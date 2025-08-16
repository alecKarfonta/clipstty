# Improved Test Recorder - Phrase Capture Enhancement

## Problem Solved

The original test recorder was capturing too much audio, including:
- The command itself ("start test recording next")
- Background conversation and noise
- Multiple repetitions of phrases
- Everything from command detection until manual stop

**Example of the problem:**
```
Expected: "enable vad"
Got: " test recording next. Start test recording next. and then. and then. start test recording next..."
```

## Solution Implemented

### 1. **Separated Command Detection from Phrase Recording**
- Commands are detected and processed separately
- Recording only starts when the target phrase is detected
- Added cooldown period to avoid detecting commands as phrases

### 2. **Smart Phrase Detection**
- Fuzzy matching with STT error correction
- Handles common misheard words (e.g., "Label VAD" → "enable vad")
- Word-by-word similarity matching
- Robust against extra words and variations

### 3. **Precise Audio Extraction**
- Maintains a rolling 10-second detection buffer
- Extracts only the estimated phrase duration plus padding
- Applies aggressive silence trimming for clean results
- Estimates phrase length based on word count and speaking rate

### 4. **Enhanced User Experience**
- Clear instructions about the new workflow
- Real-time feedback about what's happening
- Better error messages and guidance
- TTS announcements for audio feedback

## New Workflow

1. **Command Phase**: Say "start test recording next"
2. **Waiting Phase**: System waits for target phrase detection
3. **Detection Phase**: System automatically detects and extracts the phrase
4. **Validation Phase**: Automatic transcription and similarity checking

## Key Features

### Phrase Detection Algorithm
```rust
fn phrase_detected_in_transcription(transcription: &str, target_phrase: &str) -> bool {
    // 1. Direct text matching
    // 2. Normalized STT error correction
    // 3. Fuzzy word-by-word matching with 70% threshold
}
```

### Audio Extraction
```rust
fn extract_phrase_from_buffer(buffer: &[f32], transcription: &str, target_phrase: &str) -> Option<Vec<f32>> {
    // 1. Estimate phrase duration (words / 2.5 words per second)
    // 2. Add padding (0.5 seconds each side)
    // 3. Extract from end of detection buffer
    // 4. Apply aggressive silence trimming
}
```

### Recording State Management
- `waiting_for_phrase`: Indicates listening for target phrase
- `phrase_detection_buffer`: Rolling audio buffer for phrase detection
- `command_cooldown_until`: Prevents command interference
- `is_recording`: Active phrase recording state

## Improvements Made

### 1. **Audio Quality**
- Only captures the target phrase, not commands
- Automatic silence trimming
- Consistent audio length based on phrase content
- Reduced background noise and artifacts

### 2. **Accuracy**
- Better transcription matching through fuzzy algorithms
- Handles common STT substitutions
- Reduced false positives from command detection
- More precise phrase boundary detection

### 3. **User Experience**
- Clear workflow instructions
- Real-time status updates
- Automatic phrase detection (no manual timing)
- Better error messages and recovery

### 4. **Robustness**
- Handles various speaking patterns
- Tolerates extra words and hesitations
- Recovers from detection failures
- Prevents command/phrase confusion

## Test Coverage

Added comprehensive tests for:
- Phrase detection with exact matches
- Fuzzy matching with STT errors
- Audio extraction from buffers
- Edge cases and error conditions

## Usage Example

```bash
# Start the test recorder
cargo run --bin test_recorder

# Say: "start test recording next"
# System responds: "🎯 Waiting for phrase #1: 'enable vad'"
# System responds: "Now say ONLY the target phrase: 'enable vad'"

# Say: "enable vad" (just the phrase, nothing else)
# System responds: "🎯 Target phrase detected! Starting precise recording..."
# System responds: "✨ Extracted phrase audio (8000 samples)"

# Automatic validation and progression to next phrase
```

## Technical Details

### State Machine
1. **Idle** → Command detected → **Waiting for Phrase**
2. **Waiting for Phrase** → Phrase detected → **Recording**
3. **Recording** → Stop command → **Validation**
4. **Validation** → Success → **Next Phrase** | Failure → **Retry**

### Audio Processing Pipeline
1. Continuous audio capture to detection buffer
2. Voice activity detection triggers transcription
3. Phrase detection algorithm checks for target
4. Audio extraction from detection buffer
5. Silence trimming and optimization
6. Save and validate result

### Error Handling
- Transcription failures → Retry with guidance
- Phrase not detected → Continue waiting with hints
- Audio too short/quiet → Specific feedback
- Similarity too low → Detailed mismatch analysis

## Future Enhancements

The current implementation provides a solid foundation. Future improvements could include:

1. **Word-level timestamps** from Whisper for exact phrase boundaries
2. **Voice activity detection** improvements for better phrase segmentation  
3. **Speaker adaptation** for personalized detection thresholds
4. **Real-time feedback** during phrase detection
5. **Batch processing** for multiple phrase recordings

## Impact

This enhancement transforms the test recorder from a manual, error-prone tool into an intelligent, automated system that:
- Produces clean, consistent test recordings
- Reduces user effort and frustration
- Improves transcription accuracy validation
- Enables reliable voice command testing

The improved phrase capture mechanism ensures that test recordings contain only the target phrases, making them much more suitable for voice command testing and validation.
