//! Basic voice commands for core STT Clippy functionality.
//! 
//! This module implements the essential voice commands that correspond to the
//! existing basic functionality in stt_to_clipboard.rs.

use std::time::Duration;
use regex::Regex;

use super::*;


/// Enable VAD (Voice Activity Detection) command
pub struct EnableVADCommand;

impl VoiceCommand for EnableVADCommand {
    fn execute(&self, _params: CommandParams, context: &mut SystemContext, _services: Option<&ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        context.audio_state.vad_enabled = true;
        
        Ok(CommandResult::success("Voice Activity Detection enabled".to_string())
            .with_execution_time(Duration::from_millis(10)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("enable vad".to_string()),
            PatternType::Exact("enable the vad".to_string()),
            PatternType::Exact("turn on vad".to_string()),
            PatternType::Exact("turn vad on".to_string()),
            PatternType::Exact("start vad".to_string()),
            PatternType::Exact("voice on".to_string()),
            PatternType::Exact("vad on".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Enables Voice Activity Detection for automatic speech detection"
    }
    
    fn get_name(&self) -> &str {
        "enable_vad"
    }
    
    fn get_description(&self) -> &str {
        "Enable Voice Activity Detection"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "enable vad".to_string(),
            "turn on vad".to_string(),
            "voice on".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["disable_vad".to_string(), "adjust_sensitivity".to_string()]
    }
}

/// Disable VAD command
pub struct DisableVADCommand;

impl VoiceCommand for DisableVADCommand {
    fn execute(&self, _params: CommandParams, context: &mut SystemContext, _services: Option<&ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        context.audio_state.vad_enabled = false;
        
        Ok(CommandResult::success("Voice Activity Detection disabled".to_string())
            .with_execution_time(Duration::from_millis(10)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("disable vad".to_string()),
            PatternType::Exact("disable the vad".to_string()),
            PatternType::Exact("turn off vad".to_string()),
            PatternType::Exact("turn vad off".to_string()),
            PatternType::Exact("stop vad".to_string()),
            PatternType::Exact("voice off".to_string()),
            PatternType::Exact("vad off".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Disables Voice Activity Detection"
    }
    
    fn get_name(&self) -> &str {
        "disable_vad"
    }
    
    fn get_description(&self) -> &str {
        "Disable Voice Activity Detection"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "disable vad".to_string(),
            "turn off vad".to_string(),
            "voice off".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["enable_vad".to_string()]
    }
}

/// Increase sensitivity command
pub struct IncreaseSensitivityCommand;

impl VoiceCommand for IncreaseSensitivityCommand {
    fn execute(&self, _params: CommandParams, context: &mut SystemContext, _services: Option<&ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        let old_sensitivity = context.audio_state.sensitivity;
        context.audio_state.sensitivity = (old_sensitivity + 0.05).min(1.0);
        
        Ok(CommandResult::success_with_data(
            format!("Sensitivity increased from {:.2} to {:.2}", old_sensitivity, context.audio_state.sensitivity),
            CommandData::Number(context.audio_state.sensitivity as f64)
        ).with_execution_time(Duration::from_millis(5)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("increase sensitivity".to_string()),
            PatternType::Exact("raise sensitivity".to_string()),
            PatternType::Exact("more sensitive".to_string()),
            PatternType::Exact("turn up sensitivity".to_string()),
            PatternType::Exact("higher sensitivity".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Increases VAD sensitivity for detecting quieter speech"
    }
    
    fn get_name(&self) -> &str {
        "increase_sensitivity"
    }
    
    fn get_description(&self) -> &str {
        "Increase VAD sensitivity"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "increase sensitivity".to_string(),
            "more sensitive".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["decrease_sensitivity".to_string(), "set_sensitivity".to_string()]
    }
}

/// Decrease sensitivity command
pub struct DecreaseSensitivityCommand;

impl VoiceCommand for DecreaseSensitivityCommand {
    fn execute(&self, _params: CommandParams, context: &mut SystemContext, _services: Option<&ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        let old_sensitivity = context.audio_state.sensitivity;
        context.audio_state.sensitivity = (old_sensitivity - 0.05).max(0.0);
        
        Ok(CommandResult::success_with_data(
            format!("Sensitivity decreased from {:.2} to {:.2}", old_sensitivity, context.audio_state.sensitivity),
            CommandData::Number(context.audio_state.sensitivity as f64)
        ).with_execution_time(Duration::from_millis(5)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("decrease sensitivity".to_string()),
            PatternType::Exact("lower sensitivity".to_string()),
            PatternType::Exact("less sensitive".to_string()),
            PatternType::Exact("turn down sensitivity".to_string()),
            PatternType::Exact("reduce sensitivity".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Decreases VAD sensitivity to reduce false positives"
    }
    
    fn get_name(&self) -> &str {
        "decrease_sensitivity"
    }
    
    fn get_description(&self) -> &str {
        "Decrease VAD sensitivity"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "decrease sensitivity".to_string(),
            "less sensitive".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["increase_sensitivity".to_string(), "set_sensitivity".to_string()]
    }
}

/// Toggle instant output command
pub struct ToggleInstantOutputCommand;

impl VoiceCommand for ToggleInstantOutputCommand {
    fn execute(&self, _params: CommandParams, context: &mut SystemContext, _services: Option<&ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        context.stt_state.instant_output = !context.stt_state.instant_output;
        
        let mode = if context.stt_state.instant_output { "enabled" } else { "disabled" };
        
        Ok(CommandResult::success_with_data(
            format!("Instant output {}", mode),
            CommandData::Boolean(context.stt_state.instant_output)
        ).with_execution_time(Duration::from_millis(5)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("toggle instant".to_string()),
            PatternType::Exact("toggle instant output".to_string()),
            PatternType::Exact("toggle paste mode".to_string()),
            PatternType::Exact("paste mode".to_string()),
            PatternType::Exact("switch output mode".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::STT
    }
    
    fn get_help_text(&self) -> &str {
        "Toggles between clipboard and instant paste output modes"
    }
    
    fn get_name(&self) -> &str {
        "toggle_instant_output"
    }
    
    fn get_description(&self) -> &str {
        "Toggle instant output mode"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "toggle instant output".to_string(),
            "paste mode".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["enable_instant_output".to_string(), "disable_instant_output".to_string()]
    }
}

/// Enable narration command
pub struct EnableNarrationCommand;

impl VoiceCommand for EnableNarrationCommand {
    fn execute(&self, _params: CommandParams, context: &mut SystemContext, _services: Option<&ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        context.current_mode = SystemMode::Narration;
        
        Ok(CommandResult::success("Continuous narration mode enabled".to_string())
            .with_execution_time(Duration::from_millis(10)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("enable narration".to_string()),
            PatternType::Exact("enter narration mode".to_string()),
            PatternType::Exact("start narration".to_string()),
            PatternType::Exact("dictation on".to_string()),
            PatternType::Exact("start dictation".to_string()),
            PatternType::Exact("continuous mode".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::STT
    }
    
    fn get_help_text(&self) -> &str {
        "Enables continuous narration mode for live dictation"
    }
    
    fn get_name(&self) -> &str {
        "enable_narration"
    }
    
    fn get_description(&self) -> &str {
        "Enable continuous narration mode"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "enable narration".to_string(),
            "start dictation".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["disable_narration".to_string()]
    }
}

/// Disable narration command
pub struct DisableNarrationCommand;

impl VoiceCommand for DisableNarrationCommand {
    fn execute(&self, _params: CommandParams, context: &mut SystemContext, _services: Option<&ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        context.current_mode = SystemMode::Normal;
        
        Ok(CommandResult::success("Continuous narration mode disabled".to_string())
            .with_execution_time(Duration::from_millis(10)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("disable narration".to_string()),
            PatternType::Exact("exit narration mode".to_string()),
            PatternType::Exact("stop narration".to_string()),
            PatternType::Exact("dictation off".to_string()),
            PatternType::Exact("stop dictation".to_string()),
            PatternType::Exact("normal mode".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::STT
    }
    
    fn get_help_text(&self) -> &str {
        "Disables continuous narration mode and returns to normal operation"
    }
    
    fn get_name(&self) -> &str {
        "disable_narration"
    }
    
    fn get_description(&self) -> &str {
        "Disable continuous narration mode"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "disable narration".to_string(),
            "stop dictation".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["enable_narration".to_string()]
    }
}

/// Set sensitivity command with specific value
pub struct SetSensitivityCommand;

impl VoiceCommand for SetSensitivityCommand {
    fn execute(&self, params: CommandParams, context: &mut SystemContext, _services: Option<&ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        // Extract number from command text
        let regex = Regex::new(r"sensitivity to ([\d.]+)").unwrap();
        if let Some(captures) = regex.captures(&params.text) {
            if let Some(value_str) = captures.get(1) {
                if let Ok(value) = value_str.as_str().parse::<f32>() {
                    let clamped_value = value.clamp(0.0, 1.0);
                    let old_sensitivity = context.audio_state.sensitivity;
                    context.audio_state.sensitivity = clamped_value;
                    
                    return Ok(CommandResult::success_with_data(
                        format!("Sensitivity set from {:.2} to {:.2}", old_sensitivity, clamped_value),
                        CommandData::Number(clamped_value as f64)
                    ).with_execution_time(Duration::from_millis(5)));
                }
            }
        }
        
        Err(VoiceCommandError::InvalidParameters("Could not parse sensitivity value".to_string()))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Regex(Regex::new(r"set sensitivity to ([\d.]+)").unwrap()),
            PatternType::Regex(Regex::new(r"sensitivity ([\d.]+)").unwrap()),
            PatternType::Regex(Regex::new(r"adjust sensitivity to ([\d.]+)").unwrap()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Sets VAD sensitivity to a specific value between 0.0 and 1.0"
    }
    
    fn get_name(&self) -> &str {
        "set_sensitivity"
    }
    
    fn get_description(&self) -> &str {
        "Set VAD sensitivity to specific value"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "set sensitivity to 0.7".to_string(),
            "sensitivity 0.5".to_string(),
            "adjust sensitivity to 0.3".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["increase_sensitivity".to_string(), "decrease_sensitivity".to_string()]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Intermediate
    }
}

/// Show system status command
pub struct ShowStatusCommand;

impl VoiceCommand for ShowStatusCommand {
    fn execute(&self, _params: CommandParams, context: &mut SystemContext, _services: Option<&ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        let mut status_data = HashMap::new();
        status_data.insert("mode".to_string(), serde_json::Value::String(format!("{:?}", context.current_mode)));
        status_data.insert("vad_enabled".to_string(), serde_json::Value::Bool(context.audio_state.vad_enabled));
        status_data.insert("sensitivity".to_string(), serde_json::Value::Number(
            serde_json::Number::from_f64(context.audio_state.sensitivity as f64).unwrap()
        ));
        status_data.insert("instant_output".to_string(), serde_json::Value::Bool(context.stt_state.instant_output));
        status_data.insert("current_model".to_string(), serde_json::Value::String(context.stt_state.current_model.clone()));
        
        let status_message = format!(
            "System Status:\n• Mode: {:?}\n• VAD: {}\n• Sensitivity: {:.2}\n• Output: {}\n• Model: {}",
            context.current_mode,
            if context.audio_state.vad_enabled { "enabled" } else { "disabled" },
            context.audio_state.sensitivity,
            if context.stt_state.instant_output { "instant" } else { "clipboard" },
            context.stt_state.current_model
        );
        
        Ok(CommandResult::success_with_data(
            status_message,
            CommandData::Object(status_data)
        ).with_execution_time(Duration::from_millis(20)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("show status".to_string()),
            PatternType::Exact("system status".to_string()),
            PatternType::Exact("status".to_string()),
            PatternType::Exact("show system status".to_string()),
            PatternType::Exact("what's the status".to_string()),
            PatternType::Exact("current status".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::System
    }
    
    fn get_help_text(&self) -> &str {
        "Displays current system status including VAD, sensitivity, and output mode"
    }
    
    fn get_name(&self) -> &str {
        "show_status"
    }
    
    fn get_description(&self) -> &str {
        "Show current system status"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "show status".to_string(),
            "system status".to_string(),
            "what's the status".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["show_help".to_string(), "show_metrics".to_string()]
    }
}

/// Help command
pub struct ShowHelpCommand;

impl VoiceCommand for ShowHelpCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext, _services: Option<&ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
        let help_text = "Available Voice Commands:\n\
            • 'enable vad' / 'disable vad' - Control voice detection\n\
            • 'increase sensitivity' / 'decrease sensitivity' - Adjust sensitivity\n\
            • 'set sensitivity to X' - Set specific sensitivity (0.0-1.0)\n\
            • 'toggle instant output' - Switch output modes\n\
            • 'enable narration' / 'disable narration' - Continuous dictation\n\
            • 'show status' - Display system status\n\
            • 'help' - Show this help message\n\
            \n\
            Say 'help [command name]' for detailed help on specific commands.";
        
        Ok(CommandResult::success(help_text.to_string())
            .with_execution_time(Duration::from_millis(5)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("help".to_string()),
            PatternType::Exact("show help".to_string()),
            PatternType::Exact("what can I say".to_string()),
            PatternType::Exact("available commands".to_string()),
            PatternType::Exact("list commands".to_string()),
            PatternType::Exact("show commands".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Help
    }
    
    fn get_help_text(&self) -> &str {
        "Shows available voice commands and their usage"
    }
    
    fn get_name(&self) -> &str {
        "show_help"
    }
    
    fn get_description(&self) -> &str {
        "Show help and available commands"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "help".to_string(),
            "what can I say".to_string(),
            "show commands".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["show_status".to_string()]
    }
}

/// Create a basic command registry with essential commands
pub fn create_basic_command_registry() -> VoiceCommandEngine {
    let mut engine = VoiceCommandEngine::new();
    
    // Register all basic commands
    engine.register_command(EnableVADCommand).unwrap();
    engine.register_command(DisableVADCommand).unwrap();
    engine.register_command(IncreaseSensitivityCommand).unwrap();
    engine.register_command(DecreaseSensitivityCommand).unwrap();
    engine.register_command(SetSensitivityCommand).unwrap();
    engine.register_command(ToggleInstantOutputCommand).unwrap();
    engine.register_command(EnableNarrationCommand).unwrap();
    engine.register_command(DisableNarrationCommand).unwrap();
    engine.register_command(ShowStatusCommand).unwrap();
    engine.register_command(ShowHelpCommand).unwrap();
    
    engine
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_commands() {
        let mut engine = create_basic_command_registry();
        
        // Test VAD commands
        let result = engine.process_voice_input("enable vad", 0.95).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
        
        let result = engine.process_voice_input("disable vad", 0.95).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
        
        // Test sensitivity commands
        let result = engine.process_voice_input("increase sensitivity", 0.95).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
        
        let result = engine.process_voice_input("set sensitivity to 0.7", 0.95).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
        
        // Test help command
        let result = engine.process_voice_input("help", 0.95).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.message.contains("Available Voice Commands"));
    }
    
    #[test]
    fn test_command_patterns() {
        let mut engine = create_basic_command_registry();
        
        // Test various patterns for the same command
        let patterns = vec![
            "enable vad",
            "turn on vad",
            "voice on",
            "vad on",
        ];
        
        for pattern in patterns {
            let parsed = engine.parse_command(pattern);
            assert!(parsed.is_ok());
            assert_eq!(parsed.unwrap().command_name, "enable_vad");
        }
    }
    
    #[test]
    fn test_command_suggestions() {
        let engine = create_basic_command_registry();
        
        let suggestions = engine.get_suggestions("ena");
        assert!(!suggestions.is_empty());
        
        let suggestions = engine.get_suggestions("help");
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.command_name == "show_help"));
    }
}
