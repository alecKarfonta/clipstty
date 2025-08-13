//! Voice Activity Detection (VAD) service.

use crate::{core::types::VADResult, Result};
use chrono::Utc;
use std::time::{Duration, Instant};
use tracing::{info};

/// VAD operating mode
#[derive(Debug, Clone, Copy)]
pub enum VADMode {
    /// Voice detection gates recording automatically
    Auto,
    /// Push-to-talk (managed by hotkeys elsewhere)
    PushToTalk,
    /// Toggle recording on/off (managed by hotkeys elsewhere)
    Toggle,
}

/// VAD service scaffold
pub struct VADService {
    sensitivity: f32,
    timeout_ms: u64,
    mode: VADMode,
    active: bool,
    gate_open: bool,

    // Internal state for simple energy-based VAD
    energy_threshold: f32,
    last_voice_instant: Option<Instant>,
    current_segment_start: Option<Instant>,
}

impl VADService {
    /// Create a new VAD service
    pub fn new(sensitivity: f32, timeout_ms: u64, mode: VADMode) -> Result<Self> {
        let mut service = Self {
            sensitivity,
            timeout_ms,
            mode,
            active: false,
            gate_open: false,
            energy_threshold: 0.0,
            last_voice_instant: None,
            current_segment_start: None,
        };
        service.recompute_threshold();
        Ok(service)
    }

    /// Start VAD processing
    pub fn start(&mut self) -> Result<()> {
        self.active = true;
        self.last_voice_instant = None;
        self.current_segment_start = None;
        Ok(())
    }

    /// Stop VAD processing
    pub fn stop(&mut self) -> Result<()> {
        if self.active {
            info!("Stopping VAD");
            self.active = false;
        }
        Ok(())
    }

    /// Check if VAD is running
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Update sensitivity (0.0..=1.0)
    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity.clamp(0.0, 1.0);
        self.recompute_threshold();
    }

    /// Get current sensitivity
    pub fn sensitivity(&self) -> f32 {
        self.sensitivity
    }

    /// Set VAD mode
    pub fn set_mode(&mut self, mode: VADMode) {
        self.mode = mode;
    }

    /// Get VAD mode
    pub fn mode(&self) -> VADMode {
        self.mode
    }

    /// Open/close the gate used by Push-to-Talk/Toggle modes
    pub fn set_gate(&mut self, open: bool) {
        self.gate_open = open;
        if !open {
            // Reset segment tracking when gate closes
            self.current_segment_start = None;
            self.last_voice_instant = None;
        }
    }

    /// Process a frame of audio samples (mono f32 in [-1.0, 1.0]) and produce a `VADResult`.
    ///
    /// `sample_rate` is used to compute timing for hangover/timeout behavior.
    pub fn process_frame(&mut self, samples: &[f32], sample_rate: u32) -> Result<VADResult> {
        if !self.active {
            // When inactive, always return no voice
            return Ok(VADResult {
                voice_detected: false,
                confidence: 0.0,
                timestamp: Utc::now(),
                duration_ms: 0,
            });
        }

        let now = Instant::now();
        // If PTT/Toggle and gate is closed, do not signal voice
        match self.mode {
            VADMode::PushToTalk | VADMode::Toggle => {
                if !self.gate_open {
                    return Ok(VADResult {
                        voice_detected: false,
                        confidence: 0.0,
                        timestamp: Utc::now(),
                        duration_ms: 0,
                    });
                }
            }
            VADMode::Auto => {}
        }

        let frame_energy = compute_frame_energy(samples);

        let mut voice_detected = frame_energy >= self.energy_threshold;

        // Hangover logic based on timeout_ms
        let timeout = Duration::from_millis(self.timeout_ms);
        if voice_detected {
            if self.current_segment_start.is_none() {
                self.current_segment_start = Some(now);
            }
            self.last_voice_instant = Some(now);
        } else if let Some(last) = self.last_voice_instant {
            if now.duration_since(last) <= timeout {
                voice_detected = true;
            } else {
                // Segment ended
                self.current_segment_start = None;
                self.last_voice_instant = None;
            }
        }

        let duration_ms: u64 = if let Some(start) = self.current_segment_start {
            now.duration_since(start).as_millis() as u64
        } else {
            0
        };

        // Confidence: relative energy to threshold, capped at 1.0
        let confidence = if self.energy_threshold > 0.0 {
            (frame_energy / (self.energy_threshold * 4.0)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let result = VADResult {
            voice_detected,
            confidence,
            timestamp: Utc::now(),
            duration_ms,
        };

        // Optional light adaptation: slightly raise/lower threshold towards observed energy
        self.adapt_threshold(frame_energy, sample_rate);

        Ok(result)
    }

    fn recompute_threshold(&mut self) {
        // Map sensitivity (0..1) to an energy threshold range.
        // Higher sensitivity -> lower threshold.
        let sensitivity = self.sensitivity.clamp(0.0, 1.0);
        let min_thr = 5e-4_f32;
        let max_thr = 1e-2_f32;
        self.energy_threshold = min_thr + (1.0 - sensitivity) * (max_thr - min_thr);
    }

    fn adapt_threshold(&mut self, observed_energy: f32, _sample_rate: u32) {
        // Gentle adaptation to track background noise while keeping stability
        let alpha = 0.01_f32;
        let target = observed_energy.max(self.energy_threshold);
        self.energy_threshold = (1.0 - alpha) * self.energy_threshold + alpha * target;
        // Keep within sane bounds
        let min_thr = 1e-5_f32;
        let max_thr = 5e-2_f32;
        if self.energy_threshold < min_thr {
            self.energy_threshold = min_thr;
        } else if self.energy_threshold > max_thr {
            self.energy_threshold = max_thr;
        }
    }
}

fn compute_frame_energy(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    sum_sq / samples.len() as f32
}


