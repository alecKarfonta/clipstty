use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use stt_clippy::services::{audio::AudioService, clipboard::ClipboardService, paste::PasteService, stt::STTService};
use tracing::{info, debug, error};
use tracing_subscriber::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setup tracing to stdout with colors
    init_logging();
    // Ensure model path is provided for local backend
    let model_path = std::env::var("WHISPER_MODEL_PATH")?;
    println!("Using WHISPER_MODEL_PATH={}", model_path);
    log_kv("stt_to_clipboard", "main", "WHISPER_MODEL_PATH", &model_path);

    // Capture audio continuously with simple segment window
    let mut audio_service = AudioService::new()?;
    let captured: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_ref = captured.clone();
    let sample_rate_holder: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
    let sr_ref = sample_rate_holder.clone();
    audio_service.on_audio_frame(move |frame, _sr| {
        if let Ok(mut buf) = captured_ref.lock() {
            buf.extend_from_slice(frame);
        }
        if let Ok(mut sro) = sr_ref.lock() {
            *sro = Some(_sr);
        }
    });

    audio_service.start_capture()?;
    info!(target: "runner", "[stt_to_clipboard].main started audio capture");

    let mut stt = STTService::new()?;
    let mut clipboard = ClipboardService::new()?;
    let mut instant_output: bool = false;
    let mut narration_enabled: bool = false; // not used for continuous injection in this runner
    // Cooldown to prevent duplicate command triggers back-to-back
    let mut last_command: Option<VoiceCommand> = None;
    let mut last_command_instant: Option<Instant> = None;
    let command_cooldown = Duration::from_millis(1500);
    // Quiet period after command: skip STT to avoid feedback/retrigger
    let mut command_quiet_until: Option<Instant> = None;
    // Extended quiet period after TTS to avoid microphone feedback
    let mut tts_quiet_until: Option<Instant> = None;

    // simple continuous loop: every window_ms collect & transcribe if there is speech
    let window_ms: u64 = 60000; // keep up to 60s of recent audio
    let poll_ms: u64 = 80; // slightly faster polling
    let frame_ms: usize = 60; // energy computed over last ~60ms
    let mut last_log = Instant::now();
    // Simple VAD-like gating
    // Raise threshold so silence drops below it and can finalize segments
    let energy_threshold: f32 = 1.0e-4;
    let hangover_ms: u64 = 600; // a bit longer silence to finalize
    let min_speech_ms: u64 = 100; // allow short single-word utterances
    let mut voice_active: bool = false;
    let mut last_voice_instant: Option<Instant> = None;
    let mut segment_first_instant: Option<Instant> = None;
    // Narration helpers
    let mut narration_state = NarrationState::new();
    let mut last_narration_check = Instant::now();
    loop {
        std::thread::sleep(Duration::from_millis(poll_ms));

        // pull buffer snapshot
        let (audio_raw, input_sr) = {
            let buf = captured.lock().unwrap().clone();
            let sr = sample_rate_holder.lock().ok().and_then(|g| *g).unwrap_or(16000);
            (buf, sr)
        };
        if audio_raw.is_empty() {
            if last_log.elapsed() > Duration::from_secs(2) {
                debug!(target: "runner", "[stt_to_clipboard].main waiting for audio...");
                last_log = Instant::now();
            }
            continue;
        }

        // keep only last window
        let samples_per_ms = (input_sr as usize) / 1000;
        let keep = (window_ms as usize) * samples_per_ms;
        let tail: Vec<f32> = if audio_raw.len() > keep {
            audio_raw[audio_raw.len() - keep..].to_vec()
        } else {
            audio_raw
        };

        // resample to 16k
        let audio = if input_sr == 16000 { tail } else { resample_linear(&tail, input_sr, 16000) };

        // Honor quiet period after a command or TTS
        let now = Instant::now();
        if let Some(until) = command_quiet_until {
            if now < until {
                continue;
            } else {
                command_quiet_until = None;
            }
        }
        if let Some(until) = tts_quiet_until {
            if now < until {
                continue;
            } else {
                tts_quiet_until = None;
            }
        }

        // If narration mode is enabled, perform continuous delta injection
        if narration_enabled {
            let window_ms: u64 = 8000;
            let hangover_ms: u64 = 500;
            let now = Instant::now();
            if now.duration_since(last_narration_check).as_millis() as u64 >= 120 {
                last_narration_check = now;
                // Keep only last window_ms
                let samples_to_keep = ((window_ms as usize) * 16).min(audio.len());
                if samples_to_keep >= 16000 { // enforce >= 1s to avoid short-input warnings
                    let start_idx = audio.len().saturating_sub(samples_to_keep);
                    let segment = &audio[start_idx..];
                    if !segment.is_empty() {
                        match stt.transcribe(segment) {
                            Ok(res) => {
                                // Hangover handling: if gap longer than hangover, finalize
                                if now.duration_since(last_narration_check).as_millis() as u64 > hangover_ms {
                                    narration_state.maybe_finalize_pause();
                                }
                                let new_text = narration_state.diff_and_update(&res.text);
                                if !new_text.is_empty() {
                                    if let Ok(mut p) = PasteService::new() {
                                        if p.inject_text(&new_text).is_err() {
                                            let _ = p.clipboard_paste(&new_text, 100);
                                        }
                                    } else {
                                        let _ = clipboard.copy_text(&new_text);
                                    }
                                }
                            }
                            Err(e) => {
                                error!(target: "runner", "[stt_to_clipboard].main stt narration error: {}", e);
                            }
                        }
                    }
                }
            }
            // Skip normal end-of-speech path while narration is enabled
            continue;
        }

        // very light energy gate on a short trailing frame + hangover to detect segment boundaries
        let trailing_samples = (frame_ms * 16).min(audio.len());
        let energy: f32 = if trailing_samples == 0 {
            0.0
        } else {
            let start = audio.len() - trailing_samples;
            let slice = &audio[start..];
            slice.iter().map(|s| s * s).sum::<f32>() / (slice.len() as f32)
        };
        log_kv("stt_to_clipboard", "main", "energy", &format!("{:.6}", energy));
        let now = Instant::now();
        if energy >= energy_threshold {
            last_voice_instant = Some(now);
            if !voice_active {
                voice_active = true;
                segment_first_instant = Some(now);
                info!(target: "runner", "[stt_to_clipboard].main VAD start");
            }
            continue;
        }

        // energy below threshold
        if voice_active {
            if let Some(last_voice) = last_voice_instant {
                if now.duration_since(last_voice) >= Duration::from_millis(hangover_ms) {
                    // finalize segment: compute segment duration from first->now, capped by window
                    let seg_duration_ms = segment_first_instant
                        .map(|start| now.duration_since(start).as_millis() as u64)
                        .unwrap_or(0)
                        .min(window_ms);
                    if seg_duration_ms >= min_speech_ms {
                        info!(target: "runner", "[stt_to_clipboard].main VAD end seg_ms={}", seg_duration_ms);
                        // Extract just the last seg_duration_ms from the resampled tail
                        let samples_to_take = ((seg_duration_ms as usize) * 16).min(audio.len());
                        let start_idx = audio.len().saturating_sub(samples_to_take);
                        let seg_audio: Vec<f32> = audio[start_idx..].to_vec();
                        // Enforce minimum segment of 1s to avoid short-input warnings
                        if seg_audio.len() < 16000 { // 1s at 16k
                            // reset state and wait for more audio
                            voice_active = false;
                            last_voice_instant = None;
                            segment_first_instant = None;
                            continue;
                        }
                        let st = Instant::now();
                        match stt.transcribe(&seg_audio) {
                            Ok(result) => {
                                println!("Transcription: {}", result.text);
                                let wall = st.elapsed();
                                let audio_s = (seg_audio.len() as f64) / 16000.0_f64;
                                let wall_s = wall.as_secs_f64();
                                let rtf = if audio_s > 0.0 { wall_s / audio_s } else { 0.0 };
                                info!(target: "runner", "[stt_to_clipboard].main transcribed len={} text=\"{}\" audio_s={:.3} wall_s={:.3} rtf={:.3}", result.text.len(), result.text, audio_s, wall_s, rtf);
                                // Command recognition: intercept voice commands
                                if let Some(cmd) = parse_voice_command(&result.text) {
                                    let now = Instant::now();
                                    let suppress = match (last_command, last_command_instant) {
                                        (Some(prev), Some(ts)) if prev == cmd && now.duration_since(ts) < command_cooldown => true,
                                        _ => false,
                                    };
                                    if suppress {
                                        info!(target: "runner", "Suppressing duplicate voice command within cooldown");
                                    } else {
                                        if let Some(tts_until) = apply_voice_command(cmd, &mut audio_service, &mut instant_output, &mut narration_enabled) {
                                            tts_quiet_until = Some(tts_until);
                                        }
                                        last_command = Some(cmd);
                                        last_command_instant = Some(now);
                                        // Clear audio buffer and reset gating to avoid re-processing the same command segment
                                        if let Ok(mut buf) = captured.lock() { buf.clear(); }
                                        voice_active = false;
                                        last_voice_instant = None;
                                        segment_first_instant = None;
                                        // Start quiet period to avoid TTS feedback and retrigger
                                        command_quiet_until = Some(Instant::now() + command_cooldown);
                                    }
                                    continue;
                                }
                                // If there was text transcribed, output based on mode
                                if !result.text.is_empty() {
                                    if instant_output {
                                        if let Ok(mut p) = PasteService::new() {
                                            // Try direct inject; fall back to clipboard paste
                                            if p.inject_text(&result.text).is_err() {
                                                let _ = p.clipboard_paste(&result.text, 100);
                                            }
                                            info!(target: "runner", "[stt_to_clipboard].main pasted text_length={}", result.text.len());
                                        } else if let Err(e) = clipboard.copy_text(&result.text) {
                                            error!(target: "runner", "[stt_to_clipboard].main clipboard error: {}", e);
                                        }
                                    } else {
                                        if let Err(e) = clipboard.copy_text(&result.text) {
                                            error!(target: "runner", "[stt_to_clipboard].main clipboard error: {}", e);
                                        } else {
                                            info!(target: "runner", "[stt_to_clipboard].main clipboard saved text_length={}", result.text.len());
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!(target: "runner", "[stt_to_clipboard].main stt error: {}", e);
                            }
                        }
                    }
                    // reset state and wait for next speech
                    voice_active = false;
                    last_voice_instant = None;
                    segment_first_instant = None;
                }
            }
        }
    }
}


fn resample_linear(input: &[f32], from_sr: u32, to_sr: u32) -> Vec<f32> {
    if input.is_empty() || from_sr == 0 || to_sr == 0 {
        return Vec::new();
    }
    if from_sr == to_sr {
        return input.to_vec();
    }

    let ratio = to_sr as f64 / from_sr as f64;
    let out_len = ((input.len() as f64) * ratio).round() as usize;
    if out_len == 0 {
        return Vec::new();
    }

    let mut output = Vec::with_capacity(out_len);
    for n in 0..out_len {
        let pos = (n as f64) / ratio;
        let i0 = pos.floor() as usize;
        let i1 = (i0 + 1).min(input.len() - 1);
        let t = (pos - (i0 as f64)) as f32;
        let s0 = input[i0];
        let s1 = input[i1];
        // Linear interpolation
        let sample = s0 + (s1 - s0) * t;
        // Clamp to [-1.0, 1.0]
        output.push(sample.max(-1.0).min(1.0));
    }
    output
}

fn init_logging() {
    use tracing_subscriber::fmt::time::UtcTime;
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_timer(UtcTime::rfc_3339())
        .with_target(true)
        .with_ansi(true);
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("stt_clippy=debug,runner=debug,stt=debug"))
        .with(stdout_layer)
        .try_init();
}

fn log_kv(class_name: &str, func_name: &str, var: &str, value: &str) {
    const C_CLASS: &str = "\x1b[35m"; // magenta
    const C_FUNC: &str = "\x1b[36m"; // cyan
    const C_VAR: &str = "\x1b[33m"; // yellow
    const C_VAL: &str = "\x1b[32m"; // green
    const C_RESET: &str = "\x1b[0m";
    println!(
        "[{}{}{}].{}{}{} {}{}{}={}{}{}",
        C_CLASS, class_name, C_RESET,
        C_FUNC, func_name, C_RESET,
        C_VAR, var, C_RESET,
        C_VAL, value, C_RESET
    );
}


// --- Voice command parsing and application for the runner ---
// Smart narration state with intelligent formatting and deduplication
struct NarrationState {
    last_text: String,
    last_emit_len: usize,
    last_output_time: Instant,
    accumulated_output: String,
    sentence_start: bool,
    // Enhanced deduplication
    output_history: Vec<String>,
    last_stable_text: String,
    stability_count: usize,
    text_hash_history: std::collections::HashSet<u64>,
}

impl NarrationState {
    fn new() -> Self { 
        Self { 
            last_text: String::new(), 
            last_emit_len: 0,
            last_output_time: Instant::now(),
            accumulated_output: String::new(),
            sentence_start: true,
            output_history: Vec::new(),
            last_stable_text: String::new(),
            stability_count: 0,
            text_hash_history: std::collections::HashSet::new(),
        } 
    }
    
    fn diff_and_update(&mut self, full_text: &str) -> String {
        let now = Instant::now();
        
        // Enhanced deduplication: check if this is stable new content
        if full_text == self.last_stable_text {
            self.stability_count += 1;
        } else {
            self.stability_count = 1;
            self.last_stable_text = full_text.to_string();
        }
        
        // Only process if text has been stable for at least 2 iterations to avoid flicker
        if self.stability_count < 2 {
            return String::new();
        }
        
        // Check if we've already processed this exact text
        let text_hash = self.hash_text(full_text);
        if self.text_hash_history.contains(&text_hash) {
            return String::new();
        }
        
        // Find genuinely new content using smart delta detection
        let new_content = self.extract_new_content(full_text);
        if new_content.trim().is_empty() {
            return String::new();
        }
        
        // Check for repeated content in the new content itself
        if self.is_repeated_content(&new_content) {
            return String::new();
        }
        
        // Update tracking state
        self.text_hash_history.insert(text_hash);
        self.last_text = full_text.to_string();
        
        // Smart formatting
        let formatted = self.format_delta(&new_content, now);
        
        if !formatted.is_empty() {
            self.last_output_time = now;
            self.accumulated_output.push_str(&formatted);
            self.output_history.push(formatted.clone());
            
            // Keep history manageable
            if self.output_history.len() > 20 {
                self.output_history.remove(0);
            }
            if self.text_hash_history.len() > 50 {
                // Clear old hashes but keep recent ones
                let old_hashes: Vec<u64> = self.text_hash_history.iter().take(25).cloned().collect();
                for hash in old_hashes {
                    self.text_hash_history.remove(&hash);
                }
            }
        }
        
        formatted
    }
    
    fn hash_text(&self, text: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }
    
    fn extract_new_content(&mut self, full_text: &str) -> String {
        // If no previous text, everything is new
        if self.last_text.is_empty() {
            self.last_emit_len = full_text.len();
            return full_text.to_string();
        }
        
        // Find the longest common prefix between last_text and full_text
        let mut lcp = 0usize;
        let a = full_text.as_bytes();
        let b = self.last_text.as_bytes();
        let max = a.len().min(b.len());
        while lcp < max && a[lcp] == b[lcp] { 
            lcp += 1; 
        }
        
        // Extract only the truly new part
        let new_part = if full_text.len() > lcp {
            &full_text[lcp..]
        } else {
            ""
        };
        
        // Update emit length to prevent re-processing
        self.last_emit_len = full_text.len();
        
        new_part.to_string()
    }
    
    fn is_repeated_content(&self, content: &str) -> bool {
        let content_words: Vec<&str> = content.split_whitespace().collect();
        if content_words.len() < 3 {
            return false; // Too short to be a meaningful repeat
        }
        
        // Check if this content appears in recent output history
        for previous_output in self.output_history.iter().rev().take(5) {
            let prev_words: Vec<&str> = previous_output.split_whitespace().collect();
            
            // Check for substantial overlap (> 70% of words)
            let overlap = self.calculate_word_overlap(&content_words, &prev_words);
            if overlap > 0.7 {
                return true;
            }
        }
        
        // Check for internal repetition within the content itself
        self.has_internal_repetition(&content_words)
    }
    
    fn calculate_word_overlap(&self, words1: &[&str], words2: &[&str]) -> f64 {
        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }
        
        let set1: std::collections::HashSet<&str> = words1.iter().cloned().collect();
        let set2: std::collections::HashSet<&str> = words2.iter().cloned().collect();
        
        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
    
    fn has_internal_repetition(&self, words: &[&str]) -> bool {
        if words.len() < 6 {
            return false;
        }
        
        // Look for repeated sequences of 3+ words
        for start in 0..words.len() - 5 {
            for length in 3..=((words.len() - start) / 2) {
                let phrase1 = &words[start..start + length];
                let phrase2 = &words[start + length..start + 2 * length];
                
                if phrase1 == phrase2 {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn format_delta(&mut self, raw_delta: &str, now: Instant) -> String {
        // Time since last output to detect pauses
        let time_gap = now.duration_since(self.last_output_time).as_millis() as u64;
        let is_long_pause = time_gap > 800; // 800ms+ indicates sentence/phrase boundary
        let is_short_pause = time_gap > 200; // 200ms+ indicates word boundary
        
        // Clean and tokenize the delta
        let cleaned = raw_delta.trim().to_lowercase();
        if cleaned.is_empty() {
            return String::new();
        }
        
        // Split into potential words, handling punctuation
        let words = self.extract_words(&cleaned);
        if words.is_empty() {
            return String::new();
        }
        
        let mut result = String::new();
        
        for (i, word) in words.iter().enumerate() {
            let is_first_word = i == 0;
            let _is_last_word = i == words.len() - 1;
            
            // Handle spacing and capitalization
            if is_first_word {
                // First word in this delta
                if self.accumulated_output.is_empty() {
                    // Very first word - capitalize if sentence start
                    result.push_str(&self.capitalize_if_needed(word));
                    self.sentence_start = false;
                } else if is_long_pause {
                    // Long pause - likely new sentence
                    if !self.ends_with_punctuation(&self.accumulated_output) {
                        result.push('.');
                    }
                    result.push(' ');
                    result.push_str(&self.capitalize_word(word));
                    self.sentence_start = false;
                } else if is_short_pause || self.needs_space_before(&self.accumulated_output, word) {
                    // Normal word boundary
                    result.push(' ');
                    result.push_str(&self.format_word(word));
                } else {
                    // No pause - might be continuation of previous word
                    result.push_str(&self.format_word(word));
                }
            } else {
                // Subsequent words in this delta
                if self.is_punctuation(word) {
                    result.push_str(word); // Punctuation goes directly
                } else {
                    result.push(' ');
                    result.push_str(&self.format_word(word));
                }
            }
            
            // Update sentence state
            if self.is_sentence_ending(word) {
                self.sentence_start = true;
            }
        }
        
        result
    }
    
    fn extract_words(&self, text: &str) -> Vec<String> {
        let mut words = Vec::new();
        let mut current_word = String::new();
        
        for ch in text.chars() {
            if ch.is_whitespace() {
                if !current_word.is_empty() {
                    words.push(current_word.clone());
                    current_word.clear();
                }
            } else if self.is_punctuation_char(ch) {
                if !current_word.is_empty() {
                    words.push(current_word.clone());
                    current_word.clear();
                }
                words.push(ch.to_string());
            } else {
                current_word.push(ch);
            }
        }
        
        if !current_word.is_empty() {
            words.push(current_word);
        }
        
        words
    }
    
    fn capitalize_if_needed(&self, word: &str) -> String {
        if self.sentence_start || self.accumulated_output.is_empty() {
            self.capitalize_word(word)
        } else {
            word.to_string()
        }
    }
    
    fn capitalize_word(&self, word: &str) -> String {
        if word.is_empty() {
            return word.to_string();
        }
        let mut chars: Vec<char> = word.chars().collect();
        chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
        chars.into_iter().collect()
    }
    
    fn format_word(&self, word: &str) -> String {
        // Handle common speech-to-text corrections
        match word {
            "i" => "I".to_string(),
            "im" => "I'm".to_string(),
            "ive" => "I've".to_string(),
            "ill" => "I'll".to_string(),
            "dont" => "don't".to_string(),
            "wont" => "won't".to_string(),
            "cant" => "can't".to_string(),
            "shouldnt" => "shouldn't".to_string(),
            "wouldnt" => "wouldn't".to_string(),
            "couldnt" => "couldn't".to_string(),
            "thats" => "that's".to_string(),
            "its" => "it's".to_string(),
            "youre" => "you're".to_string(),
            "theyre" => "they're".to_string(),
            "were" => "we're".to_string(),
            _ => word.to_string(),
        }
    }
    
    fn needs_space_before(&self, previous_text: &str, word: &str) -> bool {
        if previous_text.is_empty() {
            return false;
        }
        
        // No space before punctuation
        if self.is_punctuation(word) {
            return false;
        }
        
        // Always space before normal words unless previous ends with specific chars
        let last_char = previous_text.chars().last().unwrap_or(' ');
        !matches!(last_char, '(' | '[' | '{' | '"' | '\'')
    }
    
    fn ends_with_punctuation(&self, text: &str) -> bool {
        text.chars().last().map_or(false, |ch| matches!(ch, '.' | '!' | '?' | ':' | ';'))
    }
    
    fn is_punctuation(&self, word: &str) -> bool {
        word.len() == 1 && self.is_punctuation_char(word.chars().next().unwrap())
    }
    
    fn is_punctuation_char(&self, ch: char) -> bool {
        matches!(ch, '.' | ',' | '!' | '?' | ':' | ';' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\'' | '-')
    }
    
    fn is_sentence_ending(&self, word: &str) -> bool {
        word.chars().any(|ch| matches!(ch, '.' | '!' | '?'))
    }
    
    fn maybe_finalize_pause(&mut self) {
        // Add final punctuation if needed
        if !self.accumulated_output.is_empty() && !self.ends_with_punctuation(&self.accumulated_output) {
            // Don't auto-add punctuation here as it might be mid-sentence
        }
        self.last_emit_len = self.last_text.len();
    }
}


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

#[cfg(target_os = "macos")]
fn speak(text: &str) {
    let _ = std::process::Command::new("say").arg(text).spawn();
}
#[cfg(not(target_os = "macos"))]
fn speak(_text: &str) { }

fn apply_voice_command(cmd: VoiceCommand, audio: &mut AudioService, instant_output: &mut bool, narration_enabled: &mut bool) -> Option<Instant> {
    let tts_quiet_duration = Duration::from_millis(3000); // 3s quiet after TTS to avoid feedback
    match cmd {
        VoiceCommand::EnableVAD => {
            audio.set_vad_enabled(true);
            info!(target: "runner", "Voice command: enabled VAD");
            speak("Enabled V A D");
            Some(Instant::now() + tts_quiet_duration)
        }
        VoiceCommand::DisableVAD => {
            audio.set_vad_enabled(false);
            info!(target: "runner", "Voice command: disabled VAD");
            speak("Disabled V A D");
            Some(Instant::now() + tts_quiet_duration)
        }
        VoiceCommand::IncreaseSensitivity => {
            audio.adjust_vad_sensitivity(0.05);
            info!(target: "runner", "Voice command: increased VAD sensitivity");
            speak("Increased sensitivity");
            Some(Instant::now() + tts_quiet_duration)
        }
        VoiceCommand::DecreaseSensitivity => {
            audio.adjust_vad_sensitivity(-0.05);
            info!(target: "runner", "Voice command: decreased VAD sensitivity");
            speak("Decreased sensitivity");
            Some(Instant::now() + tts_quiet_duration)
        }
        VoiceCommand::ToggleInstantOutput => {
            *instant_output = !*instant_output;
            info!(target: "runner", instant_output = *instant_output, "Voice command: toggled instant output");
            if *instant_output { speak("Instant output on"); } else { speak("Instant output off"); }
            Some(Instant::now() + tts_quiet_duration)
        }
        VoiceCommand::EnableNarration => {
            *narration_enabled = true;
            info!(target: "runner", "Voice command: enabled narration");
            speak("Narration on");
            Some(Instant::now() + tts_quiet_duration)
        }
        VoiceCommand::DisableNarration => {
            *narration_enabled = false;
            info!(target: "runner", "Voice command: disabled narration");
            speak("Narration off");
            Some(Instant::now() + tts_quiet_duration)
        }
    }
}

