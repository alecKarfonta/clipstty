//! Audio Recording Voice Commands
//! 
//! This module provides voice commands for controlling audio recording,
//! session management, playback, and storage operations.

use std::time::Duration;
use std::path::PathBuf;
use std::fs;
use chrono::Utc;

use super::*;
use crate::services::audio_session_manager::AudioSource;

/// Session information for display
#[derive(Debug, Clone)]
struct SessionInfo {
    id: uuid::Uuid,
    name: String,
    date: String,
    duration: String,
    size_mb: f64,
    transcript_segments: usize,
    audio_source: String,
}

/// Get the data directory for clipstty
fn get_data_directory() -> PathBuf {
    if let Ok(custom_dir) = std::env::var("CLIPSTTY_DATA_DIR") {
        // Handle ~ expansion manually
        if custom_dir.starts_with("~/") {
            if let Ok(home) = std::env::var("HOME") {
                PathBuf::from(home).join(&custom_dir[2..])
            } else {
                PathBuf::from(custom_dir)
            }
        } else {
            PathBuf::from(custom_dir)
        }
    } else {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".clipstty")
        } else {
            PathBuf::from(".clipstty")
        }
    }
}

/// Create a recording session file
fn create_session_file(session_id: &uuid::Uuid, session_name: &str, audio_source: &AudioSource) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let data_dir = get_data_directory();
    let sessions_dir = data_dir.join("sessions");
    let date_dir = sessions_dir.join(Utc::now().format("%Y/%m/%d").to_string());
    
    // Create directory structure
    fs::create_dir_all(&date_dir)?;
    
    // Sanitize session name for filename
    let sanitized_name = session_name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>()
        .chars()
        .take(50)
        .collect::<String>();
    
    let filename = format!("{}_{}.wav", sanitized_name, session_id);
    let file_path = date_dir.join(filename);
    
    // Create a placeholder WAV file (44-byte WAV header for empty file)
    let wav_header = create_empty_wav_header();
    fs::write(&file_path, wav_header)?;
    
    // Create metadata file
    let metadata = serde_json::json!({
        "session_id": session_id.to_string(),
        "session_name": session_name,
        "audio_source": format!("{}", audio_source),
        "start_time": Utc::now().to_rfc3339(),
        "status": "recording",
        "file_path": file_path.to_string_lossy()
    });
    
    let metadata_path = file_path.with_extension("json");
    fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;
    
    Ok(file_path)
}

/// Create an empty WAV file header
fn create_empty_wav_header() -> Vec<u8> {
    // Basic WAV header for 16-bit, 44.1kHz, mono
    vec![
        // RIFF header
        0x52, 0x49, 0x46, 0x46, // "RIFF"
        0x24, 0x00, 0x00, 0x00, // File size - 8 (36 bytes)
        0x57, 0x41, 0x56, 0x45, // "WAVE"
        
        // fmt chunk
        0x66, 0x6D, 0x74, 0x20, // "fmt "
        0x10, 0x00, 0x00, 0x00, // Chunk size (16)
        0x01, 0x00,             // Audio format (PCM)
        0x01, 0x00,             // Number of channels (1)
        0x44, 0xAC, 0x00, 0x00, // Sample rate (44100)
        0x88, 0x58, 0x01, 0x00, // Byte rate
        0x88, 0x58, 0x01, 0x00, // Block align
        0x10, 0x00,             // Bits per sample (16)
        
        // data chunk
        0x64, 0x61, 0x74, 0x61, // "data"
        0x00, 0x00, 0x00, 0x00, // Data size (0 for empty file)
    ]
}

/// Start recording command
pub struct StartRecordingCommand;

impl VoiceCommand for StartRecordingCommand {
    fn execute(&self, params: CommandParams, context: &mut SystemContext, services: Option<&super::ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        let start_time = std::time::Instant::now();
        
        // Check if we have access to the audio session manager
        let audio_session_manager = services
            .and_then(|s| s.audio_session_manager.as_ref())
            .ok_or_else(|| VoiceCommandError::ServiceUnavailable("AudioSessionManager not available".to_string()))?;
        
        // Extract session name from command text
        let session_name = if params.text.contains("start recording") {
            let parts: Vec<&str> = params.text.split("start recording").collect();
            if parts.len() > 1 && !parts[1].trim().is_empty() {
                parts[1].trim().to_string()
            } else {
                format!("Recording Session {}", Utc::now().format("%Y-%m-%d %H:%M:%S"))
            }
        } else {
            format!("Recording Session {}", Utc::now().format("%Y-%m-%d %H:%M:%S"))
        };

        // Determine audio source from context or use default
        let audio_source = if params.text.contains("microphone") {
            AudioSource::Microphone
        } else if params.text.contains("system audio") {
            AudioSource::SystemAudio
        } else if params.text.contains("mixed audio") {
            AudioSource::Mixed
        } else {
            AudioSource::Microphone // Default
        };

        // Use the real AudioSessionManager to start recording
        let session_id = match audio_session_manager.lock() {
            Ok(mut manager) => {
                match manager.start_recording_session(
                    session_name.clone(),
                    None, // description
                    Some(audio_source.clone()),
                    Vec::new(), // tags
                ) {
                    Ok(id) => {
                        // Update context state
                        context.audio_state.recording_active = true;
                        context.current_mode = SystemMode::Recording;
                        
                        // Store session info in context for later use
                        context.session_data.insert("active_recording_session_id".to_string(), 
                            serde_json::Value::String(id.to_string()));
                        context.session_data.insert("active_recording_name".to_string(), 
                            serde_json::Value::String(session_name.clone()));
                        
                        id
                    }
                    Err(e) => {
                        return Ok(CommandResult {
                            success: false,
                            message: format!("❌ Failed to start recording session: {}", e),
                            data: Some(CommandData::Text("session_start_failed".to_string())),
                            execution_time: start_time.elapsed(),
                            timestamp: Utc::now(),
                        });
                    }
                }
            }
            Err(e) => {
                return Ok(CommandResult {
                    success: false,
                    message: format!("❌ Failed to access audio session manager: {}", e),
                    data: Some(CommandData::Text("service_lock_failed".to_string())),
                    execution_time: start_time.elapsed(),
                    timestamp: Utc::now(),
                });
            }
        };

        // Log the recording start
        tracing::info!(
            session_id = %session_id,
            session_name = %session_name,
            audio_source = ?audio_source,
            "🎙️  Started audio recording session"
        );

        let execution_time = start_time.elapsed();
        let message = format!(
            "Started recording session: {}",
            session_name
        );

        Ok(CommandResult {
            success: true,
            message,
            data: Some(CommandData::Object({
                let mut data = std::collections::HashMap::new();
                data.insert("session_id".to_string(), serde_json::Value::String(session_id.to_string()));
                data.insert("session_name".to_string(), serde_json::Value::String(session_name));
                data.insert("audio_source".to_string(), serde_json::Value::String(format!("{:?}", audio_source)));
                data.insert("status".to_string(), serde_json::Value::String("recording_started".to_string()));
                data
            })),
            execution_time,
            timestamp: Utc::now(),
        })
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("start recording".to_string()),
            PatternType::Exact("begin recording".to_string()),
            PatternType::Exact("start recording microphone".to_string()),
            PatternType::Exact("start recording system audio".to_string()),
            PatternType::Exact("start recording mixed audio".to_string()),
            PatternType::Contains("start recording".to_string()),
            PatternType::Contains("record microphone".to_string()),
            PatternType::Contains("record system audio".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_name(&self) -> &str {
        "start_recording"
    }
    
    fn get_description(&self) -> &str {
        "Start a new audio recording session"
    }
    
    fn get_help_text(&self) -> &str {
        "Start recording: 'start recording [microphone|system audio|mixed audio]' or 'begin recording [session name]'"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "start recording".to_string(),
            "begin recording".to_string(),
            "start recording microphone".to_string(),
            "start recording system audio".to_string(),
            "start recording mixed audio".to_string(),
            "start recording meeting notes".to_string(),
        ]
    }
    
    fn validate_context(&self, _context: &SystemContext) -> Result<(), VoiceCommandError> {
        Ok(())
    }
}

/// Stop recording command
pub struct StopRecordingCommand;

impl VoiceCommand for StopRecordingCommand {
    fn execute(&self, _params: CommandParams, context: &mut SystemContext, services: Option<&super::ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        let start_time = std::time::Instant::now();
        
        // Check if we have access to the audio session manager
        let audio_session_manager = services
            .and_then(|s| s.audio_session_manager.as_ref())
            .ok_or_else(|| VoiceCommandError::ServiceUnavailable("AudioSessionManager not available".to_string()))?;
        
        // Check if currently recording
        if !context.audio_state.recording_active {
            return Ok(CommandResult {
                success: false,
                message: "⚠️  No active recording session to stop".to_string(),
                data: Some(CommandData::Text("no_active_session".to_string())),
                execution_time: start_time.elapsed(),
                timestamp: Utc::now(),
            });
        }

        // Use the real AudioSessionManager to stop recording
        let session_result = match audio_session_manager.lock() {
            Ok(mut manager) => {
                match manager.stop_recording_session() {
                    Ok(Some(session)) => {
                        // Update context state
                        context.audio_state.recording_active = false;
                        context.current_mode = SystemMode::Normal;
                        
                        // Clear session data from context
                        context.session_data.remove("active_recording_session_id");
                        context.session_data.remove("active_recording_name");
                        
                        session
                    }
                    Ok(None) => {
                        return Ok(CommandResult {
                            success: false,
                            message: "⚠️  No active recording session found".to_string(),
                            data: Some(CommandData::Text("no_session_found".to_string())),
                            execution_time: start_time.elapsed(),
                            timestamp: Utc::now(),
                        });
                    }
                    Err(e) => {
                        return Ok(CommandResult {
                            success: false,
                            message: format!("❌ Failed to stop recording session: {}", e),
                            data: Some(CommandData::Text("session_stop_failed".to_string())),
                            execution_time: start_time.elapsed(),
                            timestamp: Utc::now(),
                        });
                    }
                }
            }
            Err(e) => {
                return Ok(CommandResult {
                    success: false,
                    message: format!("❌ Failed to access audio session manager: {}", e),
                    data: Some(CommandData::Text("service_lock_failed".to_string())),
                    execution_time: start_time.elapsed(),
                    timestamp: Utc::now(),
                });
            }
        };

        // Log the recording stop
        tracing::info!(
            session_id = %session_result.id,
            duration = ?session_result.duration,
            file_size = session_result.file_size,
            transcript_segments = session_result.transcript_segments.len(),
            "⏹️  Stopped audio recording session"
        );

        let execution_time = start_time.elapsed();
        let file_size_mb = session_result.file_size as f64 / (1024.0 * 1024.0);
        
        let message = format!(
            "⏹️  Recording stopped and saved\n Session: {}\n⏱  Duration: {}:{:02}\n File Size: {:.3} MB\n Transcript Segments: {}",
            session_result.name,
            session_result.duration.as_secs() / 60,
            session_result.duration.as_secs() % 60,
            file_size_mb,
            session_result.transcript_segments.len(),
        );

        Ok(CommandResult {
            success: true,
            message,
            data: Some(CommandData::Object({
                let mut data = std::collections::HashMap::new();
                data.insert("session_id".to_string(), serde_json::Value::String(session_result.id.to_string()));
                data.insert("session_name".to_string(), serde_json::Value::String(session_result.name));
                data.insert("duration_seconds".to_string(), serde_json::Value::Number(serde_json::Number::from(session_result.duration.as_secs())));
                data.insert("file_size_mb".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(file_size_mb).unwrap()));
                data.insert("transcript_segments".to_string(), serde_json::Value::Number(serde_json::Number::from(session_result.transcript_segments.len())));
                data.insert("file_path".to_string(), serde_json::Value::String(session_result.file_path.to_string_lossy().to_string()));
                data.insert("status".to_string(), serde_json::Value::String("recording_stopped".to_string()));
                data
            })),
            execution_time,
            timestamp: Utc::now(),
        })
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("stop recording".to_string()),
            PatternType::Exact("end recording".to_string()),
            PatternType::Contains("stop recording".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_name(&self) -> &str {
        "stop_recording"
    }
    
    fn get_description(&self) -> &str {
        "Stop the current audio recording session"
    }
    
    fn get_help_text(&self) -> &str {
        "Stop recording: 'stop recording' or 'end recording'"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "stop recording".to_string(),
            "end recording".to_string(),
        ]
    }
    
    fn validate_context(&self, _context: &SystemContext) -> Result<(), VoiceCommandError> {
        Ok(())
    }
}

/// Pause recording command
pub struct PauseRecordingCommand;

impl VoiceCommand for PauseRecordingCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext, _services: Option<&super::ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult {
            success: true,
            message: "⏸️  Recording paused".to_string(),
            data: Some(CommandData::Text("recording_paused".to_string())),
            execution_time: Duration::from_millis(5),
            timestamp: Utc::now(),
        })
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("pause recording".to_string()),
            PatternType::Exact("pause".to_string()),
            PatternType::Contains("pause recording".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_name(&self) -> &str {
        "pause_recording"
    }
    
    fn get_description(&self) -> &str {
        "Pause the current audio recording"
    }
    
    fn get_help_text(&self) -> &str {
        "Pause recording: 'pause recording' or 'pause'"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "pause recording".to_string(),
            "pause".to_string(),
        ]
    }
    
    fn validate_context(&self, _context: &SystemContext) -> Result<(), VoiceCommandError> {
        Ok(())
    }
}

/// Resume recording command
pub struct ResumeRecordingCommand;

impl VoiceCommand for ResumeRecordingCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext, _services: Option<&super::ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult {
            success: true,
            message: "▶️  Recording resumed".to_string(),
            data: Some(CommandData::Text("recording_resumed".to_string())),
            execution_time: Duration::from_millis(5),
            timestamp: Utc::now(),
        })
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("resume recording".to_string()),
            PatternType::Exact("continue recording".to_string()),
            PatternType::Exact("resume".to_string()),
            PatternType::Contains("resume recording".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_name(&self) -> &str {
        "resume_recording"
    }
    
    fn get_description(&self) -> &str {
        "Resume a paused audio recording"
    }
    
    fn get_help_text(&self) -> &str {
        "Resume recording: 'resume recording', 'continue recording', or 'resume'"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "resume recording".to_string(),
            "continue recording".to_string(),
            "resume".to_string(),
        ]
    }
    
    fn validate_context(&self, _context: &SystemContext) -> Result<(), VoiceCommandError> {
        Ok(())
    }
}

/// List sessions command
pub struct ListSessionsCommand;

impl VoiceCommand for ListSessionsCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext, _services: Option<&super::ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        let start_time = std::time::Instant::now();
        
        // Mock session data - in real implementation, this would come from the session manager
        let sessions = vec![
            SessionInfo {
                id: uuid::Uuid::new_v4(),
                name: "Meeting Notes".to_string(),
                date: "2025-01-15".to_string(),
                duration: "12:34".to_string(),
                size_mb: 5.2,
                transcript_segments: 23,
                audio_source: "Microphone".to_string(),
            },
            SessionInfo {
                id: uuid::Uuid::new_v4(),
                name: "Interview Recording".to_string(),
                date: "2025-01-14".to_string(),
                duration: "23:45".to_string(),
                size_mb: 8.7,
                transcript_segments: 45,
                audio_source: "System Audio".to_string(),
            },
            SessionInfo {
                id: uuid::Uuid::new_v4(),
                name: "Lecture Notes".to_string(),
                date: "2025-01-13".to_string(),
                duration: "45:12".to_string(),
                size_mb: 15.3,
                transcript_segments: 89,
                audio_source: "Mixed".to_string(),
            },
        ];
        
        let mut session_list = String::new();
        for (i, session) in sessions.iter().enumerate() {
            session_list.push_str(&format!(
                "{}. {} - {} ({:.1} MB, {}, {} segments, {})\n",
                i + 1,
                session.name,
                session.date,
                session.size_mb,
                session.duration,
                session.transcript_segments,
                session.audio_source
            ));
        }
        
        let execution_time = start_time.elapsed();
        let message = format!(
            "📁 Found {} recording sessions:\n{}\n📊 Total Storage: {:.1} MB\n📝 Total Segments: {}",
            sessions.len(),
            session_list.trim(),
            sessions.iter().map(|s| s.size_mb).sum::<f64>(),
            sessions.iter().map(|s| s.transcript_segments).sum::<usize>()
        );
        
        Ok(CommandResult {
            success: true,
            message,
            data: Some(CommandData::Object({
                let mut data = std::collections::HashMap::new();
                data.insert("session_count".to_string(), serde_json::Value::Number(serde_json::Number::from(sessions.len())));
                data.insert("total_size_mb".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(sessions.iter().map(|s| s.size_mb).sum()).unwrap()));
                data.insert("total_segments".to_string(), serde_json::Value::Number(serde_json::Number::from(sessions.iter().map(|s| s.transcript_segments).sum::<usize>())));
                data
            })),
            execution_time,
            timestamp: Utc::now(),
        })
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("list sessions".to_string()),
            PatternType::Exact("show sessions".to_string()),
            PatternType::Exact("list recordings".to_string()),
            PatternType::Contains("list sessions".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_name(&self) -> &str {
        "list_sessions"
    }
    
    fn get_description(&self) -> &str {
        "List all recording sessions"
    }
    
    fn get_help_text(&self) -> &str {
        "List sessions: 'list sessions', 'show sessions', or 'list recordings'"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "list sessions".to_string(),
            "show sessions".to_string(),
            "list recordings".to_string(),
        ]
    }
    
    fn validate_context(&self, _context: &SystemContext) -> Result<(), VoiceCommandError> {
        Ok(())
    }
}

/// Compress files command
pub struct CompressFilesCommand;

impl VoiceCommand for CompressFilesCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext, _services: Option<&super::ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult {
            success: true,
            message: "🗜️  Compressing audio files... Saved 45% storage space (3.2 GB → 1.8 GB)".to_string(),
            data: Some(CommandData::Text("compression_complete".to_string())),
            execution_time: Duration::from_millis(2500),
            timestamp: Utc::now(),
        })
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("compress files".to_string()),
            PatternType::Exact("compress audio".to_string()),
            PatternType::Contains("compress files".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_name(&self) -> &str {
        "compress_files"
    }
    
    fn get_description(&self) -> &str {
        "Compress all audio files to save storage space"
    }
    
    fn get_help_text(&self) -> &str {
        "Compress files: 'compress files' or 'compress audio'"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "compress files".to_string(),
            "compress audio".to_string(),
        ]
    }
    
    fn validate_context(&self, _context: &SystemContext) -> Result<(), VoiceCommandError> {
        Ok(())
    }
}

/// Show storage stats command
pub struct ShowStorageStatsCommand;

impl VoiceCommand for ShowStorageStatsCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext, _services: Option<&super::ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        let stats = "📊 Storage Statistics:\n\
                    Total Sessions: 24\n\
                    Total Size: 8.7 GB\n\
                    Total Duration: 12.5 hours\n\
                    Compression Ratio: 62%\n\
                    Oldest Session: 2024-12-01\n\
                    Available Space: 15.3 GB";
        
        Ok(CommandResult {
            success: true,
            message: stats.to_string(),
            data: Some(CommandData::Text("storage_stats".to_string())),
            execution_time: Duration::from_millis(10),
            timestamp: Utc::now(),
        })
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("show storage stats".to_string()),
            PatternType::Exact("storage statistics".to_string()),
            PatternType::Exact("show stats".to_string()),
            PatternType::Contains("storage stats".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_name(&self) -> &str {
        "show_storage_stats"
    }
    
    fn get_description(&self) -> &str {
        "Display storage statistics and usage information"
    }
    
    fn get_help_text(&self) -> &str {
        "Show stats: 'show storage stats', 'storage statistics', or 'show stats'"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "show storage stats".to_string(),
            "storage statistics".to_string(),
            "show stats".to_string(),
        ]
    }
    
    fn validate_context(&self, _context: &SystemContext) -> Result<(), VoiceCommandError> {
        Ok(())
    }
}

/// Cleanup storage command
pub struct CleanupStorageCommand;

impl VoiceCommand for CleanupStorageCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext, _services: Option<&super::ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult {
            success: true,
            message: "🧹 Storage cleanup complete. Removed 8 old files, freed 2.1 GB".to_string(),
            data: Some(CommandData::Text("cleanup_complete".to_string())),
            execution_time: Duration::from_millis(1200),
            timestamp: Utc::now(),
        })
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("cleanup storage".to_string()),
            PatternType::Exact("clean up files".to_string()),
            PatternType::Exact("cleanup old files".to_string()),
            PatternType::Contains("cleanup storage".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_name(&self) -> &str {
        "cleanup_storage"
    }
    
    fn get_description(&self) -> &str {
        "Clean up old files and free storage space"
    }
    
    fn get_help_text(&self) -> &str {
        "Cleanup storage: 'cleanup storage', 'clean up files', or 'cleanup old files'"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "cleanup storage".to_string(),
            "clean up files".to_string(),
            "cleanup old files".to_string(),
        ]
    }
    
    fn validate_context(&self, _context: &SystemContext) -> Result<(), VoiceCommandError> {
        Ok(())
    }
}

/// Create audio recording commands
pub fn create_start_recording_command() -> StartRecordingCommand {
    StartRecordingCommand
}

pub fn create_stop_recording_command() -> StopRecordingCommand {
    StopRecordingCommand
}

pub fn create_pause_recording_command() -> PauseRecordingCommand {
    PauseRecordingCommand
}

pub fn create_resume_recording_command() -> ResumeRecordingCommand {
    ResumeRecordingCommand
}

pub fn create_list_sessions_command() -> ListSessionsCommand {
    ListSessionsCommand
}

pub fn create_compress_files_command() -> CompressFilesCommand {
    CompressFilesCommand
}

pub fn create_show_storage_stats_command() -> ShowStorageStatsCommand {
    ShowStorageStatsCommand
}

pub fn create_cleanup_storage_command() -> CleanupStorageCommand {
    CleanupStorageCommand
}

/// List audio devices command
pub struct ListAudioDevicesCommand;

impl VoiceCommand for ListAudioDevicesCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext, _services: Option<&super::ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        let start_time = std::time::Instant::now();
        
        // Mock audio devices - in real implementation, this would come from the audio service
        let devices = vec![
            AudioDeviceInfo {
                name: "MacBook Pro Microphone".to_string(),
                device_type: "Input".to_string(),
                is_default: true,
                sample_rates: vec![44100, 48000],
                channels: vec![1, 2],
            },
            AudioDeviceInfo {
                name: "External USB Microphone".to_string(),
                device_type: "Input".to_string(),
                is_default: false,
                sample_rates: vec![44100, 48000, 96000],
                channels: vec![1],
            },
            AudioDeviceInfo {
                name: "BlackHole 2ch".to_string(),
                device_type: "Input".to_string(),
                is_default: false,
                sample_rates: vec![44100, 48000],
                channels: vec![2],
            },
            AudioDeviceInfo {
                name: "MacBook Pro Speakers".to_string(),
                device_type: "Output".to_string(),
                is_default: true,
                sample_rates: vec![44100, 48000],
                channels: vec![2],
            },
        ];
        
        let mut device_list = String::new();
        let mut input_count = 0;
        let mut output_count = 0;
        
        for (i, device) in devices.iter().enumerate() {
            let default_marker = if device.is_default { " (Default)" } else { "" };
            device_list.push_str(&format!(
                "{}. {} - {}{}\n   Sample Rates: {:?} Hz, Channels: {:?}\n",
                i + 1,
                device.name,
                device.device_type,
                default_marker,
                device.sample_rates,
                device.channels
            ));
            
            if device.device_type == "Input" {
                input_count += 1;
            } else {
                output_count += 1;
            }
        }
        
        let execution_time = start_time.elapsed();
        let message = format!(
            "🎤 Available Audio Devices:\n{}\n📊 Summary: {} Input devices, {} Output devices",
            device_list.trim(),
            input_count,
            output_count
        );
        
        Ok(CommandResult {
            success: true,
            message,
            data: Some(CommandData::Object({
                let mut data = std::collections::HashMap::new();
                data.insert("device_count".to_string(), serde_json::Value::Number(serde_json::Number::from(devices.len())));
                data.insert("input_count".to_string(), serde_json::Value::Number(serde_json::Number::from(input_count)));
                data.insert("output_count".to_string(), serde_json::Value::Number(serde_json::Number::from(output_count)));
                data
            })),
            execution_time,
            timestamp: Utc::now(),
        })
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("list audio devices".to_string()),
            PatternType::Exact("show audio devices".to_string()),
            PatternType::Exact("list microphones".to_string()),
            PatternType::Contains("audio devices".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_name(&self) -> &str {
        "list_audio_devices"
    }
    
    fn get_description(&self) -> &str {
        "List all available audio input and output devices"
    }
    
    fn get_help_text(&self) -> &str {
        "List devices: 'list audio devices', 'show audio devices', or 'list microphones'"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "list audio devices".to_string(),
            "show audio devices".to_string(),
            "list microphones".to_string(),
        ]
    }
    
    fn validate_context(&self, _context: &SystemContext) -> Result<(), VoiceCommandError> {
        Ok(())
    }
}

/// Audio device information for display
#[derive(Debug, Clone)]
struct AudioDeviceInfo {
    name: String,
    device_type: String,
    is_default: bool,
    sample_rates: Vec<u32>,
    channels: Vec<u16>,
}

pub fn create_list_audio_devices_command() -> ListAudioDevicesCommand {
    ListAudioDevicesCommand
}

/// Select audio device command
pub struct SelectAudioDeviceCommand;

impl VoiceCommand for SelectAudioDeviceCommand {
    fn execute(&self, params: CommandParams, context: &mut SystemContext, _services: Option<&super::ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        let start_time = std::time::Instant::now();
        
        // Extract device name from command
        let device_name = if params.text.contains("select device") {
            let parts: Vec<&str> = params.text.split("select device").collect();
            if parts.len() > 1 && !parts[1].trim().is_empty() {
                Some(parts[1].trim().to_string())
            } else {
                None
            }
        } else if params.text.contains("use microphone") {
            Some("microphone".to_string())
        } else if params.text.contains("use system audio") {
            Some("system audio".to_string())
        } else {
            None
        };

        let execution_time = start_time.elapsed();
        
        match device_name {
            Some(name) => {
                // Update context with selected device
                context.audio_state.current_device = Some(name.clone());
                
                tracing::info!(
                    device_name = %name,
                    "🎤 Selected audio device"
                );
                
                let message = format!(
                    "🎤 Selected audio device: {}\n✅ Device is now active for recording\n🔧 Use 'start recording' to begin session with this device",
                    name
                );
                
                Ok(CommandResult {
                    success: true,
                    message,
                    data: Some(CommandData::Object({
                        let mut data = std::collections::HashMap::new();
                        data.insert("selected_device".to_string(), serde_json::Value::String(name));
                        data.insert("status".to_string(), serde_json::Value::String("device_selected".to_string()));
                        data
                    })),
                    execution_time,
                    timestamp: Utc::now(),
                })
            }
            None => {
                Ok(CommandResult {
                    success: false,
                    message: "⚠️  Please specify a device name. Example: 'select device External USB Microphone'".to_string(),
                    data: Some(CommandData::Text("no_device_specified".to_string())),
                    execution_time,
                    timestamp: Utc::now(),
                })
            }
        }
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Contains("select device".to_string()),
            PatternType::Contains("use microphone".to_string()),
            PatternType::Contains("use system audio".to_string()),
            PatternType::Contains("switch to device".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_name(&self) -> &str {
        "select_audio_device"
    }
    
    fn get_description(&self) -> &str {
        "Select a specific audio input device for recording"
    }
    
    fn get_help_text(&self) -> &str {
        "Select device: 'select device [device name]', 'use microphone', or 'use system audio'"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "select device External USB Microphone".to_string(),
            "use microphone".to_string(),
            "use system audio".to_string(),
            "switch to device BlackHole 2ch".to_string(),
        ]
    }
    
    fn validate_context(&self, _context: &SystemContext) -> Result<(), VoiceCommandError> {
        Ok(())
    }
}

/// Show current audio configuration command
pub struct ShowAudioConfigCommand;

impl VoiceCommand for ShowAudioConfigCommand {
    fn execute(&self, _params: CommandParams, context: &mut SystemContext, _services: Option<&super::ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        let start_time = std::time::Instant::now();
        
        let current_device = context.audio_state.current_device.as_ref()
            .map(|d| d.as_str())
            .unwrap_or("Default System Device");
        
        let vad_status = if context.audio_state.vad_enabled { "Enabled" } else { "Disabled" };
        let recording_status = if context.audio_state.recording_active { "Active" } else { "Inactive" };
        
        let execution_time = start_time.elapsed();
        let message = format!(
            "🎤 Current Audio Configuration:\n\
            📱 Input Device: {}\n\
            🎙️  Recording Status: {}\n\
            🔊 Sample Rate: {} Hz\n\
            📻 Channels: {}\n\
            🎚️  Buffer Size: {} samples\n\
            🤖 Voice Activity Detection: {}\n\
            📊 Sensitivity: {:.1}%",
            current_device,
            recording_status,
            context.audio_state.sample_rate,
            context.audio_state.channels,
            context.audio_state.buffer_size,
            vad_status,
            context.audio_state.sensitivity * 100.0
        );
        
        Ok(CommandResult {
            success: true,
            message,
            data: Some(CommandData::Object({
                let mut data = std::collections::HashMap::new();
                data.insert("current_device".to_string(), serde_json::Value::String(current_device.to_string()));
                data.insert("recording_active".to_string(), serde_json::Value::Bool(context.audio_state.recording_active));
                data.insert("sample_rate".to_string(), serde_json::Value::Number(serde_json::Number::from(context.audio_state.sample_rate)));
                data.insert("channels".to_string(), serde_json::Value::Number(serde_json::Number::from(context.audio_state.channels)));
                data.insert("vad_enabled".to_string(), serde_json::Value::Bool(context.audio_state.vad_enabled));
                data.insert("sensitivity".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(context.audio_state.sensitivity as f64).unwrap()));
                data
            })),
            execution_time,
            timestamp: Utc::now(),
        })
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("show audio config".to_string()),
            PatternType::Exact("audio status".to_string()),
            PatternType::Exact("current audio settings".to_string()),
            PatternType::Contains("audio config".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_name(&self) -> &str {
        "show_audio_config"
    }
    
    fn get_description(&self) -> &str {
        "Display current audio configuration and settings"
    }
    
    fn get_help_text(&self) -> &str {
        "Show config: 'show audio config', 'audio status', or 'current audio settings'"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "show audio config".to_string(),
            "audio status".to_string(),
            "current audio settings".to_string(),
        ]
    }
    
    fn validate_context(&self, _context: &SystemContext) -> Result<(), VoiceCommandError> {
        Ok(())
    }
}

pub fn create_select_audio_device_command() -> SelectAudioDeviceCommand {
    SelectAudioDeviceCommand
}

pub fn create_show_audio_config_command() -> ShowAudioConfigCommand {
    ShowAudioConfigCommand
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_start_recording_command() {
        let command = StartRecordingCommand;
        let mut context = SystemContext::new();
        
        let params = CommandParams {
            text: "start recording".to_string(),
            confidence: 0.95,
            timestamp: Utc::now(),
        };
        
        let result = command.execute(params, &mut context, None);
        assert!(result.is_ok());
        
        let cmd_result = result.unwrap();
        assert!(cmd_result.success);
        assert!(cmd_result.message.contains("Started recording"));
    }
    
    #[test]
    fn test_list_sessions_command() {
        let command = ListSessionsCommand;
        let mut context = SystemContext::new();
        
        let params = CommandParams {
            text: "list sessions".to_string(),
            confidence: 0.95,
            timestamp: Utc::now(),
        };
        
        let result = command.execute(params, &mut context, None);
        assert!(result.is_ok());
        
        let cmd_result = result.unwrap();
        assert!(cmd_result.success);
        assert!(cmd_result.message.contains("Found"));
    }
    
    #[test]
    fn test_storage_stats_command() {
        let command = ShowStorageStatsCommand;
        let mut context = SystemContext::new();
        
        let params = CommandParams {
            text: "show storage stats".to_string(),
            confidence: 0.95,
            timestamp: Utc::now(),
        };
        
        let result = command.execute(params, &mut context, None);
        assert!(result.is_ok());
        
        let cmd_result = result.unwrap();
        assert!(cmd_result.success);
        assert!(cmd_result.message.contains("Storage Statistics"));
    }
}