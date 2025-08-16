//! File-based Transcript Storage Implementation
//! 
//! This module provides a concrete implementation of the TranscriptStorage trait
//! using JSON files for persistence with efficient indexing and search capabilities.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Write, BufRead};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;

use super::transcription_log::{
    TranscriptStorage, TranscriptEntry, TranscriptId, SearchCriteria, TranscriptError,
    TranscriptStorageStats, BackupResult, RestoreResult, DeduplicationSavings
};

/// File-based transcript storage implementation
pub struct FileTranscriptStorage {
    /// Storage directory path
    storage_path: PathBuf,
    /// Index file path
    index_path: PathBuf,
    /// In-memory index for fast lookups
    index: TranscriptIndex,
    /// Storage statistics
    stats: TranscriptStorageStats,
    /// Configuration
    config: FileStorageConfig,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStorageConfig {
    /// Maximum transcripts per file
    pub max_transcripts_per_file: usize,
    /// Enable compression
    pub enable_compression: bool,
    /// Auto-flush interval
    pub auto_flush_interval: Duration,
    /// Index rebuild threshold
    pub index_rebuild_threshold: usize,
}

/// In-memory index for fast lookups
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TranscriptIndex {
    /// Map from transcript ID to file location
    transcript_locations: HashMap<TranscriptId, FileLocation>,
    /// File metadata
    file_metadata: HashMap<String, FileMetadata>,
    /// Last update timestamp
    last_updated: DateTime<Utc>,
}

/// File location information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileLocation {
    /// File name
    file_name: String,
    /// Offset in file (for future optimization)
    offset: Option<u64>,
    /// Size in bytes
    size: Option<u64>,
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileMetadata {
    /// Number of transcripts in file
    transcript_count: usize,
    /// File size in bytes
    file_size: u64,
    /// Creation timestamp
    created_at: DateTime<Utc>,
    /// Last modified timestamp
    modified_at: DateTime<Utc>,
}

/// Storage file format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StorageFile {
    /// File format version
    version: String,
    /// Creation timestamp
    created_at: DateTime<Utc>,
    /// Transcripts in this file
    transcripts: Vec<TranscriptEntry>,
}

impl FileTranscriptStorage {
    /// Create a new file-based transcript storage
    pub fn new(storage_path: PathBuf, config: FileStorageConfig) -> Result<Self, TranscriptError> {
        // Ensure storage directory exists
        if !storage_path.exists() {
            fs::create_dir_all(&storage_path)
                .map_err(|e| TranscriptError::StorageError(format!("Failed to create storage directory: {}", e)))?;
        }

        let index_path = storage_path.join("index.json");
        
        // Load or create index
        let index = if index_path.exists() {
            Self::load_index(&index_path)?
        } else {
            TranscriptIndex {
                transcript_locations: HashMap::new(),
                file_metadata: HashMap::new(),
                last_updated: Utc::now(),
            }
        };

        // Calculate initial statistics
        let stats = Self::calculate_stats(&storage_path, &index)?;

        let mut storage = Self {
            storage_path,
            index_path,
            index,
            stats,
            config,
        };

        // Verify index integrity
        storage.verify_index()?;

        println!("📁 Initialized file transcript storage at {}", storage.storage_path.display());
        println!("📊 Found {} transcripts in {} files", storage.stats.total_transcripts, storage.index.file_metadata.len());

        Ok(storage)
    }

    /// Load index from file
    fn load_index(index_path: &Path) -> Result<TranscriptIndex, TranscriptError> {
        let file = File::open(index_path)
            .map_err(|e| TranscriptError::StorageError(format!("Failed to open index file: {}", e)))?;
        
        let reader = BufReader::new(file);
        let index: TranscriptIndex = serde_json::from_reader(reader)
            .map_err(|e| TranscriptError::StorageError(format!("Failed to parse index file: {}", e)))?;
        
        Ok(index)
    }

    /// Save index to file
    fn save_index(&self) -> Result<(), TranscriptError> {
        let file = File::create(&self.index_path)
            .map_err(|e| TranscriptError::StorageError(format!("Failed to create index file: {}", e)))?;
        
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.index)
            .map_err(|e| TranscriptError::StorageError(format!("Failed to write index file: {}", e)))?;
        
        Ok(())
    }

    /// Calculate storage statistics
    fn calculate_stats(storage_path: &Path, index: &TranscriptIndex) -> Result<TranscriptStorageStats, TranscriptError> {
        let mut total_size = 0u64;
        let mut oldest_transcript: Option<DateTime<Utc>> = None;
        let mut newest_transcript: Option<DateTime<Utc>> = None;

        for metadata in index.file_metadata.values() {
            total_size += metadata.file_size;
            
            if oldest_transcript.is_none() || metadata.created_at < oldest_transcript.unwrap() {
                oldest_transcript = Some(metadata.created_at);
            }
            
            if newest_transcript.is_none() || metadata.modified_at > newest_transcript.unwrap() {
                newest_transcript = Some(metadata.modified_at);
            }
        }

        let total_transcripts = index.transcript_locations.len();
        let average_length = if total_transcripts > 0 {
            total_size as f32 / total_transcripts as f32
        } else {
            0.0
        };

        Ok(TranscriptStorageStats {
            total_transcripts,
            storage_size_bytes: total_size,
            total_words: 0, // Would need to calculate from actual transcripts
            average_length,
            oldest_transcript,
            newest_transcript,
            deduplication_savings: DeduplicationSavings {
                duplicates_detected: 0,
                storage_saved_bytes: 0,
                processing_time_saved: Duration::from_secs(0),
            },
        })
    }

    /// Verify index integrity
    fn verify_index(&mut self) -> Result<(), TranscriptError> {
        let mut corrupted_files = Vec::new();
        
        for (file_name, _metadata) in &self.index.file_metadata {
            let file_path = self.storage_path.join(file_name);
            if !file_path.exists() {
                corrupted_files.push(file_name.clone());
            }
        }

        // Remove corrupted file entries
        for file_name in corrupted_files {
            println!("⚠️  Removing corrupted file from index: {}", file_name);
            self.index.file_metadata.remove(&file_name);
            
            // Remove transcript locations for this file
            self.index.transcript_locations.retain(|_, location| {
                location.file_name != file_name
            });
        }

        if !self.index.file_metadata.is_empty() {
            self.save_index()?;
        }

        Ok(())
    }

    /// Get next available file name
    fn get_next_file_name(&self) -> String {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let mut counter = 0;
        
        loop {
            let file_name = if counter == 0 {
                format!("transcripts_{}.json", timestamp)
            } else {
                format!("transcripts_{}_{}.json", timestamp, counter)
            };
            
            if !self.index.file_metadata.contains_key(&file_name) {
                return file_name;
            }
            
            counter += 1;
        }
    }

    /// Load storage file
    fn load_storage_file(&self, file_name: &str) -> Result<StorageFile, TranscriptError> {
        let file_path = self.storage_path.join(file_name);
        let file = File::open(&file_path)
            .map_err(|e| TranscriptError::StorageError(format!("Failed to open storage file {}: {}", file_name, e)))?;
        
        let reader = BufReader::new(file);
        let storage_file: StorageFile = serde_json::from_reader(reader)
            .map_err(|e| TranscriptError::StorageError(format!("Failed to parse storage file {}: {}", file_name, e)))?;
        
        Ok(storage_file)
    }

    /// Save storage file
    fn save_storage_file(&mut self, file_name: &str, storage_file: &StorageFile) -> Result<(), TranscriptError> {
        let file_path = self.storage_path.join(file_name);
        let file = File::create(&file_path)
            .map_err(|e| TranscriptError::StorageError(format!("Failed to create storage file {}: {}", file_name, e)))?;
        
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, storage_file)
            .map_err(|e| TranscriptError::StorageError(format!("Failed to write storage file {}: {}", file_name, e)))?;

        // Update file metadata
        let file_size = fs::metadata(&file_path)
            .map_err(|e| TranscriptError::StorageError(format!("Failed to get file metadata: {}", e)))?
            .len();

        let now = Utc::now();
        let metadata = FileMetadata {
            transcript_count: storage_file.transcripts.len(),
            file_size,
            created_at: storage_file.created_at,
            modified_at: now,
        };

        self.index.file_metadata.insert(file_name.to_string(), metadata);
        self.index.last_updated = now;

        Ok(())
    }

    /// Find file with space for new transcript
    fn find_available_file(&self) -> Option<String> {
        for (file_name, metadata) in &self.index.file_metadata {
            if metadata.transcript_count < self.config.max_transcripts_per_file {
                return Some(file_name.clone());
            }
        }
        None
    }
}

impl TranscriptStorage for FileTranscriptStorage {
    fn store_transcript(&mut self, entry: &TranscriptEntry) -> Result<TranscriptId, TranscriptError> {
        let start_time = Instant::now();
        
        // Find or create file for storage
        let file_name = self.find_available_file()
            .unwrap_or_else(|| self.get_next_file_name());

        // Load existing file or create new one
        let mut storage_file = if self.index.file_metadata.contains_key(&file_name) {
            self.load_storage_file(&file_name)?
        } else {
            StorageFile {
                version: "1.0".to_string(),
                created_at: Utc::now(),
                transcripts: Vec::new(),
            }
        };

        // Add transcript to file
        storage_file.transcripts.push(entry.clone());

        // Save file
        self.save_storage_file(&file_name, &storage_file)?;

        // Update index
        self.index.transcript_locations.insert(entry.id, FileLocation {
            file_name: file_name.clone(),
            offset: None, // Could be optimized with actual file offsets
            size: None,
        });

        // Update statistics
        self.stats.total_transcripts += 1;
        self.stats.storage_size_bytes += serde_json::to_string(entry)
            .map_err(|e| TranscriptError::SerializationError(e))?
            .len() as u64;

        // Save index
        self.save_index()?;

        let storage_time = start_time.elapsed();
        println!("💾 Stored transcript {} in {} ({:.2}ms)", 
                entry.id, file_name, storage_time.as_millis());

        Ok(entry.id)
    }

    fn get_transcript(&self, id: TranscriptId) -> Result<Option<TranscriptEntry>, TranscriptError> {
        if let Some(location) = self.index.transcript_locations.get(&id) {
            let storage_file = self.load_storage_file(&location.file_name)?;
            
            for transcript in storage_file.transcripts {
                if transcript.id == id {
                    return Ok(Some(transcript));
                }
            }
        }
        
        Ok(None)
    }

    fn search_transcripts(&self, criteria: &SearchCriteria) -> Result<Vec<TranscriptEntry>, TranscriptError> {
        let start_time = Instant::now();
        let mut results = Vec::new();
        
        // Load all transcripts (could be optimized with better indexing)
        for (file_name, _metadata) in &self.index.file_metadata {
            let storage_file = self.load_storage_file(file_name)?;
            
            for transcript in storage_file.transcripts {
                // Apply search criteria
                let mut matches = true;
                
                // Text search
                if let Some(query) = &criteria.query {
                    if !transcript.text.to_lowercase().contains(&query.to_lowercase()) {
                        matches = false;
                    }
                }
                
                // Date range filter
                if let Some((start_date, end_date)) = &criteria.date_range {
                    if transcript.timestamp < *start_date || transcript.timestamp > *end_date {
                        matches = false;
                    }
                }
                
                // Confidence range filter
                if let Some((min_conf, max_conf)) = &criteria.confidence_range {
                    if transcript.confidence < *min_conf || transcript.confidence > *max_conf {
                        matches = false;
                    }
                }
                
                // Tag filter
                if !criteria.tags.is_empty() {
                    let has_all_tags = criteria.tags.iter().all(|tag| transcript.tags.contains(tag));
                    if !has_all_tags {
                        matches = false;
                    }
                }
                
                // Session filter
                if let Some(session_id) = &criteria.session_id {
                    if transcript.session_id != Some(*session_id) {
                        matches = false;
                    }
                }
                
                // Language filter
                if let Some(language) = &criteria.language {
                    if transcript.language.as_ref() != Some(language) {
                        matches = false;
                    }
                }
                
                if matches {
                    results.push(transcript);
                }
            }
        }
        
        // Apply limit
        if let Some(limit) = criteria.limit {
            results.truncate(limit);
        }
        
        let search_time = start_time.elapsed();
        println!("🔍 Search completed in {:.2}ms, found {} results", 
                search_time.as_millis(), results.len());
        
        Ok(results)
    }

    fn delete_transcript(&mut self, id: TranscriptId) -> Result<(), TranscriptError> {
        if let Some(location) = self.index.transcript_locations.remove(&id) {
            // Load file and remove transcript
            let mut storage_file = self.load_storage_file(&location.file_name)?;
            storage_file.transcripts.retain(|t| t.id != id);
            
            // Save updated file
            self.save_storage_file(&location.file_name, &storage_file)?;
            
            // Update statistics
            self.stats.total_transcripts -= 1;
            
            // Save index
            self.save_index()?;
            
            println!("🗑️  Deleted transcript {}", id);
        }
        
        Ok(())
    }

    fn get_storage_stats(&self) -> TranscriptStorageStats {
        self.stats.clone()
    }

    fn backup_transcripts(&self, path: &Path) -> Result<BackupResult, TranscriptError> {
        let start_time = Instant::now();
        
        // Create backup file
        let backup_file = File::create(path)
            .map_err(|e| TranscriptError::StorageError(format!("Failed to create backup file: {}", e)))?;
        
        let mut writer = BufWriter::new(backup_file);
        
        // Write backup header
        let backup_header = serde_json::json!({
            "version": "1.0",
            "created_at": Utc::now(),
            "total_transcripts": self.stats.total_transcripts,
            "source": "file_transcript_storage"
        });
        
        writeln!(writer, "{}", serde_json::to_string(&backup_header)?)?;
        
        let mut transcripts_backed_up = 0;
        
        // Write all transcripts
        for (file_name, _metadata) in &self.index.file_metadata {
            let storage_file = self.load_storage_file(file_name)?;
            
            for transcript in storage_file.transcripts {
                let transcript_json = serde_json::to_string(&transcript)?;
                writeln!(writer, "{}", transcript_json)?;
                transcripts_backed_up += 1;
            }
        }
        
        writer.flush()?;
        
        let backup_size = fs::metadata(path)
            .map_err(|e| TranscriptError::StorageError(format!("Failed to get backup file size: {}", e)))?
            .len();
        
        let backup_duration = start_time.elapsed();
        
        println!("💾 Created backup with {} transcripts ({} bytes) in {:.2}s", 
                transcripts_backed_up, backup_size, backup_duration.as_secs_f64());
        
        Ok(BackupResult {
            transcripts_backed_up,
            backup_size_bytes: backup_size,
            backup_duration,
            backup_path: path.to_path_buf(),
        })
    }

    fn restore_transcripts(&mut self, path: &Path) -> Result<RestoreResult, TranscriptError> {
        let start_time = Instant::now();
        
        let file = File::open(path)
            .map_err(|e| TranscriptError::StorageError(format!("Failed to open backup file: {}", e)))?;
        
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        // Read header
        if let Some(header_line) = lines.next() {
            let header_line = header_line?;
            let _header: serde_json::Value = serde_json::from_str(&header_line)?;
            // Could validate backup format here
        }
        
        let mut transcripts_restored = 0;
        let mut duplicates_skipped = 0;
        
        // Read transcripts
        for line in lines {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            
            let transcript: TranscriptEntry = serde_json::from_str(&line)?;
            
            // Check if transcript already exists
            if self.index.transcript_locations.contains_key(&transcript.id) {
                duplicates_skipped += 1;
                continue;
            }
            
            // Store transcript
            self.store_transcript(&transcript)?;
            transcripts_restored += 1;
        }
        
        let restore_duration = start_time.elapsed();
        
        println!("📥 Restored {} transcripts, skipped {} duplicates in {:.2}s", 
                transcripts_restored, duplicates_skipped, restore_duration.as_secs_f64());
        
        Ok(RestoreResult {
            transcripts_restored,
            duplicates_skipped,
            restore_duration,
        })
    }
}

impl Default for FileStorageConfig {
    fn default() -> Self {
        Self {
            max_transcripts_per_file: 1000,
            enable_compression: false,
            auto_flush_interval: Duration::from_secs(300), // 5 minutes
            index_rebuild_threshold: 10000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use uuid::Uuid;
    use crate::services::transcription_log::*;
    use std::collections::HashMap;

    fn create_test_transcript(text: &str, confidence: f32) -> TranscriptEntry {
        TranscriptEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            text: text.to_string(),
            confidence,
            model: "test-model".to_string(),
            duration_ms: 1000,
            audio_file_id: None,
            session_id: None,
            hash: 0,
            tags: vec!["test".to_string()],
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
                        snr_db: 20.0,
                        audio_levels: AudioLevels {
                            peak: 0.8,
                            rms: 0.3,
                            dynamic_range: 40.0,
                        },
                        frequency_analysis: FrequencyAnalysis {
                            dominant_frequency: 440.0,
                            frequency_spread: 100.0,
                            spectral_centroid: 1000.0,
                        },
                    },
                },
                annotations: Vec::new(),
            },
            language: Some("en".to_string()),
            speaker: None,
        }
    }

    #[test]
    fn test_file_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = FileStorageConfig::default();
        
        let storage = FileTranscriptStorage::new(temp_dir.path().to_path_buf(), config);
        assert!(storage.is_ok());
        
        let storage = storage.unwrap();
        assert_eq!(storage.stats.total_transcripts, 0);
    }

    #[test]
    fn test_store_and_retrieve_transcript() {
        let temp_dir = TempDir::new().unwrap();
        let config = FileStorageConfig::default();
        
        let mut storage = FileTranscriptStorage::new(temp_dir.path().to_path_buf(), config).unwrap();
        let transcript = create_test_transcript("Hello, world!", 0.95);
        let transcript_id = transcript.id;
        
        // Store transcript
        let result = storage.store_transcript(&transcript);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), transcript_id);
        
        // Retrieve transcript
        let retrieved = storage.get_transcript(transcript_id).unwrap();
        assert!(retrieved.is_some());
        
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, transcript_id);
        assert_eq!(retrieved.text, "Hello, world!");
        assert_eq!(retrieved.confidence, 0.95);
    }

    #[test]
    fn test_search_transcripts() {
        let temp_dir = TempDir::new().unwrap();
        let config = FileStorageConfig::default();
        
        let mut storage = FileTranscriptStorage::new(temp_dir.path().to_path_buf(), config).unwrap();
        
        // Store multiple transcripts
        let transcript1 = create_test_transcript("Hello, world!", 0.95);
        let transcript2 = create_test_transcript("Goodbye, world!", 0.85);
        let transcript3 = create_test_transcript("Hello, universe!", 0.90);
        
        storage.store_transcript(&transcript1).unwrap();
        storage.store_transcript(&transcript2).unwrap();
        storage.store_transcript(&transcript3).unwrap();
        
        // Search for "hello"
        let criteria = SearchCriteria {
            query: Some("hello".to_string()),
            ..Default::default()
        };
        
        let results = storage.search_transcripts(&criteria).unwrap();
        assert_eq!(results.len(), 2);
        
        // Search with confidence filter
        let criteria = SearchCriteria {
            confidence_range: Some((0.90, 1.0)),
            ..Default::default()
        };
        
        let results = storage.search_transcripts(&criteria).unwrap();
        assert_eq!(results.len(), 2); // transcript1 and transcript3
    }

    #[test]
    fn test_backup_and_restore() {
        let temp_dir = TempDir::new().unwrap();
        let config = FileStorageConfig::default();
        
        let mut storage = FileTranscriptStorage::new(temp_dir.path().to_path_buf(), config).unwrap();
        
        // Store some transcripts
        let transcript1 = create_test_transcript("Test transcript 1", 0.95);
        let transcript2 = create_test_transcript("Test transcript 2", 0.85);
        
        storage.store_transcript(&transcript1).unwrap();
        storage.store_transcript(&transcript2).unwrap();
        
        // Create backup
        let backup_path = temp_dir.path().join("backup.json");
        let backup_result = storage.backup_transcripts(&backup_path).unwrap();
        
        assert_eq!(backup_result.transcripts_backed_up, 2);
        assert!(backup_path.exists());
        
        // Create new storage and restore
        let temp_dir2 = TempDir::new().unwrap();
        let mut storage2 = FileTranscriptStorage::new(temp_dir2.path().to_path_buf(), FileStorageConfig::default()).unwrap();
        
        let restore_result = storage2.restore_transcripts(&backup_path).unwrap();
        assert_eq!(restore_result.transcripts_restored, 2);
        assert_eq!(restore_result.duplicates_skipped, 0);
        
        // Verify restored transcripts
        let stats = storage2.get_storage_stats();
        assert_eq!(stats.total_transcripts, 2);
    }
}
