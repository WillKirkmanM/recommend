use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::RecommendationEngine;

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
struct InteractionRequest {
    user_id: String,
    video_id: String,
    interaction_type: String,
    data: Option<serde_json::Value>,
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
    
use std::collections::HashMap;

    let mut response = HashMap::new();
    response.insert("status", "success");
    HttpResponse::Ok().json(response)
}

async fn record_like(
    data: web::Data<Arc<Mutex<RecommendationEngine>>>,
    req: web::Json<LikeRequest>,
) -> impl Responder {
    let mut engine = data.lock().unwrap();
    
    engine.process_like(&req.user_id, &req.video_id, req.is_like);

    use std::collections::HashMap;
    let mut response = HashMap::new();
    response.insert("status", "success".to_string());
    HttpResponse::Ok().json(response)
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

async fn record_interaction(
    _data: web::Data<Arc<Mutex<RecommendationEngine>>>,
    _req: web::Json<InteractionRequest>,
) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "success"}))
}

pub async fn run_server(engine: Arc<Mutex<RecommendationEngine>>) -> std::io::Result<()> {
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
            
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(engine.clone()))
            .service(
                web::scope("/api")
                    .route("/recommendations", web::post().to(get_recommendations))
                    .route("/watch", web::post().to(record_watch))
                    .route("/like", web::post().to(record_like))
                    .route("/comment", web::post().to(record_comment))
                    .route("/share", web::post().to(record_share))
                    .route("/subscribe", web::post().to(record_subscribe))
                    .route("/interaction", web::post().to(record_interaction))
            )
            .service(actix_files::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}