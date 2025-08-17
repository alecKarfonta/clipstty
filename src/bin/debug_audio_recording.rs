//! Debug audio recording functionality
//! 
//! This binary helps debug audio recording issues by:
//! 1. Testing audio device availability
//! 2. Starting a real recording session
//! 3. Monitoring audio buffer in real-time
//! 4. Generating comprehensive outputs

use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::path::PathBuf;
use tracing::{info, debug, warn};
use tracing_subscriber::prelude::*;
use tokio::time::sleep;

use stt_clippy::services::{
    audio::AudioService,
    audio_session_manager::{AudioSessionManager, SessionConfig, AudioSource},
};

fn init_logging() {
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("debug_audio_recording=debug".parse().unwrap())
        .add_directive("stt_clippy::services::audio_session_manager=debug".parse().unwrap())
        .add_directive("stt_clippy::services::audio=debug".parse().unwrap());

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .with(filter)
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    
    println!("╭─────────────────────────────────────────────────────────────────────────╮");
    println!("│                    Audio Recording Debug Tool                          │");
    println!("│              Testing real-time audio capture and processing            │");
    println!("╰─────────────────────────────────────────────────────────────────────────╯");
    println!();
    
    // Create permanent directory for test results
    let storage_dir = PathBuf::from("./debug_audio_sessions");
    if !storage_dir.exists() {
        std::fs::create_dir_all(&storage_dir)?;
    }
    
    info!("Debug storage directory: {}", storage_dir.display());
    println!("📁 Debug files will be saved to: {}", storage_dir.display());
    
    // Create audio service
    let audio_service = Arc::new(Mutex::new(AudioService::new()?));
    
    // List available audio devices
    println!("📱 AVAILABLE AUDIO DEVICES:");
    if let Ok(audio_svc) = audio_service.lock() {
        match audio_svc.get_devices() {
            Ok(devices) => {
                for (i, device) in devices.iter().enumerate() {
                    println!("├─ {}: {} ({})", 
                        i + 1, 
                        device.name,
                        if device.is_default { "DEFAULT" } else { "AVAILABLE" }
                    );
                    println!("│  ├─ Type: {:?}", device.device_type);
                    println!("│  ├─ Sample Rates: {:?}", device.sample_rates);
                    println!("│  └─ Channels: {:?}", device.channels);
                }
            }
            Err(e) => {
                warn!("Failed to get audio devices: {}", e);
            }
        }
    }
    println!();
    
    // Create session manager with debug configuration
    let config = SessionConfig {
        auto_transcribe: false,          // Disable for debug
        real_time_transcription: false,  // Disable for debug
        save_raw_audio: true,
        compress_audio: false,
        max_session_duration: Duration::from_secs(30), // Short test
        silence_timeout: Duration::from_millis(300),
        quality_monitoring: true,
        backup_enabled: false,
        default_audio_source: AudioSource::Microphone,
    };
    
    let mut session_manager = AudioSessionManager::new(
        audio_service.clone(),
        storage_dir.clone(),
        config,
    )?;
    
    // Start a debug recording session
    println!("🎙️  Starting debug recording session...");
    println!("📢 PLEASE SPEAK INTO YOUR MICROPHONE FOR THE NEXT 10 SECONDS");
    println!();
    
    let session_id = session_manager.start_recording_session(
        "Debug Audio Test".to_string(),
        Some("Testing real-time audio capture and buffer monitoring".to_string()),
        Some(AudioSource::Microphone),
        vec!["debug".to_string(), "test".to_string()],
    )?;
    
    info!("Session ID: {}", session_id);
    
    // Monitor audio capture for 10 seconds
    let monitoring_duration = Duration::from_secs(10);
    let check_interval = Duration::from_millis(500);
    let total_checks = monitoring_duration.as_millis() / check_interval.as_millis();
    
    println!("📊 MONITORING AUDIO CAPTURE:");
    for i in 0..total_checks {
        sleep(check_interval).await;
        
        let buffer_size = session_manager.get_audio_buffer_size();
        let is_capturing = session_manager.is_audio_service_capturing();
        let is_recording = session_manager.is_recording();
        
        println!("├─ Check {}/{}: Buffer={} samples, Capturing={}, Recording={}", 
            i + 1, total_checks, buffer_size, is_capturing, is_recording);
        
        if buffer_size > 0 {
            println!("│  ✅ Audio data is being captured!");
        }
    }
    
    println!();
    
    // Add some test transcript segments
    session_manager.add_transcript_segment(
        "This is a debug test of the audio recording system.".to_string(),
        0.95,
        Duration::from_secs(1),
        Duration::from_secs(4),
    )?;
    
    session_manager.add_transcript_segment(
        "Testing comprehensive output generation.".to_string(),
        0.90,
        Duration::from_secs(5),
        Duration::from_secs(8),
    )?;
    
    // Stop the session
    println!("⏹️  Stopping recording session...");
    let final_buffer_size = session_manager.get_audio_buffer_size();
    println!("📊 Final buffer size: {} samples", final_buffer_size);
    
    if let Some(completed_session) = session_manager.stop_recording_session()? {
        println!("✅ Session completed successfully!");
        
        // Display session information
        println!("\n📊 SESSION SUMMARY:");
        println!("├─ Session ID: {}", completed_session.id);
        println!("├─ Name: {}", completed_session.name);
        println!("├─ Duration: {:?}", completed_session.duration);
        println!("├─ File Size: {} bytes", completed_session.file_size);
        println!("├─ Transcript Segments: {}", completed_session.transcript_segments.len());
        println!("└─ State: {:?}", completed_session.state);
        
        // Check what files were generated
        println!("\n📁 GENERATED FILES:");
        let session_dir = completed_session.file_path.parent().unwrap();
        
        if session_dir.exists() {
            println!("Session directory: {}", session_dir.display());
            
            if let Ok(entries) = std::fs::read_dir(session_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        let metadata = std::fs::metadata(&path)?;
                        let file_type = if path.is_dir() { "📁" } else { "📄" };
                        println!("├─ {} {} ({} bytes)", 
                            file_type, 
                            path.file_name().unwrap().to_string_lossy(),
                            metadata.len()
                        );
                        
                        // If it's the segments directory, list its contents
                        if path.is_dir() && path.file_name().unwrap() == "segments" {
                            if let Ok(segment_entries) = std::fs::read_dir(&path) {
                                for segment_entry in segment_entries {
                                    if let Ok(segment_entry) = segment_entry {
                                        let segment_path = segment_entry.path();
                                        let segment_metadata = std::fs::metadata(&segment_path)?;
                                        println!("│  └─ 🎵 {} ({} bytes)",
                                            segment_path.file_name().unwrap().to_string_lossy(),
                                            segment_metadata.len()
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            warn!("Session directory does not exist: {}", session_dir.display());
        }
        
    } else {
        println!("❌ No session was active to stop");
    }
    
    println!("\n🔍 DIAGNOSIS:");
    if final_buffer_size == 0 {
        println!("❌ No audio data was captured. Possible issues:");
        println!("   • Microphone permissions not granted");
        println!("   • No default audio input device");
        println!("   • Audio service not properly initialized");
        println!("   • Audio callback not being triggered");
    } else {
        println!("✅ Audio data was captured successfully!");
        println!("   • Buffer contained {} samples", final_buffer_size);
        println!("   • Check generated files for comprehensive outputs");
    }
    
    println!("\n✅ Audio recording debug completed!");
    println!("📁 Debug files saved to: {}", storage_dir.display());
    
    Ok(())
}
