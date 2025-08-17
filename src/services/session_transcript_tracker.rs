//! Session Transcript Tracker
//! 
//! This module provides comprehensive session tracking with phrase detection,
//! timestamps, and meeting transcript functionality with speaker preparation.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug, error};
use uuid::Uuid;

/// Transcript phrase with detailed timing and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptPhrase {
    pub id: Uuid,
    pub session_id: Uuid,
    pub text: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration: Duration,
    pub confidence: f32,
    pub speaker_id: Option<String>,
    pub language: Option<String>,
    pub word_count: usize,
    pub is_final: bool,
    pub audio_segment_start: Duration,
    pub audio_segment_end: Duration,
    pub volume_level: f32,
    pub background_noise_level: f32,
    pub phrase_type: PhraseType,
    pub keywords: Vec<String>,
    pub sentiment: Option<SentimentScore>,
}

/// Type of phrase detected
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PhraseType {
    /// Regular speech
    Speech,
    /// Question being asked
    Question,
    /// Command or instruction
    Command,
    /// Exclamation or emphasis
    Exclamation,
    /// Pause or silence
    Silence,
    /// Background noise
    Noise,
    /// Music or non-speech audio
    Music,
}

/// Sentiment analysis score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentScore {
    pub positive: f32,
    pub negative: f32,
    pub neutral: f32,
    pub compound: f32,
}

/// Speaker information for diarization preparation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerProfile {
    pub id: String,
    pub name: Option<String>,
    pub voice_characteristics: VoiceCharacteristics,
    pub speaking_patterns: SpeakingPatterns,
    pub total_speaking_time: Duration,
    pub phrase_count: usize,
    pub average_confidence: f32,
    pub first_appearance: DateTime<Utc>,
    pub last_appearance: DateTime<Utc>,
}

/// Voice characteristics for speaker identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCharacteristics {
    pub fundamental_frequency: f32,
    pub frequency_range: (f32, f32),
    pub speaking_rate: f32, // words per minute
    pub volume_range: (f32, f32),
    pub pitch_variance: f32,
    pub formant_frequencies: Vec<f32>,
}

/// Speaking patterns and habits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakingPatterns {
    pub average_phrase_length: f32,
    pub pause_frequency: f32,
    pub common_words: HashMap<String, u32>,
    pub speaking_rhythm: f32,
    pub interruption_frequency: f32,
    pub question_frequency: f32,
}

/// Session transcript with comprehensive tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTranscript {
    pub session_id: Uuid,
    pub session_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub total_duration: Duration,
    pub phrases: Vec<TranscriptPhrase>,
    pub speakers: HashMap<String, SpeakerProfile>,
    pub session_statistics: SessionStatistics,
    pub keywords: HashMap<String, u32>,
    pub topics: Vec<String>,
    pub summary: Option<String>,
    pub language: String,
    pub quality_metrics: TranscriptQualityMetrics,
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatistics {
    pub total_phrases: usize,
    pub total_words: usize,
    pub total_speaking_time: Duration,
    pub total_silence_time: Duration,
    pub average_phrase_length: f32,
    pub average_confidence: f32,
    pub speaker_count: usize,
    pub interruption_count: usize,
    pub question_count: usize,
    pub command_count: usize,
    pub words_per_minute: f32,
    pub silence_percentage: f32,
}

/// Transcript quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptQualityMetrics {
    pub overall_confidence: f32,
    pub audio_quality_score: f32,
    pub background_noise_level: f32,
    pub speaker_clarity: f32,
    pub transcription_accuracy_estimate: f32,
    pub missing_audio_segments: Vec<(Duration, Duration)>,
    pub low_confidence_segments: Vec<Uuid>,
}

/// Real-time session tracker
pub struct SessionTranscriptTracker {
    /// Current active session
    current_session: Option<SessionTranscript>,
    /// Phrase detection buffer
    phrase_buffer: VecDeque<String>,
    /// Timing tracker
    timing_tracker: TimingTracker,
    /// Speaker detection (preparation for diarization)
    speaker_detector: SpeakerDetector,
    /// Keyword extractor
    keyword_extractor: KeywordExtractor,
    /// Phrase classifier
    phrase_classifier: PhraseClassifier,
    /// Configuration
    config: TrackerConfig,
    /// Statistics
    session_stats: HashMap<Uuid, SessionStatistics>,
}

/// Timing tracker for precise phrase timing
#[derive(Debug)]
pub struct TimingTracker {
    session_start: Option<Instant>,
    last_phrase_end: Option<Instant>,
    silence_start: Option<Instant>,
    current_phrase_start: Option<Instant>,
}

/// Speaker detection system (preparation for full diarization)
#[derive(Debug)]
pub struct SpeakerDetector {
    known_speakers: HashMap<String, SpeakerProfile>,
    current_speaker: Option<String>,
    speaker_change_threshold: f32,
    voice_feature_buffer: VecDeque<VoiceFeatures>,
}

/// Voice features for speaker detection
#[derive(Debug, Clone)]
pub struct VoiceFeatures {
    pub timestamp: Instant,
    pub fundamental_frequency: f32,
    pub spectral_centroid: f32,
    pub mfcc_coefficients: Vec<f32>,
    pub volume: f32,
    pub zero_crossing_rate: f32,
}

/// Keyword extraction system
#[derive(Debug)]
pub struct KeywordExtractor {
    stop_words: std::collections::HashSet<String>,
    keyword_frequency: HashMap<String, u32>,
    min_keyword_length: usize,
    max_keywords_per_phrase: usize,
}

/// Phrase classification system
#[derive(Debug)]
pub struct PhraseClassifier {
    question_indicators: Vec<String>,
    command_indicators: Vec<String>,
    exclamation_indicators: Vec<String>,
    confidence_threshold: f32,
}

/// Tracker configuration
#[derive(Debug, Clone)]
pub struct TrackerConfig {
    pub min_phrase_duration: Duration,
    pub max_silence_gap: Duration,
    pub speaker_change_sensitivity: f32,
    pub keyword_extraction_enabled: bool,
    pub sentiment_analysis_enabled: bool,
    pub real_time_processing: bool,
    pub auto_summarization: bool,
    pub quality_monitoring: bool,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            min_phrase_duration: Duration::from_millis(500),
            max_silence_gap: Duration::from_secs(2),
            speaker_change_sensitivity: 0.7,
            keyword_extraction_enabled: true,
            sentiment_analysis_enabled: true,
            real_time_processing: true,
            auto_summarization: false,
            quality_monitoring: true,
        }
    }
}

impl SessionTranscriptTracker {
    /// Create a new session transcript tracker
    pub fn new(config: TrackerConfig) -> Self {
        Self {
            current_session: None,
            phrase_buffer: VecDeque::new(),
            timing_tracker: TimingTracker::new(),
            speaker_detector: SpeakerDetector::new(),
            keyword_extractor: KeywordExtractor::new(),
            phrase_classifier: PhraseClassifier::new(),
            config,
            session_stats: HashMap::new(),
        }
    }

    /// Start tracking a new session
    pub fn start_session(&mut self, session_id: Uuid, session_name: String) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_session.is_some() {
            warn!("Starting new session while another is active");
            self.end_current_session()?;
        }

        let session = SessionTranscript {
            session_id,
            session_name: session_name.clone(),
            start_time: Utc::now(),
            end_time: None,
            total_duration: Duration::from_secs(0),
            phrases: Vec::new(),
            speakers: HashMap::new(),
            session_statistics: SessionStatistics::default(),
            keywords: HashMap::new(),
            topics: Vec::new(),
            summary: None,
            language: "en".to_string(),
            quality_metrics: TranscriptQualityMetrics::default(),
        };

        self.timing_tracker.start_session();
        self.current_session = Some(session);

        info!(
            session_id = %session_id,
            session_name = %session_name,
            "📝 Started transcript tracking for session"
        );

        Ok(())
    }

    /// Add a new phrase to the current session
    pub fn add_phrase(
        &mut self,
        text: String,
        confidence: f32,
        audio_start: Duration,
        audio_end: Duration,
        volume_level: f32,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let session = self.current_session.as_mut()
            .ok_or("No active session")?;

        let phrase_id = Uuid::new_v4();
        let now = Utc::now();
        let duration = audio_end - audio_start;

        // Classify phrase type
        let phrase_type = self.phrase_classifier.classify(&text);

        // Extract keywords
        let keywords = if self.config.keyword_extraction_enabled {
            self.keyword_extractor.extract_keywords(&text)
        } else {
            Vec::new()
        };

        // Update session keywords
        for keyword in &keywords {
            *session.keywords.entry(keyword.clone()).or_insert(0) += 1;
        }

        // Detect speaker (preparation for diarization)
        let speaker_id = self.speaker_detector.detect_speaker(&text, volume_level, confidence);

        // Calculate sentiment if enabled
        let sentiment = if self.config.sentiment_analysis_enabled {
            Some(Self::calculate_sentiment_static(&text))
        } else {
            None
        };

        let phrase = TranscriptPhrase {
            id: phrase_id,
            session_id: session.session_id,
            text: text.clone(),
            start_time: now,
            end_time: now,
            duration,
            confidence,
            speaker_id: speaker_id.clone(),
            language: Some(session.language.clone()),
            word_count: text.split_whitespace().count(),
            is_final: true,
            audio_segment_start: audio_start,
            audio_segment_end: audio_end,
            volume_level,
            background_noise_level: 0.1, // Mock value
            phrase_type,
            keywords,
            sentiment,
        };

        session.phrases.push(phrase.clone());

        // Update speaker profile after adding phrase
        if let Some(speaker_id) = &speaker_id {
            self.update_speaker_profile(speaker_id, &phrase);
        }

        // Update session statistics
        self.update_current_session_statistics();

        debug!(
            phrase_id = %phrase_id,
            text = %text,
            confidence = confidence,
            speaker_id = ?speaker_id,
            "📝 Added phrase to session transcript"
        );

        Ok(phrase_id)
    }

    /// End the current session
    pub fn end_current_session(&mut self) -> Result<Option<SessionTranscript>, Box<dyn std::error::Error>> {
        if let Some(mut session) = self.current_session.take() {
            session.end_time = Some(Utc::now());
            session.total_duration = session.end_time.unwrap()
                .signed_duration_since(session.start_time)
                .to_std()
                .unwrap_or_default();

            // Final statistics update
            self.update_session_statistics(&mut session);

            // Generate summary if enabled
            if self.config.auto_summarization {
                session.summary = Some(self.generate_summary(&session));
            }

            // Store final statistics
            self.session_stats.insert(session.session_id, session.session_statistics.clone());

            info!(
                session_id = %session.session_id,
                duration = ?session.total_duration,
                phrase_count = session.phrases.len(),
                speaker_count = session.speakers.len(),
                "📝 Ended transcript tracking for session"
            );

            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    /// Get current session info
    pub fn get_current_session(&self) -> Option<&SessionTranscript> {
        self.current_session.as_ref()
    }

    /// Get session statistics
    pub fn get_session_statistics(&self, session_id: &Uuid) -> Option<&SessionStatistics> {
        self.session_stats.get(session_id)
    }

    /// Update speaker profile with new phrase data
    fn update_speaker_profile(&mut self, speaker_id: &str, phrase: &TranscriptPhrase) {
        let session = self.current_session.as_mut().unwrap();
        
        let profile = session.speakers.entry(speaker_id.to_string()).or_insert_with(|| {
            SpeakerProfile {
                id: speaker_id.to_string(),
                name: None,
                voice_characteristics: VoiceCharacteristics::default(),
                speaking_patterns: SpeakingPatterns::default(),
                total_speaking_time: Duration::from_secs(0),
                phrase_count: 0,
                average_confidence: 0.0,
                first_appearance: phrase.start_time,
                last_appearance: phrase.start_time,
            }
        });

        profile.phrase_count += 1;
        profile.total_speaking_time += phrase.duration;
        profile.last_appearance = phrase.start_time;
        profile.average_confidence = (profile.average_confidence * (profile.phrase_count - 1) as f32 + phrase.confidence) / profile.phrase_count as f32;

        // Update speaking patterns
        profile.speaking_patterns.average_phrase_length = 
            (profile.speaking_patterns.average_phrase_length * (profile.phrase_count - 1) as f32 + phrase.word_count as f32) / profile.phrase_count as f32;

        // Update common words
        for word in phrase.text.split_whitespace() {
            let word = word.to_lowercase();
            *profile.speaking_patterns.common_words.entry(word).or_insert(0) += 1;
        }

        // Update question frequency
        if phrase.phrase_type == PhraseType::Question {
            profile.speaking_patterns.question_frequency += 1.0;
        }
    }

    /// Update current session statistics
    fn update_current_session_statistics(&mut self) {
        if self.current_session.is_some() {
            // Extract the data we need without holding a mutable reference
            let (phrases, speakers) = if let Some(session) = &self.current_session {
                (session.phrases.clone(), session.speakers.clone())
            } else {
                return;
            };

            // Now update the session
            if let Some(session) = &mut self.current_session {
                let stats = &mut session.session_statistics;
                
                stats.total_phrases = phrases.len();
                stats.total_words = phrases.iter().map(|p| p.word_count).sum();
                stats.total_speaking_time = phrases.iter().map(|p| p.duration).sum();
                stats.speaker_count = speakers.len();
                stats.question_count = phrases.iter().filter(|p| p.phrase_type == PhraseType::Question).count();
                stats.command_count = phrases.iter().filter(|p| p.phrase_type == PhraseType::Command).count();

                if !phrases.is_empty() {
                    stats.average_phrase_length = stats.total_words as f32 / stats.total_phrases as f32;
                    stats.average_confidence = phrases.iter().map(|p| p.confidence).sum::<f32>() / stats.total_phrases as f32;
                }

                if session.total_duration.as_secs() > 0 {
                    stats.words_per_minute = (stats.total_words as f32 * 60.0) / session.total_duration.as_secs() as f32;
                    stats.silence_percentage = ((session.total_duration - stats.total_speaking_time).as_secs() as f32 / session.total_duration.as_secs() as f32) * 100.0;
                }
            }
        }
    }

    /// Update session statistics
    fn update_session_statistics(&mut self, session: &mut SessionTranscript) {
        let stats = &mut session.session_statistics;
        
        stats.total_phrases = session.phrases.len();
        stats.total_words = session.phrases.iter().map(|p| p.word_count).sum();
        stats.total_speaking_time = session.phrases.iter().map(|p| p.duration).sum();
        stats.speaker_count = session.speakers.len();
        stats.question_count = session.phrases.iter().filter(|p| p.phrase_type == PhraseType::Question).count();
        stats.command_count = session.phrases.iter().filter(|p| p.phrase_type == PhraseType::Command).count();

        if !session.phrases.is_empty() {
            stats.average_phrase_length = stats.total_words as f32 / stats.total_phrases as f32;
            stats.average_confidence = session.phrases.iter().map(|p| p.confidence).sum::<f32>() / stats.total_phrases as f32;
        }

        if session.total_duration.as_secs() > 0 {
            stats.words_per_minute = (stats.total_words as f32 * 60.0) / session.total_duration.as_secs() as f32;
            stats.silence_percentage = ((session.total_duration - stats.total_speaking_time).as_secs() as f32 / session.total_duration.as_secs() as f32) * 100.0;
        }
    }

    /// Generate session summary
    fn generate_summary(&self, session: &SessionTranscript) -> String {
        let top_keywords: Vec<_> = session.keywords.iter()
            .collect::<Vec<_>>();
        let mut top_keywords = top_keywords;
        top_keywords.sort_by(|a, b| b.1.cmp(a.1));
        top_keywords.truncate(5);

        let keyword_list = top_keywords.iter()
            .map(|(word, count)| format!("{} ({})", word, count))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            "Session Summary:\n\
            Duration: {}:{:02}\n\
            Phrases: {}\n\
            Words: {}\n\
            Speakers: {}\n\
            Questions: {}\n\
            Top Keywords: {}",
            session.total_duration.as_secs() / 60,
            session.total_duration.as_secs() % 60,
            session.session_statistics.total_phrases,
            session.session_statistics.total_words,
            session.session_statistics.speaker_count,
            session.session_statistics.question_count,
            keyword_list
        )
    }

    /// Calculate sentiment score (mock implementation)
    fn calculate_sentiment_static(text: &str) -> SentimentScore {
        // Mock sentiment analysis - in real implementation, use a proper NLP library
        let positive_words = ["good", "great", "excellent", "happy", "love", "amazing"];
        let negative_words = ["bad", "terrible", "awful", "hate", "sad", "angry"];
        
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        let positive_count = words.iter().filter(|w| positive_words.contains(w)).count() as f32;
        let negative_count = words.iter().filter(|w| negative_words.contains(w)).count() as f32;
        let total_words = words.len() as f32;
        
        if total_words == 0.0 {
            return SentimentScore {
                positive: 0.0,
                negative: 0.0,
                neutral: 1.0,
                compound: 0.0,
            };
        }
        
        let positive = positive_count / total_words;
        let negative = negative_count / total_words;
        let neutral = 1.0 - positive - negative;
        let compound = positive - negative;
        
        SentimentScore {
            positive,
            negative,
            neutral,
            compound,
        }
    }
}

// Implementation of helper structs
impl TimingTracker {
    fn new() -> Self {
        Self {
            session_start: None,
            last_phrase_end: None,
            silence_start: None,
            current_phrase_start: None,
        }
    }

    fn start_session(&mut self) {
        self.session_start = Some(Instant::now());
    }
}

impl SpeakerDetector {
    fn new() -> Self {
        Self {
            known_speakers: HashMap::new(),
            current_speaker: None,
            speaker_change_threshold: 0.7,
            voice_feature_buffer: VecDeque::new(),
        }
    }

    fn detect_speaker(&mut self, _text: &str, _volume: f32, _confidence: f32) -> Option<String> {
        // Mock speaker detection - in real implementation, use voice characteristics
        // For now, assign speakers based on simple heuristics
        if self.current_speaker.is_none() {
            self.current_speaker = Some("Speaker_1".to_string());
        }
        self.current_speaker.clone()
    }
}

impl KeywordExtractor {
    fn new() -> Self {
        let stop_words = ["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by", "is", "are", "was", "were", "be", "been", "have", "has", "had", "do", "does", "did", "will", "would", "could", "should", "may", "might", "can", "this", "that", "these", "those", "i", "you", "he", "she", "it", "we", "they", "me", "him", "her", "us", "them"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        Self {
            stop_words,
            keyword_frequency: HashMap::new(),
            min_keyword_length: 3,
            max_keywords_per_phrase: 5,
        }
    }

    fn extract_keywords(&mut self, text: &str) -> Vec<String> {
        let words: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() >= self.min_keyword_length && !self.stop_words.contains(*w))
            .map(|w| w.to_string())
            .collect();

        // Update frequency counts
        for word in &words {
            *self.keyword_frequency.entry(word.clone()).or_insert(0) += 1;
        }

        // Return top keywords for this phrase
        words.into_iter().take(self.max_keywords_per_phrase).collect()
    }
}

impl PhraseClassifier {
    fn new() -> Self {
        Self {
            question_indicators: vec!["what", "where", "when", "why", "how", "who", "which", "?"].iter().map(|s| s.to_string()).collect(),
            command_indicators: vec!["please", "start", "stop", "begin", "end", "do", "make", "create", "delete"].iter().map(|s| s.to_string()).collect(),
            exclamation_indicators: vec!["!", "wow", "amazing", "incredible", "fantastic"].iter().map(|s| s.to_string()).collect(),
            confidence_threshold: 0.5,
        }
    }

    fn classify(&self, text: &str) -> PhraseType {
        let text_lower = text.to_lowercase();
        
        if text.contains('?') || self.question_indicators.iter().any(|q| text_lower.contains(q)) {
            PhraseType::Question
        } else if text.contains('!') || self.exclamation_indicators.iter().any(|e| text_lower.contains(e)) {
            PhraseType::Exclamation
        } else if self.command_indicators.iter().any(|c| text_lower.contains(c)) {
            PhraseType::Command
        } else if text.trim().is_empty() {
            PhraseType::Silence
        } else {
            PhraseType::Speech
        }
    }
}

// Default implementations
impl Default for SessionStatistics {
    fn default() -> Self {
        Self {
            total_phrases: 0,
            total_words: 0,
            total_speaking_time: Duration::from_secs(0),
            total_silence_time: Duration::from_secs(0),
            average_phrase_length: 0.0,
            average_confidence: 0.0,
            speaker_count: 0,
            interruption_count: 0,
            question_count: 0,
            command_count: 0,
            words_per_minute: 0.0,
            silence_percentage: 0.0,
        }
    }
}

impl Default for TranscriptQualityMetrics {
    fn default() -> Self {
        Self {
            overall_confidence: 0.0,
            audio_quality_score: 0.0,
            background_noise_level: 0.0,
            speaker_clarity: 0.0,
            transcription_accuracy_estimate: 0.0,
            missing_audio_segments: Vec::new(),
            low_confidence_segments: Vec::new(),
        }
    }
}

impl Default for VoiceCharacteristics {
    fn default() -> Self {
        Self {
            fundamental_frequency: 150.0,
            frequency_range: (80.0, 300.0),
            speaking_rate: 150.0,
            volume_range: (0.1, 0.8),
            pitch_variance: 0.2,
            formant_frequencies: vec![800.0, 1200.0, 2500.0],
        }
    }
}

impl Default for SpeakingPatterns {
    fn default() -> Self {
        Self {
            average_phrase_length: 0.0,
            pause_frequency: 0.0,
            common_words: HashMap::new(),
            speaking_rhythm: 1.0,
            interruption_frequency: 0.0,
            question_frequency: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_creation() {
        let config = TrackerConfig::default();
        let tracker = SessionTranscriptTracker::new(config);
        assert!(tracker.current_session.is_none());
    }

    #[test]
    fn test_session_start() {
        let mut tracker = SessionTranscriptTracker::new(TrackerConfig::default());
        let session_id = Uuid::new_v4();
        let result = tracker.start_session(session_id, "Test Session".to_string());
        assert!(result.is_ok());
        assert!(tracker.current_session.is_some());
    }

    #[test]
    fn test_phrase_classification() {
        let classifier = PhraseClassifier::new();
        assert_eq!(classifier.classify("What is your name?"), PhraseType::Question);
        assert_eq!(classifier.classify("Please start recording"), PhraseType::Command);
        assert_eq!(classifier.classify("That's amazing!"), PhraseType::Exclamation);
        assert_eq!(classifier.classify("Hello world"), PhraseType::Speech);
    }

    #[test]
    fn test_keyword_extraction() {
        let mut extractor = KeywordExtractor::new();
        let keywords = extractor.extract_keywords("The meeting discussion about project management was very productive");
        assert!(keywords.contains(&"meeting".to_string()));
        assert!(keywords.contains(&"discussion".to_string()));
        assert!(keywords.contains(&"project".to_string()));
        assert!(!keywords.contains(&"the".to_string())); // Stop word should be filtered
    }
}
