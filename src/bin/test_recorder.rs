//! Test Audio Recorder for Voice Commands
//! 
//! This tool records audio files for testing voice commands with automatic
//! silence trimming and organized file management.

// run with  cd /Users/alec/git/clipstty && cargo build --bin test_recorder && cargo run --bin test_recorder

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::path::{Path, PathBuf};
use std::fs;

use stt_clippy::services::{
    audio::AudioService, 
    stt::STTService,
    tts::TTSService,
    voice_commands::comprehensive_registry::create_comprehensive_command_engine,
};
use tracing::{info, debug, error};
use tracing_subscriber::prelude::*;
use hound::{WavWriter, WavSpec};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    
    println!("🎙️ === Enhanced Voice Command Test Recorder with TTS Feedback === 🎙️");
    println!("This tool helps you record audio files for testing voice commands with audio guidance.");
    println!();
    
    // Initialize TTS service for audio feedback
    let mut tts_service = match TTSService::new() {
        Ok(tts) => {
            println!("✅ TTS service initialized for audio feedback");
            println!("🔊 You should hear audio guidance throughout the recording process");
            Some(tts)
        }
        Err(e) => {
            println!("⚠️ TTS service unavailable: {}. Continuing without audio feedback.", e);
            println!("💡 Note: You'll only see text instructions without audio guidance");
            None
        }
    };
    
    // Announce startup with comprehensive intro - wait for each to complete
    if let Some(ref tts) = tts_service {
        println!("🎵 Playing audio introduction...");
        if let Err(e) = tts.announce_phase_instructions("startup").await {
            println!("⚠️ TTS error during startup: {}", e);
        }
        
        if let Err(e) = tts.announce_phase_instructions("intro").await {
            println!("⚠️ TTS error during intro: {}", e);
        }
        
        if let Err(e) = tts.announce_recording_tips().await {
            println!("⚠️ TTS error during tips: {}", e);
        }
        println!("🎵 Audio introduction complete.");
    }
    
    // Create test recordings directory
    let test_dir = PathBuf::from("test_recordings");
    fs::create_dir_all(&test_dir)?;
    info!("Created test recordings directory: {}", test_dir.display());
    
    // Initialize services
    let mut audio_service = AudioService::new()?;
    let _voice_command_engine = create_comprehensive_command_engine();
    
    // Initialize STT service once and reuse it to keep model loaded
    let mut stt_service = STTService::new()?;
    info!("STT service initialized - model will stay loaded between transcriptions");
    
    // Audio capture setup
    let captured: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_ref = captured.clone();
    let sample_rate_holder: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
    let sr_ref = sample_rate_holder.clone();
    
    audio_service.on_audio_frame(move |frame, sr| {
        if let Ok(mut buf) = captured_ref.lock() {
            buf.extend_from_slice(frame);
        }
        if let Ok(mut sro) = sr_ref.lock() {
            *sro = Some(sr);
        }
    });
    
    audio_service.start_capture()?;
    info!("Audio capture started");
    
    // Recording state
    let mut recording_state = RecordingState::new();
    let mut current_phrase_index = 0;
    
    // Test phrases list
    let test_phrases = get_test_phrases();
    
    println!("📝 Available Commands:");
    println!("  • 'start test recording [phrase_number]' - Start waiting for a specific test phrase");
    println!("  • 'start test recording next' - Start waiting for the next phrase in sequence");
    println!("  • 'start recording with countdown' - Start waiting for phrase with countdown");
    println!("  • 'stop test recording' - Stop current recording and save with validation");
    println!("  • 'validate last recording' - Check if the last recording matches expected phrase");
    println!("  • 'clean and save test file' - Trim and optimize the last recording for testing");
    println!("  • 'toggle tts feedback' - Enable/disable audio feedback");
    println!("  • 'show test phrases' - Display all test phrases with numbers");
    println!("  • 'show current phrase' - Show the current phrase to record");
    println!("  • 'show recording tips' - Get audio guidance on recording best practices");
    println!("  • 'test tts' - Test if TTS audio feedback is working");
    println!("  • 'skip to phrase [number]' - Skip to a specific phrase number");
    println!("  • 'quit test recorder' - Exit the test recorder");
    println!();
    println!("💡 How it works:");
    println!("   1. Say a command like 'start test recording next'");
    println!("   2. Wait for the prompt, then say ONLY the target phrase");
    println!("   3. The system will detect and record just that phrase");
    println!("   4. Say 'stop test recording' when done");
    println!();
    let default_phrase = "<end of list>".to_string();
    let current_phrase = test_phrases.get(current_phrase_index).unwrap_or(&default_phrase);
    println!("🎯 Current phrase to record: #{} - \"{}\"", 
             current_phrase_index + 1, 
             current_phrase);
    println!();
    
    // Announce current phrase via TTS with phase instructions - wait for each to complete
    if let Some(ref tts) = tts_service {
        if let Err(e) = tts.announce_phase_instructions("ready").await {
            println!("⚠️ TTS error: {}", e);
        }
        
        if let Err(e) = tts.announce_instruction(&format!("Current phrase is number {}: {}", current_phrase_index + 1, current_phrase)).await {
            println!("⚠️ TTS error: {}", e);
        }
    }
    
    // Main processing loop
    let poll_ms = 80;
    let energy_threshold = 1e-4_f32;
    let hangover_ms = 600;
    let min_speech_ms = 100;
    
    let mut voice_active = false;
    let mut last_voice_instant: Option<Instant> = None;
    let mut segment_first_instant: Option<Instant> = None;
    
    loop {
        std::thread::sleep(Duration::from_millis(poll_ms));
        
        // Get audio buffer
        let (audio_raw, input_sr) = {
            let buf = captured.lock().unwrap().clone();
            let sr = sample_rate_holder.lock().ok().and_then(|g| *g).unwrap_or(16000);
            (buf, sr)
        };
        
        if audio_raw.is_empty() {
            continue;
        }
        
        // Resample to 16kHz
        let audio = if input_sr == 16000 { 
            audio_raw 
        } else { 
            resample_linear(&audio_raw, input_sr, 16000) 
        };
        
        // Voice activity detection
        let frame_ms = 60;
        let trailing_samples = (frame_ms * 16).min(audio.len());
        let energy: f32 = if trailing_samples == 0 {
            0.0
        } else {
            let start = audio.len() - trailing_samples;
            let slice = &audio[start..];
            slice.iter().map(|s| s * s).sum::<f32>() / (slice.len() as f32)
        };
        
        let now = Instant::now();
        
        if energy >= energy_threshold {
            last_voice_instant = Some(now);
            if !voice_active {
                voice_active = true;
                segment_first_instant = Some(now);
                debug!("VAD start");
                
                // If we're recording, capture this segment
                if recording_state.is_recording {
                    recording_state.start_segment(now);
                }
            }
            
            // Handle audio capture based on recording state
            if recording_state.is_recording {
                recording_state.add_audio_samples(&audio);
            } else if recording_state.waiting_for_phrase {
                // Always collect audio for phrase detection when waiting
                recording_state.add_detection_samples(&audio);
            }
            
            continue;
        }
        
        // Energy below threshold - check for end of speech
        if voice_active {
            if let Some(last_voice) = last_voice_instant {
                if now.duration_since(last_voice) >= Duration::from_millis(hangover_ms) {
                    let seg_duration_ms = segment_first_instant
                        .map(|start| now.duration_since(start).as_millis() as u64)
                        .unwrap_or(0);
                    
                    if seg_duration_ms >= min_speech_ms {
                        debug!("VAD end seg_ms={}", seg_duration_ms);
                        
                        // Extract segment audio
                        let samples_to_take = ((seg_duration_ms as usize) * 16).min(audio.len());
                        let start_idx = audio.len().saturating_sub(samples_to_take);
                        let seg_audio: Vec<f32> = audio[start_idx..].to_vec();
                        
                        if seg_audio.len() >= 16000 {
                            // Process voice command using the persistent STT service
                            if let Ok(result) = stt_service.transcribe(&seg_audio) {
                                println!("🎤 Heard: \"{}\"", result.text);
                                
                                // Skip processing during command cooldown to avoid detecting commands in phrase recordings
                                if recording_state.is_in_command_cooldown(now) {
                                    debug!("Skipping command processing during cooldown");
                                    continue;
                                }
                                
                                // Check if we're waiting for a target phrase
                                if recording_state.waiting_for_phrase {
                                    if phrase_detected_in_transcription(&result.text, &recording_state.phrase) {
                                        println!("🎯 Target phrase detected! Starting precise recording...");
                                        recording_state.start_phrase_recording(now);
                                        
                                        // Extract just the phrase portion from the detection buffer
                                        if let Some(phrase_audio) = extract_phrase_from_buffer(&recording_state.phrase_detection_buffer, &result.text, &recording_state.phrase) {
                                            recording_state.audio_buffer = phrase_audio;
                                            println!("✨ Extracted phrase audio ({} samples)", recording_state.audio_buffer.len());
                                        }
                                        
                                        if let Some(ref tts) = tts_service {
                                            let _ = tts.announce_instruction("Target phrase detected. Recording in progress.").await;
                                        }
                                        continue;
                                    }
                                }
                                
                                // Check for test recorder commands
                                if let Some(action) = parse_test_command(&result.text) {
                                    match action {
                                        TestCommand::StartRecording(phrase_num) => {
                                            if let Some(num) = phrase_num {
                                                if num > 0 && num <= test_phrases.len() {
                                                    current_phrase_index = num - 1;
                                                }
                                            }
                                            
                                            if current_phrase_index < test_phrases.len() {
                                                let phrase = &test_phrases[current_phrase_index];
                                                recording_state.start_recording(phrase.clone(), current_phrase_index + 1);
                                                println!("🎯 Waiting for phrase #{}: \"{}\"", 
                                                         current_phrase_index + 1, phrase);
                                                println!("   Now say ONLY the target phrase: \"{}\"", phrase);
                                                println!("   The system will automatically detect and record it.");
                                                
                                                // TTS announcement with phase instructions - wait for each to complete
                                                if let Some(ref tts) = tts_service {
                                                    if let Err(e) = tts.announce_recording_beginning().await {
                                                        println!("⚠️ TTS error: {}", e);
                                                    }
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                                                    
                                                    if let Err(e) = tts.announce_recording_start(current_phrase_index + 1, phrase).await {
                                                        println!("⚠️ TTS error: {}", e);
                                                    }
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                                                    
                                                    if let Err(e) = tts.announce_phase_instructions("recording_active").await {
                                                        println!("⚠️ TTS error: {}", e);
                                                    }
                                                }
                                            } else {
                                                println!("✅ All test phrases completed!");
                                                if let Some(ref tts) = tts_service {
                                                    let _ = tts.announce_completion().await;
                                                }
                                            }
                                        }
                                        TestCommand::StartNext => {
                                            if current_phrase_index < test_phrases.len() {
                                                let phrase = &test_phrases[current_phrase_index];
                                                recording_state.start_recording(phrase.clone(), current_phrase_index + 1);
                                                println!("🎯 Waiting for phrase #{}: \"{}\"", 
                                                         current_phrase_index + 1, phrase);
                                                println!("   Now say ONLY the target phrase: \"{}\"", phrase);
                                                println!("   The system will automatically detect and record it.");
                                                
                                                // TTS announcement with phase instructions - wait for each to complete
                                                if let Some(ref tts) = tts_service {
                                                    if let Err(e) = tts.announce_recording_beginning().await {
                                                        println!("⚠️ TTS error: {}", e);
                                                    }
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                                                    
                                                    if let Err(e) = tts.announce_recording_start(current_phrase_index + 1, phrase).await {
                                                        println!("⚠️ TTS error: {}", e);
                                                    }
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                                                    
                                                    if let Err(e) = tts.announce_phase_instructions("recording_active").await {
                                                        println!("⚠️ TTS error: {}", e);
                                                    }
                                                }
                                            } else {
                                                println!("✅ All test phrases completed!");
                                                if let Some(ref tts) = tts_service {
                                                    let _ = tts.announce_completion().await;
                                                }
                                            }
                                        }
                                        TestCommand::StartWithCountdown => {
                                            if current_phrase_index < test_phrases.len() {
                                                let phrase = &test_phrases[current_phrase_index];
                                                println!("🎯 Countdown for phrase #{}: \"{}\"", 
                                                         current_phrase_index + 1, phrase);
                                                
                                                // TTS countdown sequence
                                                if let Some(ref tts) = tts_service {
                                                    if let Err(e) = tts.announce_recording_countdown(phrase).await {
                                                        println!("⚠️ TTS countdown error: {}", e);
                                                    }
                                                }
                                                
                                                // Start waiting for phrase after countdown
                                                recording_state.start_recording(phrase.clone(), current_phrase_index + 1);
                                                println!("   Now say ONLY the target phrase: \"{}\"", phrase);
                                                println!("   The system will automatically detect and record it.");
                                                
                                                if let Some(ref tts) = tts_service {
                                                    let _ = tts.announce_phase_instructions("recording_active").await;
                                                }
                                            } else {
                                                println!("✅ All test phrases completed!");
                                                if let Some(ref tts) = tts_service {
                                                    let _ = tts.announce_completion().await;
                                                }
                                            }
                                        }
                                        TestCommand::StopRecording => {
                                            if recording_state.is_recording {
                                                // Announce validation phase
                                                if let Some(ref tts) = tts_service {
                                                    let _ = tts.announce_phase_instructions("validation").await;
                                                }
                                                
                                                match save_recording(&recording_state, &test_dir, &mut stt_service) {
                                                    Ok((filename, transcription)) => {
                                                        println!("✅ Recording saved: {}", filename);
                                                        
                                                        // Store recording info for validation
                                                        let full_path = test_dir.join(&filename).to_string_lossy().to_string();
                                                        recording_state.set_last_recording(full_path, transcription.clone());
                                                        
                                                        // TTS feedback
                                                        if let Some(ref tts) = tts_service {
                                                            let _ = tts.announce_recording_success(&filename).await;
                                                        }
                                                        
                                                        // Auto-validate if transcription available
                                                        if let Some(ref actual) = transcription {
                                                            let expected = &recording_state.phrase;
                                                            let similarity = calculate_transcription_similarity(expected, actual);
                                                            recording_state.validation_score = Some(similarity);
                                                            
                                                            println!("📝 Transcription: \"{}\"", actual);
                                                            println!("🎯 Expected: \"{}\"", expected);
                                                            println!("📈 Similarity: {:.1}%", similarity * 100.0);
                                                            
                                                            let is_good = similarity >= 0.7; // Lowered threshold to account for improved algorithm
                                                            if let Some(ref tts) = tts_service {
                                                                let _ = tts.announce_validation_result(expected, actual, is_good).await;
                                                            }
                                                            
                                                            if is_good {
                                                                println!("✅ Good recording! Moving to next phrase.");
                                                                recording_state.stop_recording();
                                                                current_phrase_index += 1;
                                                                
                                                                if current_phrase_index < test_phrases.len() {
                                                                    println!("🎯 Next phrase: #{} - \"{}\"", 
                                                                             current_phrase_index + 1, 
                                                                             &test_phrases[current_phrase_index]);
                                                                    if let Some(ref tts) = tts_service {
                                                                        if let Err(e) = tts.announce_phase_instructions("next_phrase").await {
                                                                            println!("⚠️ TTS error: {}", e);
                                                                        }
                                                                        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                                                                        
                                                                        if let Err(e) = tts.announce_progress(current_phrase_index, test_phrases.len()).await {
                                                                            println!("⚠️ TTS error: {}", e);
                                                                        }
                                                                    }
                                                                } else {
                                                                    println!("🎉 All test phrases completed!");
                                                                    if let Some(ref tts) = tts_service {
                                                                        let _ = tts.announce_phase_instructions("completion").await;
                                                                    }
                                                                }
                                                            } else {
                                                                println!("⚠️ Recording doesn't match well enough (similarity: {:.1}%)", similarity * 100.0);
                                                                println!("   Expected: \"{}\"", expected);
                                                                println!("   Got: \"{}\"", actual);
                                                                
                                                                // Provide specific feedback on what went wrong
                                                                let feedback = analyze_transcription_mismatch(expected, actual);
                                                                println!("💡 Tip: {}", feedback);
                                                                println!("   Please try again. Say 'start test recording next' to re-record this phrase.");
                                                                
                                                                if let Some(ref tts) = tts_service {
                                                                    let _ = tts.announce_specific_guidance(expected, actual, &feedback).await;
                                                                }
                                                                
                                                                // Don't advance to next phrase - stay on current one
                                                                recording_state.stop_recording();
                                                                // Don't increment current_phrase_index
                                                            }
                                                        } else {
                                                            println!("⚠️ Could not transcribe recording for validation.");
                                                            println!("   Please try again. Say 'start test recording next' to re-record this phrase.");
                                                            
                                                            if let Some(ref tts) = tts_service {
                                                                let _ = tts.announce_transcription_failure(current_phrase_index + 1).await;
                                                            }
                                                            
                                                            recording_state.stop_recording();
                                                            // Don't increment current_phrase_index
                                                        }
                                                        }
                                                    Err(e) => {
                                                        println!("❌ Recording failed: {}", e);
                                                        
                                                        // Determine the type of failure and provide specific TTS feedback
                                                        if let Some(ref tts) = tts_service {
                                                            if e.to_string().contains("silence") {
                                                                let _ = tts.announce_silence_detected(current_phrase_index + 1).await;
                                                            } else {
                                                                let _ = tts.announce_recording_failure(&e.to_string(), current_phrase_index + 1).await;
                                                            }
                                                        }
                                                        
                                                        recording_state.stop_recording();
                                                        // Don't increment current_phrase_index - stay on current phrase
                                                    }
                                                }
                                            } else {
                                                println!("❌ No recording in progress");
                                                if let Some(ref tts) = tts_service {
                                                    let _ = tts.announce_error("No recording in progress").await;
                                                }
                                            }
                                        }
                                        TestCommand::ShowPhrases => {
                                            show_test_phrases(&test_phrases, current_phrase_index);
                                        }
                                        TestCommand::ShowCurrent => {
                                            if current_phrase_index < test_phrases.len() {
                                                let current_phrase = &test_phrases[current_phrase_index];
                                                println!("🎯 Current phrase: #{} - \"{}\"", 
                                                         current_phrase_index + 1, 
                                                         current_phrase);
                                                if let Some(ref tts) = tts_service {
                                                    let _ = tts.announce_instruction(&format!("Current phrase is number {}: {}", current_phrase_index + 1, current_phrase)).await;
                                                }
                                            } else {
                                                println!("✅ All phrases completed!");
                                                if let Some(ref tts) = tts_service {
                                                    let _ = tts.announce_completion().await;
                                                }
                                            }
                                        }
                                        TestCommand::ShowRecordingTips => {
                                            println!("💡 Recording Tips:");
                                            println!("   • Speak clearly and at normal pace");
                                            println!("   • Wait for the recording prompt before speaking");
                                            println!("   • Make sure you're in a quiet environment");
                                            println!("   • Say each word distinctly");
                                            println!("   • Keep microphone at consistent distance");
                                            println!("   • Avoid background noise and interruptions");
                                            
                                            if let Some(ref tts) = tts_service {
                                                let _ = tts.announce_recording_tips().await;
                                            }
                                        }
                                        TestCommand::TestTTS => {
                                            println!("🔊 Testing TTS audio feedback...");
                                            if let Some(ref tts) = tts_service {
                                                let _ = tts.speak_and_wait("TTS audio feedback is working correctly. You should hear this message clearly.").await;
                                                println!("✅ TTS test completed. Did you hear the audio message?");
                                            } else {
                                                println!("❌ TTS service is not available");
                                            }
                                        }
                                        TestCommand::SkipTo(num) => {
                                            if num > 0 && num <= test_phrases.len() {
                                                current_phrase_index = num - 1;
                                                println!("⏭️ Skipped to phrase #{}: \"{}\"", 
                                                         num, &test_phrases[current_phrase_index]);
                                                if let Some(ref tts) = tts_service {
                                                    let _ = tts.announce_instruction(&format!("Skipped to phrase {}. Ready to record.", num)).await;
                                                }
                                            } else {
                                                println!("❌ Invalid phrase number. Use 1-{}", test_phrases.len());
                                                if let Some(ref tts) = tts_service {
                                                    let _ = tts.announce_error(&format!("Invalid phrase number. Use 1 to {}", test_phrases.len())).await;
                                                }
                                            }
                                        }
                                        TestCommand::ValidateLastRecording => {
                                            if let Some(ref _last_path) = recording_state.last_recording_path {
                                                if let Some(ref transcription) = recording_state.last_transcription {
                                                    let expected = &recording_state.phrase;
                                                    let similarity = calculate_transcription_similarity(expected, transcription);
                                                    
                                                    println!("📝 Last transcription: \"{}\"", transcription);
                                                    println!("🎯 Expected: \"{}\"", expected);
                                                    println!("📈 Similarity: {:.1}%", similarity * 100.0);
                                                    
                                                    let is_good = similarity >= 0.8;
                                                    if let Some(ref tts) = tts_service {
                                                        let _ = tts.announce_validation_result(expected, transcription, is_good).await;
                                                    }
                                                } else {
                                                    println!("❌ No transcription available for last recording");
                                                    if let Some(ref tts) = tts_service {
                                                        let _ = tts.announce_error("No transcription available for validation").await;
                                                    }
                                                }
                                            } else {
                                                println!("❌ No recording to validate");
                                                if let Some(ref tts) = tts_service {
                                                    let _ = tts.announce_error("No recording to validate").await;
                                                }
                                            }
                                        }
                                        TestCommand::CleanAndSaveTestFile => {
                                            if let Some(ref last_path) = recording_state.last_recording_path {
                                                match create_clean_test_file(last_path, &test_dir, recording_state.phrase_number, &recording_state.phrase) {
                                                    Ok(clean_filename) => {
                                                        println!("✨ Clean test file created: {}", clean_filename);
                                                        if let Some(ref tts) = tts_service {
                                                            let _ = tts.announce_instruction(&format!("Clean test file created: {}", clean_filename)).await;
                                                        }
                                                    }
                                                    Err(e) => {
                                                        println!("❌ Failed to create clean test file: {}", e);
                                                        if let Some(ref tts) = tts_service {
                                                            let _ = tts.announce_error(&format!("Failed to create clean test file: {}", e)).await;
                                                        }
                                                    }
                                                }
                                            } else {
                                                println!("❌ No recording to clean");
                                                if let Some(ref tts) = tts_service {
                                                    let _ = tts.announce_error("No recording to clean").await;
                                                }
                                            }
                                        }
                                        TestCommand::ToggleTTSFeedback => {
                                            if let Some(ref mut tts) = tts_service {
                                                let new_state = !tts.is_enabled();
                                                tts.set_enabled(new_state);
                                                if new_state {
                                                    println!("🔊 TTS feedback enabled");
                                                    let _ = tts.announce_instruction("TTS feedback is now enabled").await;
                                                } else {
                                                    println!("🔇 TTS feedback disabled");
                                                }
                                            } else {
                                                println!("❌ TTS service not available");
                                            }
                                        }
                                        TestCommand::Quit => {
                                            println!("👋 Exiting test recorder...");
                                            return Ok(());
                                        }
                                    }
                                } else {
                                    // No test command recognized - provide helpful guidance
                                    if !recording_state.is_recording && !recording_state.waiting_for_phrase {
                                        println!("💡 Tip: To start recording, say 'start test recording next' or 'start recording with countdown'");
                                        println!("   Current phrase to record: #{} - \"{}\"", 
                                                 current_phrase_index + 1, 
                                                 test_phrases.get(current_phrase_index).unwrap_or(&"<end of list>".to_string()));
                                        
                                        if let Some(ref tts) = tts_service {
                                            let _ = tts.announce_instruction("To start recording, say 'start test recording next' or 'start recording with countdown'").await;
                                        }
                                    } else if recording_state.waiting_for_phrase {
                                        println!("🎯 Still waiting for target phrase: \"{}\"", recording_state.phrase);
                                        println!("   Say ONLY the target phrase, not commands.");
                                    }
                                }
                            }
                        }
                    }
                    
                    // Reset VAD state
                    voice_active = false;
                    last_voice_instant = None;
                    segment_first_instant = None;
                }
            }
        }
    }
}

#[derive(Debug)]
struct RecordingState {
    is_recording: bool,
    phrase: String,
    phrase_number: usize,
    audio_buffer: Vec<f32>,
    start_time: Option<Instant>,
    segment_start: Option<Instant>,
    last_recording_path: Option<String>,
    last_transcription: Option<String>,
    validation_score: Option<f32>,
    // New fields for improved phrase detection
    waiting_for_phrase: bool,
    phrase_detection_buffer: Vec<f32>,
    phrase_start_time: Option<Instant>,
    command_cooldown_until: Option<Instant>,
}

impl RecordingState {
    fn new() -> Self {
        Self {
            is_recording: false,
            phrase: String::new(),
            phrase_number: 0,
            audio_buffer: Vec::new(),
            start_time: None,
            segment_start: None,
            last_recording_path: None,
            last_transcription: None,
            validation_score: None,
            waiting_for_phrase: false,
            phrase_detection_buffer: Vec::new(),
            phrase_start_time: None,
            command_cooldown_until: None,
        }
    }
    
    fn start_recording(&mut self, phrase: String, phrase_number: usize) {
        // Don't start recording immediately - wait for the target phrase
        self.waiting_for_phrase = true;
        self.phrase = phrase;
        self.phrase_number = phrase_number;
        self.audio_buffer.clear();
        self.phrase_detection_buffer.clear();
        self.start_time = None; // Will be set when phrase is detected
        self.segment_start = None;
        self.phrase_start_time = None;
        // Set cooldown to avoid detecting the command itself
        self.command_cooldown_until = Some(Instant::now() + Duration::from_millis(2000));
    }
    
    fn stop_recording(&mut self) {
        self.is_recording = false;
        self.waiting_for_phrase = false;
        // Don't clear phrase and phrase_number - keep for validation
        self.audio_buffer.clear();
        self.phrase_detection_buffer.clear();
        self.start_time = None;
        self.segment_start = None;
        self.phrase_start_time = None;
        self.command_cooldown_until = None;
    }
    
    fn set_last_recording(&mut self, path: String, transcription: Option<String>) {
        self.last_recording_path = Some(path);
        self.last_transcription = transcription;
        self.validation_score = None;
    }
    
    fn clear_session(&mut self) {
        self.phrase.clear();
        self.phrase_number = 0;
        self.last_recording_path = None;
        self.last_transcription = None;
        self.validation_score = None;
        self.waiting_for_phrase = false;
        self.phrase_detection_buffer.clear();
        self.phrase_start_time = None;
        self.command_cooldown_until = None;
    }
    
    fn start_segment(&mut self, now: Instant) {
        if self.segment_start.is_none() {
            self.segment_start = Some(now);
        }
    }
    
    fn add_audio_samples(&mut self, samples: &[f32]) {
        self.audio_buffer.extend_from_slice(samples);
    }
    
    fn add_detection_samples(&mut self, samples: &[f32]) {
        self.phrase_detection_buffer.extend_from_slice(samples);
        // Keep only the last 10 seconds for phrase detection
        let max_samples = 10 * 16000; // 10 seconds at 16kHz
        if self.phrase_detection_buffer.len() > max_samples {
            let excess = self.phrase_detection_buffer.len() - max_samples;
            self.phrase_detection_buffer.drain(0..excess);
        }
    }
    
    fn start_phrase_recording(&mut self, now: Instant) {
        self.is_recording = true;
        self.waiting_for_phrase = false;
        self.start_time = Some(now);
        self.phrase_start_time = Some(now);
        // Clear the detection buffer and start fresh recording
        self.audio_buffer.clear();
    }
    
    fn is_in_command_cooldown(&self, now: Instant) -> bool {
        if let Some(cooldown_until) = self.command_cooldown_until {
            now < cooldown_until
        } else {
            false
        }
    }
}

#[derive(Debug)]
enum TestCommand {
    StartRecording(Option<usize>),
    StartNext,
    StartWithCountdown,
    StopRecording,
    ValidateLastRecording,
    CleanAndSaveTestFile,
    ToggleTTSFeedback,
    ShowPhrases,
    ShowCurrent,
    ShowRecordingTips,
    TestTTS,
    SkipTo(usize),
    Quit,
}

fn parse_test_command(text: &str) -> Option<TestCommand> {
    let text = text.to_lowercase();
    
    if text.contains("start test recording next") {
        Some(TestCommand::StartNext)
    } else if text.contains("start recording with countdown") {
        Some(TestCommand::StartWithCountdown)
    } else if text.contains("start test recording") {
        // Try to extract phrase number
        if let Some(num_str) = text.split_whitespace().last() {
            if let Ok(num) = num_str.parse::<usize>() {
                return Some(TestCommand::StartRecording(Some(num)));
            }
        }
        Some(TestCommand::StartRecording(None))
    } else if text.contains("stop test recording") {
        Some(TestCommand::StopRecording)
    } else if text.contains("validate last recording") {
        Some(TestCommand::ValidateLastRecording)
    } else if text.contains("clean and save test file") {
        Some(TestCommand::CleanAndSaveTestFile)
    } else if text.contains("toggle tts feedback") {
        Some(TestCommand::ToggleTTSFeedback)
    } else if text.contains("show test phrases") {
        Some(TestCommand::ShowPhrases)
    } else if text.contains("show current phrase") {
        Some(TestCommand::ShowCurrent)
    } else if text.contains("show recording tips") {
        Some(TestCommand::ShowRecordingTips)
    } else if text.contains("test tts") {
        Some(TestCommand::TestTTS)
    } else if text.contains("skip to phrase") {
        if let Some(num_str) = text.split_whitespace().last() {
            if let Ok(num) = num_str.parse::<usize>() {
                return Some(TestCommand::SkipTo(num));
            }
        }
        None
    } else if text.contains("quit test recorder") {
        Some(TestCommand::Quit)
    } else {
        None
    }
}

fn save_recording(state: &RecordingState, test_dir: &Path, stt_service: &mut STTService) -> Result<(String, Option<String>), Box<dyn std::error::Error>> {
    if state.audio_buffer.is_empty() {
        return Err("No audio data to save".into());
    }
    
    // Trim silence from beginning and end
    let trimmed_audio = trim_silence(&state.audio_buffer, 0.01, 0.1);
    
    if trimmed_audio.is_empty() {
        return Err("Audio contains only silence".into());
    }
    
    // Generate filename
    let filename = format!("phrase_{:03}_{}.wav", 
                          state.phrase_number,
                          state.phrase.replace(' ', "_").replace(&['\'', '"', '?', '!', '.', ','][..], ""));
    let filepath = test_dir.join(&filename);
    
    // Save as WAV file
    let spec = WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = WavWriter::create(&filepath, spec)?;
    let duration_s = trimmed_audio.len() as f32 / 16000.0;
    let sample_count = trimmed_audio.len();
    
    for sample in &trimmed_audio {
        let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(sample_i16)?;
    }
    writer.finalize()?;
    
    info!("Saved recording: {} ({:.2}s, {} samples)", 
          filename, 
          duration_s,
          sample_count);
    
    // Transcribe the saved audio for validation
    let transcription = transcribe_audio_file(&filepath, stt_service);
    
    Ok((filename, transcription))
}

fn trim_silence(audio: &[f32], silence_threshold: f32, min_silence_duration: f32) -> Vec<f32> {
    if audio.is_empty() {
        return Vec::new();
    }
    
    let min_silence_samples = (min_silence_duration * 16000.0) as usize;
    let window_size = 160; // 10ms at 16kHz
    
    // Find start of speech
    let mut start_idx = 0;
    let mut silence_count = 0;
    
    for i in (0..audio.len()).step_by(window_size) {
        let end = (i + window_size).min(audio.len());
        let window = &audio[i..end];
        let energy = window.iter().map(|s| s * s).sum::<f32>() / window.len() as f32;
        
        if energy > silence_threshold {
            if silence_count >= min_silence_samples {
                start_idx = i.saturating_sub(min_silence_samples / 2);
            } else {
                start_idx = i;
            }
            break;
        }
        silence_count += window_size;
    }
    
    // Find end of speech
    let mut end_idx = audio.len();
    silence_count = 0;
    
    for i in (0..audio.len()).step_by(window_size).rev() {
        let end = (i + window_size).min(audio.len());
        let window = &audio[i..end];
        let energy = window.iter().map(|s| s * s).sum::<f32>() / window.len() as f32;
        
        if energy > silence_threshold {
            if silence_count >= min_silence_samples {
                end_idx = (end + min_silence_samples / 2).min(audio.len());
            } else {
                end_idx = end;
            }
            break;
        }
        silence_count += window_size;
    }
    
    if start_idx >= end_idx {
        return Vec::new();
    }
    
    audio[start_idx..end_idx].to_vec()
}

fn show_test_phrases(phrases: &[String], current_index: usize) {
    println!("📝 Test Phrases List:");
    println!("═══════════════════════════════════════");
    
    for (i, phrase) in phrases.iter().enumerate() {
        let marker = if i == current_index { "👉" } else { "  " };
        let status = if i < current_index { "✅" } else { "⏳" };
        println!("{} {} {:3}. {}", marker, status, i + 1, phrase);
    }
    
    println!("═══════════════════════════════════════");
    println!("Progress: {}/{} phrases", current_index, phrases.len());
}

fn get_test_phrases() -> Vec<String> {
    vec![
        // Basic Commands
        "enable vad".to_string(),
        "disable vad".to_string(),
        "turn on vad".to_string(),
        "turn off vad".to_string(),
        "increase sensitivity".to_string(),
        "decrease sensitivity".to_string(),
        "toggle instant output".to_string(),
        "enable narration".to_string(),
        "disable narration".to_string(),
        "show status".to_string(),
        
        // Audio & Recording Commands
        "start recording".to_string(),
        "start recording test session".to_string(),
        "stop recording".to_string(),
        "pause recording".to_string(),
        "resume recording".to_string(),
        "list sessions".to_string(),
        "show sessions".to_string(),
        "compress files".to_string(),
        "compress audio".to_string(),
        "show storage stats".to_string(),
        "cleanup storage".to_string(),
        "set sample rate to 44100".to_string(),
        
        // Transcription Management Commands
        "search transcripts".to_string(),
        "search transcripts for meeting".to_string(),
        "show recent transcripts".to_string(),
        "export transcripts".to_string(),
        "delete duplicate transcripts".to_string(),
        "show transcription statistics".to_string(),
        "create transcript backup".to_string(),
        "tag last transcript as important".to_string(),
        "find transcripts containing project".to_string(),
        "show transcription accuracy trends".to_string(),
        "merge similar transcripts".to_string(),
        "show word frequency analysis".to_string(),
        "export transcript as text".to_string(),
        "backup transcripts".to_string(),
        "transcription stats".to_string(),
        
        // STT Commands
        "switch to base model".to_string(),
        "switch to large model".to_string(),
        "use small model".to_string(),
        "set language to english".to_string(),
        "set language to spanish".to_string(),
        "enable instant output".to_string(),
        "disable instant output".to_string(),
        "set confidence threshold to 0.8".to_string(),
        "enable punctuation".to_string(),
        "disable punctuation".to_string(),
        "show model info".to_string(),
        "reload model".to_string(),
        
        // System Commands
        "show system status".to_string(),
        "restart service".to_string(),
        "show metrics".to_string(),
        "clear clipboard history".to_string(),
        "show available hotkeys".to_string(),
        "show uptime".to_string(),
        "show memory usage".to_string(),
        "toggle debug mode".to_string(),
        "benchmark system".to_string(),
        "quick test".to_string(),
        "quick save".to_string(),
        "quick reset".to_string(),
        "show shortcuts".to_string(),
        "performance test".to_string(),
        "memory stats".to_string(),
        
        // Navigation & Help Commands
        "go to settings".to_string(),
        "show history".to_string(),
        "show logs".to_string(),
        "open config file".to_string(),
        "list all commands".to_string(),
        "search commands for audio".to_string(),
        "explain command enable vad".to_string(),
        "what does start recording do".to_string(),
        "help with voice commands".to_string(),
        "show all commands".to_string(),
    ]
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
        let sample = s0 + (s1 - s0) * t;
        output.push(sample.max(-1.0).min(1.0));
    }
    output
}

/// Transcribe an audio file for validation
fn transcribe_audio_file(filepath: &std::path::Path, stt_service: &mut STTService) -> Option<String> {
    // Read the audio file
    let mut reader = match hound::WavReader::open(filepath) {
        Ok(reader) => reader,
        Err(e) => {
            error!("Failed to open audio file for transcription: {}", e);
            return None;
        }
    };
    
    // Convert to f32 samples
    let samples: Result<Vec<f32>, _> = reader.samples::<i16>()
        .map(|s| s.map(|sample| sample as f32 / 32768.0))
        .collect();
    
    let audio_samples = match samples {
        Ok(samples) => samples,
        Err(e) => {
            error!("Failed to read audio samples: {}", e);
            return None;
        }
    };
    
    // Transcribe using the provided STT service
    match stt_service.transcribe(&audio_samples) {
        Ok(result) => Some(result.text),
        Err(e) => {
            error!("STT transcription failed: {}", e);
            None
        }
    }
}

/// Calculate similarity between expected and actual transcription
fn calculate_transcription_similarity(expected: &str, actual: &str) -> f32 {
    let expected_lower = expected.to_lowercase();
    let actual_lower = actual.to_lowercase();
    let expected = expected_lower.trim();
    let actual = actual_lower.trim();
    
    if expected == actual {
        return 1.0;
    }
    
    // Handle common STT substitutions and phonetic similarities
    let expected_normalized = normalize_for_stt_comparison(expected);
    let actual_normalized = normalize_for_stt_comparison(actual);
    
    if expected_normalized == actual_normalized {
        return 1.0;
    }
    
    // Word-based similarity with fuzzy matching
    let expected_words: Vec<&str> = expected_normalized.split_whitespace().collect();
    let actual_words: Vec<&str> = actual_normalized.split_whitespace().collect();
    
    if expected_words.is_empty() && actual_words.is_empty() {
        return 1.0;
    }
    
    if expected_words.is_empty() || actual_words.is_empty() {
        return 0.0;
    }
    
    let mut matches = 0;
    let mut partial_matches = 0;
    
    for expected_word in &expected_words {
        if actual_words.contains(expected_word) {
            matches += 1;
        } else {
            // Check for partial matches (edit distance)
            for actual_word in &actual_words {
                if words_are_similar(expected_word, actual_word) {
                    partial_matches += 1;
                    break;
                }
            }
        }
    }
    
    let total_matches = matches as f32 + (partial_matches as f32 * 0.7); // Partial matches worth 70%
    total_matches / expected_words.len().max(actual_words.len()) as f32
}

/// Normalize text for STT comparison by handling common substitutions
fn normalize_for_stt_comparison(text: &str) -> String {
    let mut normalized = text.to_lowercase();
    
    // Common STT substitutions - map misheard words to correct ones
    let substitutions = [
        ("label", "enable"), // "Label VAD" -> "enable VAD"
        ("able", "enable"),
        ("unable", "enable"),
        ("this able", "disable"),
        ("disabled", "disable"),
        ("according", "recording"),
        ("record", "recording"),
        ("transcript", "transcripts"),
        ("transcription", "transcripts"),
        ("modal", "model"),
        ("models", "model"),
        ("fresh hold", "threshold"),
        ("thresh hold", "threshold"),
        ("sense activity", "sensitivity"),
        ("punk situation", "punctuation"),
        ("puncture nation", "punctuation"),
        // Phonetic variations for "vad"
        ("bad", "vad"),
        ("pad", "vad"),
        ("mad", "vad"),
        ("lad", "vad"),
        ("dad", "vad"),
        ("had", "vad"),
        ("sad", "vad"),
        ("fad", "vad"),
        ("bat", "vad"),
        ("pat", "vad"),
        ("mat", "vad"),
        ("lat", "vad"),
        ("dat", "vad"),
        ("hat", "vad"),
        ("sat", "vad"),
        ("fat", "vad"),
    ];
    
    // Apply substitutions
    for (wrong, correct) in &substitutions {
        normalized = normalized.replace(wrong, correct);
    }
    
    // Remove common filler words and punctuation
    normalized = normalized
        .replace(&['.', ',', '!', '?', ';', ':'][..], "")
        .replace("  ", " ")
        .trim()
        .to_string();
    
    normalized
}

/// Check if two words are similar using simple edit distance
fn words_are_similar(word1: &str, word2: &str) -> bool {
    if word1.len() == 0 || word2.len() == 0 {
        return false;
    }
    
    // If words are very different in length, they're probably not similar
    let len_diff = (word1.len() as i32 - word2.len() as i32).abs();
    if len_diff > 2 && len_diff > (word1.len().min(word2.len()) / 2) as i32 {
        return false;
    }
    
    // Simple edit distance calculation
    let distance = edit_distance(word1, word2);
    let max_len = word1.len().max(word2.len());
    
    // Allow up to 30% character differences for similarity
    distance as f32 / max_len as f32 <= 0.3
}

/// Calculate edit distance between two strings
fn edit_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let m = s1_chars.len();
    let n = s2_chars.len();
    
    let mut dp = vec![vec![0; n + 1]; m + 1];
    
    // Initialize base cases
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }
    
    // Fill the DP table
    for i in 1..=m {
        for j in 1..=n {
            if s1_chars[i - 1] == s2_chars[j - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = 1 + dp[i - 1][j].min(dp[i][j - 1]).min(dp[i - 1][j - 1]);
            }
        }
    }
    
    dp[m][n]
}

/// Analyze what went wrong with transcription and provide helpful feedback
fn analyze_transcription_mismatch(expected: &str, actual: &str) -> String {
    let expected_lower = expected.to_lowercase();
    let actual_lower = actual.to_lowercase();
    let expected_words: Vec<&str> = expected_lower.split_whitespace().collect();
    let actual_words: Vec<&str> = actual_lower.split_whitespace().collect();
    
    // Check for common issues
    if actual_words.is_empty() {
        return "The recording was too quiet or contained only silence. Speak louder and closer to the microphone.".to_string();
    }
    
    if actual_words.len() < expected_words.len() {
        return "Some words were missed. Speak more clearly and ensure each word is pronounced distinctly.".to_string();
    }
    
    if actual_words.len() > expected_words.len() {
        return "Extra words were detected. Speak the exact phrase without adding extra words.".to_string();
    }
    
    // Check for specific word mismatches
    let mut mismatched_words = Vec::new();
    for (i, expected_word) in expected_words.iter().enumerate() {
        if let Some(actual_word) = actual_words.get(i) {
            if expected_word != actual_word && !words_are_similar(expected_word, actual_word) {
                mismatched_words.push((expected_word, actual_word));
            }
        }
    }
    
    if !mismatched_words.is_empty() {
        let (expected_word, actual_word) = mismatched_words[0];
        
        // Provide specific guidance for common misheard words
        match *expected_word {
            "vad" => {
                if actual_word.contains("bad") || actual_word.contains("pad") || actual_word.contains("mad") {
                    return "Try pronouncing 'VAD' more clearly - emphasize the 'V' sound at the beginning.".to_string();
                }
            }
            "enable" => {
                if actual_word.contains("able") || actual_word.contains("label") {
                    return "Try pronouncing 'enable' more clearly - emphasize the 'en' at the beginning.".to_string();
                }
            }
            "disable" => {
                if actual_word.contains("able") {
                    return "Try pronouncing 'disable' more clearly - emphasize the 'dis' at the beginning.".to_string();
                }
            }
            "recording" => {
                if actual_word.contains("according") {
                    return "Try pronouncing 'recording' more clearly - emphasize the 'rec' at the beginning.".to_string();
                }
            }
            _ => {}
        }
        
        return format!("The word '{}' was heard as '{}'. Try speaking more clearly and distinctly.", expected_word, actual_word);
    }
    
    // General advice
    "Try speaking more slowly and clearly. Ensure you're in a quiet environment.".to_string()
}

/// Create a cleaned and optimized test file
fn create_clean_test_file(original_path: &str, test_dir: &std::path::Path, phrase_number: usize, phrase: &str) -> Result<String, Box<dyn std::error::Error>> {
    let original_path = std::path::Path::new(original_path);
    
    // Read the original file
    let mut reader = hound::WavReader::open(original_path)?;
    let samples: Result<Vec<i16>, _> = reader.samples().collect();
    let samples = samples?;
    
    // Convert to f32 for processing
    let audio_f32: Vec<f32> = samples.iter().map(|&s| s as f32 / 32768.0).collect();
    
    // Apply more aggressive trimming for test files
    let trimmed_audio = trim_silence(&audio_f32, 0.005, 0.05); // Lower thresholds for cleaner files
    
    if trimmed_audio.is_empty() {
        return Err("Cleaned audio is empty".into());
    }
    
    // Generate clean filename
    let clean_filename = format!("clean_test_{:03}_{}.wav", 
                                phrase_number,
                                phrase.replace(' ', "_").replace(&['\\', '"', '?', '!', '.', ','][..], ""));
    let clean_filepath = test_dir.join(&clean_filename);
    
    // Save the cleaned file
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = hound::WavWriter::create(&clean_filepath, spec)?;
    for sample in &trimmed_audio {
        let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(sample_i16)?;
    }
    writer.finalize()?;
    
    let duration_s = trimmed_audio.len() as f32 / 16000.0;
    info!("Created clean test file: {} ({:.2}s, {} samples)", 
          clean_filename, duration_s, trimmed_audio.len());
    
    Ok(clean_filename)
}

/// Check if the target phrase is detected in the transcription
fn phrase_detected_in_transcription(transcription: &str, target_phrase: &str) -> bool {
    let transcription_lower = transcription.to_lowercase();
    let target_lower = target_phrase.to_lowercase();
    
    // Direct match
    if transcription_lower.contains(&target_lower) {
        return true;
    }
    
    // Fuzzy match with normalization
    let normalized_transcription = normalize_for_stt_comparison(&transcription_lower);
    let normalized_target = normalize_for_stt_comparison(&target_lower);
    
    if normalized_transcription.contains(&normalized_target) {
        return true;
    }
    
    // Word-by-word fuzzy matching
    let transcription_words: Vec<&str> = normalized_transcription.split_whitespace().collect();
    let target_words: Vec<&str> = normalized_target.split_whitespace().collect();
    
    if target_words.is_empty() {
        return false;
    }
    
    // Look for the target phrase as a sequence in the transcription
    for i in 0..=transcription_words.len().saturating_sub(target_words.len()) {
        let window = &transcription_words[i..i + target_words.len()];
        let mut matches = 0;
        
        for (j, target_word) in target_words.iter().enumerate() {
            if j < window.len() {
                if window[j] == *target_word || words_are_similar(window[j], target_word) {
                    matches += 1;
                }
            }
        }
        
        // If most words match, consider it detected
        if matches as f32 / target_words.len() as f32 >= 0.7 {
            return true;
        }
    }
    
    false
}

/// Extract the phrase portion from the detection buffer
/// This is a simplified version - in a full implementation, we'd use word timestamps
fn extract_phrase_from_buffer(buffer: &[f32], transcription: &str, target_phrase: &str) -> Option<Vec<f32>> {
    if buffer.is_empty() {
        return None;
    }
    
    // For now, we'll extract the last portion of the buffer that likely contains the phrase
    // In a more sophisticated implementation, we'd use word timestamps to get exact boundaries
    
    let transcription_words: Vec<&str> = transcription.split_whitespace().collect();
    let target_words: Vec<&str> = target_phrase.split_whitespace().collect();
    
    if target_words.is_empty() {
        return None;
    }
    
    // Estimate the duration of the target phrase
    // Assume average speaking rate of 150 words per minute (2.5 words per second)
    let estimated_phrase_duration_s = target_words.len() as f32 / 2.5;
    let estimated_samples = (estimated_phrase_duration_s * 16000.0) as usize;
    
    // Add some padding around the phrase
    let padding_samples = (0.5 * 16000.0) as usize; // 0.5 second padding on each side
    let total_samples = estimated_samples + (2 * padding_samples);
    
    // Extract from the end of the buffer
    let start_idx = buffer.len().saturating_sub(total_samples);
    let phrase_audio = buffer[start_idx..].to_vec();
    
    // Apply more aggressive silence trimming for cleaner phrase extraction
    let trimmed = trim_silence(&phrase_audio, 0.005, 0.05);
    
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn init_logging() {
    use tracing_subscriber::fmt::time::UtcTime;
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_timer(UtcTime::rfc_3339())
        .with_target(true)
        .with_ansi(true);
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("test_recorder=debug,stt_clippy=info"))
        .with(stdout_layer)
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcription_similarity_improvements() {
        // Test the specific case mentioned by the user
        let expected = "enable vad";
        let actual = "Label VAD";
        let similarity = calculate_transcription_similarity(expected, actual);
        
        println!("Testing: '{}' vs '{}'", expected, actual);
        println!("Similarity: {:.1}%", similarity * 100.0);
        
        // With our improvements, this should now have high similarity
        assert!(similarity >= 0.7, "Similarity should be at least 70% but was {:.1}%", similarity * 100.0);
        
        // Test other common cases
        let test_cases = [
            ("enable vad", "enable bad", 0.7),
            ("disable vad", "disable pad", 0.7),
            ("start recording", "start according", 0.7),
            ("show transcripts", "show transcript", 0.8),
            ("set threshold", "set fresh hold", 0.6),
        ];
        
        for (expected, actual, min_similarity) in &test_cases {
            let similarity = calculate_transcription_similarity(expected, actual);
            println!("Testing: '{}' vs '{}' = {:.1}%", expected, actual, similarity * 100.0);
            assert!(similarity >= *min_similarity, 
                   "Similarity for '{}' vs '{}' should be at least {:.1}% but was {:.1}%", 
                   expected, actual, min_similarity * 100.0, similarity * 100.0);
        }
    }

    #[test]
    fn test_feedback_analysis() {
        let feedback = analyze_transcription_mismatch("enable vad", "Label VAD");
        println!("Feedback for 'enable vad' vs 'Label VAD': {}", feedback);
        
        let feedback2 = analyze_transcription_mismatch("start recording", "start according");
        println!("Feedback for 'start recording' vs 'start according': {}", feedback2);
        
        // Should provide helpful feedback
        assert!(!feedback.is_empty());
        assert!(!feedback2.is_empty());
    }

    #[test]
    fn test_phrase_detection() {
        // Test exact matches
        assert!(phrase_detected_in_transcription("enable vad", "enable vad"));
        assert!(phrase_detected_in_transcription("Enable VAD", "enable vad"));
        
        // Test with extra words (should still detect)
        assert!(phrase_detected_in_transcription("start test recording next enable vad stop", "enable vad"));
        assert!(phrase_detected_in_transcription("okay so enable vad please", "enable vad"));
        
        // Test with STT substitutions
        assert!(phrase_detected_in_transcription("Label VAD", "enable vad"));
        assert!(phrase_detected_in_transcription("enable bad", "enable vad"));
        assert!(phrase_detected_in_transcription("start according", "start recording"));
        
        // Test fuzzy matching
        assert!(phrase_detected_in_transcription("enable pad", "enable vad"));
        assert!(phrase_detected_in_transcription("disable mad", "disable vad"));
        
        // Test non-matches
        assert!(!phrase_detected_in_transcription("show status", "enable vad"));
        assert!(!phrase_detected_in_transcription("completely different", "enable vad"));
        
        // Test multi-word phrases
        assert!(phrase_detected_in_transcription("start recording test session", "start recording"));
        assert!(phrase_detected_in_transcription("show transcription statistics", "show transcription statistics"));
    }

    #[test]
    fn test_phrase_extraction() {
        // Create a mock buffer with some audio
        let buffer: Vec<f32> = (0..16000*3).map(|i| (i as f32 * 0.001).sin()).collect(); // 3 seconds of sine wave
        
        // Test extraction
        let extracted = extract_phrase_from_buffer(&buffer, "enable vad", "enable vad");
        assert!(extracted.is_some());
        
        let extracted_audio = extracted.unwrap();
        assert!(!extracted_audio.is_empty());
        assert!(extracted_audio.len() < buffer.len()); // Should be shorter than original
        
        // Test with empty buffer
        let empty_extracted = extract_phrase_from_buffer(&[], "enable vad", "enable vad");
        assert!(empty_extracted.is_none());
    }
}
