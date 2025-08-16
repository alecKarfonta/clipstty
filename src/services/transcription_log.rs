//! Transcription Logging and Management System
//! 
//! This module provides comprehensive transcription logging with intelligent
//! deduplication, search capabilities, and analytics.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc, Date, NaiveDate};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use regex::Regex;

use super::audio_archive::{SessionId, AudioFileId};

/// Transcription logging service with deduplication and analytics
pub struct TranscriptionLogService {
    /// Storage backend for transcripts
    storage: Box<dyn TranscriptStorage>,
    /// Deduplication engine
    deduplicator: TranscriptDeduplicator,
    /// Full-text search indexer
    indexer: TranscriptIndexer,
    /// Analytics engine
    analytics: TranscriptAnalytics,
    /// Service configuration
    config: TranscriptionLogConfig,
    /// In-memory cache for recent transcripts
    recent_cache: VecDeque<TranscriptEntry>,
}

/// Transcript storage trait for different backends
pub trait TranscriptStorage: Send + Sync {
    /// Store a new transcript entry
    fn store_transcript(&mut self, entry: &TranscriptEntry) -> Result<TranscriptId, TranscriptError>;
    
    /// Retrieve transcript by ID
    fn get_transcript(&self, id: TranscriptId) -> Result<Option<TranscriptEntry>, TranscriptError>;
    
    /// Search transcripts with criteria
    fn search_transcripts(&self, criteria: &SearchCriteria) -> Result<Vec<TranscriptEntry>, TranscriptError>;
    
    /// Delete transcript by ID
    fn delete_transcript(&mut self, id: TranscriptId) -> Result<(), TranscriptError>;
    
    /// Get storage statistics
    fn get_storage_stats(&self) -> TranscriptStorageStats;
    
    /// Backup transcripts to file
    fn backup_transcripts(&self, path: &Path) -> Result<BackupResult, TranscriptError>;
    
    /// Restore transcripts from backup
    fn restore_transcripts(&mut self, path: &Path) -> Result<RestoreResult, TranscriptError>;
}

/// Transcript deduplication engine
pub struct TranscriptDeduplicator {
    /// Similarity threshold for fuzzy matching
    similarity_threshold: f64,
    /// Hash cache for exact matches
    hash_cache: HashMap<u64, TranscriptId>,
    /// Fuzzy matcher implementation
    fuzzy_matcher: Box<dyn FuzzyMatcher>,
    /// Recent transcripts window for comparison
    recent_window: Duration,
    /// Configuration
    config: DeduplicationConfig,
}

/// Fuzzy matching trait for transcript similarity
pub trait FuzzyMatcher: Send + Sync {
    /// Calculate similarity between two texts (0.0 to 1.0)
    fn similarity(&self, text1: &str, text2: &str) -> f64;
    
    /// Check if texts are similar above threshold
    fn is_similar(&self, text1: &str, text2: &str, threshold: f64) -> bool {
        self.similarity(text1, text2) >= threshold
    }
}

/// Full-text search indexer
pub struct TranscriptIndexer {
    /// Word-based index for fast text search
    word_index: HashMap<String, HashSet<TranscriptId>>,
    /// Phrase index for multi-word searches
    phrase_index: HashMap<String, HashSet<TranscriptId>>,
    /// Tag index for metadata searches
    tag_index: HashMap<String, HashSet<TranscriptId>>,
    /// Date index for temporal searches
    date_index: HashMap<NaiveDate, HashSet<TranscriptId>>,
    /// Configuration
    config: IndexConfig,
}

/// Transcription analytics engine
pub struct TranscriptAnalytics {
    /// Word frequency analysis
    word_frequency: HashMap<String, u64>,
    /// Daily transcription statistics
    daily_stats: HashMap<NaiveDate, DailyStats>,
    /// Accuracy trend tracking
    accuracy_trends: Vec<AccuracyPoint>,
    /// Session correlation data
    session_correlations: HashMap<SessionId, Vec<TranscriptId>>,
    /// Configuration
    config: AnalyticsConfig,
}

/// Transcript entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEntry {
    /// Unique transcript identifier
    pub id: TranscriptId,
    /// Transcript creation timestamp
    pub timestamp: DateTime<Utc>,
    /// Transcribed text content
    pub text: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// STT model used
    pub model: String,
    /// Processing duration in milliseconds
    pub duration_ms: u64,
    /// Associated audio file ID
    pub audio_file_id: Option<AudioFileId>,
    /// Associated recording session ID
    pub session_id: Option<SessionId>,
    /// Content hash for deduplication
    pub hash: u64,
    /// User-defined tags
    pub tags: Vec<String>,
    /// Additional metadata
    pub metadata: TranscriptMetadata,
    /// Language detected/specified
    pub language: Option<String>,
    /// Speaker information (if available)
    pub speaker: Option<String>,
}

/// Additional transcript metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptMetadata {
    /// Source of the transcription
    pub source: TranscriptSource,
    /// Processing information
    pub processing_info: ProcessingInfo,
    /// Quality metrics
    pub quality_metrics: QualityMetrics,
    /// User annotations
    pub annotations: Vec<Annotation>,
}

/// Transcript source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranscriptSource {
    /// Live audio input
    LiveAudio,
    /// Recorded audio file
    RecordedAudio { file_path: PathBuf },
    /// Imported from external source
    Imported { source: String },
    /// Manual entry
    Manual,
}

/// Processing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingInfo {
    /// Processing start time
    pub start_time: DateTime<Utc>,
    /// Processing end time
    pub end_time: DateTime<Utc>,
    /// Model parameters used
    pub model_params: HashMap<String, String>,
    /// Processing warnings
    pub warnings: Vec<String>,
}

/// Quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Overall quality score (0.0 to 1.0)
    pub quality_score: f32,
    /// Word-level confidence scores
    pub word_confidences: Vec<f32>,
    /// Detected issues
    pub issues: Vec<QualityIssue>,
    /// Signal quality metrics
    pub signal_metrics: SignalMetrics,
}

/// Quality issues detected in transcription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityIssue {
    /// Low confidence words
    LowConfidence { word: String, confidence: f32 },
    /// Potential hallucination
    PossibleHallucination { text: String, reason: String },
    /// Audio quality issues
    AudioQuality { issue: String, severity: f32 },
    /// Language detection uncertainty
    LanguageUncertainty { detected: String, confidence: f32 },
}

/// Signal quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMetrics {
    /// Signal-to-noise ratio
    pub snr_db: f32,
    /// Audio level statistics
    pub audio_levels: AudioLevels,
    /// Frequency analysis
    pub frequency_analysis: FrequencyAnalysis,
}

/// Audio level statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioLevels {
    pub peak: f32,
    pub rms: f32,
    pub dynamic_range: f32,
}

/// Frequency analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyAnalysis {
    pub dominant_frequency: f32,
    pub frequency_spread: f32,
    pub spectral_centroid: f32,
}

/// User annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    /// Annotation ID
    pub id: Uuid,
    /// Annotation timestamp
    pub timestamp: DateTime<Utc>,
    /// Annotation type
    pub annotation_type: AnnotationType,
    /// Annotation content
    pub content: String,
    /// Position in transcript (character offset)
    pub position: Option<usize>,
    /// Length of annotated text
    pub length: Option<usize>,
}

/// Types of annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationType {
    /// Correction of transcription error
    Correction,
    /// Additional context or note
    Note,
    /// Highlight important section
    Highlight,
    /// Mark as questionable
    Question,
    /// Speaker identification
    Speaker,
    /// Timestamp marker
    Timestamp,
}

/// Deduplication result
#[derive(Debug, Clone)]
pub enum DuplicationResult {
    /// Transcript is unique
    Unique,
    /// Exact duplicate found
    ExactDuplicate(TranscriptId),
    /// Similar transcript found
    SimilarTranscript { id: TranscriptId, similarity: f64 },
}

/// Merged transcript from similar entries
#[derive(Debug, Clone)]
pub struct MergedTranscript {
    /// Primary transcript ID
    pub primary_id: TranscriptId,
    /// Merged text content
    pub merged_text: String,
    /// Combined confidence score
    pub combined_confidence: f32,
    /// Source transcript IDs
    pub source_ids: Vec<TranscriptId>,
    /// Merge strategy used
    pub merge_strategy: MergeStrategy,
}

/// Merge strategies for similar transcripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Use highest confidence transcript
    HighestConfidence,
    /// Combine text from all sources
    TextCombination,
    /// Use longest transcript
    Longest,
    /// Custom merge logic
    Custom(String),
}

/// Search criteria for transcripts
#[derive(Debug, Clone)]
pub struct SearchCriteria {
    /// Text search query
    pub query: Option<String>,
    /// Search type
    pub search_type: SearchType,
    /// Date range filter
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Confidence range filter
    pub confidence_range: Option<(f32, f32)>,
    /// Tag filters
    pub tags: Vec<String>,
    /// Session ID filter
    pub session_id: Option<SessionId>,
    /// Language filter
    pub language: Option<String>,
    /// Result limit
    pub limit: Option<usize>,
    /// Sort order
    pub sort_order: SortOrder,
}

/// Search types
#[derive(Debug, Clone)]
pub enum SearchType {
    /// Full-text search
    FullText,
    /// Exact phrase search
    ExactPhrase,
    /// Regular expression search
    Regex(Regex),
    /// Fuzzy search
    Fuzzy { threshold: f64 },
    /// Tag-based search
    Tags,
}

/// Sort order for search results
#[derive(Debug, Clone)]
pub enum SortOrder {
    /// Most recent first
    Newest,
    /// Oldest first
    Oldest,
    /// Highest confidence first
    HighestConfidence,
    /// Best relevance match
    Relevance,
    /// Longest text first
    Longest,
}

/// Transcript match result
#[derive(Debug, Clone)]
pub struct TranscriptMatch {
    /// Matched transcript
    pub transcript: TranscriptEntry,
    /// Relevance score (0.0 to 1.0)
    pub relevance: f64,
    /// Matched text snippets
    pub snippets: Vec<TextSnippet>,
    /// Match type
    pub match_type: MatchType,
}

/// Text snippet with highlighting
#[derive(Debug, Clone)]
pub struct TextSnippet {
    /// Snippet text
    pub text: String,
    /// Character position in original text
    pub position: usize,
    /// Highlighted portions
    pub highlights: Vec<(usize, usize)>,
}

/// Match type classification
#[derive(Debug, Clone)]
pub enum MatchType {
    /// Exact text match
    Exact,
    /// Fuzzy text match
    Fuzzy { similarity: f64 },
    /// Tag match
    Tag,
    /// Metadata match
    Metadata,
}

/// Daily transcription statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    /// Date
    pub date: NaiveDate,
    /// Total transcripts
    pub total_transcripts: usize,
    /// Total words transcribed
    pub total_words: usize,
    /// Average confidence
    pub average_confidence: f32,
    /// Total processing time
    pub total_processing_time: Duration,
    /// Unique sessions
    pub unique_sessions: usize,
    /// Most active hour
    pub peak_hour: u32,
}

/// Accuracy tracking point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Accuracy score
    pub accuracy: f32,
    /// Sample size
    pub sample_size: usize,
    /// Model used
    pub model: String,
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct TranscriptStorageStats {
    /// Total transcripts stored
    pub total_transcripts: usize,
    /// Total storage size in bytes
    pub storage_size_bytes: u64,
    /// Total words stored
    pub total_words: usize,
    /// Average transcript length
    pub average_length: f32,
    /// Oldest transcript date
    pub oldest_transcript: Option<DateTime<Utc>>,
    /// Newest transcript date
    pub newest_transcript: Option<DateTime<Utc>>,
    /// Deduplication savings
    pub deduplication_savings: DeduplicationSavings,
}

/// Deduplication savings metrics
#[derive(Debug, Clone)]
pub struct DeduplicationSavings {
    /// Duplicates detected
    pub duplicates_detected: usize,
    /// Storage saved in bytes
    pub storage_saved_bytes: u64,
    /// Processing time saved
    pub processing_time_saved: Duration,
}

/// Backup operation result
#[derive(Debug, Clone)]
pub struct BackupResult {
    /// Number of transcripts backed up
    pub transcripts_backed_up: usize,
    /// Backup file size
    pub backup_size_bytes: u64,
    /// Backup duration
    pub backup_duration: Duration,
    /// Backup file path
    pub backup_path: PathBuf,
}

/// Restore operation result
#[derive(Debug, Clone)]
pub struct RestoreResult {
    /// Number of transcripts restored
    pub transcripts_restored: usize,
    /// Number of duplicates skipped
    pub duplicates_skipped: usize,
    /// Restore duration
    pub restore_duration: Duration,
}

/// Configuration types
pub type TranscriptId = Uuid;

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionLogConfig {
    /// Storage directory
    pub storage_path: PathBuf,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Auto-save interval
    pub auto_save_interval: Duration,
    /// Enable deduplication
    pub enable_deduplication: bool,
    /// Enable analytics
    pub enable_analytics: bool,
    /// Backup configuration
    pub backup_config: Option<BackupConfig>,
}

/// Deduplication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationConfig {
    /// Similarity threshold (0.0 to 1.0)
    pub similarity_threshold: f64,
    /// Recent window for comparison
    pub recent_window_minutes: u32,
    /// Enable fuzzy matching
    pub enable_fuzzy_matching: bool,
    /// Hash algorithm
    pub hash_algorithm: HashAlgorithm,
}

/// Hash algorithms for deduplication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HashAlgorithm {
    /// Simple hash
    Simple,
    /// Content-based hash
    ContentBased,
    /// Semantic hash
    Semantic,
}

/// Index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Enable word indexing
    pub enable_word_index: bool,
    /// Enable phrase indexing
    pub enable_phrase_index: bool,
    /// Maximum phrase length
    pub max_phrase_length: usize,
    /// Stop words to ignore
    pub stop_words: HashSet<String>,
    /// Minimum word length
    pub min_word_length: usize,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// Enable word frequency analysis
    pub enable_word_frequency: bool,
    /// Enable daily statistics
    pub enable_daily_stats: bool,
    /// Enable accuracy tracking
    pub enable_accuracy_tracking: bool,
    /// Retention period for analytics
    pub retention_days: u32,
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Enable automatic backups
    pub enabled: bool,
    /// Backup directory
    pub backup_directory: PathBuf,
    /// Backup interval
    pub backup_interval: Duration,
    /// Maximum backup files to keep
    pub max_backup_files: usize,
    /// Compress backups
    pub compress_backups: bool,
}

/// Transcription errors
#[derive(Debug, Error)]
pub enum TranscriptError {
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Deduplication error: {0}")]
    DeduplicationError(String),
    
    #[error("Search error: {0}")]
    SearchError(String),
    
    #[error("Analytics error: {0}")]
    AnalyticsError(String),
    
    #[error("Transcript not found: {0}")]
    TranscriptNotFound(TranscriptId),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
}

// Implementation of TranscriptionLogService
impl TranscriptionLogService {
    /// Create a new transcription log service
    pub fn new(
        storage: Box<dyn TranscriptStorage>,
        config: TranscriptionLogConfig,
    ) -> Result<Self, TranscriptError> {
        // Ensure storage directory exists
        if !config.storage_path.exists() {
            fs::create_dir_all(&config.storage_path)?;
        }
        
        let deduplicator = TranscriptDeduplicator::new(DeduplicationConfig {
            similarity_threshold: 0.85,
            recent_window_minutes: 10,
            enable_fuzzy_matching: true,
            hash_algorithm: HashAlgorithm::ContentBased,
        });
        
        let indexer = TranscriptIndexer::new(IndexConfig {
            enable_word_index: true,
            enable_phrase_index: true,
            max_phrase_length: 5,
            stop_words: Self::default_stop_words(),
            min_word_length: 2,
        });
        
        let analytics = TranscriptAnalytics::new(AnalyticsConfig {
            enable_word_frequency: true,
            enable_daily_stats: true,
            enable_accuracy_tracking: true,
            retention_days: 365,
        });
        
        Ok(Self {
            storage,
            deduplicator,
            indexer,
            analytics,
            config,
            recent_cache: VecDeque::new(),
        })
    }
    
    /// Log a new transcription
    pub fn log_transcription(&mut self, text: &str, confidence: f32, model: &str, duration_ms: u64) -> Result<TranscriptEntry, TranscriptError> {
        let entry = TranscriptEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            text: text.to_string(),
            confidence,
            model: model.to_string(),
            duration_ms,
            audio_file_id: None,
            session_id: None,
            hash: self.calculate_hash(text),
            tags: Vec::new(),
            metadata: TranscriptMetadata {
                source: TranscriptSource::LiveAudio,
                processing_info: ProcessingInfo {
                    start_time: Utc::now(),
                    end_time: Utc::now(),
                    model_params: HashMap::new(),
                    warnings: Vec::new(),
                },
                quality_metrics: QualityMetrics {
                    quality_score: confidence,
                    word_confidences: Vec::new(),
                    issues: Vec::new(),
                    signal_metrics: SignalMetrics {
                        snr_db: 0.0,
                        audio_levels: AudioLevels {
                            peak: 0.0,
                            rms: 0.0,
                            dynamic_range: 0.0,
                        },
                        frequency_analysis: FrequencyAnalysis {
                            dominant_frequency: 0.0,
                            frequency_spread: 0.0,
                            spectral_centroid: 0.0,
                        },
                    },
                },
                annotations: Vec::new(),
            },
            language: None,
            speaker: None,
        };
        
        // Check for duplicates if enabled
        if self.config.enable_deduplication {
            match self.deduplicator.is_duplicate(text)? {
                DuplicationResult::ExactDuplicate(existing_id) => {
                    println!("🔄 Exact duplicate detected, skipping: {}", existing_id);
                    return self.storage.get_transcript(existing_id)?
                        .ok_or_else(|| TranscriptError::TranscriptNotFound(existing_id));
                }
                DuplicationResult::SimilarTranscript { id, similarity } => {
                    println!("🔄 Similar transcript detected ({}% similarity): {}", 
                            (similarity * 100.0) as u32, id);
                    // For now, still store but could implement merging logic
                }
                DuplicationResult::Unique => {
                    // Continue with storage
                }
            }
        }
        
        // Store the transcript
        let transcript_id = self.storage.store_transcript(&entry)?;
        
        // Update indexes
        if self.config.enable_analytics {
            self.indexer.index_transcript(&entry)?;
            self.analytics.update_with_transcript(&entry)?;
        }
        
        // Update cache
        self.recent_cache.push_back(entry.clone());
        if self.recent_cache.len() > self.config.max_cache_size {
            self.recent_cache.pop_front();
        }
        
        println!("📝 Logged transcription: {} characters, {:.1}% confidence", 
                text.len(), confidence * 100.0);
        
        Ok(entry)
    }
    
    /// Search transcripts
    pub fn search_transcripts(&self, criteria: &SearchCriteria) -> Result<Vec<TranscriptMatch>, TranscriptError> {
        self.indexer.search(criteria)
    }
    
    /// Get transcription analytics
    pub fn get_analytics(&self) -> &TranscriptAnalytics {
        &self.analytics
    }
    
    /// Get storage statistics
    pub fn get_storage_stats(&self) -> TranscriptStorageStats {
        self.storage.get_storage_stats()
    }
    
    /// Export transcripts to file
    pub fn export_transcripts(&self, criteria: &SearchCriteria, output_path: &Path) -> Result<usize, TranscriptError> {
        let transcripts = self.storage.search_transcripts(criteria)?;
        
        let mut file = File::create(output_path)?;
        let mut count = 0;
        
        for transcript in transcripts {
            writeln!(file, "=== Transcript {} ===", transcript.id)?;
            writeln!(file, "Timestamp: {}", transcript.timestamp.format("%Y-%m-%d %H:%M:%S UTC"))?;
            writeln!(file, "Confidence: {:.1}%", transcript.confidence * 100.0)?;
            writeln!(file, "Model: {}", transcript.model)?;
            if let Some(session_id) = transcript.session_id {
                writeln!(file, "Session: {}", session_id)?;
            }
            writeln!(file, "Text: {}", transcript.text)?;
            writeln!(file)?;
            count += 1;
        }
        
        println!("📤 Exported {} transcripts to {}", count, output_path.display());
        Ok(count)
    }
    
    /// Backup transcripts
    pub fn backup_transcripts(&self, backup_path: &Path) -> Result<BackupResult, TranscriptError> {
        self.storage.backup_transcripts(backup_path)
    }
    
    /// Calculate content hash
    fn calculate_hash(&self, text: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.to_lowercase().trim().hash(&mut hasher);
        hasher.finish()
    }
    
    /// Default stop words for indexing
    fn default_stop_words() -> HashSet<String> {
        vec![
            "a", "an", "and", "are", "as", "at", "be", "by", "for", "from",
            "has", "he", "in", "is", "it", "its", "of", "on", "that", "the",
            "to", "was", "will", "with", "the", "this", "but", "they", "have",
            "had", "what", "said", "each", "which", "she", "do", "how", "their",
            "if", "up", "out", "many", "then", "them", "these", "so", "some",
            "her", "would", "make", "like", "into", "him", "time", "two", "more",
            "go", "no", "way", "could", "my", "than", "first", "been", "call",
            "who", "oil", "sit", "now", "find", "down", "day", "did", "get",
            "come", "made", "may", "part"
        ].into_iter().map(|s| s.to_string()).collect()
    }
}

// Default implementations
impl Default for TranscriptionLogConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from("./transcription_logs"),
            max_cache_size: 1000,
            auto_save_interval: Duration::from_secs(300), // 5 minutes
            enable_deduplication: true,
            enable_analytics: true,
            backup_config: None,
        }
    }
}

impl Default for SearchCriteria {
    fn default() -> Self {
        Self {
            query: None,
            search_type: SearchType::FullText,
            date_range: None,
            confidence_range: None,
            tags: Vec::new(),
            session_id: None,
            language: None,
            limit: Some(50),
            sort_order: SortOrder::Newest,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    // Mock implementations for testing
    struct MockTranscriptStorage {
        transcripts: HashMap<TranscriptId, TranscriptEntry>,
    }
    
    impl TranscriptStorage for MockTranscriptStorage {
        fn store_transcript(&mut self, entry: &TranscriptEntry) -> Result<TranscriptId, TranscriptError> {
            self.transcripts.insert(entry.id, entry.clone());
            Ok(entry.id)
        }
        
        fn get_transcript(&self, id: TranscriptId) -> Result<Option<TranscriptEntry>, TranscriptError> {
            Ok(self.transcripts.get(&id).cloned())
        }
        
        fn search_transcripts(&self, _criteria: &SearchCriteria) -> Result<Vec<TranscriptEntry>, TranscriptError> {
            Ok(self.transcripts.values().cloned().collect())
        }
        
        fn delete_transcript(&mut self, id: TranscriptId) -> Result<(), TranscriptError> {
            self.transcripts.remove(&id);
            Ok(())
        }
        
        fn get_storage_stats(&self) -> TranscriptStorageStats {
            TranscriptStorageStats {
                total_transcripts: self.transcripts.len(),
                storage_size_bytes: 0,
                total_words: 0,
                average_length: 0.0,
                oldest_transcript: None,
                newest_transcript: None,
                deduplication_savings: DeduplicationSavings {
                    duplicates_detected: 0,
                    storage_saved_bytes: 0,
                    processing_time_saved: Duration::from_secs(0),
                },
            }
        }
        
        fn backup_transcripts(&self, _path: &Path) -> Result<BackupResult, TranscriptError> {
            Ok(BackupResult {
                transcripts_backed_up: self.transcripts.len(),
                backup_size_bytes: 1024,
                backup_duration: Duration::from_secs(1),
                backup_path: PathBuf::from("test_backup.json"),
            })
        }
        
        fn restore_transcripts(&mut self, _path: &Path) -> Result<RestoreResult, TranscriptError> {
            Ok(RestoreResult {
                transcripts_restored: 0,
                duplicates_skipped: 0,
                restore_duration: Duration::from_secs(1),
            })
        }
    }
    
    #[test]
    fn test_transcription_log_service_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = TranscriptionLogConfig {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let storage = Box::new(MockTranscriptStorage {
            transcripts: HashMap::new(),
        });
        
        let service = TranscriptionLogService::new(storage, config);
        assert!(service.is_ok());
    }
    
    #[test]
    fn test_log_transcription() {
        let temp_dir = TempDir::new().unwrap();
        let config = TranscriptionLogConfig {
            storage_path: temp_dir.path().to_path_buf(),
            enable_deduplication: false, // Disable for simple test
            ..Default::default()
        };
        
        let storage = Box::new(MockTranscriptStorage {
            transcripts: HashMap::new(),
        });
        
        let mut service = TranscriptionLogService::new(storage, config).unwrap();
        
        let result = service.log_transcription(
            "Hello, this is a test transcription.",
            0.95,
            "whisper-base",
            1500
        );
        
        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.text, "Hello, this is a test transcription.");
        assert_eq!(entry.confidence, 0.95);
        assert_eq!(entry.model, "whisper-base");
    }
}
