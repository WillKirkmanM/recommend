use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use rand::{rng, Rng};
use crate::{User, Video};

pub fn content_based_recommendations(
    user_id: &str,
    count: usize,
    users: &Arc<Mutex<HashMap<String, User>>>,
    videos: &Arc<Mutex<HashMap<String, Video>>>
) -> Vec<(String, f64)> {
    let mut recommendations = Vec::new();
    let mut rng = rng();
    
    let users_guard = users.lock().unwrap();
    let user_opt = users_guard.get(user_id);
    
    let videos_guard = videos.lock().unwrap();
    
    
    if let Some(user) = user_opt {
        let watched_videos: HashSet<_> = user.watch_history.iter()
            .map(|event| &event.video_id)
            .collect();
        
        for (video_id, video) in videos_guard.iter() {
            if watched_videos.contains(video_id) {
                continue;
            }
            
            let mut score = 0.0;
            
            for category in &video.categories {
                if let Some(preference) = user.content_preferences.get(category) {
                    score += preference * (0.8 + rng.random::<f64>() * 0.4);
                }
            }
            
            if score == 0.0 {
                score = rng.random_range(0.1..0.3);
            }
            
            let video_age_days = (chrono::Utc::now() - video.upload_date).num_days();
            let recency_boost = if video_age_days < 30 {
                0.2 * (30 - video_age_days) as f64 / 30.0
            } else {
                0.0
            };
            
            score += recency_boost;
            
            recommendations.push((video_id.clone(), score));
        }
    } else {
        let dummy_videos = vec![
            "video_cb_1", "video_cb_2", "video_cb_3", "video_cb_4", "video_cb_5",
            "video_cb_6", "video_cb_7", "video_cb_8", "video_cb_9", "video_cb_10"
        ];
        
        for video_id in dummy_videos {
            recommendations.push((video_id.to_string(), rng.random_range(0.2..0.7)));
        }
    }
    
    recommendations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    let mut seen = HashSet::new();
    recommendations.retain(|(id, _)| seen.insert(id.clone()));
    
    recommendations.truncate(count);
    recommendations
}

fn calculate_content_similarity(
    user_preferences: &HashMap<String, f64>,
    video_categories: &[String],
    _video_tags: &[String]
) -> f64 {
    let mut score = 0.0;
    let mut matched_features = 0;
    
    for category in video_categories {
        if let Some(preference) = user_preferences.get(category) {
            score += preference;
            matched_features += 1;
        }
    }
    
    
    if matched_features > 0 {
        score / matched_features as f64
    } else {
        0.0
    }
}