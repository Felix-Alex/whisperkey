use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio::net::TcpStream;

use crate::asr::r#trait::{AsrProvider, AsrResponse};
use crate::config::schema::AsrConfig;
use crate::error::{AppError, AppResult};

pub struct VolcengineAsr;

// ── Streaming session for raw mode (real-time) ──

type WsConn = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct VolcengineStreamer {
    ws: WsConn,
    sample_rate: u32,
    seq: i32,
}

impl VolcengineStreamer {
    /// Connect and send full-client-request. Returns ready-to-stream session.
    pub async fn connect(config: &AsrConfig, sample_rate: u32) -> AppResult<Self> {
        let api_key = &config.api_key;
        if api_key.is_empty() {
            return Err(AppError::AsrAuth);
        }

        let host = config
            .base_url
            .trim_end_matches('/')
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        let ws_url = format!("wss://{host}/api/v3/sauc/bigmodel_async");

        let connect_id = uuid::Uuid::new_v4().to_string();
        let request_id = uuid::Uuid::new_v4().to_string();

        let mut req = ws_url
            .as_str()
            .into_client_request()
            .map_err(|_| AppError::Network)?;
        let headers = req.headers_mut();
        headers.insert("X-Api-Key", api_key.as_str().parse().map_err(|_| {
            tracing::error!("volcengine: API key contains invalid header characters");
            AppError::AsrAuth
        })?);
        headers.insert("X-Api-Resource-Id", "volc.seedasr.sauc.duration".parse().map_err(|_| AppError::Internal)?);
        headers.insert("X-Api-Request-Id", request_id.as_str().parse().map_err(|_| AppError::Internal)?);
        headers.insert("X-Api-Connect-Id", connect_id.as_str().parse().map_err(|_| AppError::Internal)?);
        headers.insert("X-Api-Sequence", "-1".parse().map_err(|_| AppError::Internal)?);

        let (mut ws, _resp) = tokio_tungstenite::connect_async(req)
            .await
            .map_err(|e| {
                tracing::error!("volcengine streamer WS connect: {e}");
                AppError::Network
            })?;

        // Send full-client-request
        ws.send(tokio_tungstenite::tungstenite::Message::Binary(
            build_full_client_request(sample_rate).into(),
        ))
        .await
        .map_err(|_| AppError::Network)?;

        // Wait for server ack
        match tokio::time::timeout(tokio::time::Duration::from_secs(5), ws.next()).await {
            Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(data)))) => {
                tracing::info!("volcengine streamer: config ack len={}", data.len());
            }
            Ok(Some(Ok(other))) => {
                tracing::warn!("volcengine streamer: unexpected ack: {:?}", other);
            }
            other => {
                tracing::error!("volcengine streamer: no config ack: {:?}", other);
                return Err(AppError::Network);
            }
        }

        tracing::info!("volcengine streamer: connected, sample_rate={sample_rate}");
        Ok(Self { ws, sample_rate, seq: 2 }) // seq starts at 2: full-client-request consumed seq 1
    }

    /// Send a chunk of raw PCM audio bytes. Set `is_final=true` for the last chunk.
    pub async fn send_audio(&mut self, pcm_bytes: &[u8], is_final: bool) -> AppResult<()> {
        let chunk_seq = if is_final { -self.seq } else { self.seq };
        let frame = build_audio_frame(pcm_bytes, chunk_seq);
        self.ws
            .send(tokio_tungstenite::tungstenite::Message::Binary(frame.into()))
            .await
            .map_err(|_| AppError::Network)?;
        self.seq += 1;
        Ok(())
    }

    /// Try to read a partial text result (non-blocking, returns immediately if no data).
    pub async fn try_recv_text(&mut self) -> AppResult<Option<String>> {
        let timeout = tokio::time::Duration::from_millis(50);
        match tokio::time::timeout(timeout, self.ws.next()).await {
            Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(data)))) => {
                Ok(parse_response(&data))
            }
            Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_)))) |
            Ok(None) => {
                Ok(None)
            }
            Ok(Some(Err(e))) => {
                tracing::warn!("volcengine streamer recv error: {e}");
                Ok(None)
            }
            Err(_) => {
                // Timeout — no data available
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Consume the streamer, return final accumulated text.
    pub async fn finalize(mut self) -> AppResult<String> {
        // Send empty final frame to signal end of audio
        let frame = build_audio_frame(&[], -self.seq);
        self.ws
            .send(tokio_tungstenite::tungstenite::Message::Binary(frame.into()))
            .await
            .map_err(|_| AppError::Network)?;

        // Read remaining responses — last one has complete text
        let mut text = String::new();
        let deadline = tokio::time::Duration::from_secs(10);
        loop {
            match tokio::time::timeout(deadline, self.ws.next()).await {
                Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(data)))) => {
                    if let Some(t) = parse_response(&data) {
                        text = t;
                    }
                }
                _ => break,
            }
        }

        // Close after collecting all responses
        let _ = self.ws.close(None).await;

        if text.is_empty() {
            return Err(AppError::AsrTimeout);
        }
        Ok(text)
    }
}

// ── Binary protocol frame builder ──

fn frame_header(msg_type: u8, flags: u8, serialization: u8, compression: u8) -> [u8; 4] {
    [
        0x11, // version=1, header_size=1 (4 bytes)
        (msg_type << 4) | (flags & 0x0F),
        (serialization << 4) | (compression & 0x0F),
        0x00, // reserved
    ]
}

fn build_full_client_request(sample_rate: u32) -> Vec<u8> {
    let payload = serde_json::json!({
        "user": { "uid": "whisperkey_user" },
        "audio": {
            "format": "pcm",
            "rate": sample_rate,
            "bits": 16,
            "channel": 1,
            "language": "zh-CN"
        },
        "request": {
            "model_name": "bigmodel",
            "enable_itn": true,
            "enable_punc": true
        }
    });
    let payload_bytes = serde_json::to_vec(&payload).unwrap();
    let header = frame_header(0x01, 0x00, 0x01, 0x00); // full-client-request, JSON, no compression
    let size = (payload_bytes.len() as u32).to_be_bytes();
    let mut frame =
        Vec::with_capacity(4 + 4 + payload_bytes.len());
    frame.extend_from_slice(&header);
    frame.extend_from_slice(&size);
    frame.extend_from_slice(&payload_bytes);
    frame
}

fn build_audio_frame(pcm: &[u8], sequence: i32) -> Vec<u8> {
    let flags = if sequence < 0 { 0x03 } else { 0x01 };
    let header = frame_header(0x02, flags, 0x00, 0x00); // audio-only, raw, no compression
    let seq_bytes = sequence.to_be_bytes();
    let size = (pcm.len() as u32).to_be_bytes();
    let mut frame =
        Vec::with_capacity(4 + 4 + 4 + pcm.len());
    frame.extend_from_slice(&header);
    frame.extend_from_slice(&seq_bytes);
    frame.extend_from_slice(&size);
    frame.extend_from_slice(pcm);
    frame
}

fn parse_response(data: &[u8]) -> Option<String> {
    if data.len() < 8 {
        tracing::warn!("volcengine parse: frame too short ({} bytes)", data.len());
        return None;
    }
    let msg_type = (data[1] >> 4) & 0x0F;
    if msg_type == 0x0F {
        // Server error — log the payload
        let err_payload = String::from_utf8_lossy(&data[4..]);
        tracing::error!("volcengine server error frame: {}", err_payload.chars().take(500).collect::<String>());
        return None;
    }
    if msg_type != 0x09 {
        // Unknown message type — dump raw bytes for diagnosis
        tracing::warn!("volcengine parse: unexpected msg_type=0x{:02X}, raw: {:02X?}", msg_type, &data[..data.len().min(64)]);
        return None;
    }
    let flags = data[1] & 0x0F;
    let has_seq = (flags & 0x01) != 0;
    let off = 4 + if has_seq { 4 } else { 0 };
    if data.len() < off + 4 {
        tracing::warn!("volcengine parse: frame too short for payload size (len={}, off={})", data.len(), off);
        return None;
    }
    let payload_size =
        u32::from_be_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]) as usize;
    let start = off + 4;
    if data.len() < start + payload_size {
        tracing::warn!("volcengine parse: truncated payload (need {} bytes, have {})", start + payload_size, data.len());
        return None;
    }
    let payload = &data[start..start + payload_size];
    let json: serde_json::Value = match serde_json::from_slice(payload) {
        Ok(v) => v,
        Err(e) => {
            let raw = String::from_utf8_lossy(payload);
            tracing::warn!("volcengine parse: JSON parse error: {e}, payload: {}", raw.chars().take(300).collect::<String>());
            return None;
        }
    };
    json.get("result")?.get("text")?.as_str().map(|s| s.to_string())
}

#[async_trait]
impl AsrProvider for VolcengineAsr {
    fn name(&self) -> &'static str {
        "火山引擎语音识别"
    }

    async fn transcribe(&self, wav: Vec<u8>, config: &AsrConfig) -> AppResult<AsrResponse> {
        let api_key = &config.api_key;
        if api_key.is_empty() {
            return Err(AppError::AsrAuth);
        }

        // Extract sample rate from WAV header (bytes 24..28, LE u32)
        let sample_rate = if wav.len() >= 28 {
            u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]])
        } else {
            16000
        };

        // Extract PCM data (skip 44-byte WAV header)
        let pcm = if wav.len() > 44 { &wav[44..] } else { &[] as &[u8] };

        tracing::info!(
            "volcengine ASR: sample_rate={} pcm_bytes={} duration_ms={}",
            sample_rate,
            pcm.len(),
            pcm.len() * 1000 / (sample_rate as usize * 2)
        );

        let host = config
            .base_url
            .trim_end_matches('/')
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        let ws_url = format!("wss://{host}/api/v3/sauc/bigmodel_async");

        let connect_id = uuid::Uuid::new_v4().to_string();
        let request_id = uuid::Uuid::new_v4().to_string();

        let mut req = ws_url
            .as_str()
            .into_client_request()
            .map_err(|_| AppError::Network)?;
        let headers = req.headers_mut();
        headers.insert("X-Api-Key", api_key.as_str().parse().map_err(|_| {
            tracing::error!("volcengine batch: API key contains invalid header characters");
            AppError::AsrAuth
        })?);
        headers.insert(
            "X-Api-Resource-Id",
            "volc.seedasr.sauc.duration".parse().map_err(|_| AppError::Internal)?,
        );
        headers.insert(
            "X-Api-Request-Id",
            request_id.as_str().parse().map_err(|_| AppError::Internal)?,
        );
        headers.insert(
            "X-Api-Connect-Id",
            connect_id.as_str().parse().map_err(|_| AppError::Internal)?,
        );
        headers.insert("X-Api-Sequence", "-1".parse().map_err(|_| AppError::Internal)?);

        let (mut ws, _resp) = tokio_tungstenite::connect_async(req)
            .await
            .map_err(|e| {
                tracing::error!("volcengine WS connect: {e}");
                AppError::Network
            })?;

        // 1. Send full-client-request
        ws.send(tokio_tungstenite::tungstenite::Message::Binary(
            build_full_client_request(sample_rate).into(),
        ))
        .await
        .map_err(|_| AppError::Network)?;

        // 2. Wait for server ack (response to full-client-request)
        match tokio::time::timeout(tokio::time::Duration::from_secs(5), ws.next()).await {
            Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(data)))) => {
                tracing::info!("volcengine ASR: config ack len={}", data.len());
            }
            Ok(Some(Ok(other))) => {
                tracing::warn!("volcengine ASR: unexpected config response: {:?}", other);
            }
            other => {
                tracing::error!("volcengine ASR: no config ack: {:?}", other);
            }
        }

        // 3. Send audio chunks (~200ms each)
        let bytes_per_chunk = (sample_rate as usize * 2 / 5).max(1024);
        let total_chunks = if pcm.is_empty() { 0 } else { (pcm.len() + bytes_per_chunk - 1) / bytes_per_chunk };
        let mut seq = 2i32; // seq starts at 2: full-client-request consumed seq 1

        for chunk_idx in 0..total_chunks {
            let start = chunk_idx * bytes_per_chunk;
            let end = (start + bytes_per_chunk).min(pcm.len());
            let is_last = chunk_idx + 1 >= total_chunks;
            let chunk_seq = if is_last { -seq } else { seq };
            let frame = build_audio_frame(&pcm[start..end], chunk_seq);
            ws.send(tokio_tungstenite::tungstenite::Message::Binary(frame.into()))
                .await
                .map_err(|_| AppError::Network)?;
            seq += 1;
        }

        // If no audio, send single empty last frame
        if pcm.is_empty() {
            let frame = build_audio_frame(&[], -seq);
            ws.send(tokio_tungstenite::tungstenite::Message::Binary(frame.into()))
                .await
                .map_err(|_| AppError::Network)?;
        }

        tracing::info!("volcengine ASR: sent {} audio chunks, waiting for results", total_chunks);

        // 4. Collect responses
        let mut text = String::new();
        let mut response_count = 0u32;
        let finish = tokio::time::Duration::from_secs(30);

        loop {
            match tokio::time::timeout(finish, ws.next()).await {
                Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(data)))) => {
                    response_count += 1;
                    tracing::info!("volcengine ASR: response #{response_count} len={}", data.len());
                    if let Some(t) = parse_response(&data) {
                        tracing::info!("volcengine ASR: response text='{t}'");
                        text = t;
                    } else {
                        // Dump raw JSON for debugging
                        let raw = String::from_utf8_lossy(&data);
                        let raw_str: String = raw.chars().take(500).collect();
                        tracing::warn!("volcengine ASR: unparsed response: {}", raw_str);
                        // If server sent an error (e.g. timeout), stop waiting
                        if raw_str.contains("\"error\"") {
                            tracing::error!("volcengine ASR: server error detected, breaking loop");
                            break;
                        }
                    }
                }
                Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Close(frame)))) => {
                    tracing::info!("volcengine ASR: server closed, frame={:?}", frame);
                    break;
                }
                Ok(Some(Err(e))) => {
                    tracing::error!("volcengine WS recv error: {e}");
                    break;
                }
                Ok(None) => {
                    tracing::info!("volcengine ASR: stream ended normally");
                    break;
                }
                Err(_) => {
                    tracing::info!("volcengine ASR: response timeout after {response_count} responses");
                    break;
                }
                _ => continue,
            }
        }

        let _ = ws.close(None).await;

        if text.is_empty() {
            tracing::error!("volcengine ASR: no text in {response_count} responses");
            return Err(AppError::AsrTimeout);
        }

        tracing::info!("volcengine ASR: final text='{text}'");
        Ok(AsrResponse {
            text,
            confidence: 0.0,
        })
    }
}
