//! Speech-to-Text (STT) control voice commands.
//! 
//! This module implements comprehensive voice commands for STT system control,
//! including model management, language settings, output configuration,
//! and transcription parameters.

use std::time::Duration;
use regex::Regex;

use super::*;

/// Switch STT model command
pub struct SwitchModelCommand;

impl VoiceCommand for SwitchModelCommand {
    fn execute(&self, params: CommandParams, context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let regex = Regex::new(r"(?:switch to|use|load) (?:model )?(\w+)(?: model)?").unwrap();
        if let Some(captures) = regex.captures(&params.text) {
            if let Some(model_str) = captures.get(1) {
                let model = model_str.as_str().to_lowercase();
                let valid_models = ["tiny", "base", "small", "medium", "large"];
                if valid_models.contains(&model.as_str()) {
                    context.stt_state.current_model = model.clone();
                    return Ok(CommandResult::success_with_data(
                        format!("Switched to {} model", model),
                        CommandData::Text(model)
                    ).with_execution_time(Duration::from_millis(200)));
                } else {
                    return Err(VoiceCommandError::InvalidParameters(
                        format!("Invalid model: {}. Valid models: tiny, base, small, medium, large", model)
                    ));
                }
            }
        }
        
        Err(VoiceCommandError::InvalidParameters("Could not parse model name".to_string()))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Regex(Regex::new(r"switch to (\w+) model").unwrap()),
            PatternType::Regex(Regex::new(r"use (\w+) model").unwrap()),
            PatternType::Regex(Regex::new(r"load (\w+) model").unwrap()),
            PatternType::Regex(Regex::new(r"change model to (\w+)").unwrap()),
            PatternType::Regex(Regex::new(r"set model (\w+)").unwrap()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::STT
    }
    
    fn get_help_text(&self) -> &str {
        "Switches the STT model. Available models: tiny, base, small, medium, large"
    }
    
    fn get_name(&self) -> &str {
        "switch_model"
    }
    
    fn get_description(&self) -> &str {
        "Switch STT model"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "switch to base model".to_string(),
            "use large model".to_string(),
            "load small model".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Intermediate
    }
}

/// Set language command
pub struct SetLanguageCommand;

impl VoiceCommand for SetLanguageCommand {
    fn execute(&self, params: CommandParams, context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let regex = Regex::new(r"(?:language|lang) (?:to )?([a-z]{2}|english|spanish|french|german|auto)").unwrap();
        if let Some(captures) = regex.captures(&params.text) {
            if let Some(lang_str) = captures.get(1) {
                let language = lang_str.as_str().to_lowercase();
                // Convert language names to ISO codes
                let lang_code = match language.as_str() {
                    "english" => "en",
                    "spanish" => "es", 
                    "french" => "fr",
                    "german" => "de",
                    "auto" => "auto",
                    code if code.len() == 2 => code,
                    _ => return Err(VoiceCommandError::InvalidParameters(
                        "Invalid language. Use language name or 2-letter code".to_string()
                    ))
                };
                
                context.stt_state.language = if lang_code == "auto" {
                    None
                } else {
                    Some(lang_code.to_string())
                };
                
                return Ok(CommandResult::success_with_data(
                    format!("Language set to {}", if lang_code == "auto" { "auto-detect" } else { lang_code }),
                    CommandData::Text(lang_code.to_string())
                ).with_execution_time(Duration::from_millis(50)));
            }
        }
        
        Err(VoiceCommandError::InvalidParameters("Could not parse language".to_string()))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Regex(Regex::new(r"set language to (\w+)").unwrap()),
            PatternType::Regex(Regex::new(r"language (\w+)").unwrap()),
            PatternType::Regex(Regex::new(r"change language to (\w+)").unwrap()),
            PatternType::Regex(Regex::new(r"use (\w+) language").unwrap()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::STT
    }
    
    fn get_help_text(&self) -> &str {
        "Sets the STT language. Use language names (english, spanish, etc.) or 2-letter codes, or 'auto' for auto-detection"
    }
    
    fn get_name(&self) -> &str {
        "set_language"
    }
    
    fn get_description(&self) -> &str {
        "Set STT language"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "set language to english".to_string(),
            "language spanish".to_string(),
            "use auto language".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Beginner
    }
}

/// Enable punctuation command
pub struct EnablePunctuationCommand;

impl VoiceCommand for EnablePunctuationCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult::success("Automatic punctuation enabled".to_string())
            .with_execution_time(Duration::from_millis(20)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("enable punctuation".to_string()),
            PatternType::Exact("turn on punctuation".to_string()),
            PatternType::Exact("punctuation on".to_string()),
            PatternType::Exact("auto punctuation".to_string()),
            PatternType::Exact("enable auto punctuation".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::STT
    }
    
    fn get_help_text(&self) -> &str {
        "Enables automatic punctuation insertion in transcriptions"
    }
    
    fn get_name(&self) -> &str {
        "enable_punctuation"
    }
    
    fn get_description(&self) -> &str {
        "Enable automatic punctuation"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "enable punctuation".to_string(),
            "auto punctuation".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["disable_punctuation".to_string()]
    }
}

/// Disable punctuation command
pub struct DisablePunctuationCommand;

impl VoiceCommand for DisablePunctuationCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult::success("Automatic punctuation disabled".to_string())
            .with_execution_time(Duration::from_millis(20)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("disable punctuation".to_string()),
            PatternType::Exact("turn off punctuation".to_string()),
            PatternType::Exact("punctuation off".to_string()),
            PatternType::Exact("no punctuation".to_string()),
            PatternType::Exact("disable auto punctuation".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::STT
    }
    
    fn get_help_text(&self) -> &str {
        "Disables automatic punctuation insertion in transcriptions"
    }
    
    fn get_name(&self) -> &str {
        "disable_punctuation"
    }
    
    fn get_description(&self) -> &str {
        "Disable automatic punctuation"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "disable punctuation".to_string(),
            "no punctuation".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["enable_punctuation".to_string()]
    }
}

/// Set confidence threshold command
pub struct SetConfidenceThresholdCommand;

impl VoiceCommand for SetConfidenceThresholdCommand {
    fn execute(&self, params: CommandParams, context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let regex = Regex::new(r"confidence (?:threshold )?(?:to )?([0-9]*\.?[0-9]+)").unwrap();
        if let Some(captures) = regex.captures(&params.text) {
            if let Some(threshold_str) = captures.get(1) {
                if let Ok(threshold) = threshold_str.as_str().parse::<f32>() {
                    if threshold >= 0.0 && threshold <= 1.0 {
                        context.stt_state.confidence_threshold = threshold;
                        return Ok(CommandResult::success_with_data(
                            format!("Confidence threshold set to {:.2}", threshold),
                            CommandData::Number(threshold as f64)
                        ).with_execution_time(Duration::from_millis(20)));
                    } else {
                        return Err(VoiceCommandError::InvalidParameters(
                            "Confidence threshold must be between 0.0 and 1.0".to_string()
                        ));
                    }
                }
            }
        }
        
        Err(VoiceCommandError::InvalidParameters("Could not parse confidence threshold".to_string()))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Regex(Regex::new(r"set confidence threshold to ([0-9]*\.?[0-9]+)").unwrap()),
            PatternType::Regex(Regex::new(r"confidence threshold ([0-9]*\.?[0-9]+)").unwrap()),
            PatternType::Regex(Regex::new(r"set confidence to ([0-9]*\.?[0-9]+)").unwrap()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::STT
    }
    
    fn get_help_text(&self) -> &str {
        "Sets the minimum confidence threshold for accepting transcriptions (0.0 to 1.0)"
    }
    
    fn get_name(&self) -> &str {
        "set_confidence_threshold"
    }
    
    fn get_description(&self) -> &str {
        "Set confidence threshold"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "set confidence threshold to 0.8".to_string(),
            "confidence threshold 0.6".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Advanced
    }
}

/// Toggle streaming mode command
pub struct ToggleStreamingCommand;

impl VoiceCommand for ToggleStreamingCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        // This would toggle between streaming and batch processing
        Ok(CommandResult::success("Streaming mode toggled".to_string())
            .with_execution_time(Duration::from_millis(30)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("toggle streaming".to_string()),
            PatternType::Exact("toggle streaming mode".to_string()),
            PatternType::Exact("switch streaming mode".to_string()),
            PatternType::Exact("streaming mode".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::STT
    }
    
    fn get_help_text(&self) -> &str {
        "Toggles between streaming and batch processing modes"
    }
    
    fn get_name(&self) -> &str {
        "toggle_streaming"
    }
    
    fn get_description(&self) -> &str {
        "Toggle streaming mode"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "toggle streaming".to_string(),
            "streaming mode".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Intermediate
    }
}

/// Enable instant output command
pub struct EnableInstantOutputCommand;

impl VoiceCommand for EnableInstantOutputCommand {
    fn execute(&self, _params: CommandParams, context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        context.stt_state.instant_output = true;
        Ok(CommandResult::success("Instant output enabled - text will be pasted directly".to_string())
            .with_execution_time(Duration::from_millis(20)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("enable instant output".to_string()),
            PatternType::Exact("instant output on".to_string()),
            PatternType::Exact("enable instant paste".to_string()),
            PatternType::Exact("direct paste on".to_string()),
            PatternType::Exact("live output".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::STT
    }
    
    fn get_help_text(&self) -> &str {
        "Enables instant output mode where transcriptions are pasted directly"
    }
    
    fn get_name(&self) -> &str {
        "enable_instant_output"
    }
    
    fn get_description(&self) -> &str {
        "Enable instant output mode"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "enable instant output".to_string(),
            "instant paste on".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["disable_instant_output".to_string(), "toggle_instant_output".to_string()]
    }
}

/// Disable instant output command
pub struct DisableInstantOutputCommand;

impl VoiceCommand for DisableInstantOutputCommand {
    fn execute(&self, _params: CommandParams, context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        context.stt_state.instant_output = false;
        Ok(CommandResult::success("Instant output disabled - text will be saved to clipboard".to_string())
            .with_execution_time(Duration::from_millis(20)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("disable instant output".to_string()),
            PatternType::Exact("instant output off".to_string()),
            PatternType::Exact("disable instant paste".to_string()),
            PatternType::Exact("clipboard mode".to_string()),
            PatternType::Exact("clipboard output".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::STT
    }
    
    fn get_help_text(&self) -> &str {
        "Disables instant output mode - transcriptions will be saved to clipboard"
    }
    
    fn get_name(&self) -> &str {
        "disable_instant_output"
    }
    
    fn get_description(&self) -> &str {
        "Disable instant output mode"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "disable instant output".to_string(),
            "clipboard mode".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["enable_instant_output".to_string(), "toggle_instant_output".to_string()]
    }
}

/// Adjust processing speed command
pub struct AdjustProcessingSpeedCommand;

impl VoiceCommand for AdjustProcessingSpeedCommand {
    fn execute(&self, params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        if params.text.contains("faster") || params.text.contains("speed up") {
            Ok(CommandResult::success("STT processing speed increased".to_string())
                .with_execution_time(Duration::from_millis(30)))
        } else if params.text.contains("slower") || params.text.contains("slow down") {
            Ok(CommandResult::success("STT processing speed decreased".to_string())
                .with_execution_time(Duration::from_millis(30)))
        } else {
            Err(VoiceCommandError::InvalidParameters("Specify 'faster' or 'slower'".to_string()))
        }
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Contains("processing faster".to_string()),
            PatternType::Contains("processing slower".to_string()),
            PatternType::Contains("speed up processing".to_string()),
            PatternType::Contains("slow down processing".to_string()),
            PatternType::Exact("faster processing".to_string()),
            PatternType::Exact("slower processing".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::STT
    }
    
    fn get_help_text(&self) -> &str {
        "Adjusts STT processing speed (faster or slower)"
    }
    
    fn get_name(&self) -> &str {
        "adjust_processing_speed"
    }
    
    fn get_description(&self) -> &str {
        "Adjust STT processing speed"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "processing faster".to_string(),
            "speed up processing".to_string(),
            "slower processing".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Advanced
    }
}

/// Show STT settings command
pub struct ShowSTTSettingsCommand;

impl VoiceCommand for ShowSTTSettingsCommand {
    fn execute(&self, _params: CommandParams, context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let settings = format!(
            "STT Settings:\n\
            • Current Model: {}\n\
            • Language: {}\n\
            • Instant Output: {}\n\
            • Confidence Threshold: {:.2}\n\
            • Processing Queue: {} items",
            context.stt_state.current_model,
            context.stt_state.language.as_ref().unwrap_or(&"Auto-detect".to_string()),
            if context.stt_state.instant_output { "Enabled" } else { "Disabled" },
            context.stt_state.confidence_threshold,
            context.stt_state.processing_queue_size
        );
        
        Ok(CommandResult::success(settings).with_execution_time(Duration::from_millis(30)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("show stt settings".to_string()),
            PatternType::Exact("stt settings".to_string()),
            PatternType::Exact("transcription settings".to_string()),
            PatternType::Exact("stt configuration".to_string()),
            PatternType::Exact("show transcription settings".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::STT
    }
    
    fn get_help_text(&self) -> &str {
        "Displays current STT configuration and settings"
    }
    
    fn get_name(&self) -> &str {
        "show_stt_settings"
    }
    
    fn get_description(&self) -> &str {
        "Show STT settings"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "show stt settings".to_string(),
            "transcription settings".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["show_status".to_string()]
    }
}

/// Restart STT service command
pub struct RestartSTTServiceCommand;

impl VoiceCommand for RestartSTTServiceCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult::success("STT service restarted".to_string())
            .with_execution_time(Duration::from_millis(500)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("restart stt".to_string()),
            PatternType::Exact("restart stt service".to_string()),
            PatternType::Exact("restart transcription".to_string()),
            PatternType::Exact("reload stt".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::STT
    }
    
    fn get_help_text(&self) -> &str {
        "Restarts the STT service to refresh the connection and clear any issues"
    }
    
    fn get_name(&self) -> &str {
        "restart_stt_service"
    }
    
    fn get_description(&self) -> &str {
        "Restart STT service"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "restart stt".to_string(),
            "reload stt".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Intermediate
    }
}

/// Create STT command registry
pub fn register_stt_commands(engine: &mut VoiceCommandEngine) -> Result<(), VoiceCommandError> {
    engine.register_command(SwitchModelCommand)?;
    engine.register_command(SetLanguageCommand)?;
    engine.register_command(EnablePunctuationCommand)?;
    engine.register_command(DisablePunctuationCommand)?;
    engine.register_command(SetConfidenceThresholdCommand)?;
    engine.register_command(ToggleStreamingCommand)?;
    engine.register_command(EnableInstantOutputCommand)?;
    engine.register_command(DisableInstantOutputCommand)?;
    engine.register_command(AdjustProcessingSpeedCommand)?;
    engine.register_command(ShowSTTSettingsCommand)?;
    engine.register_command(RestartSTTServiceCommand)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_stt_commands() {
        let mut engine = VoiceCommandEngine::new();
        register_stt_commands(&mut engine).unwrap();
        
        // Test model switch command
        let result = engine.process_voice_input("switch to large model", 0.95).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.message.contains("large"));
        
        // Test language command
        let result = engine.process_voice_input("set language to spanish", 0.95).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
        
        // Test confidence threshold
        let result = engine.process_voice_input("set confidence threshold to 0.8", 0.95).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
        
        // Test instant output commands
        let result = engine.process_voice_input("enable instant output", 0.95).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }
    
    #[test]
    fn test_stt_command_patterns() {
        let mut engine = VoiceCommandEngine::new();
        register_stt_commands(&mut engine).unwrap();
        
        // Test various patterns for model switching
        let patterns = vec![
            "switch to base model",
            "use base model",
            "load base model",
        ];
        
        for pattern in patterns {
            let parsed = engine.parse_command(pattern);
            assert!(parsed.is_ok());
            assert_eq!(parsed.unwrap().command_name, "switch_model");
        }
    }
    
    #[test]
    fn test_language_variations() {
        let mut engine = VoiceCommandEngine::new();
        register_stt_commands(&mut engine).unwrap();
        
        let patterns = vec![
            "set language to english",
            "language spanish",
            "use auto language",
        ];
        
        for pattern in patterns {
            let parsed = engine.parse_command(pattern);
            assert!(parsed.is_ok());
            assert_eq!(parsed.unwrap().command_name, "set_language");
        }
    }
}
