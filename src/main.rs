//! Main entry point for stt-clippy
//! 
//! This is a simple wrapper that calls the main stt_to_clipboard functionality.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // For now, just print a message directing users to the correct binary
    eprintln!("Please use the 'stt_to_clipboard' binary instead:");
    eprintln!("  cargo run --bin stt_to_clipboard");
    eprintln!();
    eprintln!("Available binaries:");
    eprintln!("  - stt_to_clipboard: Main STT application");
    eprintln!("  - debug_tts: TTS debugging tool");
    eprintln!("  - test_recorder: Recording test tool");
    eprintln!("  - debug_audio_recording: Audio debugging tool");
    
    std::process::exit(1);
}
