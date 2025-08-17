//! Audio Session Manager
//! 
//! This module manages audio recording sessions with real file storage,
//! transcript logging, and comprehensive session tracking.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::core::types::*;
use crate::services::audio::AudioService;
use crate::services::stt::STTService;
use crate::services::vad::VADService;
use crate::services::audio_storage::FileAudioStorage;
use crate::services::audio_archive::AudioFormatInfo;

/// Audio source type for recording
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioSource {
    /// Default microphone input
    Microphone,
    /// System audio (loopback)
    SystemAudio,
    /// Both microphone and system audio mixed
    Mixed,
    /// Specific device by name
    Device(String),
}

impl std::fmt::Display for AudioSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioSource::Microphone => write!(f, "Microphone"),
            AudioSource::SystemAudio => write!(f, "System Audio"),
            AudioSource::Mixed => write!(f, "Mixed (Microphone + System Audio)"),
            AudioSource::Device(name) => write!(f, "Device: {}", name),
        }
    }
}

/// Recording session state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    Idle,
    Recording,
    Paused,
    Stopped,
    Error(String),
}

/// Audio recording session with comprehensive tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioRecordingSession {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Duration,
    pub audio_source: AudioSource,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub format_info: AudioFormatInfo,
    pub transcript_segments: Vec<TranscriptSegment>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub state: SessionState,
    pub quality_metrics: QualityMetrics,
}

/// Individual transcript segment with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub id: Uuid,
    pub start_time: Duration,
    pub end_time: Duration,
    pub text: String,
    pub confidence: f32,
    pub speaker_id: Option<String>,
    pub language: Option<String>,
    pub word_count: usize,
    pub is_final: bool,
}

/// Audio quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub average_volume: f32,
    pub peak_volume: f32,
    pub signal_to_noise_ratio: f32,
    pub silence_percentage: f32,
    pub clipping_events: u32,
    pub vad_accuracy: f32,
}

/// Session manager for handling audio recording sessions
pub struct AudioSessionManager {
    /// Current active session
    current_session: Option<AudioRecordingSession>,
    /// Audio service for recording
    audio_service: Arc<Mutex<AudioService>>,
    /// STT service for transcription
    stt_service: Option<Arc<Mutex<STTService>>>,
    /// VAD service for voice detection
    vad_service: Option<Arc<Mutex<VADService>>>,
    /// Audio storage backend
    storage: Arc<Mutex<FileAudioStorage>>,
    /// Session configuration
    config: SessionConfig,
    /// Audio buffer for recording
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    /// Recording state
    recording_state: Arc<Mutex<SessionState>>,
    /// Session history
    session_history: Vec<AudioRecordingSession>,
    /// Base storage directory
    storage_dir: PathBuf,
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub auto_transcribe: bool,
    pub real_time_transcription: bool,
    pub save_raw_audio: bool,
    pub compress_audio: bool,
    pub max_session_duration: Duration,
    pub silence_timeout: Duration,
    pub quality_monitoring: bool,
    pub backup_enabled: bool,
    pub default_audio_source: AudioSource,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            auto_transcribe: true,
            real_time_transcription: true,
            save_raw_audio: true,
            compress_audio: true,
            max_session_duration: Duration::from_secs(4 * 60 * 60), // 4 hours
            silence_timeout: Duration::from_secs(30),
            quality_monitoring: true,
            backup_enabled: true,
            default_audio_source: AudioSource::Microphone,
        }
    }
}

impl AudioSessionManager {
    /// Create a new audio session manager
    pub fn new(
        audio_service: Arc<Mutex<AudioService>>,
        storage_dir: PathBuf,
        config: SessionConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Ensure storage directory exists
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)?;
        }

        // Initialize storage backend
        let storage_config = crate::services::audio_storage::StorageConfig::default();
        let storage = Arc::new(Mutex::new(
            FileAudioStorage::new(storage_dir.clone(), storage_config)?
        ));

        Ok(Self {
            current_session: None,
            audio_service,
            stt_service: None,
            vad_service: None,
            storage,
            config,
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            recording_state: Arc::new(Mutex::new(SessionState::Idle)),
            session_history: Vec::new(),
            storage_dir,
        })
    }

    /// Attach STT service for transcription
    pub fn attach_stt_service(&mut self, stt_service: Arc<Mutex<STTService>>) {
        self.stt_service = Some(stt_service);
    }

    /// Attach VAD service for voice activity detection
    pub fn attach_vad_service(&mut self, vad_service: Arc<Mutex<VADService>>) {
        self.vad_service = Some(vad_service);
    }

    /// Start a new recording session
    pub fn start_recording_session(
        &mut self,
        name: String,
        description: Option<String>,
        audio_source: Option<AudioSource>,
        tags: Vec<String>,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        // Check if already recording
        if self.is_recording() {
            return Err(Box::new(crate::core::error::AudioError::AlreadyRecording));
        }

        let session_id = Uuid::new_v4();
        let start_time = Utc::now();
        let audio_source = audio_source.unwrap_or_else(|| self.config.default_audio_source.clone());

        // Generate file path
        let file_path = self.generate_session_file_path(&session_id, &name, &start_time);

        // Ensure directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create session
        let session = AudioRecordingSession {
            id: session_id,
            name: name.clone(),
            description,
            start_time,
            end_time: None,
            duration: Duration::from_secs(0),
            audio_source: audio_source.clone(),
            file_path: file_path.clone(),
            file_size: 0,
            format_info: AudioFormatInfo {
                sample_rate: 44100,
                channels: 1,
                bit_depth: 16,
                format: crate::services::audio_archive::AudioFormat::WAV,
            },
            transcript_segments: Vec::new(),
            tags,
            metadata: HashMap::new(),
            state: SessionState::Recording,
            quality_metrics: QualityMetrics::default(),
        };

        // Configure audio source
        self.configure_audio_source(&audio_source)?;

        // Set up audio callback to capture data
        let buffer = self.audio_buffer.clone();
        let state = self.recording_state.clone();
        
        if let Ok(mut audio_service) = self.audio_service.lock() {
            audio_service.on_audio_frame(move |samples, _sample_rate| {
                if let Ok(current_state) = state.lock() {
                    if *current_state == SessionState::Recording {
                        if let Ok(mut buf) = buffer.lock() {
                            buf.extend_from_slice(samples);
                        }
                    }
                }
            });

            // Start audio capture
            audio_service.start_capture()?;
        }

        // Update state
        *self.recording_state.lock().unwrap() = SessionState::Recording;
        self.current_session = Some(session);

        info!(
            session_id = %session_id,
            name = %name,
            audio_source = ?audio_source,
            file_path = %file_path.display(),
            "🎙️  Started audio recording session"
        );

        Ok(session_id)
    }

    /// Stop the current recording session
    pub fn stop_recording_session(&mut self) -> Result<Option<AudioRecordingSession>, Box<dyn std::error::Error>> {
        if let Some(mut session) = self.current_session.take() {
            let end_time = Utc::now();
            session.end_time = Some(end_time);
            session.duration = end_time.signed_duration_since(session.start_time)
                .to_std()
                .unwrap_or_default();
            session.state = SessionState::Stopped;

            // Stop audio capture
            if let Ok(mut audio_service) = self.audio_service.lock() {
                audio_service.stop_capture()?;
            }

            // Save recorded audio to file
            if let Ok(buffer) = self.audio_buffer.lock() {
                if !buffer.is_empty() {
                    self.save_audio_to_file(&session, &buffer)?;
                    session.file_size = self.get_file_size(&session.file_path)?;
                }
            }

            // Clear audio buffer
            if let Ok(mut buffer) = self.audio_buffer.lock() {
                buffer.clear();
            }

            // Update state
            *self.recording_state.lock().unwrap() = SessionState::Idle;

            // Store session in history
            self.session_history.push(session.clone());

            // Log session completion
            info!(
                session_id = %session.id,
                name = %session.name,
                duration = ?session.duration,
                file_size = session.file_size,
                transcript_segments = session.transcript_segments.len(),
                "⏹️  Stopped audio recording session"
            );

            // Save session metadata
            self.save_session_metadata(&session)?;

            Ok(Some(session))
        } else {
            warn!("No active recording session to stop");
            Ok(None)
        }
    }

    /// Pause the current recording session
    pub fn pause_recording_session(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(session) = &mut self.current_session {
            if session.state == SessionState::Recording {
                session.state = SessionState::Paused;
                *self.recording_state.lock().unwrap() = SessionState::Paused;
                
                info!(
                    session_id = %session.id,
                    "⏸️  Paused audio recording session"
                );
                
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Resume the current recording session
    pub fn resume_recording_session(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(session) = &mut self.current_session {
            if session.state == SessionState::Paused {
                session.state = SessionState::Recording;
                *self.recording_state.lock().unwrap() = SessionState::Recording;
                
                info!(
                    session_id = %session.id,
                    "▶️  Resumed audio recording session"
                );
                
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        if let Ok(state) = self.recording_state.lock() {
            *state == SessionState::Recording
        } else {
            false
        }
    }

    /// Get current session info
    pub fn get_current_session(&self) -> Option<&AudioRecordingSession> {
        self.current_session.as_ref()
    }

    /// Get session history
    pub fn get_session_history(&self) -> &[AudioRecordingSession] {
        &self.session_history
    }

    /// Add transcript segment to current session
    pub fn add_transcript_segment(
        &mut self,
        text: String,
        confidence: f32,
        start_time: Duration,
        end_time: Duration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(session) = &mut self.current_session {
            let segment = TranscriptSegment {
                id: Uuid::new_v4(),
                start_time,
                end_time,
                text: text.clone(),
                confidence,
                speaker_id: None,
                language: None,
                word_count: text.split_whitespace().count(),
                is_final: true,
            };

            session.transcript_segments.push(segment);

            debug!(
                session_id = %session.id,
                text = %text,
                confidence = confidence,
                start_time = ?start_time,
                end_time = ?end_time,
                "📝 Added transcript segment to session"
            );
        }
        Ok(())
    }

    /// Configure audio source for recording
    fn configure_audio_source(&self, source: &AudioSource) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut audio_service) = self.audio_service.lock() {
            match source {
                AudioSource::Microphone => {
                    audio_service.select_input_device_by_name(None);
                }
                AudioSource::Device(name) => {
                    audio_service.select_input_device_by_name(Some(name.clone()));
                }
                AudioSource::SystemAudio => {
                    // For system audio, we'd need to configure a loopback device
                    // This is platform-specific and would require additional implementation
                    warn!("System audio recording not yet implemented, falling back to microphone");
                    audio_service.select_input_device_by_name(None);
                }
                AudioSource::Mixed => {
                    // Mixed audio would require multiple input streams
                    warn!("Mixed audio recording not yet implemented, falling back to microphone");
                    audio_service.select_input_device_by_name(None);
                }
            }
        }
        Ok(())
    }

    /// Generate file path for session
    fn generate_session_file_path(&self, session_id: &Uuid, name: &str, start_time: &DateTime<Utc>) -> PathBuf {
        let date_dir = start_time.format("%Y/%m/%d").to_string();
        let sanitized_name = self.sanitize_filename(name);
        let filename = format!("{}_{}.wav", sanitized_name, session_id);
        
        self.storage_dir
            .join("sessions")
            .join(date_dir)
            .join(filename)
    }

    /// Sanitize filename for safe storage
    fn sanitize_filename(&self, name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect::<String>()
            .chars()
            .take(50) // Limit length
            .collect()
    }

    /// Save audio data to file
    fn save_audio_to_file(&self, session: &AudioRecordingSession, samples: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
        // Create WAV file
        let spec = hound::WavSpec {
            channels: session.format_info.channels,
            sample_rate: session.format_info.sample_rate,
            bits_per_sample: session.format_info.bit_depth as u16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(&session.file_path, spec)?;
        
        // Convert f32 samples to i16 and write
        for &sample in samples {
            let sample_i16 = (sample * i16::MAX as f32) as i16;
            writer.write_sample(sample_i16)?;
        }
        
        writer.finalize()?;

        info!(
            file_path = %session.file_path.display(),
            sample_count = samples.len(),
            duration = ?session.duration,
            "💾 Saved audio session to file"
        );

        Ok(())
    }

    /// Get file size
    fn get_file_size(&self, path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(fs::metadata(path)?.len())
    }

    /// Save session metadata to JSON file
    fn save_session_metadata(&self, session: &AudioRecordingSession) -> Result<(), Box<dyn std::error::Error>> {
        let metadata_path = session.file_path.with_extension("json");
        let file = File::create(&metadata_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, session)?;

        debug!(
            metadata_path = %metadata_path.display(),
            "💾 Saved session metadata"
        );

        Ok(())
    }

    /// Load session history from storage directory
    pub fn load_session_history(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let sessions_dir = self.storage_dir.join("sessions");
        if !sessions_dir.exists() {
            return Ok(());
        }

        self.session_history.clear();
        self.scan_directory_for_sessions(&sessions_dir)?;

        info!(
            session_count = self.session_history.len(),
            "📂 Loaded session history"
        );

        Ok(())
    }

    /// Recursively scan directory for session metadata files
    fn scan_directory_for_sessions(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.scan_directory_for_sessions(&path)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(session) = self.load_session_from_metadata(&path) {
                    self.session_history.push(session);
                }
            }
        }
        Ok(())
    }

    /// Load session from metadata file
    fn load_session_from_metadata(&self, metadata_path: &Path) -> Result<AudioRecordingSession, Box<dyn std::error::Error>> {
        let file = File::open(metadata_path)?;
        let session: AudioRecordingSession = serde_json::from_reader(file)?;
        Ok(session)
    }

    /// Get available audio devices
    pub fn get_available_audio_devices(&self) -> Result<Vec<AudioDevice>, Box<dyn std::error::Error>> {
        if let Ok(audio_service) = self.audio_service.lock() {
            Ok(audio_service.get_devices()?)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get storage statistics
    pub fn get_storage_stats(&self) -> StorageStats {
        let total_sessions = self.session_history.len();
        let total_size_bytes: u64 = self.session_history.iter().map(|s| s.file_size).sum();
        let total_duration: Duration = self.session_history.iter()
            .map(|s| s.duration)
            .fold(Duration::from_secs(0), |acc, d| acc + d);

        let oldest_session = self.session_history.iter()
            .min_by_key(|s| s.start_time)
            .map(|s| s.start_time);

        let newest_session = self.session_history.iter()
            .max_by_key(|s| s.start_time)
            .map(|s| s.start_time);

        StorageStats {
            total_sessions,
            total_size_bytes,
            total_duration,
            compression_ratio: 1.0, // TODO: Calculate actual compression ratio
            oldest_session,
            newest_session,
        }
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            average_volume: 0.0,
            peak_volume: 0.0,
            signal_to_noise_ratio: 0.0,
            silence_percentage: 0.0,
            clipping_events: 0,
            vad_accuracy: 0.0,
        }
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_sessions: usize,
    pub total_size_bytes: u64,
    pub total_duration: Duration,
    pub compression_ratio: f64,
    pub oldest_session: Option<DateTime<Utc>>,
    pub newest_session: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_session_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let audio_service = Arc::new(Mutex::new(AudioService::new().unwrap()));
        let config = SessionConfig::default();
        
        let manager = AudioSessionManager::new(
            audio_service,
            temp_dir.path().to_path_buf(),
            config,
        );
        
        assert!(manager.is_ok());
    }

    #[test]
    fn test_filename_sanitization() {
        let temp_dir = TempDir::new().unwrap();
        let audio_service = Arc::new(Mutex::new(AudioService::new().unwrap()));
        let config = SessionConfig::default();
        
        let manager = AudioSessionManager::new(
            audio_service,
            temp_dir.path().to_path_buf(),
            config,
        ).unwrap();
        
        let sanitized = manager.sanitize_filename("Test/Session:With*Invalid?Chars");
        assert_eq!(sanitized, "Test_Session_With_Invalid_Chars");
    }
}
