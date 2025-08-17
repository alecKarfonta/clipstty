//! Audio Storage Implementation
//! 
//! This module provides concrete implementations of audio storage backends
//! with compression, indexing, and efficient file management.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use std::fmt;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::audio_archive::{
    AudioStorage, AudioError, RecordingSession, SearchCriteria, StorageStats,
    CompressionResult, CleanupResult, RetentionPolicy, SessionId, AudioFileId,
    AudioFormatInfo, AudioFormat,
};

/// File-based audio storage implementation
pub struct FileAudioStorage {
    /// Base storage directory
    storage_path: PathBuf,
    /// Session index for fast lookups
    session_index: SessionIndex,
    /// File manager for storage operations
    file_manager: AudioFileManager,
    /// Compression engine
    compression_engine: CompressionEngine,
    /// Storage configuration
    config: StorageConfig,
}

/// Session index for efficient session management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionIndex {
    /// Session metadata by ID
    sessions: HashMap<SessionId, SessionMetadata>,
    /// Index by name patterns
    name_index: HashMap<String, Vec<SessionId>>,
    /// Index by tags
    tag_index: HashMap<String, Vec<SessionId>>,
    /// Index by date ranges
    date_index: Vec<(DateTime<Utc>, SessionId)>,
    /// Index version for compatibility
    version: u32,
}

/// Session metadata for indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub id: SessionId,
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Duration,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub format_info: AudioFormatInfo,
    pub tags: Vec<String>,
    pub transcript_count: usize,
    pub checksum: Option<String>,
    pub compression_info: Option<CompressionInfo>,
}

/// Audio file manager for storage operations
#[derive(Debug)]
pub struct AudioFileManager {
    /// Base directory
    base_path: PathBuf,
    /// File organization strategy
    organization: FileOrganization,
    /// File naming strategy
    naming: FileNaming,
}

/// Compression engine for audio files
pub struct CompressionEngine {
    /// Available compressors
    compressors: HashMap<AudioFormat, Box<dyn AudioCompressor>>,
    /// Compression statistics
    stats: CompressionStats,
    /// Configuration
    config: CompressionConfig,
}

/// Audio compressor trait
pub trait AudioCompressor: Send + Sync {
    /// Compress audio data
    fn compress(&self, data: &[f32], format_info: &AudioFormatInfo) -> Result<Vec<u8>, AudioError>;
    
    /// Decompress audio data
    fn decompress(&self, data: &[u8], format_info: &AudioFormatInfo) -> Result<Vec<f32>, AudioError>;
    
    /// Get compression ratio estimate
    fn get_compression_ratio(&self) -> f64;
    
    /// Get compressor name
    fn get_name(&self) -> &str;
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Maximum file size before splitting
    pub max_file_size_mb: u64,
    /// Enable automatic compression
    pub auto_compress: bool,
    /// Compression format preference
    pub preferred_format: AudioFormat,
    /// Enable checksums for integrity
    pub enable_checksums: bool,
    /// Index update frequency
    pub index_update_interval: Duration,
    /// Backup configuration
    pub backup_config: Option<BackupConfig>,
}

/// Compression information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionInfo {
    pub original_size: u64,
    pub compressed_size: u64,
    pub compression_ratio: f64,
    pub format: AudioFormat,
    pub compression_time: Duration,
    pub checksum: Option<String>,
}

/// Compression statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    pub total_files_processed: usize,
    pub total_original_size: u64,
    pub total_compressed_size: u64,
    pub average_compression_ratio: f64,
    pub total_compression_time: Duration,
    pub format_stats: HashMap<AudioFormat, FormatStats>,
}

/// Per-format compression statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatStats {
    pub files_processed: usize,
    pub total_original_size: u64,
    pub total_compressed_size: u64,
    pub average_ratio: f64,
    pub average_time: Duration,
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    pub quality_level: u8,  // 0-10
    pub enable_parallel: bool,
    pub chunk_size: usize,
    pub preserve_metadata: bool,
}

/// File organization strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileOrganization {
    /// Flat structure - all files in one directory
    Flat,
    /// Organize by date (YYYY/MM/DD)
    ByDate,
    /// Organize by session name
    ByName,
    /// Organize by tags
    ByTags,
    /// Custom organization pattern
    Custom(String),
}

/// File naming strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileNaming {
    /// Use session ID as filename
    SessionId,
    /// Use session name (sanitized)
    SessionName,
    /// Use timestamp
    Timestamp,
    /// Custom naming pattern
    Custom(String),
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub backup_path: PathBuf,
    pub backup_interval: Duration,
    pub max_backups: usize,
    pub compress_backups: bool,
}

// Implementation of FileAudioStorage
impl FileAudioStorage {
    /// Create a new file-based audio storage
    pub fn new(storage_path: PathBuf, config: StorageConfig) -> Result<Self, AudioError> {
        // Ensure storage directory exists
        if !storage_path.exists() {
            fs::create_dir_all(&storage_path)?;
        }
        
        // Initialize session index
        let index_path = storage_path.join("session_index.json");
        let session_index = if index_path.exists() {
            Self::load_session_index(&index_path)?
        } else {
            SessionIndex::new()
        };
        
        // Initialize file manager
        let file_manager = AudioFileManager::new(
            storage_path.clone(),
            FileOrganization::ByDate,
            FileNaming::SessionId,
        );
        
        // Initialize compression engine
        let compression_engine = CompressionEngine::new(CompressionConfig {
            quality_level: 7,
            enable_parallel: true,
            chunk_size: 4096,
            preserve_metadata: true,
        });
        
        Ok(Self {
            storage_path,
            session_index,
            file_manager,
            compression_engine,
            config,
        })
    }
    
    /// Load session index from file
    fn load_session_index(path: &Path) -> Result<SessionIndex, AudioError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let index: SessionIndex = serde_json::from_reader(reader)?;
        Ok(index)
    }
    
    /// Save session index to file
    fn save_session_index(&self) -> Result<(), AudioError> {
        let index_path = self.storage_path.join("session_index.json");
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(index_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.session_index)?;
        Ok(())
    }
    
    /// Generate file path for a session
    fn generate_file_path(&self, session: &RecordingSession) -> PathBuf {
        self.file_manager.generate_path(session)
    }
    
    /// Calculate file checksum
    fn calculate_checksum(&self, data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl AudioStorage for FileAudioStorage {
    fn store_audio(&mut self, session: &RecordingSession, data: &[f32]) -> Result<AudioFileId, AudioError> {
        let file_id = Uuid::new_v4();
        let file_path = self.generate_file_path(session);
        
        // Ensure directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Convert audio data to bytes and optionally compress
        let audio_bytes = if self.config.auto_compress {
            self.compression_engine.compress_audio(data, &session.format_info)?
        } else {
            // Convert f32 samples to bytes (WAV format)
            let mut bytes = Vec::with_capacity(data.len() * 4);
            for &sample in data {
                bytes.extend_from_slice(&sample.to_le_bytes());
            }
            bytes
        };
        
        // Calculate checksum if enabled
        let checksum = if self.config.enable_checksums {
            Some(self.calculate_checksum(&audio_bytes))
        } else {
            None
        };
        
        // Write audio data to file
        let mut file = File::create(&file_path)?;
        file.write_all(&audio_bytes)?;
        file.sync_all()?;
        
        // Create session metadata
        let metadata = SessionMetadata {
            id: session.id,
            name: session.name.clone(),
            start_time: session.start_time,
            end_time: session.end_time,
            duration: session.duration,
            file_path: file_path.clone(),
            file_size: audio_bytes.len() as u64,
            format_info: session.format_info.clone(),
            tags: session.tags.clone(),
            transcript_count: session.transcript_count,
            checksum: checksum.clone(),
            compression_info: if self.config.auto_compress {
                Some(CompressionInfo {
                    original_size: data.len() as u64 * 4,
                    compressed_size: audio_bytes.len() as u64,
                    compression_ratio: audio_bytes.len() as f64 / (data.len() * 4) as f64,
                    format: self.config.preferred_format.clone(),
                    compression_time: Duration::from_millis(10), // Mock timing
                    checksum: checksum.clone(),
                })
            } else {
                None
            },
        };
        
        // Update session index
        self.session_index.add_session(metadata)?;
        self.save_session_index()?;
        
        println!("📁 Stored audio session: {} ({:.2} MB)", 
                session.name, 
                audio_bytes.len() as f64 / 1024.0 / 1024.0);
        
        Ok(file_id)
    }
    
    fn retrieve_audio(&self, file_id: AudioFileId) -> Result<Vec<f32>, AudioError> {
        // Find session by file_id (simplified - in real implementation would have file_id index)
        let session_metadata = self.session_index.sessions.values()
            .find(|s| s.id == file_id) // Using session_id as file_id for simplicity
            .ok_or_else(|| AudioError::StorageError("Session not found".to_string()))?;
        
        // Read audio data from file
        let mut file = File::open(&session_metadata.file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        // Verify checksum if available
        if let Some(expected_checksum) = &session_metadata.checksum {
            let actual_checksum = self.calculate_checksum(&buffer);
            if actual_checksum != *expected_checksum {
                return Err(AudioError::StorageError("Checksum mismatch - file may be corrupted".to_string()));
            }
        }
        
        // Decompress if necessary
        let audio_data = if let Some(compression_info) = &session_metadata.compression_info {
            self.compression_engine.decompress_audio(&buffer, &compression_info.format, &session_metadata.format_info)?
        } else {
            // Convert bytes back to f32 samples
            let mut samples = Vec::with_capacity(buffer.len() / 4);
            for chunk in buffer.chunks_exact(4) {
                let bytes: [u8; 4] = chunk.try_into().unwrap();
                samples.push(f32::from_le_bytes(bytes));
            }
            samples
        };
        
        Ok(audio_data)
    }
    
    fn list_sessions(&self, criteria: SearchCriteria) -> Result<Vec<RecordingSession>, AudioError> {
        let mut matching_sessions = Vec::new();
        
        for metadata in self.session_index.sessions.values() {
            let mut matches = true;
            
            // Check name pattern
            if let Some(pattern) = &criteria.name_pattern {
                if !metadata.name.to_lowercase().contains(&pattern.to_lowercase()) {
                    matches = false;
                }
            }
            
            // Check tags
            if !criteria.tags.is_empty() {
                let has_any_tag = criteria.tags.iter()
                    .any(|tag| metadata.tags.contains(tag));
                if !has_any_tag {
                    matches = false;
                }
            }
            
            // Check date range
            if let Some((start_date, end_date)) = &criteria.date_range {
                if metadata.start_time < *start_date || metadata.start_time > *end_date {
                    matches = false;
                }
            }
            
            // Check duration range
            if let Some(min_duration) = &criteria.min_duration {
                if metadata.duration < *min_duration {
                    matches = false;
                }
            }
            
            if let Some(max_duration) = &criteria.max_duration {
                if metadata.duration > *max_duration {
                    matches = false;
                }
            }
            
            if matches {
                // Convert metadata to RecordingSession
                let session = RecordingSession {
                    id: metadata.id,
                    name: metadata.name.clone(),
                    description: None, // Not stored in metadata for simplicity
                    start_time: metadata.start_time,
                    end_time: metadata.end_time,
                    duration: metadata.duration,
                    file_path: metadata.file_path.clone(),
                    file_size: metadata.file_size,
                    format_info: metadata.format_info.clone(),
                    tags: metadata.tags.clone(),
                    transcript_count: metadata.transcript_count,
                    metadata: HashMap::new(), // Not stored for simplicity
                };
                matching_sessions.push(session);
            }
        }
        
        // Sort by start time (newest first)
        matching_sessions.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        
        // Apply limit
        if let Some(limit) = criteria.limit {
            matching_sessions.truncate(limit);
        }
        
        Ok(matching_sessions)
    }
    
    fn delete_session(&mut self, session_id: SessionId) -> Result<(), AudioError> {
        // Find and remove session from index
        let metadata = self.session_index.sessions.remove(&session_id)
            .ok_or_else(|| AudioError::SessionNotFound(session_id))?;
        
        // Delete the audio file
        if metadata.file_path.exists() {
            fs::remove_file(&metadata.file_path)?;
        }
        
        // Update indexes
        self.session_index.remove_from_indexes(&metadata);
        self.save_session_index()?;
        
        println!("🗑️  Deleted session: {} (freed {:.2} MB)", 
                metadata.name, 
                metadata.file_size as f64 / 1024.0 / 1024.0);
        
        Ok(())
    }
    
    fn get_storage_stats(&self) -> StorageStats {
        let sessions: Vec<_> = self.session_index.sessions.values().collect();
        
        let total_sessions = sessions.len();
        let total_size_bytes: u64 = sessions.iter().map(|s| s.file_size).sum();
        let total_duration: Duration = sessions.iter()
            .map(|s| s.duration)
            .fold(Duration::from_secs(0), |acc, d| acc + d);
        
        let compression_ratio = if total_size_bytes > 0 {
            let compressed_sessions: Vec<_> = sessions.iter()
                .filter(|s| s.compression_info.is_some())
                .collect();
            
            if !compressed_sessions.is_empty() {
                let total_original: u64 = compressed_sessions.iter()
                    .map(|s| s.compression_info.as_ref().unwrap().original_size)
                    .sum();
                let total_compressed: u64 = compressed_sessions.iter()
                    .map(|s| s.compression_info.as_ref().unwrap().compressed_size)
                    .sum();
                
                if total_original > 0 {
                    total_compressed as f64 / total_original as f64
                } else {
                    1.0
                }
            } else {
                1.0
            }
        } else {
            1.0
        };
        
        let oldest_session = sessions.iter()
            .min_by_key(|s| s.start_time)
            .map(|s| s.start_time);
        
        let newest_session = sessions.iter()
            .max_by_key(|s| s.start_time)
            .map(|s| s.start_time);
        
        StorageStats {
            total_sessions,
            total_size_bytes,
            total_duration,
            compression_ratio,
            oldest_session,
            newest_session,
        }
    }
    
    fn compress_audio_files(&mut self) -> Result<CompressionResult, AudioError> {
        let start_time = std::time::Instant::now();
        let mut files_compressed = 0;
        let mut original_size = 0u64;
        let mut compressed_size = 0u64;
        
        // Find uncompressed sessions
        let uncompressed_sessions: Vec<_> = self.session_index.sessions.values()
            .filter(|s| s.compression_info.is_none())
            .cloned()
            .collect();
        
        for session_metadata in uncompressed_sessions {
            // Read original audio data
            let audio_data = self.retrieve_audio(session_metadata.id)?;
            
            // Compress the audio data
            let compressed_data = self.compression_engine.compress_audio(&audio_data, &session_metadata.format_info)?;
            
            // Calculate sizes
            let orig_size = audio_data.len() as u64 * 4;
            let comp_size = compressed_data.len() as u64;
            
            // Write compressed data back to file
            let mut file = File::create(&session_metadata.file_path)?;
            file.write_all(&compressed_data)?;
            file.sync_all()?;
            
            // Update session metadata
            let mut updated_metadata = session_metadata.clone();
            updated_metadata.file_size = comp_size;
            updated_metadata.compression_info = Some(CompressionInfo {
                original_size: orig_size,
                compressed_size: comp_size,
                compression_ratio: comp_size as f64 / orig_size as f64,
                format: self.config.preferred_format.clone(),
                compression_time: Duration::from_millis(10), // Mock timing
                checksum: if self.config.enable_checksums {
                    Some(self.calculate_checksum(&compressed_data))
                } else {
                    None
                },
            });
            
            // Update index
            self.session_index.sessions.insert(session_metadata.id, updated_metadata);
            
            files_compressed += 1;
            original_size += orig_size;
            compressed_size += comp_size;
        }
        
        // Save updated index
        self.save_session_index()?;
        
        let time_taken = start_time.elapsed();
        let compression_ratio = if original_size > 0 {
            compressed_size as f64 / original_size as f64
        } else {
            1.0
        };
        
        Ok(CompressionResult {
            files_compressed,
            original_size,
            compressed_size,
            compression_ratio,
            time_taken,
        })
    }
    
    fn cleanup_old_files(&mut self, retention_policy: &RetentionPolicy) -> Result<CleanupResult, AudioError> {
        let mut files_deleted = 0;
        let mut space_freed = 0u64;
        let mut sessions_removed = 0;
        
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_policy.max_age_days as i64);
        
        // Find sessions to delete based on age
        let sessions_to_delete: Vec<_> = self.session_index.sessions.values()
            .filter(|s| s.start_time < cutoff_date)
            .map(|s| s.id)
            .collect();
        
        // Delete old sessions
        for session_id in sessions_to_delete {
            if let Some(metadata) = self.session_index.sessions.get(&session_id) {
                space_freed += metadata.file_size;
                sessions_removed += 1;
                files_deleted += 1;
            }
            self.delete_session(session_id)?;
        }
        
        // Check total size limit
        let current_stats = self.get_storage_stats();
        let max_size_bytes = (retention_policy.max_total_size_gb * 1024.0 * 1024.0 * 1024.0) as u64;
        
        if current_stats.total_size_bytes > max_size_bytes {
            // Delete oldest sessions until under limit
            let sessions_by_age: Vec<_> = self.session_index.sessions.values().cloned().collect();
            let mut sessions_by_age = sessions_by_age;
            sessions_by_age.sort_by_key(|s| s.start_time);
            
            let mut current_size = current_stats.total_size_bytes;
            let sessions_to_keep = retention_policy.keep_recent_count;
            
            for (i, session_metadata) in sessions_by_age.iter().enumerate() {
                if current_size <= max_size_bytes {
                    break;
                }
                
                if i >= sessions_by_age.len() - sessions_to_keep {
                    break; // Keep recent sessions
                }
                
                current_size -= session_metadata.file_size;
                space_freed += session_metadata.file_size;
                sessions_removed += 1;
                files_deleted += 1;
                
                self.delete_session(session_metadata.id)?;
            }
        }
        
        Ok(CleanupResult {
            files_deleted,
            space_freed,
            sessions_removed,
        })
    }
}

// Implementation of SessionIndex
impl SessionIndex {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            name_index: HashMap::new(),
            tag_index: HashMap::new(),
            date_index: Vec::new(),
            version: 1,
        }
    }
    
    pub fn add_session(&mut self, metadata: SessionMetadata) -> Result<(), AudioError> {
        let session_id = metadata.id;
        
        // Add to main index
        self.sessions.insert(session_id, metadata.clone());
        
        // Update name index
        let name_key = metadata.name.to_lowercase();
        self.name_index.entry(name_key).or_insert_with(Vec::new).push(session_id);
        
        // Update tag index
        for tag in &metadata.tags {
            self.tag_index.entry(tag.clone()).or_insert_with(Vec::new).push(session_id);
        }
        
        // Update date index
        self.date_index.push((metadata.start_time, session_id));
        self.date_index.sort_by_key(|(date, _)| *date);
        
        Ok(())
    }
    
    pub fn remove_from_indexes(&mut self, metadata: &SessionMetadata) {
        let session_id = metadata.id;
        
        // Remove from name index
        let name_key = metadata.name.to_lowercase();
        if let Some(sessions) = self.name_index.get_mut(&name_key) {
            sessions.retain(|&id| id != session_id);
            if sessions.is_empty() {
                self.name_index.remove(&name_key);
            }
        }
        
        // Remove from tag index
        for tag in &metadata.tags {
            if let Some(sessions) = self.tag_index.get_mut(tag) {
                sessions.retain(|&id| id != session_id);
                if sessions.is_empty() {
                    self.tag_index.remove(tag);
                }
            }
        }
        
        // Remove from date index
        self.date_index.retain(|(_, id)| *id != session_id);
    }
}

// Implementation of AudioFileManager
impl AudioFileManager {
    pub fn new(base_path: PathBuf, organization: FileOrganization, naming: FileNaming) -> Self {
        Self {
            base_path,
            organization,
            naming,
        }
    }
    
    pub fn generate_path(&self, session: &RecordingSession) -> PathBuf {
        let mut path = self.base_path.clone();
        
        // Apply organization strategy
        match &self.organization {
            FileOrganization::Flat => {
                // Keep flat structure
            }
            FileOrganization::ByDate => {
                let date_str = session.start_time.format("%Y/%m/%d").to_string();
                path = path.join(date_str);
            }
            FileOrganization::ByName => {
                let sanitized_name = self.sanitize_filename(&session.name);
                path = path.join(sanitized_name);
            }
            FileOrganization::ByTags => {
                if let Some(tag) = session.tags.first() {
                    path = path.join(self.sanitize_filename(tag));
                }
            }
            FileOrganization::Custom(pattern) => {
                // Apply custom pattern (simplified)
                path = path.join(pattern);
            }
        }
        
        // Apply naming strategy
        let filename = match &self.naming {
            FileNaming::SessionId => format!("{}.wav", session.id),
            FileNaming::SessionName => format!("{}.wav", self.sanitize_filename(&session.name)),
            FileNaming::Timestamp => format!("{}.wav", session.start_time.format("%Y%m%d_%H%M%S")),
            FileNaming::Custom(pattern) => format!("{}.wav", pattern),
        };
        
        path.join(filename)
    }
    
    fn sanitize_filename(&self, name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect()
    }
}

// Implementation of CompressionEngine
impl CompressionEngine {
    pub fn new(config: CompressionConfig) -> Self {
        let mut compressors: HashMap<AudioFormat, Box<dyn AudioCompressor>> = HashMap::new();
        
        // Add compressor implementations
        compressors.insert(AudioFormat::FLAC, Box::new(FlacCompressor::new()));
        compressors.insert(AudioFormat::Opus, Box::new(OpusCompressor::new()));
        compressors.insert(AudioFormat::MP3, Box::new(Mp3Compressor::new()));
        
        Self {
            compressors,
            stats: CompressionStats::new(),
            config,
        }
    }
    
    pub fn compress_audio(&mut self, data: &[f32], format_info: &AudioFormatInfo) -> Result<Vec<u8>, AudioError> {
        let compressor = self.compressors.get(&format_info.format)
            .ok_or_else(|| AudioError::CompressionError("Unsupported format".to_string()))?;
        
        let compressed = compressor.compress(data, format_info)?;
        
        // Update statistics
        self.stats.update_compression_stats(&format_info.format, data.len() * 4, compressed.len());
        
        Ok(compressed)
    }
    
    pub fn decompress_audio(&self, data: &[u8], format: &AudioFormat, format_info: &AudioFormatInfo) -> Result<Vec<f32>, AudioError> {
        let compressor = self.compressors.get(format)
            .ok_or_else(|| AudioError::CompressionError("Unsupported format".to_string()))?;
        
        compressor.decompress(data, format_info)
    }
}

// Mock compressor implementations
struct FlacCompressor;
impl FlacCompressor {
    fn new() -> Self { Self }
}

impl AudioCompressor for FlacCompressor {
    fn compress(&self, data: &[f32], _format_info: &AudioFormatInfo) -> Result<Vec<u8>, AudioError> {
        // Mock FLAC compression - in real implementation, use flac-sys
        let compressed_size = (data.len() as f64 * 0.6) as usize;
        Ok(vec![0u8; compressed_size.max(1)])
    }
    
    fn decompress(&self, data: &[u8], _format_info: &AudioFormatInfo) -> Result<Vec<f32>, AudioError> {
        // Mock FLAC decompression
        let sample_count = (data.len() as f64 / 0.6) as usize / 4;
        Ok(vec![0.0f32; sample_count])
    }
    
    fn get_compression_ratio(&self) -> f64 { 0.6 }
    fn get_name(&self) -> &str { "FLAC" }
}

struct OpusCompressor;
impl OpusCompressor {
    fn new() -> Self { Self }
}

impl AudioCompressor for OpusCompressor {
    fn compress(&self, data: &[f32], _format_info: &AudioFormatInfo) -> Result<Vec<u8>, AudioError> {
        // Mock Opus compression - in real implementation, use opus-sys
        let compressed_size = (data.len() as f64 * 0.3) as usize;
        Ok(vec![0u8; compressed_size.max(1)])
    }
    
    fn decompress(&self, data: &[u8], _format_info: &AudioFormatInfo) -> Result<Vec<f32>, AudioError> {
        // Mock Opus decompression
        let sample_count = (data.len() as f64 / 0.3) as usize / 4;
        Ok(vec![0.0f32; sample_count])
    }
    
    fn get_compression_ratio(&self) -> f64 { 0.3 }
    fn get_name(&self) -> &str { "Opus" }
}

struct Mp3Compressor;
impl Mp3Compressor {
    fn new() -> Self { Self }
}

impl AudioCompressor for Mp3Compressor {
    fn compress(&self, data: &[f32], _format_info: &AudioFormatInfo) -> Result<Vec<u8>, AudioError> {
        // Mock MP3 compression - in real implementation, use lame-sys
        let compressed_size = (data.len() as f64 * 0.1) as usize;
        Ok(vec![0u8; compressed_size.max(1)])
    }
    
    fn decompress(&self, data: &[u8], _format_info: &AudioFormatInfo) -> Result<Vec<f32>, AudioError> {
        // Mock MP3 decompression
        let sample_count = (data.len() as f64 / 0.1) as usize / 4;
        Ok(vec![0.0f32; sample_count])
    }
    
    fn get_compression_ratio(&self) -> f64 { 0.1 }
    fn get_name(&self) -> &str { "MP3" }
}

impl CompressionStats {
    fn new() -> Self {
        Self {
            total_files_processed: 0,
            total_original_size: 0,
            total_compressed_size: 0,
            average_compression_ratio: 0.0,
            total_compression_time: Duration::from_secs(0),
            format_stats: HashMap::new(),
        }
    }
    
    fn update_compression_stats(&mut self, format: &AudioFormat, original_size: usize, compressed_size: usize) {
        self.total_files_processed += 1;
        self.total_original_size += original_size as u64;
        self.total_compressed_size += compressed_size as u64;
        self.average_compression_ratio = self.total_compressed_size as f64 / self.total_original_size as f64;
        
        // Update format-specific stats
        let format_stat = self.format_stats.entry(format.clone()).or_insert_with(|| FormatStats {
            files_processed: 0,
            total_original_size: 0,
            total_compressed_size: 0,
            average_ratio: 0.0,
            average_time: Duration::from_secs(0),
        });
        
        format_stat.files_processed += 1;
        format_stat.total_original_size += original_size as u64;
        format_stat.total_compressed_size += compressed_size as u64;
        format_stat.average_ratio = format_stat.total_compressed_size as f64 / format_stat.total_original_size as f64;
    }
}

// Default implementations
impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_file_size_mb: 100,
            auto_compress: true,
            preferred_format: AudioFormat::FLAC,
            enable_checksums: true,
            index_update_interval: Duration::from_secs(60),
            backup_config: None,
        }
    }
}

impl fmt::Debug for FileAudioStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileAudioStorage")
            .field("storage_path", &self.storage_path)
            .field("session_index", &self.session_index)
            .field("file_manager", &self.file_manager)
            .field("compression_engine", &"CompressionEngine")
            .finish()
    }
}

impl fmt::Debug for CompressionEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompressionEngine")
            .field("compressors", &format!("{} compressors", self.compressors.len()))
            .field("stats", &self.stats)
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_file_audio_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig::default();
        let storage = FileAudioStorage::new(temp_dir.path().to_path_buf(), config);
        assert!(storage.is_ok());
    }
    
    #[test]
    fn test_session_index_operations() {
        let mut index = SessionIndex::new();
        
        let metadata = SessionMetadata {
            id: Uuid::new_v4(),
            name: "Test Session".to_string(),
            start_time: Utc::now(),
            end_time: None,
            duration: Duration::from_secs(60),
            file_path: PathBuf::from("test.wav"),
            file_size: 1024,
            format_info: AudioFormatInfo {
                sample_rate: 44100,
                channels: 1,
                bit_depth: 16,
                format: AudioFormat::WAV,
            },
            tags: vec!["test".to_string()],
            transcript_count: 0,
            checksum: None,
            compression_info: None,
        };
        
        assert!(index.add_session(metadata.clone()).is_ok());
        assert_eq!(index.sessions.len(), 1);
        assert!(index.name_index.contains_key("test session"));
        assert!(index.tag_index.contains_key("test"));
    }
}
