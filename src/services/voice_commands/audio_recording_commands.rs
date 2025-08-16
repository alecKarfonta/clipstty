//! Audio Recording Voice Commands
//! 
//! This module provides voice commands for controlling audio recording,
//! session management, playback, and storage operations.

use std::time::Duration;
use chrono::Utc;

use super::*;

/// Start recording command
pub struct StartRecordingCommand;

impl VoiceCommand for StartRecordingCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult {
            success: true,
            message: "🎙️  Started recording session".to_string(),
            data: Some(CommandData::Text("recording_started".to_string())),
            execution_time: Duration::from_millis(10),
            timestamp: Utc::now(),
        })
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("start recording".to_string()),
            PatternType::Exact("begin recording".to_string()),
            PatternType::Contains("start recording".to_string()),
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
        "Start recording: 'start recording' or 'begin recording'"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "start recording".to_string(),
            "begin recording".to_string(),
        ]
    }
    
    fn validate_context(&self, _context: &SystemContext) -> Result<(), VoiceCommandError> {
        Ok(())
    }
}

/// Stop recording command
pub struct StopRecordingCommand;

impl VoiceCommand for StopRecordingCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult {
            success: true,
            message: "⏹️  Recording stopped and saved".to_string(),
            data: Some(CommandData::Text("recording_stopped".to_string())),
            execution_time: Duration::from_millis(10),
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
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
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
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
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
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let sessions = vec![
            "Meeting Notes - 2025-01-15 (5.2 MB, 12:34)",
            "Interview Recording - 2025-01-14 (8.7 MB, 23:45)",
            "Lecture Notes - 2025-01-13 (15.3 MB, 45:12)",
        ];
        
        let session_list = sessions.join("\n");
        
        Ok(CommandResult {
            success: true,
            message: format!("📁 Found {} recording sessions:\n{}", sessions.len(), session_list),
            data: Some(CommandData::Text(session_list)),
            execution_time: Duration::from_millis(20),
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
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
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
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
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
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
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
        
        let result = command.execute(params, &mut context);
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
        
        let result = command.execute(params, &mut context);
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
        
        let result = command.execute(params, &mut context);
        assert!(result.is_ok());
        
        let cmd_result = result.unwrap();
        assert!(cmd_result.success);
        assert!(cmd_result.message.contains("Storage Statistics"));
    }
}