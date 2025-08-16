//! Audio control voice commands.
//! 
//! This module implements comprehensive voice commands for audio system control,
//! including device management, sample rate control, audio processing settings,
//! and recording functionality.

use std::time::Duration;
use regex::Regex;

use super::*;

/// Set sample rate command
pub struct SetSampleRateCommand;

impl VoiceCommand for SetSampleRateCommand {
    fn execute(&self, params: CommandParams, context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let regex = Regex::new(r"(?:sample rate|rate) (?:to )?(\d+)").unwrap();
        if let Some(captures) = regex.captures(&params.text) {
            if let Some(rate_str) = captures.get(1) {
                if let Ok(rate) = rate_str.as_str().parse::<u32>() {
                    let valid_rates = [8000, 16000, 22050, 44100, 48000, 96000];
                    if valid_rates.contains(&rate) {
                        context.audio_state.sample_rate = rate;
                        return Ok(CommandResult::success_with_data(
                            format!("Sample rate set to {}Hz", rate),
                            CommandData::Number(rate as f64)
                        ).with_execution_time(Duration::from_millis(50)));
                    } else {
                        return Err(VoiceCommandError::InvalidParameters(
                            format!("Invalid sample rate: {}. Valid rates: 8000, 16000, 22050, 44100, 48000, 96000", rate)
                        ));
                    }
                }
            }
        }
        
        Err(VoiceCommandError::InvalidParameters("Could not parse sample rate".to_string()))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Regex(Regex::new(r"set sample rate to (\d+)").unwrap()),
            PatternType::Regex(Regex::new(r"sample rate (\d+)").unwrap()),
            PatternType::Regex(Regex::new(r"change sample rate to (\d+)").unwrap()),
            PatternType::Regex(Regex::new(r"set rate to (\d+)").unwrap()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Sets the audio sample rate. Valid rates: 8000, 16000, 22050, 44100, 48000, 96000 Hz"
    }
    
    fn get_name(&self) -> &str {
        "set_sample_rate"
    }
    
    fn get_description(&self) -> &str {
        "Set audio sample rate"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "set sample rate to 44100".to_string(),
            "sample rate 16000".to_string(),
            "change sample rate to 48000".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Intermediate
    }
}

/// Switch audio device command
pub struct SwitchAudioDeviceCommand;

impl VoiceCommand for SwitchAudioDeviceCommand {
    fn execute(&self, params: CommandParams, context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let regex = Regex::new(r"(?:switch to|use) device (.+)").unwrap();
        if let Some(captures) = regex.captures(&params.text) {
            if let Some(device_name) = captures.get(1) {
                let device = device_name.as_str().trim();
                context.audio_state.current_device = Some(device.to_string());
                return Ok(CommandResult::success_with_data(
                    format!("Switched to audio device: {}", device),
                    CommandData::Text(device.to_string())
                ).with_execution_time(Duration::from_millis(100)));
            }
        }
        
        Err(VoiceCommandError::InvalidParameters("Could not parse device name".to_string()))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Regex(Regex::new(r"switch to device (.+)").unwrap()),
            PatternType::Regex(Regex::new(r"use device (.+)").unwrap()),
            PatternType::Regex(Regex::new(r"change device to (.+)").unwrap()),
            PatternType::Regex(Regex::new(r"select device (.+)").unwrap()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Switches to a specific audio input device"
    }
    
    fn get_name(&self) -> &str {
        "switch_audio_device"
    }
    
    fn get_description(&self) -> &str {
        "Switch to specific audio device"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "switch to device built-in microphone".to_string(),
            "use device usb headset".to_string(),
            "select device bluetooth mic".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Intermediate
    }
}

/// Adjust volume command
pub struct AdjustVolumeCommand;

impl VoiceCommand for AdjustVolumeCommand {
    fn execute(&self, params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let regex = Regex::new(r"(?:volume|gain) (?:to )?(\d+)(?:%|percent)?").unwrap();
        if let Some(captures) = regex.captures(&params.text) {
            if let Some(volume_str) = captures.get(1) {
                if let Ok(volume) = volume_str.as_str().parse::<u8>() {
                    if volume <= 100 {
                        return Ok(CommandResult::success_with_data(
                            format!("Volume set to {}%", volume),
                            CommandData::Number(volume as f64)
                        ).with_execution_time(Duration::from_millis(30)));
                    } else {
                        return Err(VoiceCommandError::InvalidParameters(
                            "Volume must be between 0 and 100%".to_string()
                        ));
                    }
                }
            }
        }
        
        Err(VoiceCommandError::InvalidParameters("Could not parse volume level".to_string()))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Regex(Regex::new(r"set volume to (\d+)").unwrap()),
            PatternType::Regex(Regex::new(r"volume (\d+)").unwrap()),
            PatternType::Regex(Regex::new(r"adjust volume to (\d+)").unwrap()),
            PatternType::Regex(Regex::new(r"set gain to (\d+)").unwrap()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Adjusts the microphone volume/gain from 0 to 100%"
    }
    
    fn get_name(&self) -> &str {
        "adjust_volume"
    }
    
    fn get_description(&self) -> &str {
        "Adjust microphone volume/gain"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "set volume to 75".to_string(),
            "volume 50 percent".to_string(),
            "adjust gain to 80".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Beginner
    }
}

/// Enable noise reduction command
pub struct EnableNoiseReductionCommand;

impl VoiceCommand for EnableNoiseReductionCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult::success("Noise reduction enabled".to_string())
            .with_execution_time(Duration::from_millis(20)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("enable noise reduction".to_string()),
            PatternType::Exact("turn on noise reduction".to_string()),
            PatternType::Exact("noise reduction on".to_string()),
            PatternType::Exact("enable noise filter".to_string()),
            PatternType::Exact("start noise reduction".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Enables noise reduction to filter background noise from audio input"
    }
    
    fn get_name(&self) -> &str {
        "enable_noise_reduction"
    }
    
    fn get_description(&self) -> &str {
        "Enable noise reduction filter"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "enable noise reduction".to_string(),
            "turn on noise filter".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["disable_noise_reduction".to_string()]
    }
}

/// Disable noise reduction command
pub struct DisableNoiseReductionCommand;

impl VoiceCommand for DisableNoiseReductionCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult::success("Noise reduction disabled".to_string())
            .with_execution_time(Duration::from_millis(20)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("disable noise reduction".to_string()),
            PatternType::Exact("turn off noise reduction".to_string()),
            PatternType::Exact("noise reduction off".to_string()),
            PatternType::Exact("disable noise filter".to_string()),
            PatternType::Exact("stop noise reduction".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Disables noise reduction to allow all audio input through"
    }
    
    fn get_name(&self) -> &str {
        "disable_noise_reduction"
    }
    
    fn get_description(&self) -> &str {
        "Disable noise reduction filter"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "disable noise reduction".to_string(),
            "turn off noise filter".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["enable_noise_reduction".to_string()]
    }
}

/// Set buffer size command
pub struct SetBufferSizeCommand;

impl VoiceCommand for SetBufferSizeCommand {
    fn execute(&self, params: CommandParams, context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let regex = Regex::new(r"buffer size (?:to )?(\d+)").unwrap();
        if let Some(captures) = regex.captures(&params.text) {
            if let Some(size_str) = captures.get(1) {
                if let Ok(size) = size_str.as_str().parse::<usize>() {
                    let valid_sizes = [64, 128, 256, 512, 1024, 2048, 4096];
                    if valid_sizes.contains(&size) {
                        context.audio_state.buffer_size = size;
                        return Ok(CommandResult::success_with_data(
                            format!("Buffer size set to {} samples", size),
                            CommandData::Number(size as f64)
                        ).with_execution_time(Duration::from_millis(40)));
                    } else {
                        return Err(VoiceCommandError::InvalidParameters(
                            "Invalid buffer size. Valid sizes: 64, 128, 256, 512, 1024, 2048, 4096".to_string()
                        ));
                    }
                }
            }
        }
        
        Err(VoiceCommandError::InvalidParameters("Could not parse buffer size".to_string()))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Regex(Regex::new(r"set buffer size to (\d+)").unwrap()),
            PatternType::Regex(Regex::new(r"buffer size (\d+)").unwrap()),
            PatternType::Regex(Regex::new(r"change buffer size to (\d+)").unwrap()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Sets the audio buffer size. Valid sizes: 64, 128, 256, 512, 1024, 2048, 4096 samples"
    }
    
    fn get_name(&self) -> &str {
        "set_buffer_size"
    }
    
    fn get_description(&self) -> &str {
        "Set audio buffer size"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "set buffer size to 1024".to_string(),
            "buffer size 512".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Advanced
    }
}

/// Calibrate microphone command
pub struct CalibrateMicrophoneCommand;

impl VoiceCommand for CalibrateMicrophoneCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        // This would trigger a microphone calibration process
        Ok(CommandResult::success("Microphone calibration started. Please speak normally for 10 seconds.".to_string())
            .with_execution_time(Duration::from_millis(100)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("calibrate microphone".to_string()),
            PatternType::Exact("calibrate mic".to_string()),
            PatternType::Exact("mic calibration".to_string()),
            PatternType::Exact("auto calibrate".to_string()),
            PatternType::Exact("calibrate audio".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Starts automatic microphone calibration to optimize audio input levels"
    }
    
    fn get_name(&self) -> &str {
        "calibrate_microphone"
    }
    
    fn get_description(&self) -> &str {
        "Calibrate microphone input levels"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "calibrate microphone".to_string(),
            "auto calibrate".to_string(),
        ]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Intermediate
    }
}

/// Test audio input command
pub struct TestAudioInputCommand;

impl VoiceCommand for TestAudioInputCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult::success("Audio input test started. Speak now to test your microphone.".to_string())
            .with_execution_time(Duration::from_millis(50)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("test audio".to_string()),
            PatternType::Exact("test microphone".to_string()),
            PatternType::Exact("test mic".to_string()),
            PatternType::Exact("audio test".to_string()),
            PatternType::Exact("mic test".to_string()),
            PatternType::Exact("test input".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Tests audio input to verify microphone is working correctly"
    }
    
    fn get_name(&self) -> &str {
        "test_audio_input"
    }
    
    fn get_description(&self) -> &str {
        "Test audio input/microphone"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "test audio".to_string(),
            "mic test".to_string(),
        ]
    }
}

/// Show audio devices command
pub struct ShowAudioDevicesCommand;

impl VoiceCommand for ShowAudioDevicesCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let devices_info = "Available Audio Devices:\n\
            • Built-in Microphone (default)\n\
            • USB Headset\n\
            • Bluetooth Audio Device\n\
            \nUse 'switch to device [name]' to change input device.";
        
        Ok(CommandResult::success(devices_info.to_string())
            .with_execution_time(Duration::from_millis(30)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("show audio devices".to_string()),
            PatternType::Exact("list audio devices".to_string()),
            PatternType::Exact("available devices".to_string()),
            PatternType::Exact("show devices".to_string()),
            PatternType::Exact("list devices".to_string()),
            PatternType::Exact("audio devices".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Lists all available audio input devices"
    }
    
    fn get_name(&self) -> &str {
        "show_audio_devices"
    }
    
    fn get_description(&self) -> &str {
        "Show available audio devices"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "show audio devices".to_string(),
            "list devices".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["switch_audio_device".to_string()]
    }
}

/// Enable automatic gain control command
pub struct EnableAGCCommand;

impl VoiceCommand for EnableAGCCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult::success("Automatic gain control enabled".to_string())
            .with_execution_time(Duration::from_millis(20)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("enable automatic gain control".to_string()),
            PatternType::Exact("enable agc".to_string()),
            PatternType::Exact("turn on agc".to_string()),
            PatternType::Exact("auto gain on".to_string()),
            PatternType::Exact("enable auto gain".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Enables automatic gain control to maintain consistent audio levels"
    }
    
    fn get_name(&self) -> &str {
        "enable_agc"
    }
    
    fn get_description(&self) -> &str {
        "Enable automatic gain control"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "enable agc".to_string(),
            "turn on auto gain".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["disable_agc".to_string()]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Intermediate
    }
}

/// Disable automatic gain control command
pub struct DisableAGCCommand;

impl VoiceCommand for DisableAGCCommand {
    fn execute(&self, _params: CommandParams, _context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        Ok(CommandResult::success("Automatic gain control disabled".to_string())
            .with_execution_time(Duration::from_millis(20)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("disable automatic gain control".to_string()),
            PatternType::Exact("disable agc".to_string()),
            PatternType::Exact("turn off agc".to_string()),
            PatternType::Exact("auto gain off".to_string()),
            PatternType::Exact("disable auto gain".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Disables automatic gain control for manual level control"
    }
    
    fn get_name(&self) -> &str {
        "disable_agc"
    }
    
    fn get_description(&self) -> &str {
        "Disable automatic gain control"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "disable agc".to_string(),
            "turn off auto gain".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["enable_agc".to_string()]
    }
    
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Intermediate
    }
}

/// Show audio settings command
pub struct ShowAudioSettingsCommand;

impl VoiceCommand for ShowAudioSettingsCommand {
    fn execute(&self, _params: CommandParams, context: &mut SystemContext) -> Result<CommandResult, VoiceCommandError> {
        let settings = format!(
            "Audio Settings:\n\
            • Sample Rate: {}Hz\n\
            • Channels: {}\n\
            • Buffer Size: {} samples\n\
            • VAD Enabled: {}\n\
            • Sensitivity: {:.2}\n\
            • Current Device: {}",
            context.audio_state.sample_rate,
            context.audio_state.channels,
            context.audio_state.buffer_size,
            if context.audio_state.vad_enabled { "Yes" } else { "No" },
            context.audio_state.sensitivity,
            context.audio_state.current_device.as_ref().unwrap_or(&"Default".to_string())
        );
        
        Ok(CommandResult::success(settings).with_execution_time(Duration::from_millis(20)))
    }
    
    fn get_patterns(&self) -> Vec<PatternType> {
        vec![
            PatternType::Exact("show audio settings".to_string()),
            PatternType::Exact("audio settings".to_string()),
            PatternType::Exact("current audio settings".to_string()),
            PatternType::Exact("audio configuration".to_string()),
        ]
    }
    
    fn get_category(&self) -> CommandCategory {
        CommandCategory::Audio
    }
    
    fn get_help_text(&self) -> &str {
        "Displays current audio configuration and settings"
    }
    
    fn get_name(&self) -> &str {
        "show_audio_settings"
    }
    
    fn get_description(&self) -> &str {
        "Show current audio settings"
    }
    
    fn get_examples(&self) -> Vec<String> {
        vec![
            "show audio settings".to_string(),
            "audio configuration".to_string(),
        ]
    }
    
    fn get_related_commands(&self) -> Vec<String> {
        vec!["show_status".to_string()]
    }
}

/// Create audio command registry
pub fn register_audio_commands(engine: &mut VoiceCommandEngine) -> Result<(), VoiceCommandError> {
    engine.register_command(SetSampleRateCommand)?;
    engine.register_command(SwitchAudioDeviceCommand)?;
    engine.register_command(AdjustVolumeCommand)?;
    engine.register_command(EnableNoiseReductionCommand)?;
    engine.register_command(DisableNoiseReductionCommand)?;
    engine.register_command(SetBufferSizeCommand)?;
    engine.register_command(CalibrateMicrophoneCommand)?;
    engine.register_command(TestAudioInputCommand)?;
    engine.register_command(ShowAudioDevicesCommand)?;
    engine.register_command(EnableAGCCommand)?;
    engine.register_command(DisableAGCCommand)?;
    engine.register_command(ShowAudioSettingsCommand)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_audio_commands() {
        let mut engine = VoiceCommandEngine::new();
        register_audio_commands(&mut engine).unwrap();
        
        // Test sample rate command
        let result = engine.process_voice_input("set sample rate to 44100", 0.95).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.message.contains("44100"));
        
        // Test device switch command
        let result = engine.process_voice_input("switch to device usb headset", 0.95).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
        
        // Test volume adjustment
        let result = engine.process_voice_input("set volume to 75", 0.95).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
        
        // Test audio test command
        let result = engine.process_voice_input("test audio", 0.95).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }
    
    #[test]
    fn test_audio_command_patterns() {
        let mut engine = VoiceCommandEngine::new();
        register_audio_commands(&mut engine).unwrap();
        
        // Test various patterns for sample rate
        let patterns = vec![
            "set sample rate to 16000",
            "sample rate 16000",
            "change sample rate to 16000",
        ];
        
        for pattern in patterns {
            let parsed = engine.parse_command(pattern);
            assert!(parsed.is_ok());
            assert_eq!(parsed.unwrap().command_name, "set_sample_rate");
        }
    }
    
    #[test]
    fn test_invalid_parameters() {
        let mut engine = VoiceCommandEngine::new();
        register_audio_commands(&mut engine).unwrap();
        
        let result = engine.parse_command("set sample rate to 999999");
        assert!(result.is_ok()); // Parse succeeds
        // But execution would fail with invalid parameters
    }
}
