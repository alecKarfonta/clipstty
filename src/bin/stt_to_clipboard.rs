use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use stt_clippy::services::{audio::AudioService, clipboard::ClipboardService, stt::STTService};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure model path is provided for local backend
    let model_path = std::env::var("WHISPER_MODEL_PATH")?;
    println!("Using WHISPER_MODEL_PATH={}", model_path);

    // Capture ~5 seconds of audio
    let mut audio_service = AudioService::new()?;
    let captured: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_ref = captured.clone();
    audio_service.on_audio_frame(move |frame, _sr| {
        if let Ok(mut buf) = captured_ref.lock() {
            buf.extend_from_slice(frame);
        }
    });

    audio_service.start_capture()?;
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(50));
    }
    audio_service.stop_capture()?;

    // Read captured audio
    let audio: Vec<f32> = {
        let guard = captured.lock().unwrap();
        guard.clone()
    };
    if audio.is_empty() {
        return Err("No audio captured".into());
    }

    // Transcribe with local backend
    let mut stt = STTService::new()?;
    let result = stt.transcribe(&audio)?;
    println!("Transcription: {}", result.text);

    // Copy to clipboard
    let mut clipboard = ClipboardService::new()?;
    clipboard.copy_text(&result.text)?;
    println!("Copied transcript to clipboard");

    Ok(())
}


