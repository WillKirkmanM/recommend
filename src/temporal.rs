use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use chrono::{Utc, Timelike};

use crate::{User, Video};

pub fn get_temporal_recommendations(
    user_id: &str,
    count: usize,
    users: &Arc<Mutex<HashMap<String, User>>>,
    videos: &Arc<Mutex<HashMap<String, Video>>>
) -> Vec<(String, f64)> {
    let mut recommendations = Vec::new();
    let mut scored_videos: HashMap<String, f64> = HashMap::new();
    
    let user_subscriptions = {
        let users_guard = users.lock().unwrap();
        if let Some(user) = users_guard.get(user_id) {
            user.subscriptions.clone()
        } else {
            return recommendations;
        }
    };
    
    let time_preferences = {
        let users_guard = users.lock().unwrap();
        if let Some(user) = users_guard.get(user_id) {
            user.interaction_patterns.time_of_day_preferences.clone()
        } else {
            HashMap::new()
        }
    };
    
    let current_hour = Utc::now().hour() as u8;
    
    let videos_guard = videos.lock().unwrap();
    
    let now = SystemTime::now();
    let now_secs = now.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs();
    
    for (video_id, video) in videos_guard.iter() {
        let mut score = 0.0;
        
        if user_subscriptions.contains(&video.channel_id) {
            let video_upload_timestamp = video.upload_date.timestamp() as u64;
            let days_since_upload = ((now_secs as i64 - video_upload_timestamp as i64).abs() as f64) / (24.0 * 60.0 * 60.0);
            
            if days_since_upload < 1.0 {
                score += 5.0;
            } else if days_since_upload < 3.0 {
                score += 3.0;
            } else if days_since_upload < 7.0 {
                score += 2.0;
            } else if days_since_upload < 14.0 {
                score += 1.0;
            } else if days_since_upload < 30.0 {
                score += 0.5;
            }
        }
        
        if let Some(time_preference) = time_preferences.get(&current_hour) {
            score += *time_preference;
        }
        
        let is_trending = video.metrics.views > 1000 && 
                          video.metrics.likes as f64 / video.metrics.views as f64 > 0.8;
        if is_trending {
            score += 1.5;
        }
        
        if score > 0.0 {
            scored_videos.insert(video_id.clone(), score);
        }
    }
    
    let mut scored_list: Vec<(String, f64)> = scored_videos.into_iter().collect();
    scored_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    recommendations = scored_list.into_iter().take(count).collect();
    
    recommendations
}

pub fn get_dummy_temporal_recommendations(_user_id: &str, count: usize) -> Vec<(String, f64)> {
    let mut recommendations = Vec::new();
    
    let dummy_videos = vec![
        ("recent-music-video-1", 0.95),
        ("trending-gaming-stream-1", 0.92),
        ("morning-news-summary", 0.89),
        ("latest-subscription-upload-1", 0.87),
        ("fresh-tech-review-1", 0.85),
        ("latest-subscription-upload-2", 0.82),
        ("trending-sports-highlight", 0.78),
        ("fresh-tech-review-2", 0.76),
        ("recent-music-video-2", 0.74),
        ("latest-subscription-upload-3", 0.71),
        ("trending-gaming-stream-2", 0.68),
        ("fresh-tech-review-3", 0.65),
    ];
    
    for (idx, (video_id, score)) in dummy_videos.into_iter().enumerate() {
        if idx >= count {
            break;
        }
        recommendations.push((video_id.to_string(), score));
    }
    
    recommendations
}