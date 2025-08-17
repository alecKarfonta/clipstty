//! Context-aware command resolution and management.
//! 
//! This module implements intelligent command context analysis, disambiguation,
//! and suggestion systems based on the current system state and command history.

use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use chrono::{DateTime, Utc, Timelike};
use serde::{Deserialize, Serialize};

use super::*;

/// Command context manager for intelligent command resolution
pub struct CommandContextManager {
    /// Current system mode
    current_mode: SystemMode,
    /// Recent command history
    command_history: VecDeque<ExecutedCommand>,
    /// Current session state variables
    session_state: HashMap<String, ContextValue>,
    /// User preferences and learned behaviors
    user_preferences: UserPreferences,
    /// Disambiguation rules
    disambiguation_rules: Vec<DisambiguationRule>,
    /// Context prediction model
    prediction_engine: ContextPredictionEngine,
}

/// User preferences and learned patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Frequently used commands
    pub command_frequency: HashMap<String, u64>,
    /// Preferred command variations
    pub preferred_patterns: HashMap<String, String>,
    /// Command usage by time of day
    pub time_based_usage: HashMap<String, Vec<TimeUsage>>,
    /// Context-specific preferences
    pub context_preferences: HashMap<String, ContextPreference>,
    /// Learning enabled
    pub learning_enabled: bool,
}

/// Time-based usage patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeUsage {
    pub hour: u8,
    pub frequency: u64,
}

/// Context-specific user preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPreference {
    pub preferred_commands: Vec<String>,
    pub avoided_commands: Vec<String>,
    pub custom_patterns: HashMap<String, String>,
}

/// Context value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContextValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<ContextValue>),
    Object(HashMap<String, ContextValue>),
}

/// Disambiguation rule for resolving ambiguous commands
#[derive(Debug, Clone)]
pub struct DisambiguationRule {
    /// Pattern to match ambiguous input
    pub pattern: String,
    /// Conditions that must be met
    pub conditions: Vec<ContextCondition>,
    /// Resolved command name
    pub resolved_command: String,
    /// Priority of this rule (higher = more important)
    pub priority: u8,
}

/// Context condition for disambiguation
#[derive(Debug, Clone)]
pub enum ContextCondition {
    /// System must be in specific mode
    Mode(SystemMode),
    /// Session variable must have specific value
    SessionVar(String, ContextValue),
    /// Recent command must be present
    RecentCommand(String, Duration),
    /// Time-based condition
    TimeOfDay(u8, u8), // start_hour, end_hour
    /// Audio state condition
    AudioState(AudioStateCondition),
    /// STT state condition
    STTState(STTStateCondition),
}

/// Audio state conditions
#[derive(Debug, Clone)]
pub enum AudioStateCondition {
    VADEnabled(bool),
    SensitivityAbove(f32),
    SensitivityBelow(f32),
    Recording(bool),
}

/// STT state conditions
#[derive(Debug, Clone)]
pub enum STTStateCondition {
    InstantOutput(bool),
    Model(String),
    Language(String),
    ConfidenceAbove(f32),
}

/// Context prediction engine for suggesting next commands
pub struct ContextPredictionEngine {
    /// Command transition patterns
    transition_patterns: HashMap<String, Vec<CommandTransition>>,
    /// Sequence patterns (command chains)
    sequence_patterns: Vec<CommandSequence>,
    /// Context-based predictions
    context_predictions: HashMap<String, Vec<String>>,
}

/// Command transition probability
#[derive(Debug, Clone)]
pub struct CommandTransition {
    pub next_command: String,
    pub probability: f32,
    pub conditions: Vec<ContextCondition>,
}

/// Command sequence pattern
#[derive(Debug, Clone)]
pub struct CommandSequence {
    pub commands: Vec<String>,
    pub frequency: u64,
    pub context: String,
}

/// Resolved command with context information
#[derive(Debug, Clone)]
pub struct ResolvedCommand {
    pub command_name: String,
    pub confidence: f32,
    pub resolution_method: ResolutionMethod,
    pub context_factors: Vec<String>,
}

/// Method used to resolve the command
#[derive(Debug, Clone, PartialEq)]
pub enum ResolutionMethod {
    Direct,
    Disambiguation,
    ContextPrediction,
    UserLearning,
    FuzzyMatch,
}

impl CommandContextManager {
    /// Create a new context manager
    pub fn new() -> Self {
        Self {
            current_mode: SystemMode::Normal,
            command_history: VecDeque::new(),
            session_state: HashMap::new(),
            user_preferences: UserPreferences::default(),
            disambiguation_rules: Vec::new(),
            prediction_engine: ContextPredictionEngine::new(),
        }
    }
    
    /// Update the current system context
    pub fn update_context(&mut self, context: &SystemContext) {
        self.current_mode = context.current_mode.clone();
        
        // Update session state from context
        self.session_state.insert("audio_vad_enabled".to_string(), 
            ContextValue::Boolean(context.audio_state.vad_enabled));
        self.session_state.insert("audio_sensitivity".to_string(), 
            ContextValue::Number(context.audio_state.sensitivity as f64));
        self.session_state.insert("stt_instant_output".to_string(), 
            ContextValue::Boolean(context.stt_state.instant_output));
        self.session_state.insert("stt_model".to_string(), 
            ContextValue::String(context.stt_state.current_model.clone()));
    }
    
    /// Resolve an ambiguous command using context
    pub fn resolve_ambiguous_command(&self, input: &str) -> Vec<ResolvedCommand> {
        let mut candidates = Vec::new();
        
        // Try disambiguation rules first
        for rule in &self.disambiguation_rules {
            if input.contains(&rule.pattern) && self.check_conditions(&rule.conditions) {
                candidates.push(ResolvedCommand {
                    command_name: rule.resolved_command.clone(),
                    confidence: 0.9 + (rule.priority as f32 / 100.0),
                    resolution_method: ResolutionMethod::Disambiguation,
                    context_factors: self.get_matching_factors(&rule.conditions),
                });
            }
        }
        
        // Try user learning patterns
        if let Some(preferred) = self.user_preferences.preferred_patterns.get(input) {
            candidates.push(ResolvedCommand {
                command_name: preferred.clone(),
                confidence: 0.85,
                resolution_method: ResolutionMethod::UserLearning,
                context_factors: vec!["user_preference".to_string()],
            });
        }
        
        // Try context prediction
        if let Some(predictions) = self.get_context_predictions() {
            for prediction in predictions {
                if prediction.contains(&input.to_lowercase()) {
                    candidates.push(ResolvedCommand {
                        command_name: prediction,
                        confidence: 0.7,
                        resolution_method: ResolutionMethod::ContextPrediction,
                        context_factors: vec!["context_prediction".to_string()],
                    });
                }
            }
        }
        
        // Sort by confidence
        candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        candidates
    }
    
    /// Suggest next commands based on context
    pub fn suggest_next_commands(&self) -> Vec<CommandSuggestion> {
        let mut suggestions = Vec::new();
        
        // Get suggestions based on current mode
        match self.current_mode {
            SystemMode::Normal => {
                suggestions.extend(self.get_normal_mode_suggestions());
            }
            SystemMode::Narration => {
                suggestions.extend(self.get_narration_mode_suggestions());
            }
            SystemMode::Recording => {
                suggestions.extend(self.get_recording_mode_suggestions());
            }
            SystemMode::Configuration => {
                suggestions.extend(self.get_configuration_mode_suggestions());
            }
            SystemMode::Help => {
                suggestions.extend(self.get_help_mode_suggestions());
            }
            SystemMode::Maintenance => {
                suggestions.extend(self.get_maintenance_mode_suggestions());
            }
        }
        
        // Add transition-based suggestions
        if let Some(last_command) = self.command_history.back() {
            if let Some(transitions) = self.prediction_engine.transition_patterns.get(&last_command.command_name) {
                for transition in transitions {
                    if self.check_conditions(&transition.conditions) {
                        suggestions.push(CommandSuggestion {
                            command_name: transition.next_command.clone(),
                            description: format!("Suggested after {}", last_command.command_name),
                            category: CommandCategory::System, // Will be updated by engine
                            pattern: "transition".to_string(),
                            confidence: transition.probability,
                            examples: vec![],
                        });
                    }
                }
            }
        }
        
        // Add frequently used commands
        let frequent_commands = self.get_frequent_commands(5);
        for (command, frequency) in frequent_commands {
            suggestions.push(CommandSuggestion {
                command_name: command,
                description: "Frequently used command".to_string(),
                category: CommandCategory::System,
                pattern: "frequency".to_string(),
                confidence: (frequency as f32 / 100.0).min(0.8),
                examples: vec![],
            });
        }
        
        // Sort by confidence and remove duplicates
        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        suggestions.dedup_by(|a, b| a.command_name == b.command_name);
        suggestions.truncate(10);
        
        suggestions
    }
    
    /// Record a successfully executed command for learning
    pub fn record_command_execution(&mut self, executed: &ExecutedCommand) {
        // Add to history
        self.command_history.push_back(executed.clone());
        if self.command_history.len() > 50 {
            self.command_history.pop_front();
        }
        
        // Update user preferences if learning is enabled
        if self.user_preferences.learning_enabled {
            self.update_command_frequency(&executed.command_name);
            self.update_time_based_usage(&executed.command_name, executed.timestamp);
            self.update_transition_patterns(executed);
        }
    }
    
    /// Set session variable
    pub fn set_session_var(&mut self, key: String, value: ContextValue) {
        self.session_state.insert(key, value);
    }
    
    /// Get session variable
    pub fn get_session_var(&self, key: &str) -> Option<&ContextValue> {
        self.session_state.get(key)
    }
    
    /// Add a disambiguation rule
    pub fn add_disambiguation_rule(&mut self, rule: DisambiguationRule) {
        self.disambiguation_rules.push(rule);
        // Sort by priority
        self.disambiguation_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
    
    /// Get context-aware command completions
    pub fn get_command_completions(&self, partial: &str) -> Vec<String> {
        let mut completions = Vec::new();
        
        // Get completions based on user preferences
        for (pattern, _command) in &self.user_preferences.preferred_patterns {
            if pattern.starts_with(partial) {
                completions.push(pattern.clone());
            }
        }
        
        // Get completions based on current context
        let context_key = format!("{:?}", self.current_mode);
        if let Some(context_pref) = self.user_preferences.context_preferences.get(&context_key) {
            for pattern in &context_pref.preferred_commands {
                if pattern.starts_with(partial) {
                    completions.push(pattern.clone());
                }
            }
        }
        
        completions.sort();
        completions.dedup();
        completions
    }
    
    /// Check if all conditions are met
    fn check_conditions(&self, conditions: &[ContextCondition]) -> bool {
        conditions.iter().all(|condition| self.check_condition(condition))
    }
    
    /// Check a single condition
    fn check_condition(&self, condition: &ContextCondition) -> bool {
        match condition {
            ContextCondition::Mode(mode) => self.current_mode == *mode,
            ContextCondition::SessionVar(key, expected_value) => {
                if let Some(actual_value) = self.session_state.get(key) {
                    self.compare_context_values(actual_value, expected_value)
                } else {
                    false
                }
            }
            ContextCondition::RecentCommand(command, duration) => {
                let cutoff = Utc::now() - chrono::Duration::from_std(*duration).unwrap();
                self.command_history.iter().any(|cmd| {
                    cmd.command_name == *command && cmd.timestamp > cutoff
                })
            }
            ContextCondition::TimeOfDay(start, end) => {
                let current_hour = chrono::Local::now().hour() as u8;
                if start <= end {
                    current_hour >= *start && current_hour <= *end
                } else {
                    current_hour >= *start || current_hour <= *end
                }
            }
            ContextCondition::AudioState(audio_condition) => {
                self.check_audio_condition(audio_condition)
            }
            ContextCondition::STTState(stt_condition) => {
                self.check_stt_condition(stt_condition)
            }
        }
    }
    
    /// Check audio state condition
    fn check_audio_condition(&self, condition: &AudioStateCondition) -> bool {
        match condition {
            AudioStateCondition::VADEnabled(expected) => {
                if let Some(ContextValue::Boolean(actual)) = self.session_state.get("audio_vad_enabled") {
                    *actual == *expected
                } else {
                    false
                }
            }
            AudioStateCondition::SensitivityAbove(threshold) => {
                if let Some(ContextValue::Number(sensitivity)) = self.session_state.get("audio_sensitivity") {
                    *sensitivity > *threshold as f64
                } else {
                    false
                }
            }
            AudioStateCondition::SensitivityBelow(threshold) => {
                if let Some(ContextValue::Number(sensitivity)) = self.session_state.get("audio_sensitivity") {
                    *sensitivity < *threshold as f64
                } else {
                    false
                }
            }
            AudioStateCondition::Recording(expected) => {
                if let Some(ContextValue::Boolean(actual)) = self.session_state.get("audio_recording") {
                    *actual == *expected
                } else {
                    false
                }
            }
        }
    }
    
    /// Check STT state condition
    fn check_stt_condition(&self, condition: &STTStateCondition) -> bool {
        match condition {
            STTStateCondition::InstantOutput(expected) => {
                if let Some(ContextValue::Boolean(actual)) = self.session_state.get("stt_instant_output") {
                    *actual == *expected
                } else {
                    false
                }
            }
            STTStateCondition::Model(expected_model) => {
                if let Some(ContextValue::String(actual_model)) = self.session_state.get("stt_model") {
                    actual_model == expected_model
                } else {
                    false
                }
            }
            STTStateCondition::Language(expected_lang) => {
                if let Some(ContextValue::String(actual_lang)) = self.session_state.get("stt_language") {
                    actual_lang == expected_lang
                } else {
                    false
                }
            }
            STTStateCondition::ConfidenceAbove(threshold) => {
                if let Some(ContextValue::Number(confidence)) = self.session_state.get("stt_confidence") {
                    *confidence > *threshold as f64
                } else {
                    false
                }
            }
        }
    }
    
    /// Compare context values
    fn compare_context_values(&self, actual: &ContextValue, expected: &ContextValue) -> bool {
        match (actual, expected) {
            (ContextValue::String(a), ContextValue::String(e)) => a == e,
            (ContextValue::Number(a), ContextValue::Number(e)) => (a - e).abs() < 0.001,
            (ContextValue::Boolean(a), ContextValue::Boolean(e)) => a == e,
            _ => false,
        }
    }
    
    /// Get factors that matched conditions
    fn get_matching_factors(&self, conditions: &[ContextCondition]) -> Vec<String> {
        conditions.iter().filter_map(|condition| {
            if self.check_condition(condition) {
                Some(format!("{:?}", condition))
            } else {
                None
            }
        }).collect()
    }
    
    /// Get context-based predictions
    fn get_context_predictions(&self) -> Option<Vec<String>> {
        let context_key = format!("{:?}", self.current_mode);
        self.prediction_engine.context_predictions.get(&context_key).cloned()
    }
    
    /// Update command frequency for learning
    fn update_command_frequency(&mut self, command: &str) {
        self.user_preferences.command_frequency
            .entry(command.to_string())
            .and_modify(|freq| *freq += 1)
            .or_insert(1);
    }
    
    /// Update time-based usage patterns
    fn update_time_based_usage(&mut self, command: &str, timestamp: DateTime<Utc>) {
        let hour = timestamp.hour() as u8;
        let usage_list = self.user_preferences.time_based_usage
            .entry(command.to_string())
            .or_insert_with(Vec::new);
        
        if let Some(existing) = usage_list.iter_mut().find(|u| u.hour == hour) {
            existing.frequency += 1;
        } else {
            usage_list.push(TimeUsage { hour, frequency: 1 });
        }
    }
    
    /// Update command transition patterns
    fn update_transition_patterns(&mut self, executed: &ExecutedCommand) {
        if self.command_history.len() >= 2 {
            let prev_command = &self.command_history[self.command_history.len() - 2];
            let transitions = self.prediction_engine.transition_patterns
                .entry(prev_command.command_name.clone())
                .or_insert_with(Vec::new);
            
            if let Some(existing) = transitions.iter_mut()
                .find(|t| t.next_command == executed.command_name) {
                existing.probability = (existing.probability * 0.9 + 0.1).min(1.0);
            } else {
                transitions.push(CommandTransition {
                    next_command: executed.command_name.clone(),
                    probability: 0.1,
                    conditions: vec![],
                });
            }
        }
    }
    
    /// Get frequently used commands
    fn get_frequent_commands(&self, limit: usize) -> Vec<(String, u64)> {
        let mut commands: Vec<_> = self.user_preferences.command_frequency.iter()
            .map(|(cmd, freq)| (cmd.clone(), *freq))
            .collect();
        commands.sort_by(|a, b| b.1.cmp(&a.1));
        commands.truncate(limit);
        commands
    }
    
    /// Mode-specific suggestion methods
    fn get_normal_mode_suggestions(&self) -> Vec<CommandSuggestion> {
        vec![
            CommandSuggestion {
                command_name: "enable_narration".to_string(),
                description: "Start continuous dictation".to_string(),
                category: CommandCategory::STT,
                pattern: "mode_suggestion".to_string(),
                confidence: 0.6,
                examples: vec!["enable narration".to_string()],
            },
            CommandSuggestion {
                command_name: "show_status".to_string(),
                description: "Check system status".to_string(),
                category: CommandCategory::System,
                pattern: "mode_suggestion".to_string(),
                confidence: 0.5,
                examples: vec!["show status".to_string()],
            },
        ]
    }
    
    fn get_narration_mode_suggestions(&self) -> Vec<CommandSuggestion> {
        vec![
            CommandSuggestion {
                command_name: "disable_narration".to_string(),
                description: "Stop continuous dictation".to_string(),
                category: CommandCategory::STT,
                pattern: "mode_suggestion".to_string(),
                confidence: 0.8,
                examples: vec!["disable narration".to_string()],
            },
        ]
    }
    
    fn get_recording_mode_suggestions(&self) -> Vec<CommandSuggestion> {
        vec![
            CommandSuggestion {
                command_name: "stop_recording".to_string(),
                description: "Stop audio recording".to_string(),
                category: CommandCategory::Recording,
                pattern: "mode_suggestion".to_string(),
                confidence: 0.8,
                examples: vec!["stop recording".to_string()],
            },
        ]
    }
    
    fn get_configuration_mode_suggestions(&self) -> Vec<CommandSuggestion> {
        vec![
            CommandSuggestion {
                command_name: "save_settings".to_string(),
                description: "Save current configuration".to_string(),
                category: CommandCategory::System,
                pattern: "mode_suggestion".to_string(),
                confidence: 0.7,
                examples: vec!["save settings".to_string()],
            },
        ]
    }
    
    fn get_help_mode_suggestions(&self) -> Vec<CommandSuggestion> {
        vec![
            CommandSuggestion {
                command_name: "list_commands".to_string(),
                description: "Show available commands".to_string(),
                category: CommandCategory::Help,
                pattern: "mode_suggestion".to_string(),
                confidence: 0.7,
                examples: vec!["list commands".to_string()],
            },
        ]
    }
    
    fn get_maintenance_mode_suggestions(&self) -> Vec<CommandSuggestion> {
        vec![
            CommandSuggestion {
                command_name: "run_diagnostics".to_string(),
                description: "Run system diagnostics".to_string(),
                category: CommandCategory::System,
                pattern: "mode_suggestion".to_string(),
                confidence: 0.7,
                examples: vec!["run diagnostics".to_string()],
            },
        ]
    }
}

impl ContextPredictionEngine {
    /// Create a new prediction engine
    pub fn new() -> Self {
        Self {
            transition_patterns: HashMap::new(),
            sequence_patterns: Vec::new(),
            context_predictions: Self::initialize_context_predictions(),
        }
    }
    
    /// Initialize default context predictions
    fn initialize_context_predictions() -> HashMap<String, Vec<String>> {
        let mut predictions = HashMap::new();
        
        predictions.insert("Normal".to_string(), vec![
            "enable_vad".to_string(),
            "show_status".to_string(),
            "toggle_instant_output".to_string(),
            "enable_narration".to_string(),
        ]);
        
        predictions.insert("Narration".to_string(), vec![
            "disable_narration".to_string(),
            "pause_narration".to_string(),
        ]);
        
        predictions.insert("Recording".to_string(), vec![
            "stop_recording".to_string(),
            "pause_recording".to_string(),
            "save_recording".to_string(),
        ]);
        
        predictions
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            command_frequency: HashMap::new(),
            preferred_patterns: HashMap::new(),
            time_based_usage: HashMap::new(),
            context_preferences: HashMap::new(),
            learning_enabled: true,
        }
    }
}

impl Default for CommandContextManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_context_manager_creation() {
        let manager = CommandContextManager::new();
        assert_eq!(manager.current_mode, SystemMode::Normal);
        assert!(manager.command_history.is_empty());
        assert!(manager.user_preferences.learning_enabled);
    }
    
    #[test]
    fn test_session_variables() {
        let mut manager = CommandContextManager::new();
        
        manager.set_session_var("test_var".to_string(), ContextValue::String("test_value".to_string()));
        
        assert!(manager.get_session_var("test_var").is_some());
        if let Some(ContextValue::String(value)) = manager.get_session_var("test_var") {
            assert_eq!(value, "test_value");
        }
    }
    
    #[test]
    fn test_condition_checking() {
        let mut manager = CommandContextManager::new();
        manager.current_mode = SystemMode::Narration;
        
        let condition = ContextCondition::Mode(SystemMode::Narration);
        assert!(manager.check_condition(&condition));
        
        let condition = ContextCondition::Mode(SystemMode::Normal);
        assert!(!manager.check_condition(&condition));
    }
    
    #[test]
    fn test_disambiguation_rules() {
        let mut manager = CommandContextManager::new();
        
        let rule = DisambiguationRule {
            pattern: "record".to_string(),
            conditions: vec![ContextCondition::Mode(SystemMode::Normal)],
            resolved_command: "start_recording".to_string(),
            priority: 10,
        };
        
        manager.add_disambiguation_rule(rule);
        
        let resolved = manager.resolve_ambiguous_command("record audio");
        assert!(!resolved.is_empty());
        assert_eq!(resolved[0].command_name, "start_recording");
    }
    
    #[test]
    fn test_command_frequency_learning() {
        let mut manager = CommandContextManager::new();
        
        let executed = ExecutedCommand {
            command_name: "test_command".to_string(),
            category: CommandCategory::System,
            parameters: "".to_string(),
            result: CommandResult::success("test".to_string()),
            timestamp: Utc::now(),
        };
        
        manager.record_command_execution(&executed);
        manager.record_command_execution(&executed);
        
        assert_eq!(manager.user_preferences.command_frequency.get("test_command"), Some(&2));
    }
    
    #[test]
    fn test_context_suggestions() {
        let mut manager = CommandContextManager::new();
        manager.current_mode = SystemMode::Narration;
        
        let suggestions = manager.suggest_next_commands();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.command_name == "disable_narration"));
    }
}
