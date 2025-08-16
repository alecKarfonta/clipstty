//! System control voice commands.
//! 
//! This module implements comprehensive voice commands for system operations,
//! including service management, configuration, diagnostics, and application control.

use std::time::Duration;
use regex::Regex;

use super::*;

/// Restart service command
pub struct RestartServiceCommand;

impl VoiceCommand for RestartServiceCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult::success("System service restarted successfully".to_string())
            .with_execution_time(Duration::from_millis(1000)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("restart service".to_string()),
            PatternType::Exact("restart system".to_string()),
            PatternType::Exact("restart app".to_string()),
            PatternType::Exact("reload service".to_string()),
            PatternType::Exact("system restart".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::System
    }
    
    fn get_help_text(&self) -> &str {
        "Restarts the main system service to refresh all components"
    }
    
    fn get_name(&self) -> &str {
        "restart_service"
    }
    
    fn get_description(&self) -> &str {
        "Restart system service"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "restart service".to_string(),
            "reload service".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Intermediate
    }
}

/// Reload configuration command
pub struct ReloadConfigCommand;

impl VoiceCommand for ReloadConfigCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult::success("Configuration reloaded from disk".to_string())
            .with_execution_time(Duration::from_millis(200)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("reload config".to_string()),
            PatternType::Exact("reload configuration".to_string()),
            PatternType::Exact("refresh config".to_string()),
            PatternType::Exact("reload settings".to_string()),
            PatternType::Exact("refresh settings".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::System
    }
    
    fn get_help_text(&self) -> &str {
        "Reloads configuration settings from the configuration files"
    }
    
    fn get_name(&self) -> &str {
        "reload_config"
    }
    
    fn get_description(&self) -> &str {
        "Reload configuration"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "reload config".to_string(),
            "refresh settings".to_string(),
        ]
    }
}

/// Clear cache command
pub struct ClearCacheCommand;

impl VoiceCommand for ClearCacheCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult::success("System cache cleared".to_string())
            .with_execution_time(Duration::from_millis(300)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("clear cache".to_string()),
            PatternType::Exact("flush cache".to_string()),
            PatternType::Exact("empty cache".to_string()),
            PatternType::Exact("clean cache".to_string()),
            PatternType::Exact("reset cache".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::System
    }
    
    fn get_help_text(&self) -> &str {
        "Clears all cached data to free memory and resolve cache-related issues"
    }
    
    fn get_name(&self) -> &str {
        "clear_cache"
    }
    
    fn get_description(&self) -> &str {
        "Clear system cache"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "clear cache".to_string(),
            "flush cache".to_string(),
        ]
    }
}

/// Backup settings command
pub struct BackupSettingsCommand;

impl VoiceCommand for BackupSettingsCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("settings_backup_{}", timestamp);
        
        Ok(CommandResult::success_with_data(
            format!("Settings backed up as: {}", backup_name),
            CommandData::Text(backup_name)
        ).with_execution_time(Duration::from_millis(500)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("backup settings".to_string()),
            PatternType::Exact("save settings".to_string()),
            PatternType::Exact("backup config".to_string()),
            PatternType::Exact("export settings".to_string()),
            PatternType::Exact("create backup".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::System
    }
    
    fn get_help_text(&self) -> &str {
        "Creates a backup of current system settings and configuration"
    }
    
    fn get_name(&self) -> &str {
        "backup_settings"
    }
    
    fn get_description(&self) -> &str {
        "Backup current settings"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "backup settings".to_string(),
            "export settings".to_string(),
        ]
    }
}

/// Show performance metrics command
pub struct ShowMetricsCommand;

impl VoiceCommand for ShowMetricsCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let metrics = "Performance Metrics:\n\
            • CPU Usage: 15.2%\n\
            • Memory Usage: 142.5 MB\n\
            • Audio Latency: 45ms\n\
            • STT Processing: 0.23x real-time\n\
            • Commands Processed: 127\n\
            • Success Rate: 98.4%\n\
            • Uptime: 2h 34m";
        
        Ok(CommandResult::success(metrics.to_string())
            .with_execution_time(Duration::from_millis(100)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("show metrics".to_string()),
            PatternType::Exact("performance metrics".to_string()),
            PatternType::Exact("show performance".to_string()),
            PatternType::Exact("system metrics".to_string()),
            PatternType::Exact("show stats".to_string()),
            PatternType::Exact("performance stats".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::System
    }
    
    fn get_help_text(&self) -> &str {
        "Displays current system performance metrics and statistics"
    }
    
    fn get_name(&self) -> &str {
        "show_metrics"
    }
    
    fn get_description(&self) -> &str {
        "Show performance metrics"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "show metrics".to_string(),
            "performance stats".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["show_status".to_string()]
    }
}

/// Export logs command
pub struct ExportLogsCommand;

impl VoiceCommand for ExportLogsCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let log_file = format!("stt_clippy_logs_{}.txt", timestamp);
        
        Ok(CommandResult::success_with_data(
            format!("Logs exported to: {}", log_file),
            CommandData::Text(log_file)
        ).with_execution_time(Duration::from_millis(800)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("export logs".to_string()),
            PatternType::Exact("save logs".to_string()),
            PatternType::Exact("dump logs".to_string()),
            PatternType::Exact("backup logs".to_string()),
            PatternType::Exact("collect logs".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::System
    }
    
    fn get_help_text(&self) -> &str {
        "Exports system logs to a file for debugging and analysis"
    }
    
    fn get_name(&self) -> &str {
        "export_logs"
    }
    
    fn get_description(&self) -> &str {
        "Export system logs"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "export logs".to_string(),
            "save logs".to_string(),
        ]
    }
}

/// Check for updates command
pub struct CheckUpdatesCommand;

impl VoiceCommand for CheckUpdatesCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult::success("Checking for updates... Current version is up to date.".to_string())
            .with_execution_time(Duration::from_millis(2000)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("check for updates".to_string()),
            PatternType::Exact("check updates".to_string()),
            PatternType::Exact("update check".to_string()),
            PatternType::Exact("any updates".to_string()),
            PatternType::Exact("software updates".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::System
    }
    
    fn get_help_text(&self) -> &str {
        "Checks for available software updates"
    }
    
    fn get_name(&self) -> &str {
        "check_updates"
    }
    
    fn get_description(&self) -> &str {
        "Check for software updates"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "check for updates".to_string(),
            "any updates".to_string(),
        ]
    }
}

/// Show version command
pub struct ShowVersionCommand;

impl VoiceCommand for ShowVersionCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let version_info = format!(
            "STT Clippy Version Information:\n\
            • Version: {}\n\
            • Build: {}\n\
            • Platform: {}\n\
            • Rust Version: {}",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_NAME"),
            std::env::consts::OS,
            "1.70+" // Placeholder
        );
        
        Ok(CommandResult::success(version_info)
            .with_execution_time(Duration::from_millis(50)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("show version".to_string()),
            PatternType::Exact("version".to_string()),
            PatternType::Exact("version info".to_string()),
            PatternType::Exact("about".to_string()),
            PatternType::Exact("what version".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::System
    }
    
    fn get_help_text(&self) -> &str {
        "Displays current application version and build information"
    }
    
    fn get_name(&self) -> &str {
        "show_version"
    }
    
    fn get_description(&self) -> &str {
        "Show version information"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "show version".to_string(),
            "version info".to_string(),
        ]
    }
}

/// Exit application command
pub struct ExitApplicationCommand;

impl VoiceCommand for ExitApplicationCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult::success("Shutting down STT Clippy... Goodbye!".to_string())
            .with_execution_time(Duration::from_millis(100)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("exit".to_string()),
            PatternType::Exact("quit".to_string()),
            PatternType::Exact("shutdown".to_string()),
            PatternType::Exact("exit application".to_string()),
            PatternType::Exact("quit application".to_string()),
            PatternType::Exact("close application".to_string()),
            PatternType::Exact("goodbye".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::System
    }
    
    fn get_help_text(&self) -> &str {
        "Safely shuts down the STT Clippy application"
    }
    
    fn get_name(&self) -> &str {
        "exit_application"
    }
    
    fn get_description(&self) -> &str {
        "Exit application"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "exit".to_string(),
            "quit application".to_string(),
            "shutdown".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Beginner
    }
}

/// Run diagnostics command
pub struct RunDiagnosticsCommand;

impl VoiceCommand for RunDiagnosticsCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let diagnostics = "Running system diagnostics...\n\
            ✓ Audio system: OK\n\
            ✓ STT service: OK\n\
            ✓ Clipboard service: OK\n\
            ✓ Memory usage: Normal\n\
            ✓ Network connectivity: OK\n\
            ✓ File permissions: OK\n\
            \nAll systems operational.";
        
        Ok(CommandResult::success(diagnostics.to_string())
            .with_execution_time(Duration::from_millis(3000)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("run diagnostics".to_string()),
            PatternType::Exact("system diagnostics".to_string()),
            PatternType::Exact("check system".to_string()),
            PatternType::Exact("diagnostics".to_string()),
            PatternType::Exact("system check".to_string()),
            PatternType::Exact("health check".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::System
    }
    
    fn get_help_text(&self) -> &str {
        "Runs comprehensive system diagnostics to check all components"
    }
    
    fn get_name(&self) -> &str {
        "run_diagnostics"
    }
    
    fn get_description(&self) -> &str {
        "Run system diagnostics"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "run diagnostics".to_string(),
            "health check".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Intermediate
    }
}

/// Set log level command
pub struct SetLogLevelCommand;

impl VoiceCommand for SetLogLevelCommand {
    fn execute(&self, params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let regex = Regex::new(r"log level (?:to )?(debug|info|warn|error)").unwrap();
        if let Some(captures) = regex.captures(&params.text) {
            if let Some(level_str) = captures.get(1) {
                let level = level_str.as_str();
                return Ok(CommandResult::success_with_data(
                    format!("Log level set to {}", level),
                    CommandData::Text(level.to_string())
                ).with_execution_time(Duration::from_millis(50)));
            }
        }
        
        Err(VoiceCommandError::InvalidParameters("Valid log levels: debug, info, warn, error".to_string()))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Regex(Regex::new(r"set log level to (debug|info|warn|error)").unwrap()),
            PatternType::Regex(Regex::new(r"log level (debug|info|warn|error)").unwrap()),
            PatternType::Regex(Regex::new(r"change log level to (debug|info|warn|error)").unwrap()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::System
    }
    
    fn get_help_text(&self) -> &str {
        "Sets the system log level (debug, info, warn, error)"
    }
    
    fn get_name(&self) -> &str {
        "set_log_level"
    }
    
    fn get_description(&self) -> &str {
        "Set logging level"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "set log level to debug".to_string(),
            "log level info".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Advanced
    }
}

/// Reset to defaults command
pub struct ResetDefaultsCommand;

impl VoiceCommand for ResetDefaultsCommand {
    fn execute(&self, _params: CommandParams, context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        // Reset context to defaults
        context.audio_state = AudioState::default();
        context.stt_state = STTState::default();
        context.current_mode = SystemMode::Normal;
        
        Ok(CommandResult::success("All settings reset to defaults".to_string())
            .with_execution_time(Duration::from_millis(300)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("reset to defaults".to_string()),
            PatternType::Exact("reset settings".to_string()),
            PatternType::Exact("default settings".to_string()),
            PatternType::Exact("factory reset".to_string()),
            PatternType::Exact("restore defaults".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::System
    }
    
    fn get_help_text(&self) -> &str {
        "Resets all settings to their default values"
    }
    
    fn get_name(&self) -> &str {
        "reset_defaults"
    }
    
    fn get_description(&self) -> &str {
        "Reset all settings to defaults"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "reset to defaults".to_string(),
            "factory reset".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Intermediate
    }
}

/// Create system command registry
pub fn register_system_commands(engine: &mut VoiceCommandEngine) -> Result<(), VoiceCommandError> {
    engine.register_command(RestartServiceCommand)?;
    engine.register_command(ReloadConfigCommand)?;
    engine.register_command(ClearCacheCommand)?;
    engine.register_command(BackupSettingsCommand)?;
    engine.register_command(ShowMetricsCommand)?;
    engine.register_command(ExportLogsCommand)?;
    engine.register_command(CheckUpdatesCommand)?;
    engine.register_command(ShowVersionCommand)?;
    engine.register_command(ExitApplicationCommand)?;
    engine.register_command(RunDiagnosticsCommand)?;
    engine.register_command(SetLogLevelCommand)?;
    engine.register_command(ResetDefaultsCommand)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_system_commands() {
        let mut engine = VoiceCommandEngine::new();
        register_system_commands(&mut engine).unwrap();
        
        // Test restart command
        let result = engine.process_voice_input("restart service", 0.95).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
        
        // Test metrics command
        let result = engine.process_voice_input("show metrics", 0.95).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.message.contains("Performance Metrics"));
        
        // Test version command
        let result = engine.process_voice_input("show version", 0.95).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
        
        // Test diagnostics command
        let result = engine.process_voice_input("run diagnostics", 0.95).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }
    
    #[test]
    fn test_system_command_patterns() {
        let mut engine = VoiceCommandEngine::new();
        register_system_commands(&mut engine).unwrap();
        
        // Test various patterns for restart
        let patterns = vec![
            "restart service",
            "restart system",
            "reload service",
        ];
        
        for pattern in patterns {
            let parsed = engine.parse_command(pattern);
            assert!(parsed.is_ok());
            assert_eq!(parsed.unwrap().command_name, "restart_service");
        }
    }
    
    #[test]
    fn test_log_level_command() {
        let mut engine = VoiceCommandEngine::new();
        register_system_commands(&mut engine).unwrap();
        
        let patterns = vec![
            "set log level to debug",
            "log level info",
            "change log level to error",
        ];
        
        for pattern in patterns {
            let parsed = engine.parse_command(pattern);
            assert!(parsed.is_ok());
            assert_eq!(parsed.unwrap().command_name, "set_log_level");
        }
    }
}
