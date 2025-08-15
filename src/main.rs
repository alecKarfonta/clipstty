//! STT Clippy - Main Application Entry Point
//!
//! This is the main entry point for the STT Clippy desktop application.

use std::process;
use std::sync::{Arc, Mutex};
use stt_clippy::{cleanup, init, get_config, Result};
use stt_clippy::services::{AudioService, STTService, VADService, VADMode, PasteService, ClipboardService};
use stt_clippy::core::config::ActivationMode;
use stt_clippy::services::hotkey::HotkeyService;
use stt_clippy::core::types::Hotkey;
use stt_clippy::ui::SystemTray;
use tracing::{info, debug, warn, error};
use std::time::{Duration, Instant};
// no atomic imports needed

// Styled log helpers (match style used in STT service)
const C_CLASS: &str = "\x1b[35m"; // magenta
const C_FUNC: &str = "\x1b[36m"; // cyan
const C_VAR: &str = "\x1b[33m"; // yellow
const C_VAL: &str = "\x1b[32m"; // green
const C_RESET: &str = "\x1b[0m";

fn fmt_log(class_name: &str, func_name: &str, message: &str) -> String {
    format!(
        "[{}{}{}].{}{}{} {}",
        C_CLASS, class_name, C_RESET,
        C_FUNC, func_name, C_RESET,
        message
    )
}

#[tokio::main]
async fn main() {
    // Initialize the application
    if let Err(e) = init(None, None) {
        eprintln!("Failed to initialize STT Clippy: {e}");
        process::exit(1);
    }

    // Run the main application
    if let Err(e) = run_app().await {
        eprintln!("Application error: {e}");
        process::exit(1);
    }

    // Cleanup before exit
    if let Err(e) = cleanup() {
        eprintln!("Failed to cleanup STT Clippy: {e}");
        process::exit(1);
    }
}

/// Main application loop
async fn run_app() -> Result<()> {
    info!("{}", fmt_log("Main", "run_app", &format!(
        "{}version{}={}{}{}",
        C_VAR, C_RESET, C_VAL, env!("CARGO_PKG_VERSION"), C_RESET
    )));

    let cfg = get_config().clone();

    // Minimal tray bootstrap
    let tray = SystemTray::new()?;
    if cfg.ui.show_tray { tray.show()?; }

    // Initialize STT / Clipboard with config
    let mut stt = STTService::new()?;
    stt.apply_config(cfg.stt.clone())?;
    let mut clipboard = ClipboardService::new()?;

    // Initialize VAD using config (only if enabled)
    let maybe_vad = if cfg.audio.enable_vad {
        Some(VADService::new(
            cfg.audio.vad_sensitivity,
            cfg.audio.vad_timeout,
            match cfg.audio.activation_mode {
                ActivationMode::PushToTalk => VADMode::PushToTalk,
                ActivationMode::Toggle => VADMode::Toggle,
            },
        )?)
    } else {
        None
    };

    // Enumerate audio devices and start/stop
    let mut audio = AudioService::new()?;
    if !cfg.audio.device_name.is_empty() {
        audio.select_input_device_by_name(Some(cfg.audio.device_name.clone()));
    }
    let devices = audio.get_devices()?;
    debug!("{}", fmt_log("Main", "run_app", &format!(
        "Audio devices enumerated {}available_devices{}={}{}{}",
        C_VAR, C_RESET, C_VAL, devices.len(), C_RESET
    )));

    // Attach VAD and register handler (only if enabled)
    if let Some(vad) = maybe_vad {
        let vad_shared = Arc::new(Mutex::new(vad));
        audio.attach_vad(vad_shared);
    }

    // Shared audio buffer to collect frames during capture sessions
    let captured: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let sample_rate: Arc<Mutex<u32>> = Arc::new(Mutex::new(16000));
    {
        let captured_ref = captured.clone();
        let sr_ref = sample_rate.clone();
        audio.on_audio_frame(move |frame, sr| {
            if let Ok(mut buf) = captured_ref.lock() {
                buf.extend_from_slice(frame);
            }
            if let Ok(mut s) = sr_ref.lock() { *s = sr; }
        });
    }

    // Wrap audio in Arc<Mutex<...>> for hotkey handlers
    let audio_ref = Arc::new(Mutex::new(audio));

    // Hotkey wiring for Push-to-Talk / Toggle
    let mut hotkeys = HotkeyService::new()?;
    let primary = Hotkey::from_string(&cfg.hotkeys.primary)
        .unwrap_or_else(|_| Hotkey::new("S".to_string()).with_ctrl().with_alt());
    let _ = hotkeys.register_hotkey(&primary);
    debug!("{}", fmt_log("Main", "run_app", &format!(
        "Registered hotkey {}primary{}={}{}{}",
        C_VAR, C_RESET, C_VAL, primary.to_string(), C_RESET
    )));
    let toggle_vad_hk = Hotkey::from_string(&cfg.hotkeys.toggle_vad)
        .unwrap_or_else(|_| Hotkey::new("V".to_string()).with_ctrl().with_alt());
    let _ = hotkeys.register_hotkey(&toggle_vad_hk);
    debug!("{}", fmt_log("Main", "run_app", &format!(
        "Registered hotkey {}toggle_vad{}={}{}{}",
        C_VAR, C_RESET, C_VAL, toggle_vad_hk.to_string(), C_RESET
    )));
    let toggle_instant_hk = Hotkey::from_string(&cfg.hotkeys.toggle_instant_output)
        .unwrap_or_else(|_| Hotkey::new("P".to_string()).with_ctrl().with_alt());
    let _ = hotkeys.register_hotkey(&toggle_instant_hk);
    debug!("{}", fmt_log("Main", "run_app", &format!(
        "Registered hotkey {}toggle_instant_output{}={}{}{}",
        C_VAR, C_RESET, C_VAL, toggle_instant_hk.to_string(), C_RESET
    )));
    let toggle_narration_hk = Hotkey::from_string(&cfg.hotkeys.toggle_narration)
        .unwrap_or_else(|_| Hotkey::new("N".to_string()).with_ctrl().with_alt());
    let _ = hotkeys.register_hotkey(&toggle_narration_hk);
    debug!("{}", fmt_log("Main", "run_app", &format!(
        "Registered hotkey {}toggle_narration{}={}{}{}",
        C_VAR, C_RESET, C_VAL, toggle_narration_hk.to_string(), C_RESET
    )));

    let instant_output = Arc::new(Mutex::new(false));
    let narration_enabled = Arc::new(Mutex::new(false));
    let narration_state: Arc<Mutex<NarrationState>> = Arc::new(Mutex::new(NarrationState::new()));

    match cfg.audio.activation_mode {
        ActivationMode::PushToTalk => {
            let stt_ref = Arc::new(Mutex::new(stt));
            let clipboard_ref = Arc::new(Mutex::new(clipboard));
            let captured_ref = captured.clone();
            let sr_ref = sample_rate.clone();
            let inst_ref = instant_output.clone();
            let audio_press = audio_ref.clone();
            let audio_release = audio_ref.clone();
            // Clones for command application without moving outer refs
            let audio_cmd_ref = audio_ref.clone();
            let inst_cmd_ref = instant_output.clone();
            let narr_cmd_ref = narration_enabled.clone();
            let narr_guard_ref = narration_enabled.clone();
            hotkeys.on_press(&primary, Arc::new(move || {
                if *narr_guard_ref.lock().unwrap() { debug!("{}", fmt_log("Main", "ptt_on_press", "Narration enabled; ignoring PTT press")); return; }
                if let Ok(mut a) = audio_press.lock() {
                    if let Ok(mut buf) = captured_ref.lock() { buf.clear(); }
                    let _ = a.start_capture();
                    a.set_ptt_gate(true);
                    debug!("{}", fmt_log("Main", "ptt_on_press", "Started capture and opened PTT gate"));
                }
            }));
            let narr_guard_ref2 = narration_enabled.clone();
            let captured_release_ref = captured.clone();
            hotkeys.on_release(&primary, Arc::new(move || {
                if *narr_guard_ref2.lock().unwrap() { debug!("{}", fmt_log("Main", "ptt_on_release", "Narration enabled; ignoring PTT release")); return; }
                let audio_data: Vec<f32> = {
                    if let Ok(mut a) = audio_release.lock() {
                        a.set_ptt_gate(false);
                        let _ = a.stop_capture();
                        debug!("{}", fmt_log("Main", "ptt_on_release", "Stopped capture and closed PTT gate"));
                    }
                    captured_release_ref.lock().unwrap().clone()
                };
                if audio_data.is_empty() { return; }
                debug!("{}", fmt_log("Main", "ptt_on_release", &format!(
                    "Captured {}samples{}={}{}{}",
                    C_VAR, C_RESET, C_VAL, audio_data.len(), C_RESET
                )));
                let sr = *sr_ref.lock().unwrap();
                let do_instant = *inst_ref.lock().unwrap();
                let audio = if sr == 16000 { audio_data } else { resample_linear(&audio_data, sr, 16000) };
                if let Ok(mut s) = stt_ref.lock() {
                    match s.transcribe(&audio) {
                        Ok(res) => {
                            // Parse voice command(s)
                            if let Some(cmd) = parse_voice_command(&res.text) {
                                apply_voice_command(cmd, &audio_cmd_ref, &inst_cmd_ref, &narr_cmd_ref);
                                return;
                            }
                            if do_instant {
                                if let Ok(mut p) = PasteService::new() { let _ = p.clipboard_paste(&res.text, 100); } else { if let Ok(mut cb) = clipboard_ref.lock() { let _ = cb.copy_text(&res.text); } }
                            } else {
                                if let Ok(mut cb) = clipboard_ref.lock() { let _ = cb.copy_text(&res.text); }
                            }
                        }
                        Err(e) => { error!("{}", fmt_log("Main", "ptt_on_release", &format!("STT error {}err{}={}{}{}", C_VAR, C_RESET, C_VAL, e, C_RESET))); }
                    }
                }
            }));
            // Keep a reference to prevent drop
            let _keep_audio = audio_ref.clone();
        }
        ActivationMode::Toggle => {
            // Single press toggles capture/gate
            let stt_ref = Arc::new(Mutex::new(stt));
            let clipboard_ref = Arc::new(Mutex::new(clipboard));
            let captured_ref = captured.clone();
            let sr_ref = sample_rate.clone();
            let inst_ref = instant_output.clone();
            let audio_toggle = audio_ref.clone();
            let audio_cmd_ref = audio_ref.clone();
            let inst_cmd_ref = instant_output.clone();
            let narr_cmd_ref = narration_enabled.clone();
            let narr_guard_ref = narration_enabled.clone();
            hotkeys.on_press(&primary, Arc::new(move || {
                if *narr_guard_ref.lock().unwrap() { debug!("{}", fmt_log("Main", "toggle_on_press", "Narration enabled; ignoring Toggle press")); return; }
                if let Ok(mut a) = audio_toggle.lock() {
                    if a.is_capturing() {
                        let _ = a.toggle_capture();
                        debug!("{}", fmt_log("Main", "toggle_on_press", "Stopped capture (toggle mode)"));
                        // copy out data then transcribe
                        let audio_data = captured_ref.lock().unwrap().clone();
                        if audio_data.is_empty() { return; }
                        let sr = *sr_ref.lock().unwrap();
                        let do_instant = *inst_ref.lock().unwrap();
                        let audio = if sr == 16000 { audio_data } else { resample_linear(&audio_data, sr, 16000) };
                        if let Ok(mut s) = stt_ref.lock() {
                            match s.transcribe(&audio) {
                                Ok(res) => {
                                    if let Some(cmd) = parse_voice_command(&res.text) {
                                        apply_voice_command(cmd, &audio_cmd_ref, &inst_cmd_ref, &narr_cmd_ref);
                                        return;
                                    }
                                    if do_instant {
                                        if let Ok(mut p) = PasteService::new() { let _ = p.clipboard_paste(&res.text, 100); } else { if let Ok(mut cb) = clipboard_ref.lock() { let _ = cb.copy_text(&res.text); } }
                                    } else {
                                        if let Ok(mut cb) = clipboard_ref.lock() { let _ = cb.copy_text(&res.text); }
                                    }
                                }
                                Err(e) => { error!("{}", fmt_log("Main", "toggle_on_press", &format!("STT error {}err{}={}{}{}", C_VAR, C_RESET, C_VAL, e, C_RESET))); }
                            }
                        }
                    } else {
                        if let Ok(mut buf) = captured_ref.lock() { buf.clear(); }
                        let _ = a.toggle_capture();
                        debug!("{}", fmt_log("Main", "toggle_on_press", "Started capture and opened gate (toggle mode)"));
                    }
                }
            }));
        }
    }

    // Toggle VAD runtime enable/disable
    {
        let audio_ref2 = audio_ref.clone();
        hotkeys.on_press(&toggle_vad_hk, Arc::new(move || {
            if let Ok(mut a) = audio_ref2.lock() {
                let enable = !a.is_vad_enabled();
                a.set_vad_enabled(enable);
                info!("{}", fmt_log("Main", "toggle_vad", &format!(
                    "Toggled VAD {}enabled{}={}{}{}",
                    C_VAR, C_RESET, C_VAL, enable, C_RESET
                )));
            }
        }));
    }

    // Toggle instant output mode (paste vs clipboard)
    {
        let inst_ref = instant_output.clone();
        hotkeys.on_press(&toggle_instant_hk, Arc::new(move || {
            if let Ok(mut v) = inst_ref.lock() {
                *v = !*v;
                info!("{}", fmt_log("Main", "toggle_instant_output", &format!(
                    "Instant output toggled {}instant_output{}={}{}{}",
                    C_VAR, C_RESET, C_VAL, *v, C_RESET
                )));
            }
        }));
    }

    // Toggle narration mode
    {
        let narration_enabled_ref = narration_enabled.clone();
        let narration_state_ref = narration_state.clone();
        let audio_ref2 = audio_ref.clone();
        hotkeys.on_press(&toggle_narration_hk, Arc::new(move || {
            if let Ok(mut en) = narration_enabled_ref.lock() {
                *en = !*en;
                // Reset state on disable
                if !*en {
                    if let Ok(mut st) = narration_state_ref.lock() { st.reset(); }
                    if let Ok(mut a) = audio_ref2.lock() { let _ = a.stop_capture(); }
                    info!("{}", fmt_log("Main", "toggle_narration", &format!(
                        "Narration {}enabled{}={}{}{} (stopped capture)",
                        C_VAR, C_RESET, C_VAL, *en, C_RESET
                    )));
                } else {
                    if let Ok(mut a) = audio_ref2.lock() { let _ = a.start_capture(); }
                    info!("{}", fmt_log("Main", "toggle_narration", &format!(
                        "Narration {}enabled{}={}{}{} (started capture)",
                        C_VAR, C_RESET, C_VAL, *en, C_RESET
                    )));
                }
            }
        }));
    }

    // Background narration task
    {
        let narration_enabled_ref = narration_enabled.clone();
        let narration_state_ref = narration_state.clone();
        let captured_ref = captured.clone();
        let sample_rate_ref = sample_rate.clone();
        let stt_ref = Arc::new(Mutex::new(STTService::new()?));
        let paste_delay = cfg.paste.delay;
        tokio::spawn(async move {
            let mut last_check = Instant::now();
            loop {
                tokio::time::sleep(Duration::from_millis(120)).await;
                if !*narration_enabled_ref.lock().unwrap() { continue; }

                let audio_data = {
                    let buf = captured_ref.lock().unwrap();
                    buf.clone()
                };
                if audio_data.is_empty() { debug!("{}", fmt_log("Main", "narration_task", "No audio yet; skipping")); continue; }
                let sr = *sample_rate_ref.lock().unwrap();
                // Windowing and hangover
                let window_ms: u64 = 8000;
                let hangover_ms: u64 = 500;
                let now = Instant::now();
                let mut st = narration_state_ref.lock().unwrap();
                if now.duration_since(last_check).as_millis() as u64 > hangover_ms { st.maybe_finalize_pause(); }
                last_check = now;

                // Keep only last window_ms
                let samples_to_keep = ((window_ms as usize) * (sr as usize) / 1000).min(audio_data.len());
                let start_idx = audio_data.len().saturating_sub(samples_to_keep);
                let segment = &audio_data[start_idx..];
                // Resample to 16k if needed
                let segment_16k = if sr == 16000 { segment.to_vec() } else { resample_linear(segment, sr, 16000) };

                if segment_16k.is_empty() { debug!("{}", fmt_log("Main", "narration_task", "Empty segment after resample; skipping")); continue; }

                // Transcribe segment
                if let Ok(mut s) = stt_ref.lock() {
                    if let Ok(res) = s.transcribe(&segment_16k) {
                        let new_text = st.diff_and_update(&res.text);
                        if !new_text.is_empty() {
                            debug!("{}", fmt_log("Main", "narration_task", &format!(
                                "Injecting delta {}len{}={}{}{}, {}text{}=\"{}\"",
                                C_VAR, C_RESET, C_VAL, new_text.len(), C_RESET,
                                C_VAR, C_RESET, new_text.replace('\n', "\\n")
                            )));
                            // Inject only the new delta text
                            if let Ok(mut p) = PasteService::new() {
                                let _ = p.inject_text(&new_text);
                            } else if let Ok(mut p) = PasteService::new() {
                                let _ = p.clipboard_paste(&new_text, paste_delay);
                            }
                        }
                    }
                }
            }
        });
    }

    // Placeholder event loop tick (simulate running app)
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    info!("{}", fmt_log("Main", "run_app", "Started successfully"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use stt_clippy::init;

    #[tokio::test]
    async fn test_run_app() {
        let _ = init(None, None);
        let result = run_app().await;
        assert!(result.is_ok());
    }
}

// Simple linear resampler (duplicated from bin for now to avoid cross-dep)
fn resample_linear(input: &[f32], from_sr: u32, to_sr: u32) -> Vec<f32> {
    if input.is_empty() || from_sr == 0 || to_sr == 0 { return Vec::new(); }
    if from_sr == to_sr { return input.to_vec(); }
    let ratio = to_sr as f64 / from_sr as f64;
    let out_len = ((input.len() as f64) * ratio).round() as usize;
    if out_len == 0 { return Vec::new(); }
    let mut output = Vec::with_capacity(out_len);
    for n in 0..out_len {
        let pos = (n as f64) / ratio;
        let i0 = pos.floor() as usize;
        let i1 = (i0 + 1).min(input.len() - 1);
        let t = (pos - (i0 as f64)) as f32;
        let s0 = input[i0];
        let s1 = input[i1];
        let sample = s0 + (s1 - s0) * t;
        output.push(sample.max(-1.0).min(1.0));
    }
    output
}

// Voice command infrastructure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VoiceCommand {
    EnableVAD,
    DisableVAD,
    IncreaseSensitivity,
    DecreaseSensitivity,
    ToggleInstantOutput,
    EnableNarration,
    DisableNarration,
}

// Minimal system TTS helper (macOS uses built-in `say`)
#[cfg(target_os = "macos")]
fn speak(text: &str) {
    let _ = std::process::Command::new("say").arg(text).spawn();
}

#[cfg(not(target_os = "macos"))]
fn speak(_text: &str) { }

fn parse_voice_command(transcript: &str) -> Option<VoiceCommand> {
    let t = transcript.to_lowercase();

    let candidates: &[(&[&str], VoiceCommand)] = &[
        (&["enable vad", "enable the vad", "turn on vad", "turn vad on", "start vad", "voice on", "vad on"], VoiceCommand::EnableVAD),
        (&["disable vad", "disable the vad", "turn off vad", "turn vad off", "stop vad", "voice off", "vad off"], VoiceCommand::DisableVAD),
        (&["increase sensitivity", "raise sensitivity", "more sensitive", "turn up sensitivity"], VoiceCommand::IncreaseSensitivity),
        (&["decrease sensitivity", "lower sensitivity", "less sensitive", "turn down sensitivity"], VoiceCommand::DecreaseSensitivity),
        (&["toggle instant", "toggle instant output", "toggle paste mode", "paste mode"], VoiceCommand::ToggleInstantOutput),
        (&["enable narration", "enter narration mode", "start narration", "dictation on", "start dictation"], VoiceCommand::EnableNarration),
        (&["disable narration", "exit narration mode", "stop narration", "dictation off", "stop dictation"], VoiceCommand::DisableNarration),
    ];

    for (phrases, cmd) in candidates {
        if phrases.iter().any(|p| t.contains(p)) {
            return Some(*cmd);
        }
    }
    None
}

fn apply_voice_command(cmd: VoiceCommand, audio_ref: &Arc<Mutex<AudioService>>, instant_output: &Arc<Mutex<bool>>, narration_enabled: &Arc<Mutex<bool>>) {
    match cmd {
        VoiceCommand::EnableVAD => {
            if let Ok(mut a) = audio_ref.lock() { a.set_vad_enabled(true); }
            tracing::info!("Voice command: enabled VAD");
            speak("Enabled V A D");
        }
        VoiceCommand::DisableVAD => {
            if let Ok(mut a) = audio_ref.lock() { a.set_vad_enabled(false); }
            tracing::info!("Voice command: disabled VAD");
            speak("Disabled V A D");
        }
        VoiceCommand::IncreaseSensitivity => {
            if let Ok(mut a) = audio_ref.lock() { a.adjust_vad_sensitivity(0.05); }
            tracing::info!("Voice command: increased VAD sensitivity");
            speak("Increased sensitivity");
        }
        VoiceCommand::DecreaseSensitivity => {
            if let Ok(mut a) = audio_ref.lock() { a.adjust_vad_sensitivity(-0.05); }
            tracing::info!("Voice command: decreased VAD sensitivity");
            speak("Decreased sensitivity");
        }
        VoiceCommand::ToggleInstantOutput => {
            if let Ok(mut v) = instant_output.lock() {
                *v = !*v;
                tracing::info!(instant_output = *v, "Voice command: toggled instant output");
                if *v { speak("Instant output on"); } else { speak("Instant output off"); }
            }
        }
        VoiceCommand::EnableNarration => {
            if let Ok(mut v) = narration_enabled.lock() { *v = true; }
            tracing::info!("Voice command: enabled narration");
            speak("Narration on");
        }
        VoiceCommand::DisableNarration => {
            if let Ok(mut v) = narration_enabled.lock() { *v = false; }
            tracing::info!("Voice command: disabled narration");
            speak("Narration off");
        }
    }
}

// Narration state tracks the last emitted text to compute deltas and resets on pauses
struct NarrationState {
    last_text: String,
    last_emit_len: usize,
    last_activity: Instant,
}

impl NarrationState {
    fn new() -> Self { Self { last_text: String::new(), last_emit_len: 0, last_activity: Instant::now() } }
    fn reset(&mut self) { self.last_text.clear(); self.last_emit_len = 0; self.last_activity = Instant::now(); }
    fn diff_and_update(&mut self, full_text: &str) -> String {
        // Trim leading spaces in delta to avoid double spaces when injecting word-by-word
        let delta = if full_text.len() >= self.last_emit_len {
            &full_text[self.last_emit_len..]
        } else {
            // If STT output regressed (e.g., new alignment), try simple LCP to find new delta
            let mut lcp = 0usize;
            let a = full_text.as_bytes();
            let b = self.last_text.as_bytes();
            let max = a.len().min(b.len());
            while lcp < max && a[lcp] == b[lcp] { lcp += 1; }
            &full_text[lcp..]
        };
        self.last_text = full_text.to_string();
        self.last_emit_len = full_text.len();
        if !delta.is_empty() { self.last_activity = Instant::now(); }
        delta.trim_start().to_string()
    }
    fn maybe_finalize_pause(&mut self) {
        // For now, nothing to finalize besides keeping last_emit_len aligned to last_text length
        self.last_emit_len = self.last_text.len();
    }
}
