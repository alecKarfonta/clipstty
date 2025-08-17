//! Transcription Analytics and Insights System
//! 
//! This module provides comprehensive analytics for transcription data including
//! word frequency analysis, accuracy trends, usage patterns, and performance insights.

use std::collections::{HashMap, BTreeMap, VecDeque};
use std::time::Duration;
use chrono::{DateTime, Utc, NaiveDate, Datelike, Timelike};
use serde::{Deserialize, Serialize};

use super::transcription_log::{
    TranscriptEntry, TranscriptId, DailyStats, AccuracyPoint, 
    TranscriptError, AnalyticsConfig
};
use super::audio_archive::SessionId;

/// Comprehensive transcription analytics engine
pub struct TranscriptAnalytics {
    /// Word frequency analysis
    word_frequency: HashMap<String, WordStats>,
    /// Daily transcription statistics
    daily_stats: BTreeMap<NaiveDate, DailyStats>,
    /// Accuracy trend tracking
    accuracy_trends: VecDeque<AccuracyPoint>,
    /// Session correlation data
    session_correlations: HashMap<SessionId, SessionAnalytics>,
    /// Model performance comparison
    model_performance: HashMap<String, ModelPerformance>,
    /// User behavior patterns
    usage_patterns: UsagePatterns,
    /// Quality insights
    quality_insights: QualityInsights,
    /// Performance metrics
    performance_metrics: PerformanceMetrics,
    /// Configuration
    config: AnalyticsConfig,
}

/// Word statistics and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordStats {
    /// Total occurrences
    pub frequency: u64,
    /// First seen timestamp
    pub first_seen: DateTime<Utc>,
    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
    /// Average confidence when transcribed
    pub average_confidence: f32,
    /// Contexts where word appears
    pub contexts: Vec<String>,
    /// Associated sessions
    pub sessions: Vec<SessionId>,
    /// Trend data (frequency over time)
    pub trend_data: Vec<TrendPoint>,
}

/// Trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub date: NaiveDate,
    pub frequency: u32,
    pub confidence: f32,
}

/// Session-specific analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnalytics {
    /// Session ID
    pub session_id: SessionId,
    /// Total transcripts in session
    pub total_transcripts: usize,
    /// Total words transcribed
    pub total_words: usize,
    /// Average confidence
    pub average_confidence: f32,
    /// Session duration
    pub session_duration: Duration,
    /// Words per minute
    pub words_per_minute: f32,
    /// Unique words
    pub unique_words: usize,
    /// Most common words
    pub top_words: Vec<(String, u32)>,
    /// Quality score
    pub quality_score: f32,
    /// Processing efficiency
    pub processing_efficiency: f32,
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    /// Model name
    pub model_name: String,
    /// Total transcripts processed
    pub total_transcripts: usize,
    /// Average confidence
    pub average_confidence: f32,
    /// Average processing time
    pub average_processing_time: Duration,
    /// Accuracy over time
    pub accuracy_history: Vec<AccuracyPoint>,
    /// Performance by content type
    pub content_type_performance: HashMap<String, ContentTypeMetrics>,
    /// Error patterns
    pub error_patterns: Vec<ErrorPattern>,
    /// Efficiency metrics
    pub efficiency_metrics: EfficiencyMetrics,
}

/// Content type performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTypeMetrics {
    /// Content type (e.g., "meeting", "lecture", "conversation")
    pub content_type: String,
    /// Sample count
    pub sample_count: usize,
    /// Average confidence
    pub average_confidence: f32,
    /// Common issues
    pub common_issues: Vec<String>,
}

/// Error pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    /// Pattern description
    pub pattern: String,
    /// Frequency of occurrence
    pub frequency: u32,
    /// Example cases
    pub examples: Vec<String>,
    /// Suggested improvements
    pub suggestions: Vec<String>,
}

/// Efficiency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    /// Real-time factor (processing_time / audio_duration)
    pub real_time_factor: f32,
    /// Throughput (words per second)
    pub throughput: f32,
    /// Resource utilization
    pub resource_utilization: f32,
    /// Memory usage patterns
    pub memory_patterns: Vec<MemoryUsagePoint>,
}

/// Memory usage tracking point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsagePoint {
    pub timestamp: DateTime<Utc>,
    pub memory_mb: f32,
    pub transcript_count: usize,
}

/// Usage patterns analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePatterns {
    /// Peak usage hours
    pub peak_hours: Vec<u32>,
    /// Daily usage distribution
    pub daily_distribution: HashMap<u32, u32>, // hour -> count
    /// Weekly usage distribution
    pub weekly_distribution: HashMap<u32, u32>, // weekday -> count
    /// Session length patterns
    pub session_length_distribution: HashMap<String, u32>, // range -> count
    /// User behavior insights
    pub behavior_insights: Vec<BehaviorInsight>,
}

/// User behavior insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorInsight {
    /// Insight type
    pub insight_type: InsightType,
    /// Description
    pub description: String,
    /// Confidence level
    pub confidence: f32,
    /// Supporting data
    pub supporting_data: Vec<String>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Types of behavioral insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightType {
    /// Usage pattern insight
    UsagePattern,
    /// Quality improvement opportunity
    QualityImprovement,
    /// Efficiency optimization
    EfficiencyOptimization,
    /// Feature usage
    FeatureUsage,
    /// Performance trend
    PerformanceTrend,
}

/// Quality insights and recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInsights {
    /// Overall quality score
    pub overall_quality: f32,
    /// Quality trends
    pub quality_trends: Vec<QualityTrendPoint>,
    /// Common quality issues
    pub common_issues: Vec<QualityIssue>,
    /// Improvement recommendations
    pub recommendations: Vec<QualityRecommendation>,
    /// Quality by time of day
    pub quality_by_hour: HashMap<u32, f32>,
    /// Quality by session length
    pub quality_by_session_length: HashMap<String, f32>,
}

/// Quality trend point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrendPoint {
    pub date: NaiveDate,
    pub quality_score: f32,
    pub sample_count: usize,
    pub confidence_range: (f32, f32),
}

/// Quality issue identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    /// Issue type
    pub issue_type: String,
    /// Frequency of occurrence
    pub frequency: u32,
    /// Impact on quality (0.0 to 1.0)
    pub impact: f32,
    /// Example transcripts affected
    pub examples: Vec<TranscriptId>,
    /// Potential causes
    pub potential_causes: Vec<String>,
}

/// Quality improvement recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRecommendation {
    /// Recommendation type
    pub recommendation_type: RecommendationType,
    /// Description
    pub description: String,
    /// Expected improvement
    pub expected_improvement: f32,
    /// Implementation difficulty (1-5)
    pub difficulty: u8,
    /// Priority (1-5)
    pub priority: u8,
}

/// Types of quality recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    /// Audio quality improvement
    AudioQuality,
    /// Model selection
    ModelSelection,
    /// Environment optimization
    Environment,
    /// Usage pattern adjustment
    UsagePattern,
    /// Hardware upgrade
    Hardware,
}

/// Performance metrics tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Processing latency statistics
    pub latency_stats: LatencyStats,
    /// Throughput measurements
    pub throughput_stats: ThroughputStats,
    /// Resource usage tracking
    pub resource_stats: ResourceStats,
    /// Error rate tracking
    pub error_stats: ErrorStats,
}

/// Latency statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub average_latency: Duration,
    pub median_latency: Duration,
    pub p95_latency: Duration,
    pub p99_latency: Duration,
    pub latency_history: Vec<LatencyPoint>,
}

/// Latency measurement point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyPoint {
    pub timestamp: DateTime<Utc>,
    pub latency: Duration,
    pub transcript_length: usize,
    pub model_used: String,
}

/// Throughput statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputStats {
    pub words_per_second: f32,
    pub transcripts_per_hour: f32,
    pub peak_throughput: f32,
    pub throughput_history: Vec<ThroughputPoint>,
}

/// Throughput measurement point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputPoint {
    pub timestamp: DateTime<Utc>,
    pub throughput: f32,
    pub concurrent_sessions: u32,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStats {
    pub cpu_usage: Vec<ResourcePoint>,
    pub memory_usage: Vec<ResourcePoint>,
    pub disk_usage: Vec<ResourcePoint>,
    pub network_usage: Vec<ResourcePoint>,
}

/// Resource usage point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePoint {
    pub timestamp: DateTime<Utc>,
    pub usage_percent: f32,
    pub absolute_value: f64,
}

/// Error statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    pub total_errors: u32,
    pub error_rate: f32,
    pub error_types: HashMap<String, u32>,
    pub error_history: Vec<ErrorPoint>,
}

/// Error tracking point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPoint {
    pub timestamp: DateTime<Utc>,
    pub error_type: String,
    pub error_count: u32,
}

impl TranscriptAnalytics {
    /// Create a new analytics engine
    pub fn new(config: AnalyticsConfig) -> Self {
        Self {
            word_frequency: HashMap::new(),
            daily_stats: BTreeMap::new(),
            accuracy_trends: VecDeque::new(),
            session_correlations: HashMap::new(),
            model_performance: HashMap::new(),
            usage_patterns: UsagePatterns::new(),
            quality_insights: QualityInsights::new(),
            performance_metrics: PerformanceMetrics::new(),
            config,
        }
    }
    
    /// Update analytics with a new transcript
    pub fn update_with_transcript(&mut self, transcript: &TranscriptEntry) -> Result<(), TranscriptError> {
        if !self.config.enable_word_frequency && !self.config.enable_daily_stats && !self.config.enable_accuracy_tracking {
            return Ok(());
        }
        
        // Update word frequency analysis
        if self.config.enable_word_frequency {
            self.update_word_frequency(transcript)?;
        }
        
        // Update daily statistics
        if self.config.enable_daily_stats {
            self.update_daily_stats(transcript)?;
        }
        
        // Update accuracy trends
        if self.config.enable_accuracy_tracking {
            self.update_accuracy_trends(transcript)?;
        }
        
        // Update session correlations
        if let Some(session_id) = transcript.session_id {
            self.update_session_analytics(session_id, transcript)?;
        }
        
        // Update model performance
        self.update_model_performance(transcript)?;
        
        // Update usage patterns
        self.update_usage_patterns(transcript)?;
        
        // Update quality insights
        self.update_quality_insights(transcript)?;
        
        // Update performance metrics
        self.update_performance_metrics(transcript)?;
        
        println!("📊 Updated analytics for transcript {}", transcript.id);
        Ok(())
    }
    
    /// Get word frequency analysis
    pub fn get_word_frequency(&self, limit: Option<usize>) -> Vec<(&String, &WordStats)> {
        let mut words: Vec<_> = self.word_frequency.iter().collect();
        words.sort_by(|a, b| b.1.frequency.cmp(&a.1.frequency));
        
        if let Some(limit) = limit {
            words.truncate(limit);
        }
        
        words
    }
    
    /// Get daily statistics for a date range
    pub fn get_daily_stats(&self, start_date: NaiveDate, end_date: NaiveDate) -> Vec<&DailyStats> {
        self.daily_stats
            .range(start_date..=end_date)
            .map(|(_, stats)| stats)
            .collect()
    }
    
    /// Get accuracy trends
    pub fn get_accuracy_trends(&self) -> &VecDeque<AccuracyPoint> {
        &self.accuracy_trends
    }
    
    /// Get session analytics
    pub fn get_session_analytics(&self, session_id: SessionId) -> Option<&SessionAnalytics> {
        self.session_correlations.get(&session_id)
    }
    
    /// Get model performance comparison
    pub fn get_model_performance(&self) -> &HashMap<String, ModelPerformance> {
        &self.model_performance
    }
    
    /// Get usage patterns
    pub fn get_usage_patterns(&self) -> &UsagePatterns {
        &self.usage_patterns
    }
    
    /// Get quality insights
    pub fn get_quality_insights(&self) -> &QualityInsights {
        &self.quality_insights
    }
    
    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> &PerformanceMetrics {
        &self.performance_metrics
    }
    
    /// Generate comprehensive analytics report
    pub fn generate_report(&self) -> AnalyticsReport {
        AnalyticsReport {
            generated_at: Utc::now(),
            total_transcripts: self.daily_stats.values().map(|s| s.total_transcripts).sum(),
            total_words: self.daily_stats.values().map(|s| s.total_words).sum(),
            average_confidence: self.calculate_overall_confidence(),
            top_words: self.get_word_frequency(Some(20)).into_iter()
                .map(|(word, stats)| (word.clone(), stats.frequency))
                .collect(),
            quality_score: self.quality_insights.overall_quality,
            usage_insights: self.generate_usage_insights(),
            performance_summary: self.generate_performance_summary(),
            recommendations: self.generate_recommendations(),
        }
    }
    
    // Private helper methods
    
    fn update_word_frequency(&mut self, transcript: &TranscriptEntry) -> Result<(), TranscriptError> {
        let words = self.tokenize_text(&transcript.text);
        
        for word in words {
            let word_stats = self.word_frequency.entry(word.clone()).or_insert_with(|| WordStats {
                frequency: 0,
                first_seen: transcript.timestamp,
                last_seen: transcript.timestamp,
                average_confidence: 0.0,
                contexts: Vec::new(),
                sessions: Vec::new(),
                trend_data: Vec::new(),
            });
            
            word_stats.frequency += 1;
            word_stats.last_seen = transcript.timestamp;
            word_stats.average_confidence = 
                (word_stats.average_confidence * (word_stats.frequency - 1) as f32 + transcript.confidence) 
                / word_stats.frequency as f32;
            
            // Add session if not already present
            if let Some(session_id) = transcript.session_id {
                if !word_stats.sessions.contains(&session_id) {
                    word_stats.sessions.push(session_id);
                }
            }
            
            // Update trend data
            let date = transcript.timestamp.date_naive();
            if let Some(trend_point) = word_stats.trend_data.iter_mut().find(|tp| tp.date == date) {
                trend_point.frequency += 1;
                trend_point.confidence = (trend_point.confidence + transcript.confidence) / 2.0;
            } else {
                word_stats.trend_data.push(TrendPoint {
                    date,
                    frequency: 1,
                    confidence: transcript.confidence,
                });
            }
        }
        
        Ok(())
    }
    
    fn update_daily_stats(&mut self, transcript: &TranscriptEntry) -> Result<(), TranscriptError> {
        let date = transcript.timestamp.date_naive();
        let hour = transcript.timestamp.hour();
        
        let daily_stats = self.daily_stats.entry(date).or_insert_with(|| DailyStats {
            date,
            total_transcripts: 0,
            total_words: 0,
            average_confidence: 0.0,
            total_processing_time: Duration::from_secs(0),
            unique_sessions: 0,
            peak_hour: 0,
        });
        
        let word_count = transcript.text.split_whitespace().count();
        
        daily_stats.total_transcripts += 1;
        daily_stats.total_words += word_count;
        daily_stats.average_confidence = 
            (daily_stats.average_confidence * (daily_stats.total_transcripts - 1) as f32 + transcript.confidence) 
            / daily_stats.total_transcripts as f32;
        daily_stats.total_processing_time += Duration::from_millis(transcript.duration_ms);
        
        // Update peak hour (simplified - would need more sophisticated tracking)
        daily_stats.peak_hour = hour;
        
        Ok(())
    }
    
    fn update_accuracy_trends(&mut self, transcript: &TranscriptEntry) -> Result<(), TranscriptError> {
        let accuracy_point = AccuracyPoint {
            timestamp: transcript.timestamp,
            accuracy: transcript.confidence,
            sample_size: 1,
            model: transcript.model.clone(),
        };
        
        self.accuracy_trends.push_back(accuracy_point);
        
        // Keep only recent trends (configurable window)
        let retention_duration = chrono::Duration::days(self.config.retention_days as i64);
        let cutoff_time = Utc::now() - retention_duration;
        
        while let Some(front) = self.accuracy_trends.front() {
            if front.timestamp >= cutoff_time {
                break;
            }
            self.accuracy_trends.pop_front();
        }
        
        Ok(())
    }
    
    fn update_session_analytics(&mut self, session_id: SessionId, transcript: &TranscriptEntry) -> Result<(), TranscriptError> {
        let session_analytics = self.session_correlations.entry(session_id).or_insert_with(|| SessionAnalytics {
            session_id,
            total_transcripts: 0,
            total_words: 0,
            average_confidence: 0.0,
            session_duration: Duration::from_secs(0),
            words_per_minute: 0.0,
            unique_words: 0,
            top_words: Vec::new(),
            quality_score: 0.0,
            processing_efficiency: 0.0,
        });
        
        let word_count = transcript.text.split_whitespace().count();
        
        session_analytics.total_transcripts += 1;
        session_analytics.total_words += word_count;
        session_analytics.average_confidence = 
            (session_analytics.average_confidence * (session_analytics.total_transcripts - 1) as f32 + transcript.confidence) 
            / session_analytics.total_transcripts as f32;
        session_analytics.session_duration += Duration::from_millis(transcript.duration_ms);
        
        // Calculate words per minute
        if session_analytics.session_duration.as_secs() > 0 {
            session_analytics.words_per_minute = 
                (session_analytics.total_words as f32 * 60.0) / session_analytics.session_duration.as_secs() as f32;
        }
        
        Ok(())
    }
    
    fn update_model_performance(&mut self, transcript: &TranscriptEntry) -> Result<(), TranscriptError> {
        let model_perf = self.model_performance.entry(transcript.model.clone()).or_insert_with(|| ModelPerformance {
            model_name: transcript.model.clone(),
            total_transcripts: 0,
            average_confidence: 0.0,
            average_processing_time: Duration::from_secs(0),
            accuracy_history: Vec::new(),
            content_type_performance: HashMap::new(),
            error_patterns: Vec::new(),
            efficiency_metrics: EfficiencyMetrics {
                real_time_factor: 0.0,
                throughput: 0.0,
                resource_utilization: 0.0,
                memory_patterns: Vec::new(),
            },
        });
        
        model_perf.total_transcripts += 1;
        model_perf.average_confidence = 
            (model_perf.average_confidence * (model_perf.total_transcripts - 1) as f32 + transcript.confidence) 
            / model_perf.total_transcripts as f32;
        
        let processing_time = Duration::from_millis(transcript.duration_ms);
        let avg_millis = (model_perf.average_processing_time.as_millis() * (model_perf.total_transcripts - 1) as u128 
                         + processing_time.as_millis()) / model_perf.total_transcripts as u128;
        model_perf.average_processing_time = Duration::from_millis(avg_millis.min(u64::MAX as u128) as u64);
        
        // Add accuracy point
        model_perf.accuracy_history.push(AccuracyPoint {
            timestamp: transcript.timestamp,
            accuracy: transcript.confidence,
            sample_size: 1,
            model: transcript.model.clone(),
        });
        
        Ok(())
    }
    
    fn update_usage_patterns(&mut self, transcript: &TranscriptEntry) -> Result<(), TranscriptError> {
        let hour = transcript.timestamp.hour();
        let weekday = transcript.timestamp.weekday().num_days_from_monday();
        
        *self.usage_patterns.daily_distribution.entry(hour).or_insert(0) += 1;
        *self.usage_patterns.weekly_distribution.entry(weekday).or_insert(0) += 1;
        
        // Update peak hours (simplified)
        if !self.usage_patterns.peak_hours.contains(&hour) {
            self.usage_patterns.peak_hours.push(hour);
            self.usage_patterns.peak_hours.sort();
            if self.usage_patterns.peak_hours.len() > 3 {
                self.usage_patterns.peak_hours.truncate(3);
            }
        }
        
        Ok(())
    }
    
    fn update_quality_insights(&mut self, transcript: &TranscriptEntry) -> Result<(), TranscriptError> {
        // Update overall quality (running average)
        let current_count = self.quality_insights.quality_trends.len() as f32;
        self.quality_insights.overall_quality = 
            (self.quality_insights.overall_quality * current_count + transcript.confidence) 
            / (current_count + 1.0);
        
        // Add quality trend point
        let date = transcript.timestamp.date_naive();
        if let Some(trend_point) = self.quality_insights.quality_trends.iter_mut()
            .find(|tp| tp.date == date) {
            trend_point.quality_score = (trend_point.quality_score + transcript.confidence) / 2.0;
            trend_point.sample_count += 1;
        } else {
            self.quality_insights.quality_trends.push(QualityTrendPoint {
                date,
                quality_score: transcript.confidence,
                sample_count: 1,
                confidence_range: (transcript.confidence, transcript.confidence),
            });
        }
        
        // Update quality by hour
        let hour = transcript.timestamp.hour();
        let hour_quality = self.quality_insights.quality_by_hour.entry(hour).or_insert(0.0);
        *hour_quality = (*hour_quality + transcript.confidence) / 2.0;
        
        Ok(())
    }
    
    fn update_performance_metrics(&mut self, transcript: &TranscriptEntry) -> Result<(), TranscriptError> {
        let processing_time = Duration::from_millis(transcript.duration_ms);
        
        // Update latency stats
        self.performance_metrics.latency_stats.latency_history.push(LatencyPoint {
            timestamp: transcript.timestamp,
            latency: processing_time,
            transcript_length: transcript.text.len(),
            model_used: transcript.model.clone(),
        });
        
        // Update throughput stats
        let word_count = self.count_words(&transcript.text);
        let throughput = if processing_time.as_secs_f32() > 0.0 {
            word_count as f32 / processing_time.as_secs_f32()
        } else {
            0.0
        };
        
        self.performance_metrics.throughput_stats.throughput_history.push(ThroughputPoint {
            timestamp: transcript.timestamp,
            throughput,
            concurrent_sessions: 1, // Simplified
        });
        
        Ok(())
    }
    
    fn tokenize_text(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|word| {
                word.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            .filter(|word| !word.is_empty() && word.len() >= 2)
            .collect()
    }
    
    fn count_words(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }
    
    fn calculate_overall_confidence(&self) -> f32 {
        if self.accuracy_trends.is_empty() {
            return 0.0;
        }
        
        let sum: f32 = self.accuracy_trends.iter().map(|ap| ap.accuracy).sum();
        sum / self.accuracy_trends.len() as f32
    }
    
    fn generate_usage_insights(&self) -> Vec<String> {
        let mut insights = Vec::new();
        
        // Peak usage analysis
        if !self.usage_patterns.peak_hours.is_empty() {
            insights.push(format!("Peak usage hours: {:?}", self.usage_patterns.peak_hours));
        }
        
        // Quality patterns
        if self.quality_insights.overall_quality > 0.9 {
            insights.push("Excellent transcription quality maintained".to_string());
        } else if self.quality_insights.overall_quality < 0.7 {
            insights.push("Transcription quality could be improved".to_string());
        }
        
        insights
    }
    
    fn generate_performance_summary(&self) -> String {
        format!(
            "Average confidence: {:.1}%, Total transcripts: {}, Models used: {}",
            self.calculate_overall_confidence() * 100.0,
            self.accuracy_trends.len(),
            self.model_performance.len()
        )
    }
    
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.quality_insights.overall_quality < 0.8 {
            recommendations.push("Consider improving audio quality or using a more advanced model".to_string());
        }
        
        if self.usage_patterns.peak_hours.len() > 2 {
            recommendations.push("Consider load balancing during peak usage hours".to_string());
        }
        
        recommendations
    }
}

/// Comprehensive analytics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub generated_at: DateTime<Utc>,
    pub total_transcripts: usize,
    pub total_words: usize,
    pub average_confidence: f32,
    pub top_words: Vec<(String, u64)>,
    pub quality_score: f32,
    pub usage_insights: Vec<String>,
    pub performance_summary: String,
    pub recommendations: Vec<String>,
}

// Default implementations
impl UsagePatterns {
    fn new() -> Self {
        Self {
            peak_hours: Vec::new(),
            daily_distribution: HashMap::new(),
            weekly_distribution: HashMap::new(),
            session_length_distribution: HashMap::new(),
            behavior_insights: Vec::new(),
        }
    }
}

impl QualityInsights {
    fn new() -> Self {
        Self {
            overall_quality: 0.0,
            quality_trends: Vec::new(),
            common_issues: Vec::new(),
            recommendations: Vec::new(),
            quality_by_hour: HashMap::new(),
            quality_by_session_length: HashMap::new(),
        }
    }
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            latency_stats: LatencyStats {
                average_latency: Duration::from_secs(0),
                median_latency: Duration::from_secs(0),
                p95_latency: Duration::from_secs(0),
                p99_latency: Duration::from_secs(0),
                latency_history: Vec::new(),
            },
            throughput_stats: ThroughputStats {
                words_per_second: 0.0,
                transcripts_per_hour: 0.0,
                peak_throughput: 0.0,
                throughput_history: Vec::new(),
            },
            resource_stats: ResourceStats {
                cpu_usage: Vec::new(),
                memory_usage: Vec::new(),
                disk_usage: Vec::new(),
                network_usage: Vec::new(),
            },
            error_stats: ErrorStats {
                total_errors: 0,
                error_rate: 0.0,
                error_types: HashMap::new(),
                error_history: Vec::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::transcription_log::*;
    use std::collections::HashMap;
    
    fn create_test_transcript(text: &str, confidence: f32) -> TranscriptEntry {
        TranscriptEntry {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            text: text.to_string(),
            confidence,
            model: "test-model".to_string(),
            duration_ms: 1000,
            audio_file_id: None,
            session_id: Some(uuid::Uuid::new_v4()),
            hash: 0,
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
    fn test_analytics_creation() {
        let config = AnalyticsConfig {
            enable_word_frequency: true,
            enable_daily_stats: true,
            enable_accuracy_tracking: true,
            retention_days: 30,
        };
        
        let analytics = TranscriptAnalytics::new(config);
        assert_eq!(analytics.word_frequency.len(), 0);
        assert_eq!(analytics.daily_stats.len(), 0);
    }
    
    #[test]
    fn test_word_frequency_update() {
        let config = AnalyticsConfig {
            enable_word_frequency: true,
            enable_daily_stats: false,
            enable_accuracy_tracking: false,
            retention_days: 30,
        };
        
        let mut analytics = TranscriptAnalytics::new(config);
        let transcript = create_test_transcript("hello world hello", 0.9);
        
        analytics.update_with_transcript(&transcript).unwrap();
        
        assert_eq!(analytics.word_frequency.len(), 2);
        assert_eq!(analytics.word_frequency.get("hello").unwrap().frequency, 2);
        assert_eq!(analytics.word_frequency.get("world").unwrap().frequency, 1);
    }
    
    #[test]
    fn test_daily_stats_update() {
        let config = AnalyticsConfig {
            enable_word_frequency: false,
            enable_daily_stats: true,
            enable_accuracy_tracking: false,
            retention_days: 30,
        };
        
        let mut analytics = TranscriptAnalytics::new(config);
        let transcript = create_test_transcript("hello world", 0.9);
        
        analytics.update_with_transcript(&transcript).unwrap();
        
        let today = Utc::now().date_naive();
        assert!(analytics.daily_stats.contains_key(&today));
        
        let stats = analytics.daily_stats.get(&today).unwrap();
        assert_eq!(stats.total_transcripts, 1);
        assert_eq!(stats.total_words, 2);
        assert_eq!(stats.average_confidence, 0.9);
    }
    
    #[test]
    fn test_analytics_report_generation() {
        let config = AnalyticsConfig {
            enable_word_frequency: true,
            enable_daily_stats: true,
            enable_accuracy_tracking: true,
            retention_days: 30,
        };
        
        let mut analytics = TranscriptAnalytics::new(config);
        let transcript = create_test_transcript("hello world test", 0.85);
        
        analytics.update_with_transcript(&transcript).unwrap();
        
        let report = analytics.generate_report();
        assert_eq!(report.total_transcripts, 1);
        assert_eq!(report.total_words, 3);
        assert_eq!(report.average_confidence, 0.85);
        assert!(!report.top_words.is_empty());
    }
}
