# Synthetic Audio Test Fixtures

All files in this directory are 100% synthetic, copyright-free test signals generated solely for automated decoder and pitch-analysis verification (no personal audio or third-party recordings):

- `tone_440_48k.wav` — 1.0 s 440.0 Hz sine wave (amplitude 0.5, 48 kHz, 16-bit mono PCM), generated deterministically by `cargo run -p audio-decode --example gen_fixtures --locked`.
- `tone_440_44k1.flac` — 1.0 s 440.0 Hz sine wave (amplitude 0.5, 44.1 kHz, 16-bit mono RFC 9639 verbatim FLAC), generated deterministically by `gen_fixtures`.
- `tone_440_48k.mp3` — 1.0 s 440.0 Hz sine wave (48 kHz MPEG-1 Layer III), transcoded locally from `tone_440_48k.wav` using Windows built-in Media Foundation (`Windows.Media.Transcoding.MediaTranscoder`).
