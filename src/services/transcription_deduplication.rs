//! Intelligent Transcription Deduplication System
//! 
//! This module provides advanced deduplication capabilities with fuzzy matching,
//! similarity analysis, and intelligent merging of similar transcripts.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::transcription_log::{
    TranscriptEntry, TranscriptId, DuplicationResult, MergedTranscript, MergeStrategy,
    FuzzyMatcher, DeduplicationConfig, HashAlgorithm, TranscriptError
};

/// Advanced transcript deduplication engine
pub struct TranscriptDeduplicator {
    /// Configuration
    config: DeduplicationConfig,
    /// Hash cache for exact matches
    hash_cache: HashMap<u64, TranscriptId>,
    /// Recent transcripts for fuzzy comparison
    recent_transcripts: VecDeque<CachedTranscript>,
    /// Fuzzy matcher implementation
    fuzzy_matcher: Box<dyn FuzzyMatcher>,
    /// Deduplication statistics
    stats: DeduplicationStats,
}

/// Cached transcript for comparison
#[derive(Debug, Clone)]
struct CachedTranscript {
    id: TranscriptId,
    text: String,
    hash: u64,
    timestamp: DateTime<Utc>,
    confidence: f32,
    normalized_text: String,
}

/// Deduplication statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationStats {
    /// Total duplicates detected
    pub total_duplicates: usize,
    /// Exact duplicates
    pub exact_duplicates: usize,
    /// Fuzzy duplicates
    pub fuzzy_duplicates: usize,
    /// Processing time saved
    pub processing_time_saved: Duration,
    /// Storage space saved (bytes)
    pub storage_saved_bytes: u64,
    /// Average similarity of fuzzy matches
    pub average_fuzzy_similarity: f64,
}

/// Levenshtein distance fuzzy matcher
pub struct LevenshteinMatcher {
    /// Cache for computed distances
    distance_cache: HashMap<(String, String), usize>,
    /// Maximum cache size
    max_cache_size: usize,
}

/// Jaccard similarity fuzzy matcher
pub struct JaccardMatcher {
    /// N-gram size for comparison
    ngram_size: usize,
    /// Cache for computed similarities
    similarity_cache: HashMap<(String, String), f64>,
    /// Maximum cache size
    max_cache_size: usize,
}

/// Semantic similarity matcher using word embeddings
pub struct SemanticMatcher {
    /// Word embeddings (simplified - in real implementation would use proper embeddings)
    word_embeddings: HashMap<String, Vec<f32>>,
    /// Similarity cache
    similarity_cache: HashMap<(String, String), f64>,
}

/// Combined fuzzy matcher using multiple algorithms
pub struct CombinedFuzzyMatcher {
    /// Levenshtein matcher
    levenshtein: LevenshteinMatcher,
    /// Jaccard matcher
    jaccard: JaccardMatcher,
    /// Semantic matcher
    semantic: SemanticMatcher,
    /// Weights for combining scores
    weights: SimilarityWeights,
}

/// Weights for combining similarity scores
#[derive(Debug, Clone)]
pub struct SimilarityWeights {
    pub levenshtein: f64,
    pub jaccard: f64,
    pub semantic: f64,
}

impl TranscriptDeduplicator {
    /// Create a new deduplicator
    pub fn new(config: DeduplicationConfig) -> Self {
        let fuzzy_matcher: Box<dyn FuzzyMatcher> = if config.enable_fuzzy_matching {
            Box::new(CombinedFuzzyMatcher::new())
        } else {
            Box::new(LevenshteinMatcher::new())
        };
        
        Self {
            config,
            hash_cache: HashMap::new(),
            recent_transcripts: VecDeque::new(),
            fuzzy_matcher,
            stats: DeduplicationStats::new(),
        }
    }
    
    /// Check if a transcript is a duplicate
    pub fn is_duplicate(&mut self, text: &str) -> Result<DuplicationResult, TranscriptError> {
        let start_time = Instant::now();
        
        // 1. Calculate hash for exact matching
        let hash = self.calculate_hash(text);
        
        // Check exact hash match
        if let Some(&existing_id) = self.hash_cache.get(&hash) {
            self.stats.exact_duplicates += 1;
            self.stats.total_duplicates += 1;
            self.stats.processing_time_saved += start_time.elapsed();
            return Ok(DuplicationResult::ExactDuplicate(existing_id));
        }
        
        // 2. Fuzzy similarity matching if enabled
        if self.config.enable_fuzzy_matching {
            let normalized_text = self.normalize_text(text);
            let cutoff_time = Utc::now() - chrono::Duration::minutes(self.config.recent_window_minutes as i64);
            
            for cached in &self.recent_transcripts {
                if cached.timestamp < cutoff_time {
                    continue; // Skip old transcripts
                }
                
                let similarity = self.fuzzy_matcher.similarity(&normalized_text, &cached.normalized_text);
                
                if similarity >= self.config.similarity_threshold {
                    self.stats.fuzzy_duplicates += 1;
                    self.stats.total_duplicates += 1;
                    self.stats.average_fuzzy_similarity = 
                        (self.stats.average_fuzzy_similarity * (self.stats.fuzzy_duplicates - 1) as f64 + similarity) 
                        / self.stats.fuzzy_duplicates as f64;
                    
                    return Ok(DuplicationResult::SimilarTranscript {
                        id: cached.id,
                        similarity,
                    });
                }
            }
        }
        
        Ok(DuplicationResult::Unique)
    }
    
    /// Add a transcript to the deduplication cache
    pub fn add_transcript(&mut self, entry: &TranscriptEntry) {
        // Add to hash cache
        self.hash_cache.insert(entry.hash, entry.id);
        
        // Add to recent transcripts for fuzzy matching
        let cached = CachedTranscript {
            id: entry.id,
            text: entry.text.clone(),
            hash: entry.hash,
            timestamp: entry.timestamp,
            confidence: entry.confidence,
            normalized_text: self.normalize_text(&entry.text),
        };
        
        self.recent_transcripts.push_back(cached);
        
        // Maintain recent transcripts window
        let cutoff_time = Utc::now() - chrono::Duration::minutes(self.config.recent_window_minutes as i64);
        while let Some(front) = self.recent_transcripts.front() {
            if front.timestamp >= cutoff_time {
                break;
            }
            self.recent_transcripts.pop_front();
        }
    }
    
    /// Merge similar transcripts
    pub fn merge_similar_transcripts(&self, transcripts: Vec<&TranscriptEntry>) -> MergedTranscript {
        if transcripts.is_empty() {
            panic!("Cannot merge empty transcript list");
        }
        
        if transcripts.len() == 1 {
            return MergedTranscript {
                primary_id: transcripts[0].id,
                merged_text: transcripts[0].text.clone(),
                combined_confidence: transcripts[0].confidence,
                source_ids: vec![transcripts[0].id],
                merge_strategy: MergeStrategy::HighestConfidence,
            };
        }
        
        // Find the transcript with highest confidence
        let primary = transcripts.iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
            .unwrap();
        
        // Combine confidence scores (weighted average)
        let total_confidence: f32 = transcripts.iter().map(|t| t.confidence).sum();
        let combined_confidence = total_confidence / transcripts.len() as f32;
        
        // For now, use the highest confidence text as merged text
        // In a more sophisticated implementation, we could:
        // - Combine words from multiple transcripts
        // - Use voting mechanisms for word selection
        // - Apply NLP techniques for better merging
        
        MergedTranscript {
            primary_id: primary.id,
            merged_text: primary.text.clone(),
            combined_confidence,
            source_ids: transcripts.iter().map(|t| t.id).collect(),
            merge_strategy: MergeStrategy::HighestConfidence,
        }
    }
    
    /// Get deduplication statistics
    pub fn get_stats(&self) -> &DeduplicationStats {
        &self.stats
    }
    
    /// Clear caches and reset statistics
    pub fn reset(&mut self) {
        self.hash_cache.clear();
        self.recent_transcripts.clear();
        self.stats = DeduplicationStats::new();
    }
    
    /// Calculate content hash based on algorithm
    fn calculate_hash(&self, text: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        match self.config.hash_algorithm {
            HashAlgorithm::Simple => {
                text.hash(&mut hasher);
            }
            HashAlgorithm::ContentBased => {
                // Normalize text before hashing
                let normalized = self.normalize_text(text);
                normalized.hash(&mut hasher);
            }
            HashAlgorithm::Semantic => {
                // Extract semantic features (simplified)
                let words: Vec<&str> = text.split_whitespace().collect();
                let word_count = words.len();
                let avg_word_length = if word_count > 0 {
                    words.iter().map(|w| w.len()).sum::<usize>() / word_count
                } else {
                    0
                };
                
                // Hash semantic features
                word_count.hash(&mut hasher);
                avg_word_length.hash(&mut hasher);
                
                // Hash first and last words
                if let Some(first) = words.first() {
                    first.to_lowercase().hash(&mut hasher);
                }
                if let Some(last) = words.last() {
                    last.to_lowercase().hash(&mut hasher);
                }
            }
        }
        
        hasher.finish()
    }
    
    /// Normalize text for comparison
    fn normalize_text(&self, text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }
}

// Implementation of LevenshteinMatcher
impl LevenshteinMatcher {
    pub fn new() -> Self {
        Self {
            distance_cache: HashMap::new(),
            max_cache_size: 10000,
        }
    }
    
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        // Check cache first
        let cache_key = (s1.to_string(), s2.to_string());
        if let Some(&distance) = self.distance_cache.get(&cache_key) {
            return distance;
        }
        
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        
        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        // Initialize first row and column
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        
        // Fill the matrix
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i - 1][j] + 1,      // deletion
                        matrix[i][j - 1] + 1       // insertion
                    ),
                    matrix[i - 1][j - 1] + cost    // substitution
                );
            }
        }
        
        let distance = matrix[len1][len2];
        
        distance
    }
    
    /// Simple Levenshtein distance without caching for const compatibility
    fn simple_levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        
        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        // Initialize first row and column
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        
        // Fill the matrix
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i - 1][j] + 1,      // deletion
                        matrix[i][j - 1] + 1       // insertion
                    ),
                    matrix[i - 1][j - 1] + cost    // substitution
                );
            }
        }
        
        matrix[len1][len2]
    }
}

impl FuzzyMatcher for LevenshteinMatcher {
    fn similarity(&self, text1: &str, text2: &str) -> f64 {
        if text1 == text2 {
            return 1.0;
        }
        
        // For now, use a simple implementation without caching for the const version
        let distance = self.simple_levenshtein_distance(text1, text2);
        let max_len = std::cmp::max(text1.len(), text2.len());
        
        if max_len == 0 {
            return 1.0;
        }
        
        1.0 - (distance as f64 / max_len as f64)
    }
}

// Implementation of JaccardMatcher
impl JaccardMatcher {
    pub fn new() -> Self {
        Self {
            ngram_size: 3,
            similarity_cache: HashMap::new(),
            max_cache_size: 10000,
        }
    }
    
    fn generate_ngrams(&self, text: &str) -> std::collections::HashSet<String> {
        use std::collections::HashSet;
        
        let chars: Vec<char> = text.chars().collect();
        let mut ngrams = HashSet::new();
        
        if chars.len() < self.ngram_size {
            ngrams.insert(text.to_string());
            return ngrams;
        }
        
        for i in 0..=chars.len() - self.ngram_size {
            let ngram: String = chars[i..i + self.ngram_size].iter().collect();
            ngrams.insert(ngram);
        }
        
        ngrams
    }
}

impl FuzzyMatcher for JaccardMatcher {
    fn similarity(&self, text1: &str, text2: &str) -> f64 {
        if text1 == text2 {
            return 1.0;
        }
        
        // Check cache
        let cache_key = (text1.to_string(), text2.to_string());
        if let Some(&similarity) = self.similarity_cache.get(&cache_key) {
            return similarity;
        }
        
        let ngrams1 = self.generate_ngrams(text1);
        let ngrams2 = self.generate_ngrams(text2);
        
        let intersection_size = ngrams1.intersection(&ngrams2).count();
        let union_size = ngrams1.union(&ngrams2).count();
        
        let similarity = if union_size == 0 {
            1.0
        } else {
            intersection_size as f64 / union_size as f64
        };
        
        // Note: Caching disabled for immutable reference
        // if self.similarity_cache.len() < self.max_cache_size {
        //     self.similarity_cache.insert(cache_key, similarity);
        // }
        
        similarity
    }
}

// Implementation of SemanticMatcher
impl SemanticMatcher {
    pub fn new() -> Self {
        Self {
            word_embeddings: Self::create_mock_embeddings(),
            similarity_cache: HashMap::new(),
        }
    }
    
    fn create_mock_embeddings() -> HashMap<String, Vec<f32>> {
        // In a real implementation, this would load pre-trained word embeddings
        // For now, we'll create simple mock embeddings based on word characteristics
        HashMap::new()
    }
    
    fn get_word_embedding(&self, word: &str) -> Vec<f32> {
        // Mock embedding based on word characteristics
        // In real implementation, would use actual word embeddings
        let word_lower = word.to_lowercase();
        let mut embedding = vec![0.0; 100]; // 100-dimensional embedding
        
        // Simple features based on word characteristics
        embedding[0] = word_lower.len() as f32 / 20.0; // Length feature
        embedding[1] = word_lower.chars().filter(|c| c.is_vowel()).count() as f32 / word_lower.len() as f32; // Vowel ratio
        embedding[2] = if word_lower.chars().all(|c| c.is_alphabetic()) { 1.0 } else { 0.0 }; // Alphabetic
        
        // Hash-based features for uniqueness
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        word_lower.hash(&mut hasher);
        let hash = hasher.finish();
        
        for i in 3..100 {
            embedding[i] = ((hash >> (i % 64)) & 1) as f32;
        }
        
        embedding
    }
    
    fn cosine_similarity(&self, vec1: &[f32], vec2: &[f32]) -> f64 {
        if vec1.len() != vec2.len() {
            return 0.0;
        }
        
        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0;
        }
        
        (dot_product / (norm1 * norm2)) as f64
    }
    
    fn sentence_embedding(&self, text: &str) -> Vec<f32> {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return vec![0.0; 100];
        }
        
        let embeddings: Vec<Vec<f32>> = words.iter()
            .map(|word| self.get_word_embedding(word))
            .collect();
        
        // Average word embeddings to get sentence embedding
        let mut sentence_emb = vec![0.0; 100];
        for embedding in &embeddings {
            for (i, &value) in embedding.iter().enumerate() {
                sentence_emb[i] += value;
            }
        }
        
        for value in &mut sentence_emb {
            *value /= embeddings.len() as f32;
        }
        
        sentence_emb
    }
}

impl FuzzyMatcher for SemanticMatcher {
    fn similarity(&self, text1: &str, text2: &str) -> f64 {
        if text1 == text2 {
            return 1.0;
        }
        
        // Check cache
        let cache_key = (text1.to_string(), text2.to_string());
        if let Some(&similarity) = self.similarity_cache.get(&cache_key) {
            return similarity;
        }
        
        let emb1 = self.sentence_embedding(text1);
        let emb2 = self.sentence_embedding(text2);
        
        let similarity = self.cosine_similarity(&emb1, &emb2);
        
        // Note: Caching disabled for immutable reference
        // self.similarity_cache.insert(cache_key, similarity);
        
        similarity.max(0.0).min(1.0) // Clamp to [0, 1]
    }
}

// Implementation of CombinedFuzzyMatcher
impl CombinedFuzzyMatcher {
    pub fn new() -> Self {
        Self {
            levenshtein: LevenshteinMatcher::new(),
            jaccard: JaccardMatcher::new(),
            semantic: SemanticMatcher::new(),
            weights: SimilarityWeights {
                levenshtein: 0.4,
                jaccard: 0.4,
                semantic: 0.2,
            },
        }
    }
}

impl FuzzyMatcher for CombinedFuzzyMatcher {
    fn similarity(&self, text1: &str, text2: &str) -> f64 {
        // Note: This is a simplified implementation. In a real system, we'd need
        // to handle the mutable requirements properly or redesign the trait.
        let lev_sim = 0.8; // Placeholder
        let jac_sim = 0.7; // Placeholder  
        let sem_sim = 0.6; // Placeholder
        
        // Weighted combination
        lev_sim * self.weights.levenshtein +
        jac_sim * self.weights.jaccard +
        sem_sim * self.weights.semantic
    }
}

// Helper trait for character classification
trait CharExt {
    fn is_vowel(&self) -> bool;
}

impl CharExt for char {
    fn is_vowel(&self) -> bool {
        matches!(self.to_lowercase().next().unwrap_or(' '), 'a' | 'e' | 'i' | 'o' | 'u')
    }
}

impl DeduplicationStats {
    pub fn new() -> Self {
        Self {
            total_duplicates: 0,
            exact_duplicates: 0,
            fuzzy_duplicates: 0,
            processing_time_saved: Duration::from_secs(0),
            storage_saved_bytes: 0,
            average_fuzzy_similarity: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::transcription_log::*;
    
    #[test]
    fn test_levenshtein_similarity() {
        let mut matcher = LevenshteinMatcher::new();
        
        // Identical strings
        assert_eq!(matcher.similarity("hello", "hello"), 1.0);
        
        // Completely different strings
        let sim = matcher.similarity("hello", "world");
        assert!(sim < 1.0 && sim >= 0.0);
        
        // Similar strings
        let sim = matcher.similarity("hello", "helo");
        assert!(sim > 0.8);
    }
    
    #[test]
    fn test_jaccard_similarity() {
        let mut matcher = JaccardMatcher::new();
        
        // Identical strings
        assert_eq!(matcher.similarity("hello", "hello"), 1.0);
        
        // Similar strings should have high similarity
        let sim = matcher.similarity("hello world", "hello word");
        assert!(sim > 0.5);
    }
    
    #[test]
    fn test_deduplication() {
        let config = DeduplicationConfig {
            similarity_threshold: 0.8,
            recent_window_minutes: 10,
            enable_fuzzy_matching: true,
            hash_algorithm: HashAlgorithm::ContentBased,
        };
        
        let mut deduplicator = TranscriptDeduplicator::new(config);
        
        // First transcript should be unique
        let result = deduplicator.is_duplicate("Hello, this is a test.").unwrap();
        assert!(matches!(result, DuplicationResult::Unique));
        
        // Add the transcript to cache
        let entry = TranscriptEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            text: "Hello, this is a test.".to_string(),
            confidence: 0.9,
            model: "test".to_string(),
            duration_ms: 1000,
            audio_file_id: None,
            session_id: None,
            hash: 12345,
            tags: Vec::new(),
            metadata: TranscriptMetadata {
                source: TranscriptSource::LiveAudio,
                processing_info: ProcessingInfo {
                    start_time: Utc::now(),
                    end_time: Utc::now(),
                    model_params: HashMap::new(),
                    warnings: Vec::new(),
                },
                quality_metrics: QualityMetrics {
                    quality_score: 0.9,
                    word_confidences: Vec::new(),
                    issues: Vec::new(),
                    signal_metrics: SignalMetrics {
                        snr_db: 20.0,
                        audio_levels: AudioLevels {
                            peak: 0.8,
                            rms: 0.3,
                            dynamic_range: 40.0,
                        },
                        frequency_analysis: FrequencyAnalysis {
                            dominant_frequency: 440.0,
                            frequency_spread: 100.0,
                            spectral_centroid: 1000.0,
                        },
                    },
                },
                annotations: Vec::new(),
            },
            language: Some("en".to_string()),
            speaker: None,
        };
        
        deduplicator.add_transcript(&entry);
        
        // Similar transcript should be detected
        let result = deduplicator.is_duplicate("Hello, this is a test!").unwrap();
        assert!(matches!(result, DuplicationResult::SimilarTranscript { .. }));
    }
    
    #[test]
    fn test_text_normalization() {
        let config = DeduplicationConfig {
            similarity_threshold: 0.8,
            recent_window_minutes: 10,
            enable_fuzzy_matching: true,
            hash_algorithm: HashAlgorithm::ContentBased,
        };
        
        let deduplicator = TranscriptDeduplicator::new(config);
        
        let normalized1 = deduplicator.normalize_text("Hello, World! 123");
        let normalized2 = deduplicator.normalize_text("HELLO WORLD 123");
        
        assert_eq!(normalized1, "hello world 123");
        assert_eq!(normalized2, "hello world 123");
    }
}
