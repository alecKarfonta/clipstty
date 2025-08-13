//! Audio service for capturing and processing audio input.

use crate::{core::types::*, Result};
use crate::services::vad::VADService;
use crate::core::types::VADResult;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, Stream};
use tracing::{error, info};
use std::sync::{Arc, Mutex};

/// Audio service for managing audio capture and processing
pub struct AudioService {
    input_device: Option<cpal::Device>,
    input_stream: Option<Stream>,
    capturing: bool,
    selected_device_name: Option<String>,
    vad: Option<Arc<Mutex<VADService>>>,
    vad_callback: Option<Arc<dyn Fn(VADResult) + Send + Sync>>, 
    audio_callback: Option<Arc<dyn Fn(&[f32], u32) + Send + Sync>>,
    ptt_active: bool,
}

impl AudioService {
    /// Create a new audio service
    pub fn new() -> Result<Self> {
        Ok(Self {
            input_device: None,
            input_stream: None,
            capturing: false,
            selected_device_name: None,
            vad: None,
            vad_callback: None,
            audio_callback: None,
            ptt_active: false,
        })
    }

	/// Select input device by name (None to use system default)
	pub fn select_input_device_by_name(&mut self, name: Option<String>) {
		self.selected_device_name = name;
	}

    /// Attach a VAD service used to gate/monitor voice activity
    pub fn attach_vad(&mut self, vad: Arc<Mutex<VADService>>) {
        self.vad = Some(vad);
    }

    /// Register a callback to be invoked when VAD detects voice
    pub fn on_vad_event<F>(&mut self, callback: F)
    where
        F: Fn(VADResult) + Send + Sync + 'static,
    {
        self.vad_callback = Some(Arc::new(callback));
    }

    /// Register a callback to receive raw mono f32 frames and sample rate
    pub fn on_audio_frame<F>(&mut self, callback: F)
    where
        F: Fn(&[f32], u32) + Send + Sync + 'static,
    {
        self.audio_callback = Some(Arc::new(callback));
    }

    /// Set Push-to-Talk gate state. When using VAD in PushToTalk/Toggle modes,
    /// this controls whether frames can be considered for speech.
    pub fn set_ptt_gate(&mut self, open: bool) {
        self.ptt_active = open;
        if let Some(vad) = &self.vad {
            if let Ok(mut g) = vad.lock() {
                g.set_gate(open);
            }
        }
    }

    /// Toggle capture state (used for Toggle mode)
    pub fn toggle_capture(&mut self) -> Result<bool> {
        if self.is_capturing() {
            self.stop_capture()?;
            self.set_ptt_gate(false);
            Ok(false)
        } else {
            self.start_capture()?;
            self.set_ptt_gate(true);
            Ok(true)
        }
    }

    /// Start audio capture
    pub fn start_capture(&mut self) -> Result<()> {
        if self.capturing {
            return Ok(());
        }

        let host = cpal::default_host();

        let device = match &self.selected_device_name {
            Some(name) => {
                let mut found: Option<cpal::Device> = None;
                if let Ok(mut devices) = host.input_devices() {
                    for d in devices.by_ref() {
                        if let Ok(dname) = d.name() {
                            if &dname == name {
                                found = Some(d);
                                break;
                            }
                        }
                    }
                }
                found.ok_or_else(|| crate::core::error::AudioError::DeviceNotFound(name.clone()))?
            }
            None => host
                .default_input_device()
                .ok_or_else(|| crate::core::error::AudioError::DeviceNotFound("default".into()))?,
        };

        let supported_config = device
            .default_input_config()
            .map_err(|e| crate::core::error::AudioError::DeviceInit(e.to_string()))?;

        let config: cpal::StreamConfig = supported_config.clone().config();
        let sample_format = supported_config.sample_format();

        let err_fn = |err| {
            error!("Audio input stream error: {err}");
        };

        info!(
            device = %device.name().unwrap_or_else(|_| "<unknown>".into()),
            sample_rate = config.sample_rate.0,
            channels = config.channels,
            "Starting audio capture"
        );

        // Start VAD if attached
        if let Some(vad) = &self.vad {
            if let Ok(mut g) = vad.lock() {
                let _ = g.start();
            }
        }

        let stream = match sample_format {
            SampleFormat::F32 => build_stream::<f32>(&device, &config, err_fn, self.vad.clone(), self.vad_callback.clone(), self.audio_callback.clone())?,
            SampleFormat::I16 => build_stream::<i16>(&device, &config, err_fn, self.vad.clone(), self.vad_callback.clone(), self.audio_callback.clone())?,
            SampleFormat::U16 => build_stream::<u16>(&device, &config, err_fn, self.vad.clone(), self.vad_callback.clone(), self.audio_callback.clone())?,
            _ => return Err(crate::core::error::AudioError::UnsupportedFormat(format!("{sample_format:?}")).into()),
        };

        stream
            .play()
            .map_err(|e| crate::core::error::AudioError::CaptureStart(e.to_string()))?;

        self.input_stream = Some(stream);
        self.input_device = Some(device);
        self.capturing = true;
        Ok(())
    }

    /// Stop audio capture
    pub fn stop_capture(&mut self) -> Result<()> {
        if !self.capturing {
            return Ok(());
        }

        info!("Stopping audio capture");
        // Dropping the stream stops capture
        self.input_stream = None;
        self.input_device = None;
        self.capturing = false;
        // Stop VAD if attached
        if let Some(vad) = &self.vad {
            if let Ok(mut g) = vad.lock() {
                let _ = g.stop();
            }
        }
        Ok(())
    }

    /// Check if currently capturing
    pub fn is_capturing(&self) -> bool {
        self.capturing
    }

    /// Get available audio devices
    pub fn get_devices(&self) -> Result<Vec<AudioDevice>> {
        let host = cpal::default_host();

        let default_input_name = host
            .default_input_device()
            .and_then(|d| d.name().ok())
            .unwrap_or_default();
        let default_output_name = host
            .default_output_device()
            .and_then(|d| d.name().ok())
            .unwrap_or_default();

        let mut devices_info: Vec<AudioDevice> = Vec::new();

        // Input devices
        if let Ok(devices) = host.input_devices() {
            for device in devices {
                let name = device.name().unwrap_or_else(|_| "<unknown>".into());
                let mut sample_rates: Vec<u32> = Vec::new();
                let mut channels: Vec<u16> = Vec::new();

                if let Ok(configs) = device.supported_input_configs() {
                    for cfg in configs {
                        let sr_min = cfg.min_sample_rate().0;
                        let sr_max = cfg.max_sample_rate().0;
                        // Store range endpoints to indicate capability without enumerating all
                        if !sample_rates.contains(&sr_min) {
                            sample_rates.push(sr_min);
                        }
                        if !sample_rates.contains(&sr_max) {
                            sample_rates.push(sr_max);
                        }
                        let ch = cfg.channels();
                        if !channels.contains(&ch) {
                            channels.push(ch);
                        }
                    }
                }

                devices_info.push(AudioDevice {
                    name: name.clone(),
                    id: name.clone(),
                    sample_rates,
                    channels,
                    is_default: name == default_input_name,
                    device_type: AudioDeviceType::Input,
                });
            }
        }

        // Output devices (optional, useful for duplex display)
        if let Ok(devices) = host.output_devices() {
            for device in devices {
                let name = device.name().unwrap_or_else(|_| "<unknown>".into());
                let mut sample_rates: Vec<u32> = Vec::new();
                let mut channels: Vec<u16> = Vec::new();

                if let Ok(configs) = device.supported_output_configs() {
                    for cfg in configs {
                        let sr_min = cfg.min_sample_rate().0;
                        let sr_max = cfg.max_sample_rate().0;
                        if !sample_rates.contains(&sr_min) {
                            sample_rates.push(sr_min);
                        }
                        if !sample_rates.contains(&sr_max) {
                            sample_rates.push(sr_max);
                        }
                        let ch = cfg.channels();
                        if !channels.contains(&ch) {
                            channels.push(ch);
                        }
                    }
                }

                devices_info.push(AudioDevice {
                    name: name.clone(),
                    id: name.clone(),
                    sample_rates,
                    channels,
                    is_default: name == default_output_name,
                    device_type: AudioDeviceType::Output,
                });
            }
        }

        Ok(devices_info)
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    err_fn: impl Fn(cpal::StreamError) + Send + 'static,
    vad: Option<Arc<Mutex<VADService>>>,
    vad_callback: Option<Arc<dyn Fn(VADResult) + Send + Sync>>, 
    audio_cb: Option<Arc<dyn Fn(&[f32], u32) + Send + Sync>>,
) -> Result<Stream>
where
    T: Sample + Send + 'static + cpal::SizedSample,
    T: cpal::FromSample<f32>,
    f32: cpal::FromSample<T>,
{
    // Basic skeleton: receive data, convert to mono f32, and feed into VAD (if attached)
    let channels = config.channels as usize;
    let sample_rate = config.sample_rate.0;
    let stream = device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                // Convert interleaved frames to mono f32 samples
                let mono: Vec<f32> = if channels == 1 {
                    data.iter().map(|s| (*s).to_sample::<f32>()).collect()
                } else {
                    data
                        .chunks(channels)
                        .map(|frame| {
                            let sum: f32 = frame.iter().map(|s| (*s).to_sample::<f32>()).sum();
                            sum / channels as f32
                        })
                        .collect()
                };

                if let Some(vad_ref) = vad.as_ref() {
                    if let Ok(mut g) = vad_ref.lock() {
                        if let Ok(vr) = g.process_frame(&mono, sample_rate) {
                            if vr.voice_detected {
                                if let Some(cb) = vad_callback.as_ref() {
                                    (cb)(vr.clone());
                                }
                            }
                        }
                    }
                }

                // Emit raw audio frame to registered listener
                if let Some(cb) = audio_cb.as_ref() {
                    (cb)(&mono, sample_rate);
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| crate::core::error::AudioError::CaptureStart(e.to_string()))?;
    Ok(stream)
}
