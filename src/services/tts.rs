//! Text-to-Speech service for providing audio feedback during testing.

use std::sync::{Arc, Mutex};
use std::process::Command;
use tracing::{info, error, debug};
use tts::Tts;

/// TTS service for providing audio feedback
pub struct TTSService {
    tts: Option<Arc<Mutex<Tts>>>,
    enabled: bool,
    use_native_macos: bool,
}

impl TTSService {
    /// Create a new TTS service
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing TTS service...");
        
        // Check if we're on macOS and prefer native say command
        let use_native_macos = cfg!(target_os = "macos");
        
        if use_native_macos {
            info!("Using native macOS 'say' command for TTS");
            
            // Test the say command
            match Command::new("say").arg("TTS initialized").output() {
                Ok(_) => info!("Native macOS TTS service initialized and tested successfully"),
                Err(e) => {
                    error!("Native macOS TTS test failed: {}", e);
                    return Err(format!("Failed to initialize native macOS TTS: {}", e).into());
                }
            }
            
            Ok(Self {
                tts: None, // Don't need the tts crate on macOS
                enabled: true,
                use_native_macos: true,
            })
        } else {
            let mut tts = Tts::default()?;
            
            // Set volume to maximum to ensure we can hear it
            if let Err(e) = tts.set_volume(1.0.into()) {
                error!("Failed to set TTS volume: {}", e);
            } else {
                info!("TTS volume set to maximum");
            }
            
            // Set a reasonable speaking rate
            if let Err(e) = tts.set_rate(0.8.into()) {
                error!("Failed to set TTS rate: {}", e);
            } else {
                info!("TTS rate set to 0.8");
            }
            
            // Test TTS by speaking a short message (optional - don't fail if this doesn't work)
            match tts.speak("TTS initialized", true) {
                Ok(_) => info!("TTS service initialized and tested successfully"),
                Err(e) => {
                    error!("TTS test failed but continuing: {}", e);
                    // Don't fail initialization if test speech fails
                }
            }
            
            Ok(Self {
                tts: Some(Arc::new(Mutex::new(tts))),
                enabled: true,
                use_native_macos: false,
            })
        }
    }

    /// Enable or disable TTS feedback
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            info!("TTS feedback enabled");
        } else {
            info!("TTS feedback disabled");
        }
    }

    /// Check if TTS is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Speak text with TTS (non-blocking)
    pub async fn speak(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            debug!("TTS disabled, skipping: {}", text);
            return Ok(());
        }

        info!("TTS speaking (non-blocking): {}", text);
        
        if self.use_native_macos {
            // For non-blocking on macOS, spawn the say command in background
            let text_clone = text.to_string();
            tokio::spawn(async move {
                let _ = Command::new("say")
                    .arg(&text_clone)
                    .output();
            });
            info!("TTS successfully started speaking: {}", text);
        } else if let Some(ref tts_arc) = self.tts {
            let mut tts_guard = tts_arc.lock().unwrap();
            match tts_guard.speak(text, false) {
                Ok(_) => {
                    info!("TTS successfully started speaking: {}", text);
                }
                Err(e) => {
                    error!("TTS speak error: {}", e);
                    return Err(e.into());
                }
            }
        } else {
            return Err("TTS service not properly initialized".into());
        }
        
        Ok(())
    }

    /// Speak text and wait for completion
    pub async fn speak_and_wait(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            debug!("TTS disabled, skipping: {}", text);
            return Ok(());
        }

        info!("TTS speaking and waiting: {}", text);
        
        if self.use_native_macos {
            // Use native macOS say command which properly blocks
            let result = tokio::task::spawn_blocking({
                let text = text.to_string();
                move || {
                    Command::new("say")
                        .arg(&text)
                        .output()
                }
            }).await?;
            
            match result {
                Ok(output) => {
                    if output.status.success() {
                        info!("TTS completed speaking: {}", text);
                    } else {
                        let error_msg = String::from_utf8_lossy(&output.stderr);
                        error!("TTS speak error: {}", error_msg);
                        return Err(format!("TTS speak failed: {}", error_msg).into());
                    }
                }
                Err(e) => {
                    error!("TTS speak error: {}", e);
                    return Err(e.into());
                }
            }
        } else if let Some(ref tts_arc) = self.tts {
            // Use the tts crate for non-macOS platforms
            let mut tts_guard = tts_arc.lock().unwrap();
            
            // Ensure volume is set high before speaking
            if let Err(e) = tts_guard.set_volume(1.0.into()) {
                error!("Failed to set volume before speaking: {}", e);
            }
            
            // Start speaking (non-blocking first to avoid issues with blocking)
            match tts_guard.speak(text, false) {
                Ok(_) => {
                    info!("TTS started speaking: {}", text);
                }
                Err(e) => {
                    error!("TTS speak error: {}", e);
                    return Err(e.into());
                }
            }
            
            // Release the lock while we wait
            drop(tts_guard);
            
            // Calculate estimated speaking time based on text length
            // Average speaking rate is about 150-200 words per minute
            // We'll use 120 WPM to be conservative, plus some padding
            let word_count = text.split_whitespace().count();
            let estimated_duration_ms = if word_count == 0 {
                500 // Minimum duration for very short text
            } else {
                // 120 WPM = 2 words per second = 500ms per word
                (word_count * 500).max(1000) // At least 1 second
            };
            
            info!("Estimated speaking duration: {}ms for {} words", estimated_duration_ms, word_count);
            
            // Wait for the estimated duration
            tokio::time::sleep(tokio::time::Duration::from_millis(estimated_duration_ms as u64)).await;
            
            // Wait a bit more to ensure completion
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            
            info!("TTS completed speaking: {}", text);
        } else {
            return Err("TTS service not properly initialized".into());
        }
        
        info!("TTS finished speaking: {}", text);
        Ok(())
    }

    /// Stop current speech
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.use_native_macos {
            // For macOS, we can't easily stop the say command once started
            // The say command runs to completion
            debug!("TTS stop requested (native macOS say command cannot be interrupted)");
        } else if let Some(ref tts_arc) = self.tts {
            let mut tts_guard = tts_arc.lock().unwrap();
            tts_guard.stop()?;
            debug!("TTS stopped");
        }
        Ok(())
    }

    /// Get available voices
    pub async fn get_voices(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if self.use_native_macos {
            // For macOS, we could list voices using `say -v ?` but for now return a default
            Ok(vec!["Default".to_string()])
        } else if let Some(ref tts_arc) = self.tts {
            let tts_guard = tts_arc.lock().unwrap();
            let voices = tts_guard.voices()?;
            let voice_names: Vec<String> = voices.iter()
                .map(|v| format!("{}", v.name()))
                .collect();
            Ok(voice_names)
        } else {
            Err("TTS service not properly initialized".into())
        }
    }

    /// Set speaking rate (0.1 to 10.0, 1.0 is normal)
    pub async fn set_rate(&self, rate: f32) -> Result<(), Box<dyn std::error::Error>> {
        if self.use_native_macos {
            // For macOS, we could use `say -r` parameter but for now just log
            debug!("TTS rate setting requested: {} (native macOS uses default rate)", rate);
        } else if let Some(ref tts_arc) = self.tts {
            let mut tts_guard = tts_arc.lock().unwrap();
            tts_guard.set_rate(rate.into())?;
            debug!("TTS rate set to: {}", rate);
        }
        Ok(())
    }

    /// Set volume (0.0 to 1.0)
    pub async fn set_volume(&self, volume: f32) -> Result<(), Box<dyn std::error::Error>> {
        if self.use_native_macos {
            // For macOS, we could use system volume controls but for now just log
            debug!("TTS volume setting requested: {} (native macOS uses system volume)", volume);
        } else if let Some(ref tts_arc) = self.tts {
            let mut tts_guard = tts_arc.lock().unwrap();
            tts_guard.set_volume(volume.into())?;
            debug!("TTS volume set to: {}", volume);
        }
        Ok(())
    }
}

/// Helper functions for common TTS feedback messages
impl TTSService {
    /// Announce the start of recording for a specific phrase
    pub async fn announce_recording_start(&self, phrase_number: usize, phrase: &str) -> Result<(), Box<dyn std::error::Error>> {
        let message = format!("Starting recording session for phrase {}. You will now record: {}", phrase_number, phrase);
        self.speak_and_wait(&message).await
    }

    /// Announce that we're about to begin recording
    pub async fn announce_recording_beginning(&self) -> Result<(), Box<dyn std::error::Error>> {
        let message = "Beginning test recording session. I will guide you through each step.";
        self.speak_and_wait(message).await
    }

    /// Announce successful recording completion
    pub async fn announce_recording_success(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let message = format!("Recording saved successfully as {}", filename);
        self.speak_and_wait(&message).await
    }

    /// Announce recording validation results
    pub async fn announce_validation_result(&self, expected: &str, actual: &str, is_good: bool) -> Result<(), Box<dyn std::error::Error>> {
        let message = if is_good {
            format!("Good recording! Expected '{}', got '{}'", expected, actual)
        } else {
            format!("Recording may need improvement. Expected '{}', but got '{}'", expected, actual)
        };
        self.speak_and_wait(&message).await
    }

    /// Announce recording failure with specific guidance
    pub async fn announce_recording_failure(&self, reason: &str, phrase_number: usize) -> Result<(), Box<dyn std::error::Error>> {
        let message = format!("Recording failed: {}. Please try recording phrase {} again. Say 'start test recording next' when ready.", reason, phrase_number);
        self.speak_and_wait(&message).await
    }

    /// Announce recording contains only silence
    pub async fn announce_silence_detected(&self, phrase_number: usize) -> Result<(), Box<dyn std::error::Error>> {
        let message = format!("Recording contains only silence. Please speak louder and closer to the microphone. Try recording phrase {} again.", phrase_number);
        self.speak_and_wait(&message).await
    }

    /// Announce transcription failure
    pub async fn announce_transcription_failure(&self, phrase_number: usize) -> Result<(), Box<dyn std::error::Error>> {
        let message = format!("Could not understand the recording. Please speak more clearly and try recording phrase {} again.", phrase_number);
        self.speak_and_wait(&message).await
    }

    /// Announce phase instructions
    pub async fn announce_phase_instructions(&self, phase: &str) -> Result<(), Box<dyn std::error::Error>> {
        let message = match phase {
            "startup" => "Welcome to the enhanced test recorder. I will guide you through recording training data for voice commands. Make sure you are in a quiet environment with a good microphone.",
            "intro" => "This tool will help you record 66 voice command phrases for testing. I will provide audio feedback and guidance throughout the process. To begin, say 'start test recording next' or 'start recording with countdown'.",
            "ready" => "Ready to begin recording. Say 'start test recording next' to record the current phrase, or 'show current phrase' to hear it again.",
            "recording_active" => "Recording is now active. Speak the phrase clearly, then say 'stop test recording' when finished.",
            "validation" => "Validating your recording. Please wait while I check if it matches the expected phrase.",
            "next_phrase" => "Moving to the next phrase. Say 'start test recording next' when you're ready to continue.",
            "completion" => "Excellent work! You have completed all test recordings. The training data is now ready for testing voice command recognition.",
            _ => phase,
        };
        self.speak_and_wait(message).await
    }

    /// Announce recording tips and guidance
    pub async fn announce_recording_tips(&self) -> Result<(), Box<dyn std::error::Error>> {
        let message = "Recording tips: Speak clearly and at normal pace. Wait for the recording prompt before speaking. Make sure you're in a quiet environment. Say each word distinctly.";
        self.speak_and_wait(message).await
    }

    /// Announce progress update
    pub async fn announce_progress(&self, current: usize, total: usize) -> Result<(), Box<dyn std::error::Error>> {
        let message = format!("Progress: {} of {} phrases completed. {} remaining.", current, total, total - current);
        self.speak_and_wait(&message).await
    }

    /// Announce next steps or instructions
    pub async fn announce_instruction(&self, instruction: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.speak_and_wait(instruction).await
    }

    /// Announce error conditions
    pub async fn announce_error(&self, error_msg: &str) -> Result<(), Box<dyn std::error::Error>> {
        let message = format!("Error: {}", error_msg);
        self.speak_and_wait(&message).await
    }

    /// Announce completion of all tests
    pub async fn announce_completion(&self) -> Result<(), Box<dyn std::error::Error>> {
        let message = "All test recordings completed successfully! Test files are ready for parser testing.";
        self.speak_and_wait(message).await
    }

    /// Announce countdown for recording
    pub async fn announce_countdown(&self, seconds: u8) -> Result<(), Box<dyn std::error::Error>> {
        let message = match seconds {
            3 => "Recording starts in 3",
            2 => "2",
            1 => "1",
            0 => "Now! Speak the phrase.",
            _ => return Ok(()),
        };
        self.speak_and_wait(message).await
    }

    /// Announce full countdown sequence with preparation
    pub async fn announce_recording_countdown(&self, phrase: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.speak_and_wait(&format!("Get ready to say: {}", phrase)).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        for i in (1..=3).rev() {
            self.announce_countdown(i).await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
        
        self.announce_countdown(0).await?;
        Ok(())
    }

    /// Announce specific guidance based on transcription mismatch
    pub async fn announce_specific_guidance(&self, expected: &str, actual: &str, guidance: &str) -> Result<(), Box<dyn std::error::Error>> {
        let message = format!("Expected '{}', but heard '{}'. {}. Please try again.", expected, actual, guidance);
        self.speak_and_wait(&message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tts_service_creation() {
        let result = TTSService::new();
        // TTS might not be available in test environment, so we just check it doesn't panic
        match result {
            Ok(mut service) => {
                assert!(service.is_enabled());
                service.set_enabled(false);
                assert!(!service.is_enabled());
            }
            Err(_) => {
                // TTS not available in test environment, which is fine
            }
        }
    }
}
