//! Test binary to verify voice command recording functionality

use stt_clippy::services::voice_commands::*;
use stt_clippy::services::voice_commands::audio_recording_commands::*;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Voice Command Recording Functionality");
    println!("{}", "=".repeat(60));
    
    // Create a system context
    let mut context = SystemContext::default();
    
    // Create command parameters for "start recording"
    let start_params = CommandParams {
        text: "start recording test session".to_string(),
        confidence: 0.95,
        context: context.clone(),
        timestamp: chrono::Utc::now(),
        user_id: None,
    };
    
    // Test start recording command
    println!("🎙️  Testing StartRecordingCommand...");
    let start_cmd = StartRecordingCommand;
    match start_cmd.execute(start_params, &mut context, None) {
        Ok(result) => {
            println!("✅ Start recording result:");
            println!("   Success: {}", result.success);
            println!("   Message: {}", result.message);
            
            if let Some(data) = result.data {
                println!("   Data: {:?}", data);
            }
        }
        Err(e) => {
            println!("❌ Start recording failed: {}", e);
            return Err(e.into());
        }
    }
    
    // Wait a moment
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    // Test stop recording command
    println!("\n⏹️  Testing StopRecordingCommand...");
    let stop_params = CommandParams {
        text: "stop recording".to_string(),
        confidence: 0.95,
        context: context.clone(),
        timestamp: chrono::Utc::now(),
        user_id: None,
    };
    
    let stop_cmd = StopRecordingCommand;
    match stop_cmd.execute(stop_params, &mut context, None) {
        Ok(result) => {
            println!("✅ Stop recording result:");
            println!("   Success: {}", result.success);
            println!("   Message: {}", result.message);
            
            if let Some(data) = result.data {
                println!("   Data: {:?}", data);
            }
        }
        Err(e) => {
            println!("❌ Stop recording failed: {}", e);
            return Err(e.into());
        }
    }
    
    // Check if files were created
    println!("\n📁 Checking for created files...");
    let home = std::env::var("HOME")?;
    let sessions_dir = PathBuf::from(home)
        .join(".clipstty")
        .join("sessions")
        .join(chrono::Utc::now().format("%Y/%m/%d").to_string());
    
    if sessions_dir.exists() {
        println!("✅ Session directory exists: {}", sessions_dir.display());
        
        let entries: Vec<_> = std::fs::read_dir(&sessions_dir)?
            .filter_map(|entry| entry.ok())
            .collect();
            
        if entries.is_empty() {
            println!("⚠️  Directory exists but is empty");
        } else {
            println!("📄 Found {} files:", entries.len());
            for entry in entries {
                let path = entry.path();
                let size = std::fs::metadata(&path)?.len();
                println!("   - {} ({} bytes)", path.file_name().unwrap().to_string_lossy(), size);
                
                // If it's a JSON file, show its contents
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        println!("     Content: {}", content);
                    }
                }
            }
        }
    } else {
        println!("❌ Session directory does not exist: {}", sessions_dir.display());
    }
    
    println!("\n🎉 Test completed!");
    Ok(())
}
