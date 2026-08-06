use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AudioFormat {
    Wav,
    Opus,
    Mp3,
    Gsm,
}

#[allow(dead_code)]
pub struct AudioRecorder {
    pub session_id: String,
    pub format: AudioFormat,
    pub output_path: String,
}

#[allow(dead_code)]
impl AudioRecorder {
    pub fn new(session_id: String, format: AudioFormat, output_path: String) -> Self {
        info!(
            "Initialized AudioRecorder for session {} at {} ({:?})",
            session_id, output_path, format
        );
        AudioRecorder {
            session_id,
            format,
            output_path,
        }
    }
}
