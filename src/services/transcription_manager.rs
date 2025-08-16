//! Transcription Manager - Integration Service for Phase 3
//! 
//! This module provides a unified interface for all transcription-related services,
//! integrating logging, deduplication, search, analytics, and voice commands.

use std::path::PathBuf;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};

use super::transcription_log::{
    TranscriptionLogService, TranscriptionLogConfig, TranscriptEntry, TranscriptError,
    SearchCriteria, TranscriptMatch, TranscriptStorageStats
};
use super::transcription_deduplication::TranscriptDeduplicator;
use super::transcription_log::{DeduplicationConfig, DuplicationResult};
use super::transcription_search::TranscriptIndexer;
use super::transcription_log::IndexConfig;
use super::transcription_analytics::{TranscriptAnalytics, AnalyticsReport};
use super::transcription_log::AnalyticsConfig;
use super::audio_archive::SessionId;

/// Unified transcription management service
pub struct TranscriptionManager {
    /// Core logging service
    log_service: TranscriptionLogService,
    /// Deduplication engine
    deduplicator: TranscriptDeduplicator,
    /// Search indexer
    indexer: TranscriptIndexer,
    /// Analytics engine
    analytics: TranscriptAnalytics,
    /// Manager configuration
    config: TranscriptionManagerConfig,
    /// Performance metrics
    metrics: ManagerMetrics,
}

/// Configuration for the transcription manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionManagerConfig {
    /// Storage directory for all transcription data
    pub storage_path: PathBuf,
    /// Enable comprehensive logging
    pub enable_logging: bool,
    /// Enable intelligent deduplication
    pub enable_deduplication: bool,
    /// Enable full-text search indexing
    pub enable_search: bool,
    /// Enable analytics and insights
    pub enable_analytics: bool,
    /// Auto-save interval
    pub auto_save_interval: Duration,
    /// Maximum cache size
    pub max_cache_size: usize,
}

/// Performance metrics for the manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerMetrics {
    /// Total transcriptions processed
    pub total_processed: usize,
    /// Total duplicates detected
    pub duplicates_detected: usize,
    /// Average processing time
    pub average_processing_time: Duration,
    /// Search queries performed
    pub search_queries: usize,
    /// Analytics reports generated
    pub reports_generated: usize,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

/// Comprehensive transcription result
#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    /// The transcript entry
    pub entry: TranscriptEntry,
    /// Deduplication result
    pub deduplication: DuplicationResult,
    /// Processing time
    pub processing_time: Duration,
    /// Was the transcript stored?
    pub stored: bool,
    /// Any warnings or issues
    pub warnings: Vec<String>,
}

impl TranscriptionManager {
    /// Create a new transcription manager
    pub fn new(config: TranscriptionManagerConfig) -> Result<Self, TranscriptError> {
        println!("🚀 Initializing Transcription Manager...");
        let start_time = Instant::now();

        // Ensure storage directory exists
        if !config.storage_path.exists() {
            std::fs::create_dir_all(&config.storage_path)
                .map_err(|e| TranscriptError::StorageError(format!("Failed to create storage directory: {}", e)))?;
        }

        // Initialize logging service
        let log_config = TranscriptionLogConfig {
            storage_path: config.storage_path.join("transcripts"),
            max_cache_size: config.max_cache_size,
            auto_save_interval: config.auto_save_interval,
            enable_deduplication: config.enable_deduplication,
            enable_analytics: config.enable_analytics,
            backup_config: None,
        };

        let log_service = if config.enable_logging {
            TranscriptionLogService::new(log_config)?
        } else {
            // Create a minimal service for testing
            TranscriptionLogService::new(log_config)?
        };

        // Initialize deduplication engine
        let dedup_config = DeduplicationConfig {
            similarity_threshold: 0.85,
            recent_window_minutes: 10,
            enable_fuzzy_matching: true,
            hash_algorithm: super::transcription_log::HashAlgorithm::ContentBased,
        };

        let deduplicator = if config.enable_deduplication {
            TranscriptDeduplicator::new(dedup_config)
        } else {
            TranscriptDeduplicator::new(dedup_config)
        };

        // Initialize search indexer
        let index_config = IndexConfig {
            enable_word_index: config.enable_search,
            enable_phrase_index: config.enable_search,
            max_phrase_length: 5,
            stop_words: Self::default_stop_words(),
            min_word_length: 2,
        };

        let indexer = TranscriptIndexer::new(index_config);

        // Initialize analytics engine
        let analytics_config = AnalyticsConfig {
            enable_word_frequency: config.enable_analytics,
            enable_daily_stats: config.enable_analytics,
            enable_accuracy_tracking: config.enable_analytics,
            retention_days: 365,
        };

        let analytics = TranscriptAnalytics::new(analytics_config);

        let metrics = ManagerMetrics {
            total_processed: 0,
            duplicates_detected: 0,
            average_processing_time: Duration::from_millis(0),
            search_queries: 0,
            reports_generated: 0,
            last_updated: Utc::now(),
        };

        let init_time = start_time.elapsed();
        println!("✅ Transcription Manager initialized in {:.2}ms", init_time.as_millis());

        Ok(Self {
            log_service,
            deduplicator,
            indexer,
            analytics,
            config,
            metrics,
        })
    }

    /// Process a new transcription with full pipeline
    pub fn process_transcription(
        &mut self,
        text: &str,
        confidence: f32,
        model: &str,
        duration_ms: u64,
        session_id: Option<SessionId>,
    ) -> Result<TranscriptionResult, TranscriptError> {
        let start_time = Instant::now();
        let mut warnings = Vec::new();

        println!("📝 Processing transcription: {} characters, {:.1}% confidence", 
                text.len(), confidence * 100.0);

        // Step 1: Check for duplicates
        let deduplication_result = if self.config.enable_deduplication {
            match self.deduplicator.is_duplicate(text)? {
                DuplicationResult::ExactDuplicate(existing_id) => {
                    println!("🔄 Exact duplicate detected: {}", existing_id);
                    self.metrics.duplicates_detected += 1;
                    
                    // Return existing transcript
                    if let Some(existing_entry) = self.log_service.get_transcript(existing_id)? {
                        return Ok(TranscriptionResult {
                            entry: existing_entry,
                            deduplication: DuplicationResult::ExactDuplicate(existing_id),
                            processing_time: start_time.elapsed(),
                            stored: false,
                            warnings: vec!["Exact duplicate found, returning existing transcript".to_string()],
                        });
                    }
                    DuplicationResult::ExactDuplicate(existing_id)
                }
                DuplicationResult::SimilarTranscript { id, similarity } => {
                    println!("🔄 Similar transcript detected: {} ({:.1}% similarity)", id, similarity * 100.0);
                    warnings.push(format!("Similar transcript found with {:.1}% similarity", similarity * 100.0));
                    DuplicationResult::SimilarTranscript { id, similarity }
                }
                DuplicationResult::Unique => {
                    println!("✨ Unique transcription detected");
                    DuplicationResult::Unique
                }
            }
        } else {
            DuplicationResult::Unique
        };

        // Step 2: Create transcript entry
        let mut entry = self.create_transcript_entry(text, confidence, model, duration_ms, session_id);

        // Step 3: Store transcript
        let stored = if self.config.enable_logging {
            match self.log_service.log_transcription(text, confidence, model, duration_ms) {
                Ok(logged_entry) => {
                    entry = logged_entry;
                    true
                }
                Err(e) => {
                    warnings.push(format!("Failed to store transcript: {}", e));
                    false
                }
            }
        } else {
            false
        };

        // Step 4: Update deduplication cache
        if self.config.enable_deduplication && stored {
            self.deduplicator.add_transcript(&entry);
        }

        // Step 5: Update search index
        if self.config.enable_search && stored {
            if let Err(e) = self.indexer.index_transcript(&entry) {
                warnings.push(format!("Failed to index transcript: {}", e));
            }
        }

        // Step 6: Update analytics
        if self.config.enable_analytics && stored {
            if let Err(e) = self.analytics.update_with_transcript(&entry) {
                warnings.push(format!("Failed to update analytics: {}", e));
            }
        }

        // Step 7: Update metrics
        self.metrics.total_processed += 1;
        let processing_time = start_time.elapsed();
        let avg_millis = (self.metrics.average_processing_time.as_millis() * (self.metrics.total_processed - 1) as u128
                         + processing_time.as_millis()) / self.metrics.total_processed as u128;
        self.metrics.average_processing_time = Duration::from_millis(avg_millis.min(u64::MAX as u128) as u64);
        self.metrics.last_updated = Utc::now();

        println!("✅ Transcription processed in {:.2}ms", processing_time.as_millis());

        Ok(TranscriptionResult {
            entry,
            deduplication: deduplication_result,
            processing_time,
            stored,
            warnings,
        })
    }

    /// Search transcripts with comprehensive criteria
    pub fn search_transcripts(&mut self, criteria: &SearchCriteria) -> Result<Vec<TranscriptMatch>, TranscriptError> {
        if !self.config.enable_search {
            return Err(TranscriptError::SearchError("Search is disabled".to_string()));
        }

        self.metrics.search_queries += 1;
        self.indexer.search(criteria)
    }

    /// Get comprehensive analytics report
    pub fn generate_analytics_report(&mut self) -> Result<AnalyticsReport, TranscriptError> {
        if !self.config.enable_analytics {
            return Err(TranscriptError::AnalyticsError("Analytics is disabled".to_string()));
        }

        self.metrics.reports_generated += 1;
        Ok(self.analytics.generate_report())
    }

    /// Get storage statistics
    pub fn get_storage_stats(&self) -> TranscriptStorageStats {
        self.log_service.get_storage_stats()
    }

    /// Get manager performance metrics
    pub fn get_metrics(&self) -> &ManagerMetrics {
        &self.metrics
    }

    /// Export transcripts to file
    pub fn export_transcripts(&self, criteria: &SearchCriteria, output_path: &std::path::Path) -> Result<usize, TranscriptError> {
        self.log_service.export_transcripts(criteria, output_path)
    }

    /// Create backup of all transcription data
    pub fn create_backup(&self, backup_path: &std::path::Path) -> Result<super::transcription_log::BackupResult, TranscriptError> {
        self.log_service.backup_transcripts(backup_path)
    }

    /// Get recent transcripts
    pub fn get_recent_transcripts(&mut self, limit: usize) -> Result<Vec<TranscriptMatch>, TranscriptError> {
        let criteria = SearchCriteria {
            query: None,
            search_type: super::transcription_log::SearchType::FullText,
            date_range: None,
            confidence_range: None,
            tags: Vec::new(),
            session_id: None,
            language: None,
            limit: Some(limit),
            sort_order: super::transcription_log::SortOrder::Newest,
        };

        self.search_transcripts(&criteria)
    }

    /// Get word frequency analysis
    pub fn get_word_frequency(&self, limit: Option<usize>) -> Vec<(String, u64)> {
        if !self.config.enable_analytics {
            return Vec::new();
        }

        self.analytics.get_word_frequency(limit)
            .into_iter()
            .map(|(word, stats)| (word.clone(), stats.frequency))
            .collect()
    }

    /// Get transcription accuracy trends
    pub fn get_accuracy_trends(&self) -> Vec<super::transcription_log::AccuracyPoint> {
        if !self.config.enable_analytics {
            return Vec::new();
        }

        self.analytics.get_accuracy_trends().iter().cloned().collect()
    }

    /// Delete duplicate transcripts
    pub fn delete_duplicates(&mut self) -> Result<usize, TranscriptError> {
        // This would require implementing duplicate detection and removal
        // For now, return the count from deduplication stats
        Ok(self.deduplicator.get_stats().total_duplicates)
    }

    /// Get transcript by ID
    pub fn get_transcript(&self, id: super::transcription_log::TranscriptId) -> Result<Option<TranscriptEntry>, TranscriptError> {
        self.log_service.get_transcript(id)
    }

    // Private helper methods

    fn create_transcript_entry(
        &self,
        text: &str,
        confidence: f32,
        model: &str,
        duration_ms: u64,
        session_id: Option<SessionId>,
    ) -> TranscriptEntry {
        use uuid::Uuid;
        use std::collections::HashMap;
        use super::transcription_log::*;

        TranscriptEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            text: text.to_string(),
            confidence,
            model: model.to_string(),
            duration_ms,
            audio_file_id: None,
            session_id,
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
        }
    }

    fn calculate_hash(&self, text: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.to_lowercase().trim().hash(&mut hasher);
        hasher.finish()
    }

    fn default_stop_words() -> std::collections::HashSet<String> {
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

impl Default for TranscriptionManagerConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from("./transcription_data"),
            enable_logging: true,
            enable_deduplication: true,
            enable_search: true,
            enable_analytics: true,
            auto_save_interval: Duration::from_secs(300), // 5 minutes
            max_cache_size: 1000,
        }
    }
}

impl Default for ManagerMetrics {
    fn default() -> Self {
        Self {
            total_processed: 0,
            duplicates_detected: 0,
            average_processing_time: Duration::from_millis(0),
            search_queries: 0,
            reports_generated: 0,
            last_updated: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_transcription_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = TranscriptionManagerConfig {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = TranscriptionManager::new(config);
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert_eq!(manager.metrics.total_processed, 0);
    }

    #[test]
    fn test_process_transcription() {
        let temp_dir = TempDir::new().unwrap();
        let config = TranscriptionManagerConfig {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut manager = TranscriptionManager::new(config).unwrap();

        let result = manager.process_transcription(
            "Hello, this is a test transcription.",
            0.95,
            "whisper-base",
            1500,
            None,
        );

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.stored);
        assert_eq!(result.entry.text, "Hello, this is a test transcription.");
        assert_eq!(result.entry.confidence, 0.95);
    }

    #[test]
    fn test_duplicate_detection() {
        let temp_dir = TempDir::new().unwrap();
        let config = TranscriptionManagerConfig {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut manager = TranscriptionManager::new(config).unwrap();

        // Process first transcription
        let result1 = manager.process_transcription(
            "Hello, world!",
            0.95,
            "whisper-base",
            1000,
            None,
        ).unwrap();
        assert!(result1.stored);

        // Process identical transcription
        let result2 = manager.process_transcription(
            "Hello, world!",
            0.95,
            "whisper-base",
            1000,
            None,
        ).unwrap();
        
        // Should detect as duplicate
        assert!(matches!(result2.deduplication, DuplicationResult::ExactDuplicate(_)));
        assert!(!result2.stored);
    }

    #[test]
    fn test_search_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let config = TranscriptionManagerConfig {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut manager = TranscriptionManager::new(config).unwrap();

        // Add some transcriptions
        manager.process_transcription("Hello, world!", 0.95, "whisper-base", 1000, None).unwrap();
        manager.process_transcription("Goodbye, world!", 0.85, "whisper-base", 1200, None).unwrap();
        manager.process_transcription("Hello, universe!", 0.90, "whisper-base", 1100, None).unwrap();

        // Search for "hello"
        let criteria = SearchCriteria {
            query: Some("hello".to_string()),
            search_type: super::transcription_log::SearchType::FullText,
            ..Default::default()
        };

        let results = manager.search_transcripts(&criteria).unwrap();
        assert_eq!(results.len(), 2); // Should find 2 transcripts with "hello"
    }

    #[test]
    fn test_analytics_report() {
        let temp_dir = TempDir::new().unwrap();
        let config = TranscriptionManagerConfig {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut manager = TranscriptionManager::new(config).unwrap();

        // Add some transcriptions
        manager.process_transcription("Hello, world!", 0.95, "whisper-base", 1000, None).unwrap();
        manager.process_transcription("Test transcription", 0.85, "whisper-base", 1200, None).unwrap();

        // Generate analytics report
        let report = manager.generate_analytics_report().unwrap();
        assert_eq!(report.total_transcripts, 2);
        assert!(report.average_confidence > 0.0);
    }
}
