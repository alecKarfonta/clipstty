use tts::Tts;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing TTS...");
    
    let mut tts = Tts::default()?;
    println!("TTS created successfully");
    
    // List available voices
    match tts.voices() {
        Ok(voices) => {
            println!("Available voices:");
            for voice in voices {
                println!("  - {}", voice.name());
            }
        }
        Err(e) => println!("Could not get voices: {}", e),
    }
    
    // Test speaking
    println!("Attempting to speak...");
    match tts.speak("Hello, this is a test of text to speech", true) {
        Ok(_) => println!("TTS speak completed successfully"),
        Err(e) => println!("TTS speak failed: {}", e),
    }
    
    Ok(())
}
