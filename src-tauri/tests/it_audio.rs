/// IT-02: Audio WAV encoding logic (no hardware needed)
/// Tests WAV header structure and size calculations
use whisperkey_lib::audio::encoder::encode_wav;

#[test]
fn it02_encode_empty_produces_valid_header() {
    let wav = encode_wav(&[], 16000).unwrap();
    assert_eq!(wav.len(), 44);
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");
    assert_eq!(&wav[12..16], b"fmt ");
    // data chunk header
    assert_eq!(&wav[36..40], b"data");
    // data size = 0
    let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
    assert_eq!(data_size, 0);
}

#[test]
fn it02_encode_1_second_16khz_mono() {
    // 1 second of silence at 16kHz mono = 16000 i16 samples
    let samples = vec![0i16; 16000];
    let wav = encode_wav(&samples, 16000).unwrap();
    // 44 byte header + 16000 * 2 bytes data
    assert_eq!(wav.len(), 44 + 32000);
    // Verify file size in RIFF header
    let file_size = u32::from_le_bytes([wav[4], wav[5], wav[6], wav[7]]);
    assert_eq!(file_size as usize, 36 + 32000);
}

#[test]
fn it02_encode_3_seconds_16khz() {
    let samples = vec![0i16; 48000]; // 3s * 16000
    let wav = encode_wav(&samples, 16000).unwrap();
    assert_eq!(wav.len(), 44 + 96000);
}

#[test]
fn it02_encode_sample_rate_in_header() {
    let wav = encode_wav(&[0i16; 100], 48000).unwrap();
    // Sample rate at offset 24..28
    let rate = u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]);
    assert_eq!(rate, 48000);

    // Byte rate at offset 28..32: sample_rate * 1ch * 16bit = sample_rate * 2
    let byte_rate = u32::from_le_bytes([wav[28], wav[29], wav[30], wav[31]]);
    assert_eq!(byte_rate, 96000);
}

#[test]
fn it02_encode_mono_16bit_pcm() {
    let wav = encode_wav(&[0i16; 100], 16000).unwrap();
    // Audio format (PCM = 1) at offset 20..22
    let format = u16::from_le_bytes([wav[20], wav[21]]);
    assert_eq!(format, 1);
    // Channels at offset 22..24
    let channels = u16::from_le_bytes([wav[22], wav[23]]);
    assert_eq!(channels, 1);
    // Bits per sample at offset 34..36
    let bits = u16::from_le_bytes([wav[34], wav[35]]);
    assert_eq!(bits, 16);
}

#[test]
fn it02_encode_preserves_sample_values() {
    let samples: Vec<i16> = vec![100, -200, 300, -400, 500];
    let wav = encode_wav(&samples, 16000).unwrap();
    // Data starts at offset 44
    for (i, &expected) in samples.iter().enumerate() {
        let offset = 44 + i * 2;
        let value = i16::from_le_bytes([wav[offset], wav[offset + 1]]);
        assert_eq!(value, expected, "sample at index {i} should match");
    }
}
