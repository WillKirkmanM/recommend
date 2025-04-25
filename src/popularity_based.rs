use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::cmp::Ordering;
use rand::{rng, Rng};
use crate::Video;

pub fn popularity_based_recommendations(
    count: usize,
    videos: &Arc<Mutex<HashMap<String, Video>>>
) -> Vec<(String, f64)> {
    let mut recommendations = Vec::new();
    let mut rng = rng();
    
    let videos_guard = videos.lock().unwrap();
    
    
    if !videos_guard.is_empty() {
        for (video_id, video) in videos_guard.iter() {
            let view_score = (video.metrics.views as f64).log10().max(0.0);
            let like_ratio = if video.metrics.views > 0 {
                video.metrics.likes as f64 / video.metrics.views as f64
            } else {
                0.0
            };
            
            let now = chrono::Utc::now();
            let days_old = (now - video.upload_date).num_days().max(1) as f64;
            let recency_factor = 1.0 + (30.0 / days_old).min(3.0);
            
            let score = (view_score * 0.6 + like_ratio * 0.4) * recency_factor;
            
            recommendations.push((video_id.clone(), score));
        }
    } else {
        let dummy_videos = vec![
            "trending_1", "trending_2", "trending_3", "trending_4", "trending_5",
            "trending_6", "trending_7", "trending_8", "trending_9", "trending_10"
        ];
        
        for video_id in dummy_videos {
            let views = rng.random_range(10000..1000000);
            let view_score = (views as f64).log10();
            let like_ratio = rng.random_range(0.6..0.95);
            
            let score = view_score * 0.6 + like_ratio * 0.4;
            recommendations.push((video_id.to_string(), score));
        }
    }
    
    recommendations.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal)
    });
    
    recommendations.truncate(count);
    
    recommendations
}

fn calculate_trending_score(video: &Video) -> f64 {
    let now = chrono::Utc::now();
    let video_age_hours = (now - video.upload_date).num_hours().max(1) as f64;
    
    let recent_views = video.metrics.views as f64 * (48.0 / video_age_hours).min(1.0);
    let recent_likes = video.metrics.likes as f64 * (48.0 / video_age_hours).min(1.0);
    let recent_comments = video.metrics.comment_count as f64 * (48.0 / video_age_hours).min(1.0);
    
    let gravity = 1.8;
    let base_score = recent_views + recent_likes * 4.0 + recent_comments * 2.0;
    
    base_score / (video_age_hours + 2.0).powf(gravity)
}