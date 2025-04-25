use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use rand::{rng, Rng};
use crate::User;

pub fn collaborative_filtering_recommendations(
    user_id: &str,
    count: usize, 
    user_video_matrix: &Arc<Mutex<HashMap<String, HashMap<String, f64>>>>,
    _users: &Arc<Mutex<HashMap<String, User>>>
) -> Vec<(String, f64)> {
    let mut recommendations = Vec::new();
    let mut rng = rng();
    
    let user_video_matrix_guard = user_video_matrix.lock().unwrap();
    let user_ratings = user_video_matrix_guard.get(user_id);
    
    
    let dummy_videos = vec![
        "video_cf_1", "video_cf_2", "video_cf_3", "video_cf_4", "video_cf_5",
        "video_cf_6", "video_cf_7", "video_cf_8", "video_cf_9", "video_cf_10",
        "video_cf_11", "video_cf_12", "video_cf_13", "video_cf_14", "video_cf_15",
    ];
    
    let dummy_similar_users = vec![
        ("user2", 0.85),
        ("user5", 0.74),
        ("user8", 0.68),
        ("user9", 0.61),
        ("user15", 0.58),
    ];
    
    let has_ratings = user_ratings.is_some() && !user_ratings.unwrap().is_empty();
    
    if has_ratings {
        let user_video_ids: HashSet<&String> = user_ratings.unwrap().keys().collect();
        
        for &(_, similarity) in &dummy_similar_users {
            for _ in 0..5 {
                let video_id = dummy_videos[rng.random_range(0..dummy_videos.len())].to_string();
                if user_video_ids.contains(&video_id) {
                    continue;
                }
                
                let score = similarity * (0.7 + rng.random::<f64>() * 0.3);
                recommendations.push((video_id, score));
            }
        }
    } else {
        for video_id in dummy_videos {
            recommendations.push((video_id.to_string(), rng.random_range(0.3..0.6)));
        }
    }
    
    recommendations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    let mut seen = HashSet::new();
    recommendations.retain(|(id, _)| seen.insert(id.clone()));
    
    recommendations.truncate(count);
    recommendations
}

fn calculate_user_similarity(
    user1_ratings: &HashMap<String, f64>,
    user2_ratings: &HashMap<String, f64>
) -> f64 {
    let mut dot_product = 0.0;
    let mut magnitude1 = 0.0;
    let mut magnitude2 = 0.0;
    
    for (video_id, rating1) in user1_ratings {
        if let Some(rating2) = user2_ratings.get(video_id) {
            dot_product += rating1 * rating2;
        }
        magnitude1 += rating1 * rating1;
    }
    
    for rating2 in user2_ratings.values() {
        magnitude2 += rating2 * rating2;
    }
    
    let magnitude = magnitude1.sqrt() * magnitude2.sqrt();
    if magnitude > 0.0 {
        dot_product / magnitude
    } else {
        0.0
    }
}