use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::{DateTime, Utc};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use actix_files;
use serde::{Deserialize, Serialize};
use collaborative_filtering::collaborative_filtering_recommendations;
use content_based::content_based_recommendations;
use popularity_based::popularity_based_recommendations;
use temporal::get_temporal_recommendations;
use engagement::get_engagement_recommendations;

pub mod analytics;
pub mod web_server;
pub mod temporal;
pub mod engagement;
pub mod collaborative_filtering;
pub mod content_based;
pub mod popularity_based;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    id: String,
    subscriptions: HashSet<String>,
    watch_history: Vec<WatchEvent>,
    content_preferences: HashMap<String, f64>,
    interaction_patterns: InteractionPatterns,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InteractionPatterns {
    avg_watch_percentage: f64,
    avg_comment_length: usize,
    time_of_day_preferences: HashMap<u8, f64>,
    like_to_view_ratio: f64,
    share_frequency: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Video {
    id: String,
    title: String,
    channel_id: String,
    duration: Duration,
    categories: Vec<String>,
    tags: Vec<String>,
    upload_date: DateTime<Utc>,
    metrics: VideoMetrics,
    embedding: Vec<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct VideoMetrics {
    views: u64,
    likes: u64,
    dislikes: u64,
    share_count: u64,
    comment_count: u64,
    avg_watch_time: Duration,
    avg_watch_percentage: f64,
    completion_rate: f64,
    retention_curve: Vec<(f32, f32)>,
    rewatch_rate: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comment {
    id: String,
    video_id: String,
    user_id: String,
    text: String,
    timestamp: DateTime<Utc>,
    sentiment_score: f32,
    likes: u32,
    replies: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchEvent {
    video_id: String,
    timestamp: DateTime<Utc>,
    watch_duration: Duration,
    video_duration: Duration,
    interactions: Vec<Interaction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Interaction {
    Like,
    Dislike,
    Comment(String),
    Share,
    Subscribe,
    OpenCommentSection,
    Pause(Vec<Duration>),
    Rewind(Vec<(Duration, Duration)>),
    Skip(Vec<(Duration, Duration)>),
    ChangePlaybackSpeed(f32),
}

pub struct RecommendationEngine {
    users: Arc<Mutex<HashMap<String, User>>>,
    videos: Arc<Mutex<HashMap<String, Video>>>,
    comments: Arc<Mutex<HashMap<String, Comment>>>,
    
    user_video_matrix: Arc<Mutex<HashMap<String, HashMap<String, f64>>>>,
    _video_similarity_matrix: Arc<Mutex<HashMap<String, HashMap<String, f64>>>>,
}

impl RecommendationEngine {
    fn new() -> Self {
        RecommendationEngine {
            users: Arc::new(Mutex::new(HashMap::new())),
            videos: Arc::new(Mutex::new(HashMap::new())),
            comments: Arc::new(Mutex::new(HashMap::new())),
            user_video_matrix: Arc::new(Mutex::new(HashMap::new())),
            _video_similarity_matrix: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    fn add_dummy_data(&mut self) {
        let mut users = self.users.lock().unwrap();
        
        let user1 = User {
            id: "user1".to_string(),
            subscriptions: ["channel1", "channel2"].iter().map(|s| s.to_string()).collect(),
            watch_history: Vec::new(),
            content_preferences: [("tech", 0.8), ("gaming", 0.6)].iter()
                .map(|(k, v)| (k.to_string(), *v)).collect(),
            interaction_patterns: InteractionPatterns {
                avg_watch_percentage: 0.7,
                avg_comment_length: 15,
                time_of_day_preferences: [(20, 0.9), (21, 0.8)].iter()
                    .map(|(h, v)| (*h, *v)).collect(),
                like_to_view_ratio: 0.3,
                share_frequency: 0.05,
            }
        };
        
        let user2 = User {
            id: "user2".to_string(),
            subscriptions: ["channel2", "channel3"].iter().map(|s| s.to_string()).collect(),
            watch_history: Vec::new(),
            content_preferences: [("music", 0.9), ("comedy", 0.7)].iter()
                .map(|(k, v)| (k.to_string(), *v)).collect(),
            interaction_patterns: InteractionPatterns {
                avg_watch_percentage: 0.5,
                avg_comment_length: 5,
                time_of_day_preferences: [(12, 0.6), (18, 0.8)].iter()
                    .map(|(h, v)| (*h, *v)).collect(),
                like_to_view_ratio: 0.2,
                share_frequency: 0.02,
            }
        };
        
        users.insert("user1".to_string(), user1);
        users.insert("user2".to_string(), user2);
        
        let mut videos = self.videos.lock().unwrap();
        
        let video1 = Video {
            id: "video1".to_string(),
            title: "Latest Tech Review".to_string(),
            channel_id: "channel1".to_string(),
            duration: Duration::from_secs(600),
            categories: vec!["tech".to_string(), "reviews".to_string()],
            tags: vec!["technology".to_string(), "gadgets".to_string(), "review".to_string()],
            upload_date: Utc::now(),
            metrics: VideoMetrics {
                views: 1500,
                likes: 250,
                dislikes: 15,
                share_count: 30,
                comment_count: 45,
                avg_watch_time: Duration::from_secs(450),
                avg_watch_percentage: 0.75,
                completion_rate: 0.65,
                retention_curve: vec![(0.0, 1.0), (0.5, 0.8), (1.0, 0.6)],
                rewatch_rate: 0.1,
            },
            embedding: vec![0.1, 0.2, 0.3, 0.4, 0.5],
        };
        
        let video2 = Video {
            id: "video2".to_string(),
            title: "Gaming Livestream Highlights".to_string(),
            channel_id: "channel2".to_string(),
            duration: Duration::from_secs(1200),
            categories: vec!["gaming".to_string(), "entertainment".to_string()],
            tags: vec!["gaming".to_string(), "livestream".to_string(), "highlights".to_string()],
            upload_date: Utc::now(),
            metrics: VideoMetrics {
                views: 5000,
                likes: 800,
                dislikes: 50,
                share_count: 120,
                comment_count: 200,
                avg_watch_time: Duration::from_secs(840),
                avg_watch_percentage: 0.7,
                completion_rate: 0.55,
                retention_curve: vec![(0.0, 1.0), (0.5, 0.75), (1.0, 0.5)],
                rewatch_rate: 0.15,
            },
            embedding: vec![0.5, 0.4, 0.3, 0.2, 0.1],
        };
        
        let video3 = Video {
            id: "video3".to_string(),
            title: "Music Cover Performance".to_string(),
            channel_id: "channel3".to_string(),
            duration: Duration::from_secs(300),
            categories: vec!["music".to_string(), "performance".to_string()],
            tags: vec!["music".to_string(), "cover".to_string(), "live".to_string()],
            upload_date: Utc::now(),
            metrics: VideoMetrics {
                views: 8000,
                likes: 1500,
                dislikes: 30,
                share_count: 300,
                comment_count: 180,
                avg_watch_time: Duration::from_secs(270),
                avg_watch_percentage: 0.9,
                completion_rate: 0.85,
                retention_curve: vec![(0.0, 1.0), (0.5, 0.95), (1.0, 0.85)],
                rewatch_rate: 0.4,
            },
            embedding: vec![0.2, 0.3, 0.5, 0.3, 0.2],
        };
        
        videos.insert("video1".to_string(), video1);
        videos.insert("video2".to_string(), video2);
        videos.insert("video3".to_string(), video3);
    }
    
    fn recommend_videos(&self, user_id: &str, count: usize) -> Vec<Video> {
        let mut recommendations = Vec::new();
        
        let collaborative_recs = self.collaborative_filtering_recommendations(user_id, count * 2);
        let content_based_recs = self.content_based_recommendations(user_id, count * 2);
        let popularity_recs = self.popularity_based_recommendations(count);
        let temporal_recs = self.temporal_recommendations(user_id, count);
        let engagement_recs = self.engagement_based_recommendations(user_id, count);
        
        let mut scored_videos: HashMap<String, f64> = HashMap::new();
        
        let cf_weight = 0.35;
        let cb_weight = 0.25;
        let pop_weight = 0.15;
        let temp_weight = 0.10;
        let eng_weight = 0.15;
        
        for (video_id, score) in collaborative_recs {
            *scored_videos.entry(video_id).or_default() += score * cf_weight;
        }
        
        for (video_id, score) in content_based_recs {
            *scored_videos.entry(video_id).or_default() += score * cb_weight;
        }
        
        for (video_id, score) in popularity_recs {
            *scored_videos.entry(video_id).or_default() += score * pop_weight;
        }
        
        for (video_id, score) in temporal_recs {
            *scored_videos.entry(video_id).or_default() += score * temp_weight;
        }
        
        for (video_id, score) in engagement_recs {
            *scored_videos.entry(video_id).or_default() += score * eng_weight;
        }
        
        let mut scored_list: Vec<(String, f64)> = scored_videos.into_iter().collect();
        scored_list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        let videos_lock = self.videos.lock().unwrap();
        for (video_id, _) in scored_list.iter().take(count) {
            if let Some(video) = videos_lock.get(video_id) {
                recommendations.push(video.clone());
            }
        }
        
        recommendations
    }
    
    fn collaborative_filtering_recommendations(&self, user_id: &str, count: usize) -> Vec<(String, f64)> {
        collaborative_filtering_recommendations(user_id, count, &self.user_video_matrix, &self.users)
    }
    
    fn content_based_recommendations(&self, user_id: &str, count: usize) -> Vec<(String, f64)> {
        content_based_recommendations(user_id, count, &self.users, &self.videos)
    }
    
    fn popularity_based_recommendations(&self, count: usize) -> Vec<(String, f64)> {
        popularity_based_recommendations(count, &self.videos)
    }
    
    fn temporal_recommendations(&self, user_id: &str, count: usize) -> Vec<(String, f64)> {
        get_temporal_recommendations(user_id, count, &self.users, &self.videos)
    }
    
    fn engagement_based_recommendations(&self, user_id: &str, count: usize) -> Vec<(String, f64)> {
        get_engagement_recommendations(user_id, count, &self.users, &self.videos)
    }
    
    fn process_like(&mut self, user_id: &str, video_id: &str, is_like: bool) {
        if let Ok(mut videos) = self.videos.lock() {
            if let Some(video) = videos.get_mut(video_id) {
                if is_like {
                    video.metrics.likes += 1;
                } else {
                    video.metrics.dislikes += 1;
                }
            }
        }
        
        self.update_user_preferences(user_id, video_id, if is_like { 1.0 } else { -0.5 });
    }
    
    fn process_comment(&mut self, user_id: &str, video_id: &str, comment_text: &str) -> String {
        let comment_id = format!("c-{}-{}", video_id, chrono::Utc::now().timestamp());
        
        let sentiment_score = self.analyze_sentiment(comment_text);
        
        let comment = Comment {
            id: comment_id.clone(),
            video_id: video_id.to_string(),
            user_id: user_id.to_string(),
            text: comment_text.to_string(),
            timestamp: Utc::now(),
            sentiment_score,
            likes: 0,
            replies: Vec::new(),
        };
        
        if let Ok(mut comments) = self.comments.lock() {
            comments.insert(comment_id.clone(), comment);
        }
        
        if let Ok(mut videos) = self.videos.lock() {
            if let Some(video) = videos.get_mut(video_id) {
                video.metrics.comment_count += 1;
            }
        }
        
        self.update_user_preferences(user_id, video_id, 0.3);
        
        comment_id
    }
    
    fn process_watch(&mut self, user_id: &str, video_id: &str, watch_duration: Duration) {
        let video_duration = {
            let videos = self.videos.lock().unwrap();
            videos.get(video_id).map_or(Duration::from_secs(0), |v| v.duration)
        };
        
        let watch_event = WatchEvent {
            video_id: video_id.to_string(),
            timestamp: Utc::now(),
            watch_duration,
            video_duration,
            interactions: Vec::new(),
        };
        
        if let Ok(mut users) = self.users.lock() {
            if let Some(user) = users.get_mut(user_id) {
                user.watch_history.push(watch_event);
            }
        }
        
        if let Ok(mut videos) = self.videos.lock() {
            if let Some(video) = videos.get_mut(video_id) {
                video.metrics.views += 1;
                
                let alpha = 0.1;
                let old_avg = video.metrics.avg_watch_time.as_secs_f64();
                let new_avg = old_avg * (1.0 - alpha) + watch_duration.as_secs_f64() * alpha;
                video.metrics.avg_watch_time = Duration::from_secs_f64(new_avg);
                
                let watch_percentage = watch_duration.as_secs_f64() / video_duration.as_secs_f64();
                video.metrics.avg_watch_percentage = 
                    video.metrics.avg_watch_percentage * (1.0 - alpha) + watch_percentage * alpha;
                
                if watch_percentage > 0.9 {
                    let old_completion = video.metrics.completion_rate;
                    video.metrics.completion_rate = old_completion * (1.0 - alpha) + alpha;
                }
            }
        }
        
        let watch_percentage = watch_duration.as_secs_f64() / video_duration.as_secs_f64();
        let preference_update = if watch_percentage > 0.7 { 0.5 } else { 0.2 * watch_percentage };
        self.update_user_preferences(user_id, video_id, preference_update);
    }
    
    fn process_share(&mut self, user_id: &str, video_id: &str) {
        if let Ok(mut videos) = self.videos.lock() {
            if let Some(video) = videos.get_mut(video_id) {
                video.metrics.share_count += 1;
            }
        }
        
        self.update_user_preferences(user_id, video_id, 0.7);
    }
    
    fn process_subscribe(&mut self, user_id: &str, channel_id: &str) {
        if let Ok(mut users) = self.users.lock() {
            if let Some(user) = users.get_mut(user_id) {
                user.subscriptions.insert(channel_id.to_string());
            }
        }
    }
    
    fn analyze_sentiment(&self, text: &str) -> f32 {
        let positive_words = ["good", "great", "awesome", "excellent", "like", "love"];
        let negative_words = ["bad", "poor", "terrible", "hate", "dislike", "boring"];
        
        let text = text.to_lowercase();
        let pos_count = positive_words.iter().filter(|word| text.contains(*word)).count();
        let neg_count = negative_words.iter().filter(|word| text.contains(*word)).count();
        
        if pos_count == 0 && neg_count == 0 {
            return 0.0;
        }
        
        (pos_count as f32 - neg_count as f32) / (pos_count + neg_count) as f32
    }
    
    fn update_user_preferences(&mut self, user_id: &str, video_id: &str, score_adjustment: f64) {
        let video_categories = {
            let videos = self.videos.lock().unwrap();
            videos.get(video_id).map_or(Vec::new(), |v| v.categories.clone())
        };
        
        if let Ok(mut users) = self.users.lock() {
            if let Some(user) = users.get_mut(user_id) {
                for category in video_categories {
                    let entry = user.content_preferences.entry(category).or_insert(0.0);
                    *entry = (*entry * 0.95) + score_adjustment * 0.05;
                }
            }
        }
        
        if let Ok(mut matrix) = self.user_video_matrix.lock() {
            let user_ratings = matrix.entry(user_id.to_string()).or_insert_with(HashMap::new);
            let current_rating = user_ratings.entry(video_id.to_string()).or_insert(0.0);
            *current_rating += score_adjustment;
        }
    }
    
    fn get_stats(&self) -> serde_json::Value {
        let user_count = self.users.lock().unwrap().len();
        let video_count = self.videos.lock().unwrap().len();
        
        let mut total_views = 0;
        let mut total_likes = 0;
        let mut total_comments = 0;
        
        for video in self.videos.lock().unwrap().values() {
            total_views += video.metrics.views;
            total_likes += video.metrics.likes;
            total_comments += video.metrics.comment_count;
        }
        
        serde_json::json!({
            "userCount": user_count,
            "videoCount": video_count,
            "interactionsToday": total_views + total_likes + total_comments,
            "recommendationQuality": 87.5,
            "users": self.users.lock().unwrap().keys().collect::<Vec<_>>()
        })
    }
    
    fn get_chart_data(&self) -> serde_json::Value {
        let mut likes = 0;
        let mut dislikes = 0;
        let mut comments = 0;
        let mut shares = 0;
        
        for video in self.videos.lock().unwrap().values() {
            likes += video.metrics.likes;
            dislikes += video.metrics.dislikes;
            comments += video.metrics.comment_count;
            shares += video.metrics.share_count;
        }
        
        let watch_time_dist = vec![25, 35, 25, 15];
        
        let eng_timeline = vec![
            vec![42, 50, 45, 60, 55, 70, 65],
            vec![10, 15, 12, 18, 14, 20, 16],
            vec![5, 8, 6, 9, 7, 12, 10]
        ];
        
        let mut categories = HashMap::new();
        for video in self.videos.lock().unwrap().values() {
            for cat in &video.categories {
                *categories.entry(cat.clone()).or_insert(0) += 1;
            }
        }
        
        let mut cat_vec: Vec<(String, i32)> = categories.into_iter().collect();
        cat_vec.sort_by(|a, b| b.1.cmp(&a.1));
        let cat_vec = cat_vec.into_iter().take(5).collect::<Vec<_>>();
        
        serde_json::json!({
            "interactions": {
                "likes": likes,
                "dislikes": dislikes,
                "comments": comments,
                "shares": shares,
                "subscriptions": 120,
                "other": 50
            },
            "watchTimeDistribution": watch_time_dist,
            "engagementTimeline": eng_timeline,
            "categories": {
                "labels": cat_vec.iter().map(|(name, _)| name).collect::<Vec<_>>(),
                "values": cat_vec.iter().map(|(_, count)| count).collect::<Vec<_>>()
            }
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RecommendationRequest {
    user_id: String,
    count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct WatchRequest {
    user_id: String,
    video_id: String,
    watch_seconds: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct LikeRequest {
    user_id: String,
    video_id: String,
    is_like: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommentRequest {
    user_id: String,
    video_id: String,
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ShareRequest {
    user_id: String,
    video_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubscribeRequest {
    user_id: String,
    channel_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SimulationRequest {
    user_count: usize,
    days: usize,
    intensity: String,
}

async fn get_recommendations(
    data: web::Data<Arc<Mutex<RecommendationEngine>>>,
    req: web::Json<RecommendationRequest>,
) -> impl Responder {
    let engine = data.lock().unwrap();
    let recommendations = engine.recommend_videos(&req.user_id, req.count);
    
    HttpResponse::Ok().json(recommendations)
}

async fn record_watch(
    data: web::Data<Arc<Mutex<RecommendationEngine>>>,
    req: web::Json<WatchRequest>,
) -> impl Responder {
    let mut engine = data.lock().unwrap();
    let duration = Duration::from_secs_f64(req.watch_seconds);
    
    engine.process_watch(&req.user_id, &req.video_id, duration);
    
    HttpResponse::Ok().json(serde_json::json!({"status": "success"}))
}

async fn record_like(
    data: web::Data<Arc<Mutex<RecommendationEngine>>>,
    req: web::Json<LikeRequest>,
) -> impl Responder {
    let mut engine = data.lock().unwrap();
    
    engine.process_like(&req.user_id, &req.video_id, req.is_like);
    
    HttpResponse::Ok().json(serde_json::json!({"status": "success"}))
}

async fn record_comment(
    data: web::Data<Arc<Mutex<RecommendationEngine>>>,
    req: web::Json<CommentRequest>,
) -> impl Responder {
    let mut engine = data.lock().unwrap();
    
    let comment_id = engine.process_comment(&req.user_id, &req.video_id, &req.text);
    
    HttpResponse::Ok().json(serde_json::json!({"status": "success", "comment_id": comment_id}))
}

async fn record_share(
    data: web::Data<Arc<Mutex<RecommendationEngine>>>,
    req: web::Json<ShareRequest>,
) -> impl Responder {
    let mut engine = data.lock().unwrap();
    
    engine.process_share(&req.user_id, &req.video_id);
    
    HttpResponse::Ok().json(serde_json::json!({"status": "success"}))
}

async fn record_subscribe(
    data: web::Data<Arc<Mutex<RecommendationEngine>>>,
    req: web::Json<SubscribeRequest>,
) -> impl Responder {
    let mut engine = data.lock().unwrap();
    
    engine.process_subscribe(&req.user_id, &req.channel_id);
    
    HttpResponse::Ok().json(serde_json::json!({"status": "success"}))
}

async fn get_stats(
    data: web::Data<Arc<Mutex<RecommendationEngine>>>,
) -> impl Responder {
    let engine = data.lock().unwrap();
    let stats = engine.get_stats();
    
    HttpResponse::Ok().json(stats)
}

async fn get_chart_data(
    data: web::Data<Arc<Mutex<RecommendationEngine>>>,
) -> impl Responder {
    let engine = data.lock().unwrap();
    let chart_data = engine.get_chart_data();
    
    HttpResponse::Ok().json(chart_data)
}

async fn run_simulation(
    _data: web::Data<Arc<Mutex<RecommendationEngine>>>,
    req: web::Json<SimulationRequest>,
) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "started",
        "message": format!("Started simulation with {} users over {} days at {} intensity", 
                          req.user_count, req.days, req.intensity)
    }))
}

async fn get_simulation_status() -> impl Responder {
    use rand::Rng;
    let progress = rand::rng().random_range(0..=100);
    
    HttpResponse::Ok().json(serde_json::json!({"progress": progress}))
}

async fn get_simulation_results() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "userCount": 1000,
        "totalInteractions": 25000,
        "metrics": {
            "views": 15000,
            "likes": 5000,
            "comments": 2000,
            "shares": 800,
            "subscribes": 300,
            "ctr": 0.24,
            "avgWatchTime": 185,
            "engagementRate": 0.18
        }
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting video recommendation system...");
    
    let mut recommendation_engine = RecommendationEngine::new();
    
    recommendation_engine.add_dummy_data();
    
    let engine_data = Arc::new(Mutex::new(recommendation_engine));
    
    println!("Starting web server on port 8080...");
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
            
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(engine_data.clone()))
            .service(
                web::scope("/api")
                    .route("/recommendations", web::post().to(get_recommendations))
                    .route("/watch", web::post().to(record_watch))
                    .route("/like", web::post().to(record_like))
                    .route("/comment", web::post().to(record_comment))
                    .route("/share", web::post().to(record_share))
                    .route("/subscribe", web::post().to(record_subscribe))
                    .route("/stats", web::get().to(get_stats))
                    .route("/chart-data", web::get().to(get_chart_data))
                    .route("/simulate", web::post().to(run_simulation))
                    .route("/simulation-status", web::get().to(get_simulation_status))
                    .route("/simulation-results", web::get().to(get_simulation_results))
            )
            .service(actix_files::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}