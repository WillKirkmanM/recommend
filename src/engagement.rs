use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{User, Video};

pub fn get_engagement_recommendations(
    user_id: &str,
    count: usize,
    users: &Arc<Mutex<HashMap<String, User>>>,
    videos: &Arc<Mutex<HashMap<String, Video>>>
) -> Vec<(String, f64)> {
    let mut recommendations = Vec::new();
    let mut scored_videos: HashMap<String, f64> = HashMap::new();
    
    let user_patterns = {
        let users_guard = users.lock().unwrap();
        match users_guard.get(user_id) {
            Some(user) => user.interaction_patterns.clone(),
            None => return recommendations,
        }
    };
    
    let user_avg_watch_pct = user_patterns.avg_watch_percentage;
    let user_avg_comment_length = user_patterns.avg_comment_length;
    let user_like_ratio = user_patterns.like_to_view_ratio;
    let user_share_frequency = user_patterns.share_frequency;
    
    let videos_guard = videos.lock().unwrap();
    
    for (video_id, video) in videos_guard.iter() {
        let mut score = 0.0;
        
        let watch_time_compatibility = 1.0 - (user_avg_watch_pct - video.metrics.avg_watch_percentage).abs();
        score += watch_time_compatibility * 2.0;
        
        let video_like_ratio = if video.metrics.views > 0 {
            video.metrics.likes as f64 / video.metrics.views as f64
        } else {
            0.0
        };
        
        let like_ratio_similarity = 1.0 - (user_like_ratio - video_like_ratio).abs();
        score += like_ratio_similarity * 1.5;
        
        let video_comment_ratio = if video.metrics.views > 0 {
            video.metrics.comment_count as f64 / video.metrics.views as f64
        } else {
            0.0
        };
        
        let user_comments_a_lot = user_avg_comment_length > 20;
        if user_comments_a_lot && video_comment_ratio > 0.1 {
            score += 1.0;
        }
        
        if video.metrics.rewatch_rate > 0.2 {
            score += video.metrics.rewatch_rate * 1.2;
        }
        
        if video.metrics.completion_rate > 0.7 {
            score += 0.8;
        }
        
        if (user_share_frequency > 0.05) && (video.metrics.share_count as f64 / video.metrics.views as f64 > 0.02) {
            score += 1.0;
        }
        
        score = score.min(5.0);
        
        if score > 0.0 {
            scored_videos.insert(video_id.clone(), score);
        }
    }
    
    let mut scored_list: Vec<(String, f64)> = scored_videos.into_iter().collect();
    scored_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    recommendations = scored_list.into_iter().take(count).collect();
    
    recommendations
}

pub fn get_dummy_engagement_recommendations(_user_id: &str, count: usize) -> Vec<(String, f64)> {
    let mut recommendations = Vec::new();
    
    let dummy_videos = vec![
        ("high-completion-tutorial", 0.96),
        ("popular-discussion-video", 0.92),
        ("highly-commented-debate", 0.89),
        ("long-form-interview-1", 0.86),
        ("high-retention-documentary", 0.84),
        ("high-share-viral-content", 0.81),
        ("community-discussion-thread", 0.79),
        ("long-form-interview-2", 0.76),
        ("interactive-livestream-1", 0.73),
        ("high-rewatch-tutorial", 0.71),
        ("interactive-livestream-2", 0.68),
        ("community-discussion-followup", 0.66),
    ];
    
    for (idx, (video_id, score)) in dummy_videos.into_iter().enumerate() {
        if idx >= count {
            break;
        }
        recommendations.push((video_id.to_string(), score));
    }
    
    recommendations
}