//! Test enhanced audio session manager with comprehensive outputs
//! 
//! This test demonstrates the new session saving functionality that creates:
//! 1. Raw audio file (complete recording)
//! 2. Cleaned audio file (silence removed)
//! 3. Individual segment files
//! 4. Comprehensive metadata file

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tempfile::TempDir;
use tracing::{info, debug};
use tracing_subscriber::prelude::*;

use stt_clippy::services::{
    audio::AudioService,
    audio_session_manager::{AudioSessionManager, SessionConfig, AudioSource},
};

fn init_logging() {
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("test_enhanced_session_manager=debug".parse().unwrap())
        .add_directive("stt_clippy::services::audio_session_manager=debug".parse().unwrap());

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .with(filter)
        .init();
}

fn generate_test_audio_with_silence() -> Vec<f32> {
    let sample_rate = 44100;
    let mut samples = Vec::new();
    
    // Generate test audio: speech -> silence -> speech -> silence -> speech
    // Each segment is 2 seconds
    
    // Speech segment 1 (0-2s): 440Hz tone
    for i in 0..(sample_rate * 2) {
        let t = i as f32 / sample_rate as f32;
        let sample = 0.3 * (2.0 * std::f32::consts::PI * 440.0 * t).sin();
        samples.push(sample);
    }
    
    // Silence segment 1 (2-3s): very low noise
    for _ in 0..sample_rate {
        samples.push(0.001 * (rand::random::<f32>() - 0.5));
    }
    
    // Speech segment 2 (3-5s): 880Hz tone
    for i in 0..(sample_rate * 2) {
        let t = i as f32 / sample_rate as f32;
        let sample = 0.3 * (2.0 * std::f32::consts::PI * 880.0 * t).sin();
        samples.push(sample);
    }
    
    // Silence segment 2 (5-6s): very low noise
    for _ in 0..sample_rate {
        samples.push(0.001 * (rand::random::<f32>() - 0.5));
    }
    
    // Speech segment 3 (6-8s): 1320Hz tone
    for i in 0..(sample_rate * 2) {
        let t = i as f32 / sample_rate as f32;
        let sample = 0.3 * (2.0 * std::f32::consts::PI * 1320.0 * t).sin();
        samples.push(sample);
    }
    
    info!(
        total_samples = samples.len(),
        duration_seconds = samples.len() as f32 / sample_rate as f32,
        "🎵 Generated test audio with speech and silence segments"
    );
    
    samples
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    
    println!("╭─────────────────────────────────────────────────────────────────────────╮");
    println!("│                Enhanced Audio Session Manager Test                      │");
    println!("│          Testing comprehensive session output generation                │");
    println!("╰─────────────────────────────────────────────────────────────────────────╯");
    println!();
    
    // Create temporary directory for test
    let temp_dir = TempDir::new()?;
    let storage_dir = temp_dir.path().to_path_buf();
    
    info!("Test storage directory: {}", storage_dir.display());
    
    // Create audio service (mock for testing)
    let audio_service = Arc::new(Mutex::new(AudioService::new()?));
    
    // Create session manager with test configuration
    let config = SessionConfig {
        auto_transcribe: true,
        real_time_transcription: false, // Disable for test
        save_raw_audio: true,
        compress_audio: false, // Keep uncompressed for test
        max_session_duration: Duration::from_secs(60),
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
    
    // Generate test audio data
    let test_audio = generate_test_audio_with_silence();
    
    // Start a test session
    info!("🎙️  Starting test recording session...");
    let session_id = session_manager.start_recording_session(
        "Enhanced Test Session".to_string(),
        Some("Test session demonstrating comprehensive output generation".to_string()),
        Some(AudioSource::Microphone),
        vec!["test".to_string(), "enhanced".to_string(), "demo".to_string()],
    )?;
    
    info!("Session ID: {}", session_id);
    
    // Simulate adding some transcript segments
    session_manager.add_transcript_segment(
        "This is the first speech segment with a clear tone.".to_string(),
        0.95,
        Duration::from_secs(0),
        Duration::from_secs(2),
    )?;
    
    session_manager.add_transcript_segment(
        "Here is the second speech segment at a higher frequency.".to_string(),
        0.92,
        Duration::from_secs(3),
        Duration::from_secs(5),
    )?;
    
    session_manager.add_transcript_segment(
        "Finally, the third segment demonstrates the highest tone.".to_string(),
        0.88,
        Duration::from_secs(6),
        Duration::from_secs(8),
    )?;
    
    // Simulate recording by adding test audio data
    // In real usage, this would come from the audio callback
    info!("🎵 Adding test audio data to session...");
    session_manager.add_test_audio_data(&test_audio)?;
    
    // Stop the session (this will trigger comprehensive output generation)
    info!("⏹️  Stopping recording session...");
    if let Some(completed_session) = session_manager.stop_recording_session()? {
        info!("✅ Session completed successfully!");
        
        // Display session information
        println!("\n📊 SESSION SUMMARY:");
        println!("├─ Session ID: {}", completed_session.id);
        println!("├─ Name: {}", completed_session.name);
        println!("├─ Duration: {:?}", completed_session.duration);
        println!("├─ File Size: {} bytes", completed_session.file_size);
        println!("├─ Transcript Segments: {}", completed_session.transcript_segments.len());
        println!("└─ Tags: {:?}", completed_session.tags);
        
        // List generated files
        println!("\n📁 GENERATED FILES:");
        let session_dir = completed_session.file_path.parent().unwrap();
        
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
        
        // Display metadata file content
        let metadata_path = session_dir.join("session_metadata.json");
        if metadata_path.exists() {
            println!("\n📋 METADATA PREVIEW:");
            if let Ok(metadata_content) = std::fs::read_to_string(&metadata_path) {
                // Parse and display key information
                if let Ok(metadata_json) = serde_json::from_str::<serde_json::Value>(&metadata_content) {
                    if let Some(outputs) = metadata_json.get("outputs") {
                        if let Some(segments) = outputs.get("segments") {
                            if let Some(segments_array) = segments.as_array() {
                                println!("├─ Audio Segments: {}", segments_array.len());
                                for (i, segment) in segments_array.iter().enumerate() {
                                    if let (Some(start), Some(end), Some(text)) = (
                                        segment.get("start_time"),
                                        segment.get("end_time"), 
                                        segment.get("text")
                                    ) {
                                        println!("│  ├─ Segment {}: {}s-{}s", 
                                            i + 1,
                                            start.get("secs").unwrap_or(&serde_json::Value::Number(0.into())),
                                            end.get("secs").unwrap_or(&serde_json::Value::Number(0.into()))
                                        );
                                        if let Some(text_str) = text.as_str() {
                                            println!("│  │  └─ Text: \"{}\"", text_str);
                                        }
                                    }
                                }
                            }
                        }
                        
                        if let Some(raw_duration) = outputs.get("total_raw_duration") {
                            println!("├─ Raw Duration: {}s", 
                                raw_duration.get("secs").unwrap_or(&serde_json::Value::Number(0.into()))
                            );
                        }
                        
                        if let Some(cleaned_duration) = outputs.get("total_cleaned_duration") {
                            println!("├─ Cleaned Duration: {}s", 
                                cleaned_duration.get("secs").unwrap_or(&serde_json::Value::Number(0.into()))
                            );
                        }
                        
                        if let Some(silence_removed) = outputs.get("silence_removed_duration") {
                            println!("└─ Silence Removed: {}s", 
                                silence_removed.get("secs").unwrap_or(&serde_json::Value::Number(0.into()))
                            );
                        }
                    }
                }
            }
        }
        
    } else {
        println!("❌ No session was active to stop");
    }
    
    println!("\n✅ Enhanced session manager test completed!");
    println!("📁 Test files saved to: {}", storage_dir.display());
    
    Ok(())
}
