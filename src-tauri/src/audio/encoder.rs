use crate::error::AppResult;

/// Encode i16 PCM samples to WAV bytes (16-bit, mono).
/// `sample_rate` is the actual recording sample rate (e.g. 16000, 48000).
pub fn encode_wav(samples: &[i16], sample_rate: u32) -> AppResult<Vec<u8>> {
    let data_size = (samples.len() * 2) as u32;
    let file_size = 36 + data_size;
    let byte_rate = sample_rate * 2; // 1ch * 16bit = 2 bytes per sample
    let mut buf = Vec::with_capacity(44 + samples.len() * 2);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // fmt chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());   // chunk size (PCM)
    buf.extend_from_slice(&1u16.to_le_bytes());     // audio format (PCM)
    buf.extend_from_slice(&1u16.to_le_bytes());     // channels (mono)
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());     // block align
    buf.extend_from_slice(&16u16.to_le_bytes());    // bits per sample

    // data chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());
    for &s in samples {
        buf.extend_from_slice(&s.to_le_bytes());
    }

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_empty() {
        let wav = encode_wav(&[], 16000).unwrap();
        assert_eq!(wav.len(), 44);
    }

    #[test]
    fn test_encode_samples() {
        let samples: Vec<i16> = (0..16000).map(|i| (i % 1000) as i16).collect();
        let wav = encode_wav(&samples, 16000).unwrap();
        assert_eq!(wav.len(), 44 + 32000);
    }

    #[test]
    fn test_encode_48khz() {
        let samples = vec![0i16; 48000];
        let wav = encode_wav(&samples, 48000).unwrap();
        assert_eq!(wav.len(), 44 + 96000);
        // Verify sample rate field at offset 24..28
        let rate = u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]);
        assert_eq!(rate, 48000);
    }

    #[test]
    fn test_wav_header() {
        let wav = encode_wav(&[0i16; 100], 16000).unwrap();
        // RIFF header
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        // fmt chunk
        assert_eq!(&wav[12..16], b"fmt ");
        // data chunk
        assert_eq!(&wav[36..40], b"data");
        // data size (100 samples * 2 bytes)
        let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
        assert_eq!(data_size, 200);
    }
}
