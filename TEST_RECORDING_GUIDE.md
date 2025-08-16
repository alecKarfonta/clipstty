# Voice Command Test Recording Guide

This guide helps you record audio files for testing all voice commands in the ClipSTTy system.

## 🎯 Overview

The test recorder helps you create a comprehensive test suite by recording audio files for all 74 voice command phrases. Each recording is automatically trimmed of silence and saved with a descriptive filename.

## 🚀 Quick Start

### 1. Build the Test Recorder
```bash
cargo build --bin test_recorder
```

### 2. Run the Test Recorder
```bash
WHISPER_MODEL_PATH=/path/to/your/whisper/model.bin ./target/debug/test_recorder
```

### 3. Start Recording
Say any of these commands to begin:
- `"start test recording next"` - Record the next phrase in sequence
- `"start test recording 5"` - Record phrase #5 specifically
- `"start test recording"` - Record the current phrase

### 4. Record the Phrase
When recording starts, clearly say the displayed phrase, then say:
- `"stop test recording"` - Stop and save the recording

## 📝 Complete Test Phrases List (74 phrases)

### 🎤 Basic Commands (10 phrases)
1. "enable vad"
2. "disable vad" 
3. "turn on vad"
4. "turn off vad"
5. "increase sensitivity"
6. "decrease sensitivity"
7. "toggle instant output"
8. "enable narration"
9. "disable narration"
10. "show status"

### 🎵 Audio & Recording Commands (12 phrases)
11. "start recording"
12. "start recording test session"
13. "stop recording"
14. "pause recording"
15. "resume recording"
16. "list sessions"
17. "show sessions"
18. "compress files"
19. "compress audio"
20. "show storage stats"
21. "cleanup storage"
22. "set sample rate to 44100"

### 📝 Transcription Management Commands (15 phrases)
23. "search transcripts"
24. "search transcripts for meeting"
25. "show recent transcripts"
26. "export transcripts"
27. "delete duplicate transcripts"
28. "show transcription statistics"
29. "create transcript backup"
30. "tag last transcript as important"
31. "find transcripts containing project"
32. "show transcription accuracy trends"
33. "merge similar transcripts"
34. "show word frequency analysis"
35. "export transcript as text"
36. "backup transcripts"
37. "transcription stats"

### ⚙️ STT Commands (12 phrases)
38. "switch to base model"
39. "switch to large model"
40. "use small model"
41. "set language to english"
42. "set language to spanish"
43. "enable instant output"
44. "disable instant output"
45. "set confidence threshold to 0.8"
46. "enable punctuation"
47. "disable punctuation"
48. "show model info"
49. "reload model"

### 🖥️ System Commands (15 phrases)
50. "show system status"
51. "restart service"
52. "show metrics"
53. "clear clipboard history"
54. "show available hotkeys"
55. "show uptime"
56. "show memory usage"
57. "toggle debug mode"
58. "benchmark system"
59. "quick test"
60. "quick save"
61. "quick reset"
62. "show shortcuts"
63. "performance test"
64. "memory stats"

### 🔧 Navigation & Help Commands (10 phrases)
65. "go to settings"
66. "show history"
67. "show logs"
68. "open config file"
69. "list all commands"
70. "search commands for audio"
71. "explain command enable vad"
72. "what does start recording do"
73. "help with voice commands"
74. "show all commands"

## 🎙️ Voice Commands for Test Recorder

### Recording Control
- `"start test recording next"` - Start recording the next phrase in sequence
- `"start test recording [number]"` - Start recording a specific phrase number (1-74)
- `"stop test recording"` - Stop current recording and save with silence trimming

### Navigation
- `"show test phrases"` - Display all 74 phrases with progress indicators
- `"show current phrase"` - Show the current phrase to record
- `"skip to phrase [number]"` - Jump to a specific phrase number

### System
- `"quit test recorder"` - Exit the test recorder

## 📁 Output Files

Recordings are saved in the `test_recordings/` directory with descriptive filenames:
- `phrase_001_enable_vad.wav`
- `phrase_012_start_recording_test_session.wav`
- `phrase_074_show_all_commands.wav`

### File Format
- **Format**: WAV (16-bit PCM)
- **Sample Rate**: 16kHz
- **Channels**: Mono
- **Processing**: Automatic silence trimming

## 🔧 Advanced Features

### Automatic Silence Trimming
The recorder automatically:
- Removes silence from the beginning and end of recordings
- Preserves a small buffer of silence for natural sound
- Rejects recordings that contain only silence

### Progress Tracking
- Shows completed phrases with ✅
- Shows pending phrases with ⏳
- Displays current phrase with 👉
- Tracks overall progress (e.g., "Progress: 25/74 phrases")

### Quality Control
- Minimum recording length: 1 second
- Automatic energy-based voice detection
- Real-time feedback on recording status

## 💡 Recording Tips

### For Best Results:
1. **Speak clearly** and at normal volume
2. **Wait for the recording prompt** before speaking
3. **Say the phrase exactly** as shown
4. **Pause briefly** before saying "stop test recording"
5. **Use a quiet environment** to minimize background noise

### If a Recording Fails:
- The system will show an error message
- Simply try recording the phrase again
- Check that your microphone is working properly

## 🧪 Using Test Files

Once you have recorded all phrases, you can use them for:

### Automated Testing
```rust
// Example test usage
#[test]
fn test_voice_command_recognition() {
    let audio_data = load_test_audio("phrase_001_enable_vad.wav");
    let result = voice_engine.process_audio(&audio_data);
    assert_eq!(result.command, "enable_vad");
}
```

### Regression Testing
- Test accuracy across different models
- Validate command recognition reliability
- Benchmark processing performance

### Development Testing
- Test new voice command patterns
- Validate fuzzy matching algorithms
- Debug recognition issues

## 🔍 Troubleshooting

### Common Issues:

**"No audio data to save"**
- Ensure your microphone is connected and working
- Check that the system detects voice activity
- Try speaking louder or closer to the microphone

**"Audio contains only silence"**
- The recording didn't detect any speech
- Check microphone levels and positioning
- Ensure you're speaking during the recording period

**"Command not recognized"**
- Make sure you're saying the exact test recorder commands
- Check the command list above for proper phrasing
- Ensure the STT system is working correctly

### Debug Mode:
Set the log level for more detailed output:
```bash
CLIPSTTY_LOG_LEVEL=debug WHISPER_MODEL_PATH=/path/to/model.bin ./target/debug/test_recorder
```

## 📊 Progress Tracking

The system tracks your progress and shows:
- Current phrase number and text
- Completion status for each phrase
- Overall progress percentage
- Estimated time remaining

Example output:
```
🎯 Current phrase: #25 - "show recent transcripts"
Progress: 24/74 phrases (32% complete)
```

## 🎉 Completion

When you've recorded all 74 phrases, you'll see:
```
🎉 All test phrases completed!
✅ All phrases completed!
```

Your test recordings are now ready for use in automated testing, development, and quality assurance!
