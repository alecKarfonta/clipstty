//! Advanced Transcription Search and Indexing System
//! 
//! This module provides comprehensive search capabilities with full-text indexing,
//! fuzzy search, regex support, and advanced filtering options.

use std::collections::{HashMap, HashSet, BTreeMap};
use std::time::Instant;
use chrono::{DateTime, Utc, NaiveDate};
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::transcription_log::{
    TranscriptEntry, TranscriptId, SearchCriteria, SearchType, SortOrder,
    TranscriptMatch, TextSnippet, MatchType, TranscriptError, IndexConfig
};

/// Advanced transcript indexer with multiple search capabilities
pub struct TranscriptIndexer {
    /// Word-based inverted index
    word_index: HashMap<String, PostingList>,
    /// Phrase index for multi-word searches
    phrase_index: HashMap<String, PostingList>,
    /// Tag index for metadata searches
    tag_index: HashMap<String, HashSet<TranscriptId>>,
    /// Date index for temporal searches
    date_index: BTreeMap<NaiveDate, HashSet<TranscriptId>>,
    /// Confidence index for quality filtering
    confidence_index: BTreeMap<ConfidenceRange, HashSet<TranscriptId>>,
    /// Session index for session-based searches
    session_index: HashMap<String, HashSet<TranscriptId>>,
    /// Language index
    language_index: HashMap<String, HashSet<TranscriptId>>,
    /// Full transcript storage for snippet generation
    transcript_storage: HashMap<TranscriptId, TranscriptEntry>,
    /// Configuration
    config: IndexConfig,
    /// Index statistics
    stats: IndexStats,
}

/// Posting list for inverted index
#[derive(Debug, Clone)]
struct PostingList {
    /// Document frequencies
    documents: HashMap<TranscriptId, TermFrequency>,
    /// Total document frequency
    total_frequency: usize,
}

/// Term frequency information
#[derive(Debug, Clone)]
struct TermFrequency {
    /// Number of occurrences in document
    frequency: usize,
    /// Positions of term in document
    positions: Vec<usize>,
    /// Context around each occurrence
    contexts: Vec<String>,
}

/// Confidence range for indexing
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ConfidenceRange {
    /// Range start (inclusive)
    start: u8, // 0-100
    /// Range end (exclusive)
    end: u8,   // 0-100
}

/// Search result with scoring
#[derive(Debug, Clone)]
struct ScoredResult {
    /// Transcript ID
    transcript_id: TranscriptId,
    /// Relevance score
    score: f64,
    /// Match information
    match_info: MatchInfo,
}

/// Match information for scoring
#[derive(Debug, Clone)]
struct MatchInfo {
    /// Term matches
    term_matches: Vec<TermMatch>,
    /// Match type
    match_type: MatchType,
    /// Total matched terms
    matched_terms: usize,
    /// Query term count
    query_terms: usize,
}

/// Individual term match
#[derive(Debug, Clone)]
struct TermMatch {
    /// Matched term
    term: String,
    /// Frequency in document
    frequency: usize,
    /// Positions in document
    positions: Vec<usize>,
    /// TF-IDF score
    tfidf_score: f64,
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Total indexed documents
    pub total_documents: usize,
    /// Total unique words
    pub unique_words: usize,
    /// Total unique phrases
    pub unique_phrases: usize,
    /// Average document length
    pub average_document_length: f64,
    /// Index size in bytes (estimated)
    pub estimated_size_bytes: usize,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

/// Search performance metrics
#[derive(Debug, Clone)]
pub struct SearchMetrics {
    /// Search duration
    pub search_duration: std::time::Duration,
    /// Number of documents searched
    pub documents_searched: usize,
    /// Number of results found
    pub results_found: usize,
    /// Index hits
    pub index_hits: usize,
    /// Cache hits
    pub cache_hits: usize,
}

impl TranscriptIndexer {
    /// Create a new transcript indexer
    pub fn new(config: IndexConfig) -> Self {
        Self {
            word_index: HashMap::new(),
            phrase_index: HashMap::new(),
            tag_index: HashMap::new(),
            date_index: BTreeMap::new(),
            confidence_index: BTreeMap::new(),
            session_index: HashMap::new(),
            language_index: HashMap::new(),
            transcript_storage: HashMap::new(),
            config,
            stats: IndexStats::new(),
        }
    }
    
    /// Index a transcript
    pub fn index_transcript(&mut self, transcript: &TranscriptEntry) -> Result<(), TranscriptError> {
        let start_time = Instant::now();
        
        // Store full transcript for snippet generation
        self.transcript_storage.insert(transcript.id, transcript.clone());
        
        // Index text content
        if self.config.enable_word_index {
            self.index_words(transcript)?;
        }
        
        if self.config.enable_phrase_index {
            self.index_phrases(transcript)?;
        }
        
        // Index metadata
        self.index_metadata(transcript)?;
        
        // Update statistics
        self.stats.total_documents += 1;
        self.stats.last_updated = Utc::now();
        
        let processing_time = start_time.elapsed();
        println!("📇 Indexed transcript {} in {:.2}ms", 
                transcript.id, processing_time.as_millis());
        
        Ok(())
    }
    
    /// Search transcripts with criteria
    pub fn search(&self, criteria: &SearchCriteria) -> Result<Vec<TranscriptMatch>, TranscriptError> {
        let start_time = Instant::now();
        
        let mut results = match &criteria.search_type {
            SearchType::FullText => self.full_text_search(criteria)?,
            SearchType::ExactPhrase => self.exact_phrase_search(criteria)?,
            SearchType::Regex(regex) => self.regex_search(criteria, regex)?,
            SearchType::Fuzzy { threshold } => self.fuzzy_search(criteria, *threshold)?,
            SearchType::Tags => self.tag_search(criteria)?,
        };
        
        // Apply filters
        results = self.apply_filters(results, criteria)?;
        
        // Sort results
        self.sort_results(&mut results, &criteria.sort_order);
        
        // Apply limit
        if let Some(limit) = criteria.limit {
            results.truncate(limit);
        }
        
        // Convert to TranscriptMatch
        let matches = self.convert_to_matches(results)?;
        
        let search_time = start_time.elapsed();
        println!("🔍 Search completed in {:.2}ms, found {} results", 
                search_time.as_millis(), matches.len());
        
        Ok(matches)
    }
    
    /// Remove transcript from index
    pub fn remove_transcript(&mut self, transcript_id: TranscriptId) -> Result<(), TranscriptError> {
        // Remove from storage
        if let Some(transcript) = self.transcript_storage.remove(&transcript_id) {
            // Remove from word index
            let words = self.tokenize_text(&transcript.text);
            for word in words {
                if let Some(posting_list) = self.word_index.get_mut(&word) {
                    posting_list.documents.remove(&transcript_id);
                    if posting_list.documents.is_empty() {
                        self.word_index.remove(&word);
                    }
                }
            }
            
            // Remove from phrase index
            let phrases = self.extract_phrases(&transcript.text);
            for phrase in phrases {
                if let Some(posting_list) = self.phrase_index.get_mut(&phrase) {
                    posting_list.documents.remove(&transcript_id);
                    if posting_list.documents.is_empty() {
                        self.phrase_index.remove(&phrase);
                    }
                }
            }
            
            // Remove from metadata indexes
            self.remove_from_metadata_indexes(&transcript, transcript_id);
            
            self.stats.total_documents -= 1;
        }
        
        Ok(())
    }
    
    /// Get index statistics
    pub fn get_stats(&self) -> &IndexStats {
        &self.stats
    }
    
    /// Rebuild entire index
    pub fn rebuild_index(&mut self) -> Result<(), TranscriptError> {
        println!("🔄 Rebuilding transcript index...");
        let start_time = Instant::now();
        
        // Clear all indexes
        self.word_index.clear();
        self.phrase_index.clear();
        self.tag_index.clear();
        self.date_index.clear();
        self.confidence_index.clear();
        self.session_index.clear();
        self.language_index.clear();
        
        // Reindex all transcripts
        let transcripts: Vec<_> = self.transcript_storage.values().cloned().collect();
        self.transcript_storage.clear();
        self.stats = IndexStats::new();
        
        for transcript in transcripts {
            self.index_transcript(&transcript)?;
        }
        
        let rebuild_time = start_time.elapsed();
        println!("✅ Index rebuilt in {:.2}s", rebuild_time.as_secs_f64());
        
        Ok(())
    }
    
    // Private helper methods
    
    fn index_words(&mut self, transcript: &TranscriptEntry) -> Result<(), TranscriptError> {
        let words = self.tokenize_text(&transcript.text);
        
        for (position, word) in words.iter().enumerate() {
            if word.len() < self.config.min_word_length {
                continue;
            }
            
            if self.config.stop_words.contains(word) {
                continue;
            }
            
            let posting_list = self.word_index.entry(word.clone()).or_insert_with(|| PostingList {
                documents: HashMap::new(),
                total_frequency: 0,
            });
            
            let term_freq = posting_list.documents.entry(transcript.id).or_insert_with(|| TermFrequency {
                frequency: 0,
                positions: Vec::new(),
                contexts: Vec::new(),
            });
            
            term_freq.frequency += 1;
            term_freq.positions.push(position);
            term_freq.contexts.push(self.extract_context(&transcript.text, position, word));
            
            posting_list.total_frequency += 1;
        }
        
        Ok(())
    }
    
    fn index_phrases(&mut self, transcript: &TranscriptEntry) -> Result<(), TranscriptError> {
        let phrases = self.extract_phrases(&transcript.text);
        
        for (position, phrase) in phrases.iter().enumerate() {
            let posting_list = self.phrase_index.entry(phrase.clone()).or_insert_with(|| PostingList {
                documents: HashMap::new(),
                total_frequency: 0,
            });
            
            let term_freq = posting_list.documents.entry(transcript.id).or_insert_with(|| TermFrequency {
                frequency: 0,
                positions: Vec::new(),
                contexts: Vec::new(),
            });
            
            term_freq.frequency += 1;
            term_freq.positions.push(position);
            term_freq.contexts.push(phrase.clone());
            
            posting_list.total_frequency += 1;
        }
        
        Ok(())
    }
    
    fn index_metadata(&mut self, transcript: &TranscriptEntry) -> Result<(), TranscriptError> {
        // Index tags
        for tag in &transcript.tags {
            self.tag_index.entry(tag.clone()).or_insert_with(HashSet::new).insert(transcript.id);
        }
        
        // Index date
        let date = transcript.timestamp.date_naive();
        self.date_index.entry(date).or_insert_with(HashSet::new).insert(transcript.id);
        
        // Index confidence range
        let confidence_range = self.get_confidence_range(transcript.confidence);
        self.confidence_index.entry(confidence_range).or_insert_with(HashSet::new).insert(transcript.id);
        
        // Index session
        if let Some(session_id) = transcript.session_id {
            self.session_index.entry(session_id.to_string()).or_insert_with(HashSet::new).insert(transcript.id);
        }
        
        // Index language
        if let Some(language) = &transcript.language {
            self.language_index.entry(language.clone()).or_insert_with(HashSet::new).insert(transcript.id);
        }
        
        Ok(())
    }
    
    fn remove_from_metadata_indexes(&mut self, transcript: &TranscriptEntry, transcript_id: TranscriptId) {
        // Remove from tag index
        for tag in &transcript.tags {
            if let Some(tag_set) = self.tag_index.get_mut(tag) {
                tag_set.remove(&transcript_id);
                if tag_set.is_empty() {
                    self.tag_index.remove(tag);
                }
            }
        }
        
        // Remove from date index
        let date = transcript.timestamp.date_naive();
        if let Some(date_set) = self.date_index.get_mut(&date) {
            date_set.remove(&transcript_id);
            if date_set.is_empty() {
                self.date_index.remove(&date);
            }
        }
        
        // Remove from confidence index
        let confidence_range = self.get_confidence_range(transcript.confidence);
        if let Some(conf_set) = self.confidence_index.get_mut(&confidence_range) {
            conf_set.remove(&transcript_id);
            if conf_set.is_empty() {
                self.confidence_index.remove(&confidence_range);
            }
        }
        
        // Remove from session index
        if let Some(session_id) = transcript.session_id {
            if let Some(session_set) = self.session_index.get_mut(&session_id.to_string()) {
                session_set.remove(&transcript_id);
                if session_set.is_empty() {
                    self.session_index.remove(&session_id.to_string());
                }
            }
        }
        
        // Remove from language index
        if let Some(language) = &transcript.language {
            if let Some(lang_set) = self.language_index.get_mut(language) {
                lang_set.remove(&transcript_id);
                if lang_set.is_empty() {
                    self.language_index.remove(language);
                }
            }
        }
    }
    
    fn tokenize_text(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|word| {
                word.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            .filter(|word| !word.is_empty())
            .collect()
    }
    
    fn extract_phrases(&self, text: &str) -> Vec<String> {
        let words = self.tokenize_text(text);
        let mut phrases = Vec::new();
        
        for window_size in 2..=self.config.max_phrase_length {
            for window in words.windows(window_size) {
                let phrase = window.join(" ");
                phrases.push(phrase);
            }
        }
        
        phrases
    }
    
    fn extract_context(&self, text: &str, position: usize, word: &str) -> String {
        let words: Vec<&str> = text.split_whitespace().collect();
        let context_size = 3; // Words before and after
        
        let start = position.saturating_sub(context_size);
        let end = (position + context_size + 1).min(words.len());
        
        words[start..end].join(" ")
    }
    
    fn get_confidence_range(&self, confidence: f32) -> ConfidenceRange {
        let conf_percent = (confidence * 100.0) as u8;
        let range_size = 10; // 10% ranges
        let start = (conf_percent / range_size) * range_size;
        let end = start + range_size;
        
        ConfidenceRange { start, end }
    }
    
    fn full_text_search(&self, criteria: &SearchCriteria) -> Result<Vec<ScoredResult>, TranscriptError> {
        let query = criteria.query.as_ref().ok_or_else(|| {
            TranscriptError::SearchError("Query required for full-text search".to_string())
        })?;
        
        let query_terms = self.tokenize_text(query);
        let mut candidate_docs = HashSet::new();
        let mut term_matches = HashMap::new();
        
        // Find candidate documents
        for term in &query_terms {
            if let Some(posting_list) = self.word_index.get(term) {
                for &doc_id in posting_list.documents.keys() {
                    candidate_docs.insert(doc_id);
                }
                term_matches.insert(term.clone(), posting_list);
            }
        }
        
        // Score documents
        let mut results = Vec::new();
        for doc_id in candidate_docs {
            let score = self.calculate_tfidf_score(&query_terms, doc_id, &term_matches);
            
            let match_info = MatchInfo {
                term_matches: self.build_term_matches(&query_terms, doc_id, &term_matches),
                match_type: MatchType::Exact,
                matched_terms: query_terms.len(),
                query_terms: query_terms.len(),
            };
            
            results.push(ScoredResult {
                transcript_id: doc_id,
                score,
                match_info,
            });
        }
        
        Ok(results)
    }
    
    fn exact_phrase_search(&self, criteria: &SearchCriteria) -> Result<Vec<ScoredResult>, TranscriptError> {
        let query = criteria.query.as_ref().ok_or_else(|| {
            TranscriptError::SearchError("Query required for phrase search".to_string())
        })?;
        
        let mut results = Vec::new();
        
        if let Some(posting_list) = self.phrase_index.get(query) {
            for (&doc_id, term_freq) in &posting_list.documents {
                let score = term_freq.frequency as f64;
                
                let match_info = MatchInfo {
                    term_matches: vec![TermMatch {
                        term: query.clone(),
                        frequency: term_freq.frequency,
                        positions: term_freq.positions.clone(),
                        tfidf_score: score,
                    }],
                    match_type: MatchType::Exact,
                    matched_terms: 1,
                    query_terms: 1,
                };
                
                results.push(ScoredResult {
                    transcript_id: doc_id,
                    score,
                    match_info,
                });
            }
        }
        
        Ok(results)
    }
    
    fn regex_search(&self, criteria: &SearchCriteria, regex: &Regex) -> Result<Vec<ScoredResult>, TranscriptError> {
        let mut results = Vec::new();
        
        for (doc_id, transcript) in &self.transcript_storage {
            if regex.is_match(&transcript.text) {
                let matches: Vec<_> = regex.find_iter(&transcript.text).collect();
                let score = matches.len() as f64;
                
                let match_info = MatchInfo {
                    term_matches: vec![TermMatch {
                        term: regex.as_str().to_string(),
                        frequency: matches.len(),
                        positions: matches.iter().map(|m| m.start()).collect(),
                        tfidf_score: score,
                    }],
                    match_type: MatchType::Exact,
                    matched_terms: 1,
                    query_terms: 1,
                };
                
                results.push(ScoredResult {
                    transcript_id: *doc_id,
                    score,
                    match_info,
                });
            }
        }
        
        Ok(results)
    }
    
    fn fuzzy_search(&self, criteria: &SearchCriteria, threshold: f64) -> Result<Vec<ScoredResult>, TranscriptError> {
        let query = criteria.query.as_ref().ok_or_else(|| {
            TranscriptError::SearchError("Query required for fuzzy search".to_string())
        })?;
        
        let mut results = Vec::new();
        
        for (doc_id, transcript) in &self.transcript_storage {
            let similarity = self.calculate_fuzzy_similarity(query, &transcript.text);
            
            if similarity >= threshold {
                let match_info = MatchInfo {
                    term_matches: vec![TermMatch {
                        term: query.clone(),
                        frequency: 1,
                        positions: vec![0],
                        tfidf_score: similarity,
                    }],
                    match_type: MatchType::Fuzzy { similarity },
                    matched_terms: 1,
                    query_terms: 1,
                };
                
                results.push(ScoredResult {
                    transcript_id: *doc_id,
                    score: similarity,
                    match_info,
                });
            }
        }
        
        Ok(results)
    }
    
    fn tag_search(&self, criteria: &SearchCriteria) -> Result<Vec<ScoredResult>, TranscriptError> {
        let mut candidate_docs = HashSet::new();
        
        for tag in &criteria.tags {
            if let Some(doc_set) = self.tag_index.get(tag) {
                if candidate_docs.is_empty() {
                    candidate_docs = doc_set.clone();
                } else {
                    candidate_docs = candidate_docs.intersection(doc_set).cloned().collect();
                }
            }
        }
        
        let results = candidate_docs.into_iter().map(|doc_id| {
            ScoredResult {
                transcript_id: doc_id,
                score: 1.0,
                match_info: MatchInfo {
                    term_matches: Vec::new(),
                    match_type: MatchType::Tag,
                    matched_terms: criteria.tags.len(),
                    query_terms: criteria.tags.len(),
                },
            }
        }).collect();
        
        Ok(results)
    }
    
    fn apply_filters(&self, mut results: Vec<ScoredResult>, criteria: &SearchCriteria) -> Result<Vec<ScoredResult>, TranscriptError> {
        // Date range filter
        if let Some((start_date, end_date)) = &criteria.date_range {
            results.retain(|result| {
                if let Some(transcript) = self.transcript_storage.get(&result.transcript_id) {
                    transcript.timestamp >= *start_date && transcript.timestamp <= *end_date
                } else {
                    false
                }
            });
        }
        
        // Confidence range filter
        if let Some((min_conf, max_conf)) = &criteria.confidence_range {
            results.retain(|result| {
                if let Some(transcript) = self.transcript_storage.get(&result.transcript_id) {
                    transcript.confidence >= *min_conf && transcript.confidence <= *max_conf
                } else {
                    false
                }
            });
        }
        
        // Session filter
        if let Some(session_id) = &criteria.session_id {
            results.retain(|result| {
                if let Some(transcript) = self.transcript_storage.get(&result.transcript_id) {
                    transcript.session_id == Some(*session_id)
                } else {
                    false
                }
            });
        }
        
        // Language filter
        if let Some(language) = &criteria.language {
            results.retain(|result| {
                if let Some(transcript) = self.transcript_storage.get(&result.transcript_id) {
                    transcript.language.as_ref() == Some(language)
                } else {
                    false
                }
            });
        }
        
        Ok(results)
    }
    
    fn sort_results(&self, results: &mut Vec<ScoredResult>, sort_order: &SortOrder) {
        match sort_order {
            SortOrder::Relevance => {
                results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            }
            SortOrder::Newest => {
                results.sort_by(|a, b| {
                    let timestamp_a = self.transcript_storage.get(&a.transcript_id).map(|t| t.timestamp);
                    let timestamp_b = self.transcript_storage.get(&b.transcript_id).map(|t| t.timestamp);
                    timestamp_b.cmp(&timestamp_a)
                });
            }
            SortOrder::Oldest => {
                results.sort_by(|a, b| {
                    let timestamp_a = self.transcript_storage.get(&a.transcript_id).map(|t| t.timestamp);
                    let timestamp_b = self.transcript_storage.get(&b.transcript_id).map(|t| t.timestamp);
                    timestamp_a.cmp(&timestamp_b)
                });
            }
            SortOrder::HighestConfidence => {
                results.sort_by(|a, b| {
                    let conf_a = self.transcript_storage.get(&a.transcript_id).map(|t| t.confidence);
                    let conf_b = self.transcript_storage.get(&b.transcript_id).map(|t| t.confidence);
                    conf_b.partial_cmp(&conf_a).unwrap()
                });
            }
            SortOrder::Longest => {
                results.sort_by(|a, b| {
                    let len_a = self.transcript_storage.get(&a.transcript_id).map(|t| t.text.len());
                    let len_b = self.transcript_storage.get(&b.transcript_id).map(|t| t.text.len());
                    len_b.cmp(&len_a)
                });
            }
        }
    }
    
    fn convert_to_matches(&self, results: Vec<ScoredResult>) -> Result<Vec<TranscriptMatch>, TranscriptError> {
        let mut matches = Vec::new();
        
        for result in results {
            if let Some(transcript) = self.transcript_storage.get(&result.transcript_id) {
                let snippets = self.generate_snippets(transcript, &result.match_info);
                
                matches.push(TranscriptMatch {
                    transcript: transcript.clone(),
                    relevance: result.score,
                    snippets,
                    match_type: result.match_info.match_type,
                });
            }
        }
        
        Ok(matches)
    }
    
    fn generate_snippets(&self, transcript: &TranscriptEntry, match_info: &MatchInfo) -> Vec<TextSnippet> {
        let mut snippets = Vec::new();
        let snippet_length = 100; // Characters around match
        
        for term_match in &match_info.term_matches {
            for &position in &term_match.positions {
                let text = &transcript.text;
                let words: Vec<&str> = text.split_whitespace().collect();
                
                if position < words.len() {
                    let start_word = position.saturating_sub(5);
                    let end_word = (position + 5).min(words.len());
                    
                    let snippet_text = words[start_word..end_word].join(" ");
                    let highlight_start = words[start_word..position].join(" ").len();
                    let highlight_end = highlight_start + term_match.term.len();
                    
                    snippets.push(TextSnippet {
                        text: snippet_text,
                        position: start_word,
                        highlights: vec![(highlight_start, highlight_end)],
                    });
                }
            }
        }
        
        // Remove duplicates and limit
        snippets.truncate(3);
        snippets
    }
    
    fn calculate_tfidf_score(&self, query_terms: &[String], doc_id: TranscriptId, term_matches: &HashMap<String, &PostingList>) -> f64 {
        let mut score = 0.0;
        let total_docs = self.stats.total_documents as f64;
        
        for term in query_terms {
            if let Some(posting_list) = term_matches.get(term) {
                if let Some(term_freq) = posting_list.documents.get(&doc_id) {
                    // TF: Term frequency in document
                    let tf = term_freq.frequency as f64;
                    
                    // IDF: Inverse document frequency
                    let df = posting_list.documents.len() as f64;
                    let idf = (total_docs / df).ln();
                    
                    // TF-IDF score
                    score += tf * idf;
                }
            }
        }
        
        score
    }
    
    fn build_term_matches(&self, query_terms: &[String], doc_id: TranscriptId, term_matches: &HashMap<String, &PostingList>) -> Vec<TermMatch> {
        let mut matches = Vec::new();
        
        for term in query_terms {
            if let Some(posting_list) = term_matches.get(term) {
                if let Some(term_freq) = posting_list.documents.get(&doc_id) {
                    let tfidf_score = self.calculate_term_tfidf(term, doc_id, posting_list);
                    
                    matches.push(TermMatch {
                        term: term.clone(),
                        frequency: term_freq.frequency,
                        positions: term_freq.positions.clone(),
                        tfidf_score,
                    });
                }
            }
        }
        
        matches
    }
    
    fn calculate_term_tfidf(&self, _term: &str, doc_id: TranscriptId, posting_list: &PostingList) -> f64 {
        let total_docs = self.stats.total_documents as f64;
        
        if let Some(term_freq) = posting_list.documents.get(&doc_id) {
            let tf = term_freq.frequency as f64;
            let df = posting_list.documents.len() as f64;
            let idf = (total_docs / df).ln();
            tf * idf
        } else {
            0.0
        }
    }
    
    fn calculate_fuzzy_similarity(&self, query: &str, text: &str) -> f64 {
        // Simple fuzzy similarity using Jaccard coefficient
        let query_words: HashSet<String> = self.tokenize_text(query).into_iter().collect();
        let text_words: HashSet<String> = self.tokenize_text(text).into_iter().collect();
        
        let intersection = query_words.intersection(&text_words).count();
        let union = query_words.union(&text_words).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

impl IndexStats {
    fn new() -> Self {
        Self {
            total_documents: 0,
            unique_words: 0,
            unique_phrases: 0,
            average_document_length: 0.0,
            estimated_size_bytes: 0,
            last_updated: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::transcription_log::*;
    use std::collections::HashSet;
    
    fn create_test_transcript(id: TranscriptId, text: &str, confidence: f32) -> TranscriptEntry {
        TranscriptEntry {
            id,
            timestamp: Utc::now(),
            text: text.to_string(),
            confidence,
            model: "test".to_string(),
            duration_ms: 1000,
            audio_file_id: None,
            session_id: None,
            hash: 0,
            tags: vec!["test".to_string()],
            metadata: TranscriptMetadata {
                source: TranscriptSource::LiveAudio,
                processing_info: ProcessingInfo {
                    start_time: Utc::now(),
                    end_time: Utc::now(),
                    model_params: HashMap::new(),
                    warnings: Vec::new(),
                },
                quality_metrics: QualityMetrics {
                    quality_score: confidence,
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
        }
    }
    
    #[test]
    fn test_indexer_creation() {
        let config = IndexConfig {
            enable_word_index: true,
            enable_phrase_index: true,
            max_phrase_length: 3,
            stop_words: HashSet::new(),
            min_word_length: 2,
        };
        
        let indexer = TranscriptIndexer::new(config);
        assert_eq!(indexer.stats.total_documents, 0);
    }
    
    #[test]
    fn test_transcript_indexing() {
        let config = IndexConfig {
            enable_word_index: true,
            enable_phrase_index: true,
            max_phrase_length: 3,
            stop_words: HashSet::new(),
            min_word_length: 2,
        };
        
        let mut indexer = TranscriptIndexer::new(config);
        let transcript = create_test_transcript(
            uuid::Uuid::new_v4(),
            "Hello world, this is a test transcript",
            0.9
        );
        
        let result = indexer.index_transcript(&transcript);
        assert!(result.is_ok());
        assert_eq!(indexer.stats.total_documents, 1);
    }
    
    #[test]
    fn test_full_text_search() {
        let config = IndexConfig {
            enable_word_index: true,
            enable_phrase_index: true,
            max_phrase_length: 3,
            stop_words: HashSet::new(),
            min_word_length: 2,
        };
        
        let mut indexer = TranscriptIndexer::new(config);
        let transcript = create_test_transcript(
            uuid::Uuid::new_v4(),
            "Hello world, this is a test transcript",
            0.9
        );
        
        indexer.index_transcript(&transcript).unwrap();
        
        let criteria = SearchCriteria {
            query: Some("hello world".to_string()),
            search_type: SearchType::FullText,
            ..Default::default()
        };
        
        let results = indexer.search(&criteria).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].transcript.id, transcript.id);
    }
    
    #[test]
    fn test_tag_search() {
        let config = IndexConfig {
            enable_word_index: true,
            enable_phrase_index: true,
            max_phrase_length: 3,
            stop_words: HashSet::new(),
            min_word_length: 2,
        };
        
        let mut indexer = TranscriptIndexer::new(config);
        let transcript = create_test_transcript(
            uuid::Uuid::new_v4(),
            "Hello world, this is a test transcript",
            0.9
        );
        
        indexer.index_transcript(&transcript).unwrap();
        
        let criteria = SearchCriteria {
            search_type: SearchType::Tags,
            tags: vec!["test".to_string()],
            ..Default::default()
        };
        
        let results = indexer.search(&criteria).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].transcript.id, transcript.id);
    }
}
