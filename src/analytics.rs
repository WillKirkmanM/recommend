use std::collections::{HashMap, HashSet};

use crate::{User, Video, Comment, VideoMetrics};

pub struct AnalyticsEngine {
    _hourly_views: HashMap<u8, u64>,
    _daily_views: HashMap<u8, u64>,
    
    user_segments: HashMap<String, Vec<String>>,
    
    _video_clusters: HashMap<String, Vec<String>>,
    
    _channel_engagement: HashMap<String, ChannelEngagement>,
}

struct ChannelEngagement {
    _subscriber_count: u64,
    _avg_view_per_sub: f64,
    _avg_like_per_view: f64,
    _avg_comment_per_view: f64,
    _subscriber_retention: f64,
}

impl AnalyticsEngine {
    pub fn new() -> Self {
        AnalyticsEngine {
            _hourly_views: HashMap::new(),
            _daily_views: HashMap::new(),
            user_segments: HashMap::new(),
            _video_clusters: HashMap::new(),
            _channel_engagement: HashMap::new(),
        }
    }
    
    pub fn run_user_segmentation(&mut self, users: &HashMap<String, User>) {
        let mut casual_viewers = Vec::new();
        let mut engaged_viewers = Vec::new();
        let content_creators = Vec::new();
        let mut niche_enthusiasts = Vec::new();
        
        for (user_id, user) in users {
            let interaction_count = user.watch_history.len();
            let comment_count = user.watch_history.iter()
                .flat_map(|we| &we.interactions)
                .filter(|i| matches!(i, crate::Interaction::Comment(_)))
                .count();
            
            if interaction_count > 100 && comment_count > 20 {
                engaged_viewers.push(user_id.clone());
            } else if interaction_count < 20 {
                casual_viewers.push(user_id.clone());
            } else if user.content_preferences.len() < 3 && !user.content_preferences.is_empty() {
                niche_enthusiasts.push(user_id.clone());
            }
        }
        
        self.user_segments.insert("casual_viewers".to_string(), casual_viewers);
        self.user_segments.insert("engaged_viewers".to_string(), engaged_viewers);
        self.user_segments.insert("content_creators".to_string(), content_creators);
        self.user_segments.insert("niche_enthusiasts".to_string(), niche_enthusiasts);
    }
    
    pub fn calculate_video_similarity_matrix(
        &self, 
        videos: &HashMap<String, Video>
    ) -> HashMap<String, HashMap<String, f64>> {
        let mut similarity_matrix = HashMap::new();
        
        for (id1, video1) in videos.iter() {
            let mut video_similarities = HashMap::new();
            
            for (id2, video2) in videos.iter() {
                if id1 == id2 {
                    continue;
                }
                
                let tag_similarity = self.calculate_tag_similarity(&video1.tags, &video2.tags);
                let category_similarity = self.calculate_tag_similarity(&video1.categories, &video2.categories);
                
                let engagement_similarity = self.calculate_engagement_similarity(&video1.metrics, &video2.metrics);
                
                let embedding_similarity = self.calculate_cosine_similarity(&video1.embedding, &video2.embedding);
                
                let overall_similarity = 
                    tag_similarity * 0.2 + 
                    category_similarity * 0.3 + 
                    engagement_similarity * 0.2 + 
                    embedding_similarity * 0.3;
                
                video_similarities.insert(id2.clone(), overall_similarity);
            }
            
            similarity_matrix.insert(id1.clone(), video_similarities);
        }
        
        similarity_matrix
    }
    
    fn calculate_tag_similarity(&self, tags1: &[String], tags2: &[String]) -> f64 {
        if tags1.is_empty() || tags2.is_empty() {
            return 0.0;
        }
        
        let set1: HashSet<_> = tags1.iter().collect();
        let set2: HashSet<_> = tags2.iter().collect();
        
        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();
        
        intersection as f64 / union as f64
    }
    
    fn calculate_engagement_similarity(&self, metrics1: &VideoMetrics, metrics2: &VideoMetrics) -> f64 {
        let like_ratio1 = if metrics1.views > 0 { metrics1.likes as f64 / metrics1.views as f64 } else { 0.0 };
        let like_ratio2 = if metrics2.views > 0 { metrics2.likes as f64 / metrics2.views as f64 } else { 0.0 };
        
        let comment_ratio1 = if metrics1.views > 0 { metrics1.comment_count as f64 / metrics1.views as f64 } else { 0.0 };
        let comment_ratio2 = if metrics2.views > 0 { metrics2.comment_count as f64 / metrics2.views as f64 } else { 0.0 };
        
        let like_diff = (like_ratio1 - like_ratio2).abs();
        let comment_diff = (comment_ratio1 - comment_ratio2).abs();
        let watch_diff = (metrics1.avg_watch_percentage - metrics2.avg_watch_percentage).abs();
        
        1.0 - (like_diff + comment_diff + watch_diff) / 3.0
    }
    
    fn calculate_cosine_similarity(&self, vec1: &[f32], vec2: &[f32]) -> f64 {
        if vec1.len() != vec2.len() || vec1.is_empty() {
            return 0.0;
        }
        
        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        
        let magnitude1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if magnitude1 * magnitude2 == 0.0 {
            return 0.0;
        }
        
        (dot_product / (magnitude1 * magnitude2)) as f64
    }
    
    pub fn extract_trending_topics(&self, recent_comments: &HashMap<String, Comment>) -> Vec<String> {
        let mut word_counts = HashMap::new();
        
        for comment in recent_comments.values() {
            for word in comment.text.split_whitespace() {
                let word = word.to_lowercase();
                *word_counts.entry(word).or_insert(0) += 1;
            }
        }
        
        let stopwords = vec!["the", "a", "an", "and", "or", "but", "is", "are", "was", "were", "I", "you", "he", "she"];
        for word in stopwords {
            word_counts.remove(word);
        }
        
        let mut words: Vec<(String, usize)> = word_counts.into_iter().collect();
        words.sort_by(|a, b| b.1.cmp(&a.1));
        
        words.iter().take(10).map(|(word, _)| word.clone()).collect()
    }
    
    pub fn generate_content_insights(&self, videos: &HashMap<String, Video>) -> HashMap<String, f64> {
        let mut category_metrics = HashMap::new();
        
        for video in videos.values() {
            for category in &video.categories {
                let entry = category_metrics.entry(category.clone()).or_insert_with(|| (0.0, 0));
                entry.0 += video.metrics.avg_watch_percentage;
                entry.1 += 1;
            }
        }
        
        category_metrics.iter().map(|(category, (total, count))| {
            (category.clone(), total / *count as f64)
        }).collect()
    }
}