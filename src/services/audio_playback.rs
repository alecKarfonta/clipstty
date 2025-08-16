//! Audio playback service for playing back recorded audio files.

use crate::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use hound::WavReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tracing::{error, info};

/// Audio playback service for playing WAV files
pub struct AudioPlaybackService {
    output_device: Option<cpal::Device>,
    is_playing: Arc<AtomicBool>,
}

impl AudioPlaybackService {
    /// Create a new audio playback service
    pub fn new() -> Result<Self> {
        Ok(Self {
            output_device: None,
            is_playing: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Play a WAV file and block until playback is complete
    pub fn play_wav_file(&mut self, file_path: &Path) -> Result<()> {
        info!("Playing audio file: {}", file_path.display());
        
        // Read the WAV file
        let mut reader = WavReader::open(file_path)
            .map_err(|e| crate::core::error::AudioError::PlaybackError(format!("Failed to open WAV file: {}", e)))?;
        
        let spec = reader.spec();
        let samples: std::result::Result<Vec<f32>, hound::Error> = reader.samples::<i16>()
            .map(|s| s.map(|sample| sample as f32 / 32768.0))
            .collect();
        
        let audio_samples = samples
            .map_err(|e| crate::core::error::AudioError::PlaybackError(format!("Failed to read audio samples: {}", e)))?;
        
        if audio_samples.is_empty() {
            return Err(crate::core::error::AudioError::PlaybackError("Audio file is empty".to_string()).into());
        }
        
        // Get the default output device
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| crate::core::error::AudioError::PlaybackError("No output device available".to_string()))?;
        
        // Configure the output stream to match the file's sample rate
        let config = cpal::StreamConfig {
            channels: 1, // We convert to mono
            sample_rate: cpal::SampleRate(spec.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };
        
        // Prepare audio data for playback
        let audio_data = Arc::new(Mutex::new(audio_samples));
        let sample_index = Arc::new(Mutex::new(0usize));
        let is_playing_clone = self.is_playing.clone();
        let data_ref = audio_data.clone();
        let index_ref = sample_index.clone();
        
        self.is_playing.store(true, Ordering::SeqCst);
        
        // Build the output stream
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut index = index_ref.lock().unwrap();
                let audio = data_ref.lock().unwrap();
                
                for sample in data.iter_mut() {
                    if *index < audio.len() {
                        *sample = audio[*index];
                        *index += 1;
                    } else {
                        *sample = 0.0; // Silence after audio ends
                    }
                }
                
                // Check if we've reached the end
                if *index >= audio.len() {
                    is_playing_clone.store(false, Ordering::SeqCst);
                }
            },
            |err| error!("Audio playback error: {}", err),
            None,
        ).map_err(|e| crate::core::error::AudioError::PlaybackError(format!("Failed to build output stream: {}", e)))?;
        
        // Start playback
        stream.play()
            .map_err(|e| crate::core::error::AudioError::PlaybackError(format!("Failed to start playback: {}", e)))?;
        
        // Wait for playback to complete
        while self.is_playing.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(10));
        }
        
        // Give a small buffer for the audio to finish playing
        thread::sleep(Duration::from_millis(100));
        
        info!("Audio playback completed");
        Ok(())
    }

    /// Check if currently playing audio
    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }

    /// Stop playback (if playing)
    pub fn stop(&self) {
        self.is_playing.store(false, Ordering::SeqCst);
    }
}

impl Default for AudioPlaybackService {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            output_device: None,
            is_playing: Arc::new(AtomicBool::new(false)),
        })
    }
}
