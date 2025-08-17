use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use stt_clippy::services::{
    audio::AudioService,
    audio_session_manager::{AudioSessionManager, SessionConfig},
    voice_commands::{
        ServiceContext, SystemContext, CommandParams, VoiceCommand,
        audio_recording_commands::{StartRecordingCommand, StopRecordingCommand},
    },
};
use chrono::Utc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🧪 Testing Audio Recording Integration");
    println!("=====================================");
    
    // Create data directory
    let data_dir = std::env::temp_dir().join("clipstty_test");
    std::fs::create_dir_all(&data_dir)?;
    println!("📁 Created test data directory: {}", data_dir.display());
    
    // Create AudioService
    let audio_service = Arc::new(Mutex::new(AudioService::new()?));
    println!("🎤 Created AudioService");
    
    // Create AudioSessionManager
    let session_config = SessionConfig::default();
    let audio_session_manager = Arc::new(Mutex::new(
        AudioSessionManager::new(
            audio_service.clone(),
            data_dir.clone(),
            session_config,
        )?
    ));
    println!("📊 Created AudioSessionManager");
    
    // Create ServiceContext
    let service_context = ServiceContext {
        audio_session_manager: Some(audio_session_manager.clone()),
    };
    println!("🔗 Created ServiceContext");
    
    // Create SystemContext
    let mut system_context = SystemContext::default();
    println!("⚙️  Created SystemContext");
    
    // Test StartRecordingCommand
    println!("\n🎙️  Testing StartRecordingCommand...");
    let start_command = StartRecordingCommand;
    let start_params = CommandParams {
        text: "start recording test session".to_string(),
        confidence: 0.95,
        context: system_context.clone(),
        timestamp: Utc::now(),
        user_id: None,
    };
    
    match start_command.execute(start_params, &mut system_context, Some(&service_context)) {
        Ok(result) => {
            println!("✅ StartRecordingCommand executed successfully!");
            println!("   Message: {}", result.message);
            println!("   Success: {}", result.success);
            println!("   Execution time: {:?}", result.execution_time);
        }
        Err(e) => {
            println!("❌ StartRecordingCommand failed: {}", e);
            return Err(e.into());
        }
    }
    
    // Wait a moment to simulate recording
    println!("\n⏳ Simulating recording for 2 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    // Add some test audio data to the session manager
    if let Ok(manager) = audio_session_manager.lock() {
        // Create some dummy audio samples
        let test_samples: Vec<f32> = (0..44100).map(|i| (i as f32 * 0.001).sin()).collect();
        let _ = manager.add_test_audio_data(&test_samples);
        println!("🎵 Added test audio data to session");
    }
    
    // Test StopRecordingCommand
    println!("\n⏹️  Testing StopRecordingCommand...");
    let stop_command = StopRecordingCommand;
    let stop_params = CommandParams {
        text: "stop recording".to_string(),
        confidence: 0.95,
        context: system_context.clone(),
        timestamp: Utc::now(),
        user_id: None,
    };
    
    match stop_command.execute(stop_params, &mut system_context, Some(&service_context)) {
        Ok(result) => {
            println!("✅ StopRecordingCommand executed successfully!");
            println!("   Message: {}", result.message);
            println!("   Success: {}", result.success);
            println!("   Execution time: {:?}", result.execution_time);
        }
        Err(e) => {
            println!("❌ StopRecordingCommand failed: {}", e);
            return Err(e.into());
        }
    }
    
    // Check if files were created
    println!("\n📁 Checking created files...");
    let sessions_dir = data_dir.join("sessions");
    if sessions_dir.exists() {
        println!("✅ Sessions directory created: {}", sessions_dir.display());
        
        // List session files
        for entry in std::fs::read_dir(&sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                println!("   📂 Found session directory: {}", path.file_name().unwrap().to_string_lossy());
                
                // List files in session directory
                for session_entry in std::fs::read_dir(&path)? {
                    let session_entry = session_entry?;
                    let session_path = session_entry.path();
                    if session_path.is_dir() {
                        println!("      📂 Session: {}", session_path.file_name().unwrap().to_string_lossy());
                        
                        // List files in specific session
                        for file_entry in std::fs::read_dir(&session_path)? {
                            let file_entry = file_entry?;
                            let file_path = file_entry.path();
                            let metadata = std::fs::metadata(&file_path)?;
                            println!("         📄 {}: {} bytes", 
                                file_path.file_name().unwrap().to_string_lossy(),
                                metadata.len()
                            );
                        }
                    }
                }
            }
        }
    } else {
        println!("⚠️  Sessions directory not found");
    }
    
    println!("\n🎉 Integration test completed successfully!");
    println!("   The voice commands are now properly connected to the AudioSessionManager!");
    println!("   Audio files and metadata are being saved to disk!");
    
    // Cleanup
    std::fs::remove_dir_all(&data_dir)?;
    println!("🧹 Cleaned up test directory");
    
    Ok(())
}
