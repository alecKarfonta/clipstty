//! Enhanced voice command framework for STT Clippy.
//! 
//! This module implements a comprehensive voice command system that supports:
//! - Trait-based command architecture for extensibility
//! - Context-aware command resolution
//! - Pattern matching and natural language processing
//! - Command suggestion and help system
//! - Performance metrics and validation

pub mod basic_commands;
pub mod context_manager;
pub mod audio_commands;
pub mod stt_commands;
pub mod system_commands;
pub mod comprehensive_registry;
pub mod suggestion_engine;
pub mod testing_framework;
pub mod audio_recording_commands;
pub mod transcript_management_commands;
pub mod session_tracking_commands;

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use regex::Regex;
use thiserror::Error;



/// Voice command execution errors
#[derive(Error, Debug)]
pub enum VoiceCommandError {
    #[error("Command not found: {0}")]
    CommandNotFound(String),
    #[error("Invalid command parameters: {0}")]
    InvalidParameters(String),
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Permission denied for command: {0}")]
    PermissionDenied(String),
    #[error("Command timeout")]
    Timeout,
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Context validation failed: {0}")]
    ContextValidationFailed(String),
}

/// Command execution result
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<CommandData>,
    pub execution_time: Duration,
    pub timestamp: DateTime<Utc>,
}

/// Command data payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommandData {
    Text(String),
    Number(f64),
    Boolean(bool),
    Object(HashMap<String, serde_json::Value>),
    Array(Vec<serde_json::Value>),
}

/// Command execution parameters
#[derive(Debug, Clone)]
pub struct CommandParams {
    pub text: String,
    pub confidence: f32,
    pub context: SystemContext,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
}

/// System context for command execution
#[derive(Debug, Clone)]
pub struct SystemContext {
    pub current_mode: SystemMode,
    pub audio_state: AudioState,
    pub stt_state: STTState,
    pub recent_commands: VecDeque<ExecutedCommand>,
    pub session_data: HashMap<String, serde_json::Value>,
    pub environment: EnvironmentContext,
}

/// Service context that holds references to actual services
/// This is not Clone because it contains Arc<Mutex<...>> references
#[derive(Debug)]
pub struct ServiceContext {
    pub audio_session_manager: Option<std::sync::Arc<std::sync::Mutex<crate::services::audio_session_manager::AudioSessionManager>>>,
}

/// System operating mode
#[derive(Debug, Clone, PartialEq)]
pub enum SystemMode {
    Normal,
    Narration,
    Recording,
    Configuration,
    Help,
    Maintenance,
}

/// Audio system state
#[derive(Debug, Clone)]
pub struct AudioState {
    pub vad_enabled: bool,
    pub sensitivity: f32,
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
    pub current_device: Option<String>,
    pub recording_active: bool,
}

/// STT system state
#[derive(Debug, Clone)]
pub struct STTState {
    pub current_model: String,
    pub language: Option<String>,
    pub instant_output: bool,
    pub confidence_threshold: f32,
    pub processing_queue_size: usize,
    pub last_transcription: Option<String>,
}

/// Environment context
#[derive(Debug, Clone)]
pub struct EnvironmentContext {
    pub platform: String,
    pub hostname: String,
    pub username: String,
    pub working_directory: String,
    pub active_applications: Vec<String>,
    pub time_of_day: String,
}

/// Executed command record
#[derive(Debug, Clone)]
pub struct ExecutedCommand {
    pub command_name: String,
    pub category: CommandCategory,
    pub parameters: String,
    pub result: CommandResult,
    pub timestamp: DateTime<Utc>,
}

/// Command categories for organization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandCategory {
    Audio,
    STT,
    System,
    FileManagement,
    Tools,
    Navigation,
    Help,
    Recording,
    Transcription,
    Parameters,
}

/// Command difficulty level
#[derive(Debug, Clone, PartialEq)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Pattern matching strategy
#[derive(Debug, Clone)]
pub enum PatternType {
    Exact(String),
    Contains(String),
    Regex(Regex),
    Fuzzy(String, f32), // pattern, similarity threshold
}

/// Voice command trait for extensible command system
pub trait VoiceCommand: Send + Sync {
    /// Execute the command with given parameters and context
    fn execute(&self, params: CommandParams, context: &mut SystemContext, services: Option<&ServiceContext>) -> Result<CommandResult, VoiceCommandError>;
    
    /// Get all voice patterns that trigger this command
    fn get_patterns(&self) -> Vec<PatternType>;
    
    /// Get the command category
    fn get_category(&self) -> CommandCategory;
    
    /// Get command help text
    fn get_help_text(&self) -> &str;
    
    /// Get command name/identifier
    fn get_name(&self) -> &str;
    
    /// Get command description
    fn get_description(&self) -> &str;
    
    /// Get difficulty level
    fn get_difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Beginner
    }
    
    /// Get required permissions
    fn get_permissions(&self) -> Vec<String> {
        Vec::new()
    }
    
    /// Validate command can execute in current context
    fn validate_context(&self, context: &SystemContext) -> Result<(), VoiceCommandError> {
        Ok(())
    }
    
    /// Get command examples
    fn get_examples(&self) -> Vec<String> {
        Vec::new()
    }
    
    /// Get related commands
    fn get_related_commands(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Command pattern for matching voice input
#[derive(Debug, Clone)]
pub struct CommandPattern {
    pub pattern: PatternType,
    pub command_name: String,
    pub priority: u8, // Higher priority = matched first
    pub enabled: bool,
}

/// Main voice command engine
pub struct VoiceCommandEngine {
    /// Registered commands
    commands: HashMap<String, Box<dyn VoiceCommand>>,
    /// Command patterns for matching
    patterns: Vec<CommandPattern>,
    /// Command execution history
    history: VecDeque<ExecutedCommand>,
    /// System context
    context: SystemContext,
    /// Service context
    services: Option<ServiceContext>,
    /// Configuration
    config: VoiceCommandConfig,
    /// Performance metrics
    metrics: CommandMetrics,
}

/// Engine configuration
#[derive(Debug, Clone)]
pub struct VoiceCommandConfig {
    pub max_history_size: usize,
    pub command_timeout: Duration,
    pub enable_fuzzy_matching: bool,
    pub fuzzy_threshold: f32,
    pub enable_suggestions: bool,
    pub enable_learning: bool,
    pub case_sensitive: bool,
}

/// Command execution metrics
#[derive(Debug, Clone)]
pub struct CommandMetrics {
    pub total_commands: u64,
    pub successful_commands: u64,
    pub failed_commands: u64,
    pub average_execution_time: Duration,
    pub command_usage: HashMap<String, u64>,
    pub error_counts: HashMap<String, u64>,
}

impl Default for VoiceCommandConfig {
    fn default() -> Self {
        Self {
            max_history_size: 100,
            command_timeout: Duration::from_secs(10),
            enable_fuzzy_matching: true,
            fuzzy_threshold: 0.8,
            enable_suggestions: true,
            enable_learning: true,
            case_sensitive: false,
        }
    }
}

impl Default for CommandMetrics {
    fn default() -> Self {
        Self {
            total_commands: 0,
            successful_commands: 0,
            failed_commands: 0,
            average_execution_time: Duration::from_millis(0),
            command_usage: HashMap::new(),
            error_counts: HashMap::new(),
        }
    }
}

impl VoiceCommandEngine {
    /// Create a new voice command engine
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            patterns: Vec::new(),
            history: VecDeque::new(),
            context: SystemContext::default(),
            services: None,
            config: VoiceCommandConfig::default(),
            metrics: CommandMetrics::default(),
        }
    }
    
    /// Create engine with custom configuration
    pub fn with_config(config: VoiceCommandConfig) -> Self {
        Self {
            commands: HashMap::new(),
            patterns: Vec::new(),
            history: VecDeque::new(),
            context: SystemContext::default(),
            services: None,
            config,
            metrics: CommandMetrics::default(),
        }
    }
    
    /// Register a new voice command
    pub fn register_command<T: VoiceCommand + 'static>(&mut self, command: T) -> Result<(), VoiceCommandError> {
        let name = command.get_name().to_string();
        let patterns = command.get_patterns();
        
        // Add patterns to matching system
        for pattern in patterns {
            self.patterns.push(CommandPattern {
                pattern,
                command_name: name.clone(),
                priority: 5, // Default priority
                enabled: true,
            });
        }
        
        // Register command
        self.commands.insert(name, Box::new(command));
        
        // Sort patterns by priority
        self.patterns.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(())
    }
    
    /// Unregister a voice command
    pub fn unregister_command(&mut self, name: &str) -> Result<(), VoiceCommandError> {
        self.commands.remove(name);
        self.patterns.retain(|p| p.command_name != name);
        Ok(())
    }
    
    /// Parse voice input and find matching command
    pub fn parse_command(&self, input: &str) -> Result<ParsedCommand, VoiceCommandError> {
        let input = if self.config.case_sensitive {
            input.to_string()
        } else {
            input.to_lowercase()
        };
        
        // Try exact pattern matching first
        for pattern in &self.patterns {
            if !pattern.enabled {
                continue;
            }
            
            if self.pattern_matches(&pattern.pattern, &input) {
                if let Some(command) = self.commands.get(&pattern.command_name) {
                    return Ok(ParsedCommand {
                        command_name: pattern.command_name.clone(),
                        original_input: input.clone(),
                        confidence: 1.0,
                        category: command.get_category(),
                        matched_pattern: format!("{:?}", pattern.pattern),
                    });
                }
            }
        }
        
        // Try fuzzy matching if enabled
        if self.config.enable_fuzzy_matching {
            if let Some(fuzzy_match) = self.fuzzy_match(&input) {
                return Ok(fuzzy_match);
            }
        }
        
        Err(VoiceCommandError::CommandNotFound(input))
    }
    
    /// Execute a parsed command
    pub async fn execute_command(&mut self, parsed: ParsedCommand, params: CommandParams) -> Result<CommandResult, VoiceCommandError> {
        let start_time = Instant::now();
        
        // Get command
        let command = self.commands.get(&parsed.command_name)
            .ok_or_else(|| VoiceCommandError::CommandNotFound(parsed.command_name.clone()))?;
        
        // Validate context
        command.validate_context(&self.context)?;
        
        // Execute with timeout
        let result = tokio::time::timeout(
            self.config.command_timeout,
            async { command.execute(params.clone(), &mut self.context, self.services.as_ref()) }
        ).await;
        
        let execution_time = start_time.elapsed();
        
        let command_result = match result {
            Ok(Ok(result)) => {
                self.metrics.successful_commands += 1;
                result
            }
            Ok(Err(e)) => {
                self.metrics.failed_commands += 1;
                self.metrics.error_counts
                    .entry(e.to_string())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
                return Err(e);
            }
            Err(_) => {
                self.metrics.failed_commands += 1;
                return Err(VoiceCommandError::Timeout);
            }
        };
        
        // Update metrics
        self.metrics.total_commands += 1;
        self.metrics.command_usage
            .entry(parsed.command_name.clone())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        
        // Update average execution time
        let total_time = self.metrics.average_execution_time * (self.metrics.total_commands - 1) as u32 + execution_time;
        self.metrics.average_execution_time = total_time / self.metrics.total_commands as u32;
        
        // Add to history
        let executed = ExecutedCommand {
            command_name: parsed.command_name,
            category: parsed.category,
            parameters: params.text,
            result: command_result.clone(),
            timestamp: Utc::now(),
        };
        
        self.history.push_back(executed);
        if self.history.len() > self.config.max_history_size {
            self.history.pop_front();
        }
        
        Ok(command_result)
    }
    
    /// Process voice input end-to-end
    pub async fn process_voice_input(&mut self, input: &str, confidence: f32) -> Result<CommandResult, VoiceCommandError> {
        let params = CommandParams {
            text: input.to_string(),
            confidence,
            context: self.context.clone(),
            timestamp: Utc::now(),
            user_id: None,
        };
        
        let parsed = self.parse_command(input)?;
        self.execute_command(parsed, params).await
    }
    
    /// Get command suggestions based on input
    pub fn get_suggestions(&self, partial_input: &str) -> Vec<CommandSuggestion> {
        if !self.config.enable_suggestions {
            return Vec::new();
        }
        
        let mut suggestions = Vec::new();
        let input_lower = partial_input.to_lowercase();
        
        for command in self.commands.values() {
            for pattern in command.get_patterns() {
                if let Some(score) = self.calculate_suggestion_score(&pattern, &input_lower) {
                    suggestions.push(CommandSuggestion {
                        command_name: command.get_name().to_string(),
                        description: command.get_description().to_string(),
                        category: command.get_category(),
                        pattern: format!("{:?}", pattern),
                        confidence: score,
                        examples: command.get_examples(),
                    });
                }
            }
        }
        
        // Sort by confidence score
        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        suggestions.truncate(10); // Return top 10 suggestions
        
        suggestions
    }
    
    /// Get commands by category
    pub fn get_commands_by_category(&self, category: CommandCategory) -> Vec<&str> {
        self.commands
            .values()
            .filter(|cmd| cmd.get_category() == category)
            .map(|cmd| cmd.get_name())
            .collect()
    }
    
    /// Get command help
    pub fn get_command_help(&self, command_name: &str) -> Option<CommandHelp> {
        self.commands.get(command_name).map(|cmd| CommandHelp {
            name: cmd.get_name().to_string(),
            description: cmd.get_description().to_string(),
            category: cmd.get_category(),
            patterns: cmd.get_patterns().iter().map(|p| format!("{:?}", p)).collect(),
            help_text: cmd.get_help_text().to_string(),
            examples: cmd.get_examples(),
            related_commands: cmd.get_related_commands(),
            difficulty: cmd.get_difficulty(),
            permissions: cmd.get_permissions(),
        })
    }
    
    /// Get execution metrics
    pub fn get_metrics(&self) -> &CommandMetrics {
        &self.metrics
    }
    
    /// Get recent command history
    pub fn get_recent_commands(&self, limit: usize) -> Vec<&ExecutedCommand> {
        self.history.iter().rev().take(limit).collect()
    }
    
    /// Update system context
    pub fn update_context(&mut self, context: SystemContext) {
        self.context = context;
    }
    
    /// Set service context
    pub fn set_service_context(&mut self, services: ServiceContext) {
        self.services = Some(services);
    }
    
    /// Pattern matching implementation
    fn pattern_matches(&self, pattern: &PatternType, input: &str) -> bool {
        match pattern {
            PatternType::Exact(p) => input == p,
            PatternType::Contains(p) => input.contains(p),
            PatternType::Regex(r) => r.is_match(input),
            PatternType::Fuzzy(p, threshold) => {
                let similarity = self.calculate_similarity(p, input);
                similarity >= *threshold
            }
        }
    }
    
    /// Fuzzy matching for similar commands
    fn fuzzy_match(&self, input: &str) -> Option<ParsedCommand> {
        let mut best_match: Option<(String, f32)> = None;
        
        for pattern in &self.patterns {
            if !pattern.enabled {
                continue;
            }
            
            let similarity = match &pattern.pattern {
                PatternType::Exact(p) | PatternType::Contains(p) | PatternType::Fuzzy(p, _) => {
                    self.calculate_similarity(p, input)
                }
                PatternType::Regex(_) => continue, // Skip regex for fuzzy matching
            };
            
            if similarity >= self.config.fuzzy_threshold {
                if let Some((_, current_best)) = &best_match {
                    if similarity > *current_best {
                        best_match = Some((pattern.command_name.clone(), similarity));
                    }
                } else {
                    best_match = Some((pattern.command_name.clone(), similarity));
                }
            }
        }
        
        if let Some((command_name, confidence)) = best_match {
            if let Some(command) = self.commands.get(&command_name) {
                return Some(ParsedCommand {
                    command_name,
                    original_input: input.to_string(),
                    confidence,
                    category: command.get_category(),
                    matched_pattern: "fuzzy".to_string(),
                });
            }
        }
        
        None
    }
    
    /// Calculate string similarity (Levenshtein distance)
    fn calculate_similarity(&self, a: &str, b: &str) -> f32 {
        let len_a = a.len();
        let len_b = b.len();
        
        if len_a == 0 {
            return if len_b == 0 { 1.0 } else { 0.0 };
        }
        if len_b == 0 {
            return 0.0;
        }
        
        let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];
        
        for i in 0..=len_a {
            matrix[i][0] = i;
        }
        for j in 0..=len_b {
            matrix[0][j] = j;
        }
        
        for i in 1..=len_a {
            for j in 1..=len_b {
                let cost = if a.chars().nth(i - 1) == b.chars().nth(j - 1) { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }
        
        let distance = matrix[len_a][len_b];
        let max_len = len_a.max(len_b);
        
        1.0 - (distance as f32 / max_len as f32)
    }
    
    /// Calculate suggestion score for partial input
    fn calculate_suggestion_score(&self, pattern: &PatternType, input: &str) -> Option<f32> {
        match pattern {
            PatternType::Exact(p) | PatternType::Contains(p) | PatternType::Fuzzy(p, _) => {
                if p.starts_with(input) {
                    Some(0.9) // High score for prefix match
                } else if p.contains(input) {
                    Some(0.7) // Medium score for substring match
                } else {
                    let similarity = self.calculate_similarity(p, input);
                    if similarity > 0.6 {
                        Some(similarity * 0.5) // Lower score for fuzzy match
                    } else {
                        None
                    }
                }
            }
            PatternType::Regex(_) => None, // Skip regex for suggestions
        }
    }
}

/// Parsed command result
#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub command_name: String,
    pub original_input: String,
    pub confidence: f32,
    pub category: CommandCategory,
    pub matched_pattern: String,
}

/// Command suggestion
#[derive(Debug, Clone)]
pub struct CommandSuggestion {
    pub command_name: String,
    pub description: String,
    pub category: CommandCategory,
    pub pattern: String,
    pub confidence: f32,
    pub examples: Vec<String>,
}

/// Command help information
#[derive(Debug, Clone)]
pub struct CommandHelp {
    pub name: String,
    pub description: String,
    pub category: CommandCategory,
    pub patterns: Vec<String>,
    pub help_text: String,
    pub examples: Vec<String>,
    pub related_commands: Vec<String>,
    pub difficulty: DifficultyLevel,
    pub permissions: Vec<String>,
}

impl Default for SystemContext {
    fn default() -> Self {
        Self {
            current_mode: SystemMode::Normal,
            audio_state: AudioState::default(),
            stt_state: STTState::default(),
            recent_commands: VecDeque::new(),
            session_data: HashMap::new(),
            environment: EnvironmentContext::default(),
        }
    }
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            vad_enabled: true,
            sensitivity: 0.5,
            sample_rate: 16000,
            channels: 1,
            buffer_size: 1024,
            current_device: None,
            recording_active: false,
        }
    }
}

impl Default for STTState {
    fn default() -> Self {
        Self {
            current_model: "base".to_string(),
            language: None,
            instant_output: false,
            confidence_threshold: 0.6,
            processing_queue_size: 0,
            last_transcription: None,
        }
    }
}

impl Default for EnvironmentContext {
    fn default() -> Self {
        Self {
            platform: std::env::consts::OS.to_string(),
            hostname: std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string()),
            username: std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
            working_directory: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "/".to_string()),
            active_applications: Vec::new(),
            time_of_day: chrono::Local::now().format("%H:%M").to_string(),
        }
    }
}

impl CommandResult {
    /// Create a successful result
    pub fn success(message: String) -> Self {
        Self {
            success: true,
            message,
            data: None,
            execution_time: Duration::from_millis(0),
            timestamp: Utc::now(),
        }
    }
    
    /// Create a successful result with data
    pub fn success_with_data(message: String, data: CommandData) -> Self {
        Self {
            success: true,
            message,
            data: Some(data),
            execution_time: Duration::from_millis(0),
            timestamp: Utc::now(),
        }
    }
    
    /// Create a failure result
    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
            data: None,
            execution_time: Duration::from_millis(0),
            timestamp: Utc::now(),
        }
    }
    
    /// Set execution time
    pub fn with_execution_time(mut self, time: Duration) -> Self {
        self.execution_time = time;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestCommand;
    
    impl VoiceCommand for TestCommand {
        fn execute(&self, _params: CommandParams, _context: &mut SystemContext, _services: Option<&ServiceContext>) -> Result<CommandResult, VoiceCommandError> {
            Ok(CommandResult::success("Test command executed".to_string()))
        }
        
        fn get_patterns(&self) -> Vec<PatternType> {
            vec![
                PatternType::Exact("test command".to_string()),
                PatternType::Contains("test".to_string()),
            ]
        }
        
        fn get_category(&self) -> CommandCategory {
            CommandCategory::System
        }
        
        fn get_help_text(&self) -> &str {
            "A test command for unit testing"
        }
        
        fn get_name(&self) -> &str {
            "test_command"
        }
        
        fn get_description(&self) -> &str {
            "Test command description"
        }
    }
    
    #[test]
    fn test_command_registration() {
        let mut engine = VoiceCommandEngine::new();
        let result = engine.register_command(TestCommand);
        assert!(result.is_ok());
        assert!(engine.commands.contains_key("test_command"));
    }
    
    #[test]
    fn test_command_parsing() {
        let mut engine = VoiceCommandEngine::new();
        engine.register_command(TestCommand).unwrap();
        
        let parsed = engine.parse_command("test command");
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.command_name, "test_command");
        assert_eq!(parsed.confidence, 1.0);
    }
    
    #[test]
    fn test_fuzzy_matching() {
        let mut engine = VoiceCommandEngine::new();
        engine.register_command(TestCommand).unwrap();
        
        let parsed = engine.parse_command("tst comand"); // Misspelled
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.command_name, "test_command");
        assert!(parsed.confidence > 0.8);
    }
    
    #[tokio::test]
    async fn test_command_execution() {
        let mut engine = VoiceCommandEngine::new();
        engine.register_command(TestCommand).unwrap();
        
        let result = engine.process_voice_input("test command", 0.95).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Test command executed");
    }
    
    #[test]
    fn test_suggestions() {
        let mut engine = VoiceCommandEngine::new();
        engine.register_command(TestCommand).unwrap();
        
        let suggestions = engine.get_suggestions("tes");
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].command_name, "test_command");
    }
}
