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
#[derive(Debug)]
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
    /// Actual sample rate from audio capture
    actual_sample_rate: Arc<Mutex<Option<u32>>>,
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
            actual_sample_rate: Arc::new(Mutex::new(None)),
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

        // Generate session directory (new approach - no single file)
        let session_dir = self.create_session_directory_for_new_session(&session_id, &name, &start_time)?;
        
        // Temporary file path - will be updated when comprehensive outputs are generated
        let temp_file_path = session_dir.join("session.wav");

        // Create session
        let session = AudioRecordingSession {
            id: session_id,
            name: name.clone(),
            description,
            start_time,
            end_time: None,
            duration: Duration::from_secs(0),
            audio_source: audio_source.clone(),
            file_path: temp_file_path.clone(),
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

        // Reset sample rate for new session
        *self.actual_sample_rate.lock().unwrap() = None;

        // Update state FIRST to ensure callback can capture audio
        *self.recording_state.lock().unwrap() = SessionState::Recording;
        self.current_session = Some(session);

        // Set up audio callback to capture data
        let buffer = self.audio_buffer.clone();
        let state = self.recording_state.clone();
        let sample_rate_ref = self.actual_sample_rate.clone();
        
        if let Ok(mut audio_service) = self.audio_service.lock() {
            audio_service.on_audio_frame(move |samples, sample_rate| {
                // Store the actual sample rate on first callback
                if let Ok(mut sr) = sample_rate_ref.lock() {
                    if sr.is_none() {
                        *sr = Some(sample_rate);
                        debug!(
                            actual_sample_rate = sample_rate,
                            "🎵 Captured actual audio sample rate"
                        );
                    }
                }
                
                if let Ok(current_state) = state.lock() {
                    if *current_state == SessionState::Recording {
                        if let Ok(mut buf) = buffer.lock() {
                            let before_len = buf.len();
                            buf.extend_from_slice(samples);
                            debug!(
                                samples_received = samples.len(),
                                sample_rate = sample_rate,
                                buffer_before = before_len,
                                buffer_after = buf.len(),
                                "📊 Audio callback received samples"
                            );
                        }
                    } else {
                        debug!(
                            current_state = ?*current_state,
                            samples_received = samples.len(),
                            "⚠️  Audio callback received samples but not in Recording state"
                        );
                    }
                }
            });

            // Start audio capture
            audio_service.start_capture()?;
        }

        info!(
            name = %name,
            "Started audio recording session"
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

            // Update session format info with actual sample rate
            if let Ok(actual_sr) = self.actual_sample_rate.lock() {
                if let Some(sample_rate) = *actual_sr {
                    session.format_info.sample_rate = sample_rate;
                    info!(
                        session_id = %session.id,
                        actual_sample_rate = sample_rate,
                        "🎵 Updated session with actual sample rate"
                    );
                }
            }

            // Save all audio outputs
            if let Ok(buffer) = self.audio_buffer.lock() {
                info!(
                    buffer_size = buffer.len(),
                    buffer_empty = buffer.is_empty(),
                    actual_sample_rate = session.format_info.sample_rate,
                    "📊 Checking audio buffer for session outputs"
                );
                
                if !buffer.is_empty() {
                    // Save multiple audio outputs
                    self.save_session_outputs(&mut session, &buffer)?;
                } else {
                    warn!(
                        session_id = %session.id,
                        "⚠️  Audio buffer is empty, no comprehensive outputs will be generated"
                    );
                    
                    // Still save basic session metadata even if no audio was captured
                    self.save_session_metadata(&session)?;
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

    /// Save comprehensive session outputs: raw audio, cleaned audio, segments, and metadata
    fn save_session_outputs(&self, session: &mut AudioRecordingSession, samples: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
        let sample_rate = session.format_info.sample_rate;
        
        // Create session output directory structure
        let session_dir = self.create_session_directory(session)?;
        
        // 1. Save raw audio (complete recording)
        let raw_audio_path = session_dir.join("raw_audio.wav");
        self.save_audio_to_file_path(&raw_audio_path, samples, &session.format_info)?;
        session.file_path = raw_audio_path.clone();
        session.file_size = self.get_file_size(&raw_audio_path)?;
        
        // 2. Detect speech segments and silence
        let speech_segments = self.detect_speech_segments(samples, sample_rate)?;
        
        // 3. Create cleaned audio (silence removed)
        let cleaned_audio_path = session_dir.join("cleaned_audio.wav");
        let cleaned_samples = self.remove_silence_from_audio(samples, &speech_segments);
        self.save_audio_to_file_path(&cleaned_audio_path, &cleaned_samples, &session.format_info)?;
        
        // 4. Create individual segment files
        let segments_dir = session_dir.join("segments");
        fs::create_dir_all(&segments_dir)?;
        let audio_segments = self.extract_individual_segments(samples, &speech_segments, &segments_dir, session, sample_rate)?;
        
        // 5. Create comprehensive metadata file
        let outputs = SessionOutputs {
            raw_audio_path: raw_audio_path.clone(),
            cleaned_audio_path: cleaned_audio_path.clone(),
            segments_directory: segments_dir.clone(),
            metadata_path: session_dir.join("session_metadata.json"),
            segments: audio_segments.clone(),
            total_raw_duration: Duration::from_secs_f32(samples.len() as f32 / sample_rate as f32),
            total_cleaned_duration: Duration::from_secs_f32(cleaned_samples.len() as f32 / sample_rate as f32),
            silence_removed_duration: if samples.len() >= cleaned_samples.len() {
                Duration::from_secs_f32((samples.len() - cleaned_samples.len()) as f32 / sample_rate as f32)
            } else {
                Duration::from_secs(0)
            },
        };
        
        self.save_comprehensive_metadata(session, &outputs)?;
        
        info!(
            session_id = %session.id,
            raw_file = %raw_audio_path.display(),
            cleaned_file = %cleaned_audio_path.display(),
            segments_count = audio_segments.len(),
            segments_dir = %segments_dir.display(),
            raw_duration = ?outputs.total_raw_duration,
            cleaned_duration = ?outputs.total_cleaned_duration,
            silence_removed = ?outputs.silence_removed_duration,
            "💾 Saved comprehensive session outputs"
        );
        
        Ok(())
    }

    /// Save audio data to a specific file path
    fn save_audio_to_file_path(&self, file_path: &Path, samples: &[f32], format_info: &AudioFormatInfo) -> Result<(), Box<dyn std::error::Error>> {
        // Create WAV file
        let spec = hound::WavSpec {
            channels: format_info.channels,
            sample_rate: format_info.sample_rate,
            bits_per_sample: format_info.bit_depth as u16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(file_path, spec)?;
        
        // Convert f32 samples to i16 and write
        for &sample in samples {
            let sample_i16 = (sample * i16::MAX as f32) as i16;
            writer.write_sample(sample_i16)?;
        }
        
        writer.finalize()?;

        debug!(
            file_path = %file_path.display(),
            sample_count = samples.len(),
            "💾 Saved audio to file"
        );

        Ok(())
    }

    /// Get file size
    fn get_file_size(&self, path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(fs::metadata(path)?.len())
    }

    /// Create session directory structure for new session
    fn create_session_directory_for_new_session(&self, session_id: &Uuid, name: &str, start_time: &DateTime<Utc>) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let date_dir = start_time.format("%Y/%m/%d").to_string();
        let sanitized_name = self.sanitize_filename(name);
        let session_dir_name = format!("{}_{}", sanitized_name, session_id);
        
        let session_dir = self.storage_dir
            .join("sessions")
            .join(date_dir)
            .join(session_dir_name);
            
        fs::create_dir_all(&session_dir)?;
        Ok(session_dir)
    }

    /// Create session directory structure
    fn create_session_directory(&self, session: &AudioRecordingSession) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let date_dir = session.start_time.format("%Y/%m/%d").to_string();
        let sanitized_name = self.sanitize_filename(&session.name);
        let session_dir_name = format!("{}_{}", sanitized_name, session.id);
        
        let session_dir = self.storage_dir
            .join("sessions")
            .join(date_dir)
            .join(session_dir_name);
            
        fs::create_dir_all(&session_dir)?;
        Ok(session_dir)
    }

    /// Detect speech segments using energy-based VAD
    fn detect_speech_segments(&self, samples: &[f32], sample_rate: u32) -> Result<Vec<(usize, usize)>, Box<dyn std::error::Error>> {
        let frame_size = (sample_rate as f32 * 0.025) as usize; // 25ms frames
        let hop_size = (sample_rate as f32 * 0.010) as usize;   // 10ms hop
        let energy_threshold = 0.001; // Configurable threshold
        let min_speech_duration = (sample_rate as f32 * 0.5) as usize; // 500ms minimum
        let max_silence_gap = (sample_rate as f32 * 0.3) as usize; // 300ms max gap
        
        let mut speech_segments = Vec::new();
        let mut current_segment_start: Option<usize> = None;
        let mut last_speech_frame = 0;
        
        // Process audio in frames
        for frame_start in (0..samples.len()).step_by(hop_size) {
            let frame_end = (frame_start + frame_size).min(samples.len());
            let frame = &samples[frame_start..frame_end];
            
            // Calculate frame energy
            let energy = self.calculate_frame_energy(frame);
            let is_speech = energy > energy_threshold;
            
            if is_speech {
                if current_segment_start.is_none() {
                    current_segment_start = Some(frame_start);
                }
                last_speech_frame = frame_end;
            } else if let Some(segment_start) = current_segment_start {
                // Check if silence gap is too long
                if frame_start > last_speech_frame && frame_start - last_speech_frame > max_silence_gap {
                    // End current segment if it's long enough
                    if last_speech_frame > segment_start && last_speech_frame - segment_start >= min_speech_duration {
                        speech_segments.push((segment_start, last_speech_frame));
                    }
                    current_segment_start = None;
                }
            }
        }
        
        // Handle final segment
        if let Some(segment_start) = current_segment_start {
            if samples.len() > segment_start && samples.len() - segment_start >= min_speech_duration {
                speech_segments.push((segment_start, samples.len()));
            }
        }
        
        debug!(
            segments_count = speech_segments.len(),
            total_samples = samples.len(),
            "🎯 Detected speech segments"
        );
        
        Ok(speech_segments)
    }

    /// Calculate energy of an audio frame
    fn calculate_frame_energy(&self, frame: &[f32]) -> f32 {
        if frame.is_empty() {
            return 0.0;
        }
        let sum_squares: f32 = frame.iter().map(|&s| s * s).sum();
        sum_squares / frame.len() as f32
    }

    /// Remove silence from audio based on speech segments
    fn remove_silence_from_audio(&self, samples: &[f32], speech_segments: &[(usize, usize)]) -> Vec<f32> {
        let mut cleaned_samples = Vec::new();
        
        for &(start, end) in speech_segments {
            if start < samples.len() && end <= samples.len() {
                cleaned_samples.extend_from_slice(&samples[start..end]);
            }
        }
        
        debug!(
            original_samples = samples.len(),
            cleaned_samples = cleaned_samples.len(),
            compression_ratio = cleaned_samples.len() as f32 / samples.len() as f32,
            "✂️  Removed silence from audio"
        );
        
        cleaned_samples
    }

    /// Extract individual audio segments to separate files
    fn extract_individual_segments(
        &self,
        samples: &[f32],
        speech_segments: &[(usize, usize)],
        segments_dir: &Path,
        session: &AudioRecordingSession,
        sample_rate: u32,
    ) -> Result<Vec<AudioSegment>, Box<dyn std::error::Error>> {
        let mut audio_segments = Vec::new();
        
        for (i, &(start_sample, end_sample)) in speech_segments.iter().enumerate() {
            let segment_id = Uuid::new_v4();
            let segment_filename = format!("segment_{:03}_{}.wav", i + 1, segment_id);
            let segment_path = segments_dir.join(&segment_filename);
            
            // Extract segment samples
            if start_sample < samples.len() && end_sample <= samples.len() {
                let segment_samples = &samples[start_sample..end_sample];
                
                // Save segment to file
                self.save_audio_to_file_path(&segment_path, segment_samples, &session.format_info)?;
                
                // Calculate timing
                let start_time = Duration::from_secs_f32(start_sample as f32 / sample_rate as f32);
                let end_time = Duration::from_secs_f32(end_sample as f32 / sample_rate as f32);
                let duration = end_time - start_time;
                
                // Calculate average energy
                let average_energy = self.calculate_frame_energy(segment_samples);
                
                // Try to match with transcript segments
                let (text, confidence) = self.find_matching_transcript_segment(session, start_time, end_time);
                
                let audio_segment = AudioSegment {
                    id: segment_id,
                    start_sample,
                    end_sample,
                    start_time,
                    end_time,
                    duration,
                    text,
                    confidence,
                    file_path: segment_path.clone(),
                    file_size: self.get_file_size(&segment_path)?,
                    is_speech: true,
                    average_energy,
                };
                
                audio_segments.push(audio_segment);
                
                debug!(
                    segment_id = %segment_id,
                    filename = %segment_filename,
                    start_time = ?start_time,
                    end_time = ?end_time,
                    duration = ?duration,
                    samples = segment_samples.len(),
                    "🎵 Extracted audio segment"
                );
            }
        }
        
        info!(
            segments_extracted = audio_segments.len(),
            segments_dir = %segments_dir.display(),
            "📁 Extracted individual audio segments"
        );
        
        Ok(audio_segments)
    }

    /// Find matching transcript segment for audio timing
    fn find_matching_transcript_segment(&self, session: &AudioRecordingSession, start_time: Duration, end_time: Duration) -> (Option<String>, Option<f32>) {
        for transcript_segment in &session.transcript_segments {
            // Check for overlap with some tolerance
            let tolerance = Duration::from_millis(500);
            let transcript_start = transcript_segment.start_time;
            let transcript_end = transcript_segment.end_time;
            
            // Check if there's significant overlap
            let overlap_start = start_time.max(transcript_start);
            let overlap_end = end_time.min(transcript_end);
            
            if overlap_start < overlap_end {
                let overlap_duration = overlap_end - overlap_start;
                let audio_duration = end_time - start_time;
                let transcript_duration = transcript_end - transcript_start;
                
                // If overlap is significant (>50% of either segment)
                let overlap_ratio_audio = overlap_duration.as_secs_f32() / audio_duration.as_secs_f32();
                let overlap_ratio_transcript = overlap_duration.as_secs_f32() / transcript_duration.as_secs_f32();
                
                if overlap_ratio_audio > 0.5 || overlap_ratio_transcript > 0.5 {
                    return (Some(transcript_segment.text.clone()), Some(transcript_segment.confidence));
                }
            }
        }
        
        (None, None)
    }

    /// Save comprehensive metadata including session info and all outputs
    fn save_comprehensive_metadata(&self, session: &AudioRecordingSession, outputs: &SessionOutputs) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Serialize)]
        struct ComprehensiveMetadata {
            session: AudioRecordingSession,
            outputs: SessionOutputs,
            generated_at: DateTime<Utc>,
            version: String,
        }
        
        let metadata = ComprehensiveMetadata {
            session: session.clone(),
            outputs: outputs.clone(),
            generated_at: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        
        let file = File::create(&outputs.metadata_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &metadata)?;
        
        info!(
            metadata_path = %outputs.metadata_path.display(),
            session_id = %session.id,
            "📋 Saved comprehensive session metadata"
        );
        
        Ok(())
    }

    /// Save session metadata to JSON file (legacy method, kept for compatibility)
    fn save_session_metadata(&self, session: &AudioRecordingSession) -> Result<(), Box<dyn std::error::Error>> {
        let metadata_path = session.file_path.with_extension("json");
        let file = File::create(&metadata_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, session)?;

        debug!(
            metadata_path = %metadata_path.display(),
            "💾 Saved session metadata (legacy format)"
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

    /// Add test audio data (for testing purposes only)
    pub fn add_test_audio_data(&self, samples: &[f32]) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut buffer) = self.audio_buffer.lock() {
            buffer.extend_from_slice(samples);
        }
        Ok(())
    }

    /// Get current audio buffer size (for debugging)
    pub fn get_audio_buffer_size(&self) -> usize {
        if let Ok(buffer) = self.audio_buffer.lock() {
            buffer.len()
        } else {
            0
        }
    }

    /// Check if audio service is capturing
    pub fn is_audio_service_capturing(&self) -> bool {
        if let Ok(audio_service) = self.audio_service.lock() {
            audio_service.is_capturing()
        } else {
            false
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

/// Audio segment for individual file extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSegment {
    pub id: Uuid,
    pub start_sample: usize,
    pub end_sample: usize,
    pub start_time: Duration,
    pub end_time: Duration,
    pub duration: Duration,
    pub text: Option<String>,
    pub confidence: Option<f32>,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub is_speech: bool,
    pub average_energy: f32,
}

/// Session output files structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOutputs {
    pub raw_audio_path: PathBuf,
    pub cleaned_audio_path: PathBuf,
    pub segments_directory: PathBuf,
    pub metadata_path: PathBuf,
    pub segments: Vec<AudioSegment>,
    pub total_raw_duration: Duration,
    pub total_cleaned_duration: Duration,
    pub silence_removed_duration: Duration,
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
