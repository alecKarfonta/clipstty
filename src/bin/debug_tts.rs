use stt_clippy::services::tts::TTSService;
use tracing::{info, error};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .with_target(true)
        .with_ansi(true);
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("debug"))
        .with(stdout_layer)
        .try_init();

    println!("🔊 Testing TTS Service Debug");
    
    // Create TTS service
    let tts_service = match TTSService::new() {
        Ok(tts) => {
            println!("✅ TTS service created successfully");
            tts
        }
        Err(e) => {
            println!("❌ Failed to create TTS service: {}", e);
            return Err(e);
        }
    };
    
    // Test simple speak
    println!("🎵 Testing simple speak...");
    if let Err(e) = tts_service.speak_and_wait("This is a test of the TTS service. Can you hear this message?").await {
        println!("❌ TTS speak failed: {}", e);
    } else {
        println!("✅ TTS speak completed");
    }
    
    // Wait a moment
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // Test announcement
    println!("🎵 Testing announcement...");
    if let Err(e) = tts_service.announce_phase_instructions("startup").await {
        println!("❌ TTS announcement failed: {}", e);
    } else {
        println!("✅ TTS announcement completed");
    }
    
    // Wait a moment
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // Test instruction
    println!("🎵 Testing instruction...");
    if let Err(e) = tts_service.announce_instruction("This is a test instruction. Please listen carefully.").await {
        println!("❌ TTS instruction failed: {}", e);
    } else {
        println!("✅ TTS instruction completed");
    }
    
    println!("🎵 TTS debug test completed");
    Ok(())
}
