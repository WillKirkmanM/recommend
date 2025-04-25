<p align="center">
  <img src="https://avatars.githubusercontent.com/u/138057124?s=200&v=4" width="150" />
</p>
<h1 align="center">Recommendation Algorithm</h1>

<h4 align="center">
    <a href="#overview">Overview</a> |
    <a href="#architecture">Architecture</a> |
    <a href="#recommendation-algorithms">Recommendation Algorithms</a> |
    <a href="#data-structures">Data Structures</a> |
    <a href="#system-workflow">System Workflow</a> |
    <a href="#api-reference">API Reference</a> |
    <a href="#setup--usage">Setup & Usage</a>
</h4>

<p align="center">A Sophisticated Recommendation Algorithm, Combining Collaborative Filtering, Content-Based Analysis, Temporal Dynamics, Engagement Metrics & Popularity Trends.</p>

<p align="center">
  <img src="https://github.com/user-attachments/assets/042bb95e-93cc-45bc-9ded-09cf7374656d" />
</p>

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Recommendation Algorithms](#recommendation-algorithms)
    - [Collaborative Filtering](#collaborative-filtering)
    - [Content-Based Filtering](#content-based-filtering)
    - [Popularity-Based Recommendations](#popularity-based-recommendations)
    - [Temporal Recommendations](#temporal-recommendations)
    - [Engagement-Based Recommendations](#engagement-based-recommendations)
- [Data Structures](#data-structures)
- [System Workflow](#system-workflow)
- [API Reference](#api-reference)
- [Setup & Usage](#setup--usage)

## Overview

This recommendation system uses a hybrid approach combining five different recommendation algorithms to provide personalized video suggestions. The system processes user interactions (views, likes, comments, shares) to continuously refine its recommendations.

## Architecture

The system uses many modular recommendation algorithms that work in tandem. Recommendations from each algorithm are weighted and combined to produce final recommendations that balance content discovery, personalisation, and engagement maximisation.

```
┌─────────────────────────┐
│  Recommendation Engine  │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│     Hybrid Ranking      │
│                         │
│  ┌─────────┐ ┌────────┐ │
│  │  35%    │ │  25%   │ │
│  │ Collab  │ │Content │ │
│  │ Filter  │ │ Based  │ │
│  └─────────┘ └────────┘ │
│  ┌─────────┐ ┌────────┐ │
│  │  15%    │ │  10%   │ │
│  │Popular  │ │Temporal│ │
│  │ Based   │ │ Based  │ │
│  └─────────┘ └────────┘ │
│  ┌─────────┐            │
│  │  15%    │            │
│  │Engage-  │            │
│  │  ment   │            │
│  └─────────┘            │
└─────────────────────────┘
```

## Recommendation Algorithms

### Collaborative Filtering

Identifies similar users and recommends videos they enjoyed but the current user hasn't watched yet.

```rust
fn collaborative_filtering_recommendations(
    user_id: &str,
    count: usize, 
    user_video_matrix: &Arc<Mutex<HashMap<String, HashMap<String, f64>>>>,
    users: &Arc<Mutex<HashMap<String, User>>>
) -> Vec<(String, f64)> {
    // Find users with similar preferences
    // Recommend videos they've rated highly
}
```

**Key Metrics:**

| Metric | Description |
|--------|-------------|
| User Similarity | Cosine similarity between user rating vectors |
| Confidence Score | How strongly the algorithm believes in a recommendation |
| User Rating | Implicit/explicit rating derived from watch time, likes, etc. |

### Content-Based Filtering

Analyses video attributes (categories, tags) and recommends similar content to what the user has previously enjoyed.

```rust
fn content_based_recommendations(
    user_id: &str,
    count: usize,
    users: &Arc<Mutex<HashMap<String, User>>>,
    videos: &Arc<Mutex<HashMap<String, Video>>>
) -> Vec<(String, f64)> {
    // Match user preferences to video attributes
    // Score videos based on category and tag matches
}
```

**Content Feature Analysis:**

| Feature | Weight |
|---------|--------|
| Category Match | High |
| Tag Match | Medium |
| Recency | Low |
| Channel | Medium |

### Popularity-Based Recommendations

Identifies trending or broadly popular videos across the platform.

```rust
fn popularity_based_recommendations(
    count: usize,
    videos: &Arc<Mutex<HashMap<String, Video>>>
) -> Vec<(String, f64)> {
    // Calculate popularity scores
    // Apply time decay to favor recent content
}
```

**Popularity Score Formula:**

```
score = (log10(views) * 0.6 + like_ratio * 0.4) * recency_factor
```

Where:
- `views` = Total video views
- `like_ratio` = likes/views
- `recency_factor` = 1.0 + min(3.0, (30.0/days_old))

### Temporal Recommendations

Prioritises fresh content and considers time-of-day viewing patterns.

```rust
fn get_temporal_recommendations(
    user_id: &str,
    count: usize,
    users: &Arc<Mutex<HashMap<String, User>>>,
    videos: &Arc<Mutex<HashMap<String, Video>>>
) -> Vec<(String, f64)> {
    // Favor recent videos from subscribed channels
    // Match user's time-of-day preferences
    // Boost trending videos
}
```

**Temporal Scoring Factors:**

| Factor | Description | Score Boost |
|--------|-------------|-------------|
| Recent Upload (<1 day) | Very fresh content from subscribed channels | +5.0 |
| Recent Upload (1-3 days) | Fresh content from subscribed channels | +3.0 |
| Recent Upload (3-7 days) | Relatively fresh content | +2.0 |
| Time-of-Day Match | Content matches user's usual viewing time | +0.0-1.0 |
| Trending | Sudden popularity increase | +1.5 |

### Engagement-Based Recommendations

Matches videos to the user's typical engagement patterns.

```rust
fn get_engagement_recommendations(
    user_id: &str,
    count: usize,
    users: &Arc<Mutex<HashMap<String, User>>>,
    videos: &Arc<Mutex<HashMap<String, Video>>>
) -> Vec<(String, f64)> {
    // Match videos to user's engagement habits
    // (watch time, comment frequency, etc.)
}
```

**Engagement Pattern Matching:**

| Pattern | Description | Score Impact |
|---------|-------------|--------------|
| Watch Time Compatibility | Videos with similar watch percentage | +0.0-2.0 |
| Like Ratio Similarity | Similar like-to-view ratios | +0.0-1.5 |
| Comment Engagement | Match comment activity to user preferences | +0.0-1.0 |
| Rewatchability | Higher for users who rewatch content | +0.0-1.2 |
| High Completion | Videos with high completion rates | +0.0-0.8 |
| Shareability | Match sharing patterns | +0.0-1.0 |

## Data Structures

The system uses several core data structures to model users, videos, and interactions:

```rust
pub struct User {
    id: String,
    subscriptions: HashSet<String>,  // Channel IDs
    watch_history: Vec<WatchEvent>,
    content_preferences: HashMap<String, f64>,  // Category -> preference score
    interaction_patterns: InteractionPatterns,
}

pub struct Video {
    id: String,
    title: String,
    channel_id: String,
    duration: Duration,
    categories: Vec<String>,
    tags: Vec<String>,
    upload_date: DateTime<Utc>,
    metrics: VideoMetrics,
    embedding: Vec<f32>,  // Content embedding vector
}

pub struct VideoMetrics {
    views: u64,
    likes: u64,
    dislikes: u64,
    share_count: u64,
    comment_count: u64,
    avg_watch_time: Duration,
    avg_watch_percentage: f64,
    completion_rate: f64,
    retention_curve: Vec<(f32, f32)>, // (percentage through video, percentage of viewers remaining)
    rewatch_rate: f64,
}
```

## System Workflow

1. **Data Collection**: The system collects user interactions (views, likes, comments, shares).
2. **Preference Modeling**: Interactions are processed to build user preference models.
3. **Multi-algorithm Recommendations**: Each algorithm generates candidate videos.
4. **Hybrid Ranking**: Candidates are scored, weighted, and combined.
5. **Continuous Learning**: User feedback updates preferences for future recommendations.

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  User    │    │Preference│    │Algorithm │    │  Hybrid  │    │ Feedback │
│Interact. │───>│ Modeling │───>│ Scoring  │───>│ Ranking  │───>│          |
└──────────┘    └──────────┘    └──────────┘    └──────────┘    └──────────┘
```

## API Reference

The system provides a RESTful API for frontend integration:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/recommendations` | POST | Get personalized video recommendations |
| `/api/watch` | POST | Record a video view event |
| `/api/like` | POST | Record a like/dislike event |
| `/api/comment` | POST | Record a comment event |
| `/api/share` | POST | Record a share event |
| `/api/subscribe` | POST | Record a channel subscription |
| `/api/stats` | GET | Get system statistics |
| `/api/chart-data` | GET | Get visualisation data |

## Setup & Usage

### Prerequisites

- Rust 1.70+ with Cargo
- Actix-web framework

### Installation

```bash
# Clone the repository
git clone https://github.com/WillKirkmanM/recommend
cd recommend

# Build the project
cargo build --release

# Run the application
cargo run --release
```

### Usage

```rust
// Create a recommendation engine
let mut engine = RecommendationEngine::new();

// Add data (or connect to database)
engine.add_dummy_data();

// Get recommendations for a user
let recommendations = engine.recommend_videos("user1", 10);

// Process interactions
engine.process_watch("user1", "video1", Duration::from_secs(300));
engine.process_like("user1", "video1", true);
```
