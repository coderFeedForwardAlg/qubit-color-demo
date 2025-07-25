use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use futures::{Stream, StreamExt, TryStreamExt};
use minio_rsc::{provider::StaticProvider, Minio};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, prelude::FromRow, PgPool};
use std::{env, pin::Pin, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use reqwest::Client;

// --- Structs ---

use serde::Serialize;

// --- Structs ---

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct User {
    user_id: Option<uuid::Uuid>,
    username: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Video {
    video_id: Option<uuid::Uuid>,
    video_path: String,
}

#[derive(Debug, Deserialize)]
struct UserIdQuery {
    user_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
struct UserUsernameQuery {
    username: String,
}

#[derive(Debug, Deserialize)]
struct UserEmailQuery {
    email: String,
}

#[derive(Debug, Deserialize)]
struct VideoIdQuery {
    video_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
struct VideoPathQuery {
    video_path: String,
}

// --- Handlers ---

async fn health_check() -> &'static str {
    "healthy"
}

// --- User Handlers ---

async fn add_user(State(pool): State<PgPool>, Json(payload): Json<User>) -> Json<Value> {
    let query = "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING user_id, username, email";
    match sqlx::query_as::<_, User>(query)
        .bind(payload.username)
        .bind(payload.email)
        .fetch_one(&pool)
        .await
    {
        Ok(user) => Json(json!({ "status": "success", "user": user.user_id.unwrap().to_string() })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}

async fn get_users(State(pool): State<PgPool>) -> Result<Json<Value>, (StatusCode, String)> {
    let query = "SELECT user_id, username, email FROM users";
    let users = sqlx::query_as::<_, User>(query)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let users_json: Vec<Value> = users
        .into_iter()
        .map(|user| {
            json!({
                "user_id": user.user_id,
                "username": user.username,
                "email": user.email
            })
        })
        .collect();

    Ok(Json(json!({ "payload": users_json })))
}

async fn get_user_by_id(
    State(pool): State<PgPool>,
    Path(user_id): Path<uuid::Uuid>,
) -> Result<Json<User>, (StatusCode, String)> {
    let query = "SELECT user_id, username, email FROM users WHERE user_id = $1";
    sqlx::query_as::<_, User>(query)
        .bind(user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))
}

async fn get_user_by_username(
    State(pool): State<PgPool>,
    Query(params): Query<UserUsernameQuery>,
) -> Result<Json<User>, (StatusCode, String)> {
    let query = "SELECT user_id, username, email FROM users WHERE username = $1";
    sqlx::query_as::<_, User>(query)
        .bind(params.username)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))
}

async fn get_user_by_email(
    State(pool): State<PgPool>,
    Query(params): Query<UserEmailQuery>,
) -> Result<Json<User>, (StatusCode, String)> {
    let query = "SELECT user_id, username, email FROM users WHERE email = $1";
    sqlx::query_as::<_, User>(query)
        .bind(params.email)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))
}

// --- Video Handlers ---

async fn get_videos(State(pool): State<PgPool>) -> Result<Json<Value>, (StatusCode, String)> {
    let query = "SELECT video_id, video_path FROM videos";
    let videos = sqlx::query_as::<_, Video>(query)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let videos_json: Vec<Value> = videos
        .into_iter()
        .map(|video| {
            json!({
                "video_id": video.video_id,
                "video_path": video.video_path
            })
        })
        .collect();

    Ok(Json(json!({ "payload": videos_json })))
}

async fn get_video_by_id(
    State(pool): State<PgPool>,
    Query(params): Query<VideoIdQuery>,
) -> Result<Json<Video>, (StatusCode, String)> {
    let query = "SELECT video_id, video_path FROM videos WHERE video_id = $1";
    sqlx::query_as::<_, Video>(query)
        .bind(params.video_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Video not found".to_string()))
}

async fn get_video_by_path(
    State(pool): State<PgPool>,
    Query(params): Query<VideoPathQuery>,
) -> Result<Json<Video>, (StatusCode, String)> {
    let query = "SELECT video_id, video_path FROM videos WHERE video_path = $1";
    sqlx::query_as::<_, Video>(query)
        .bind(params.video_path)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Video not found".to_string()))
}

async fn upload_video(
    State(pool): State<PgPool>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, String)> {
    println!("[DEBUG] Upload video function called");

    let minio_endpoint = env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "minio:9000".to_string());
    let minio_access_key = env::var("MINIO_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string());
    let minio_secret_key = env::var("MINIO_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string());
    let minio_bucket = env::var("MINIO_BUCKET").unwrap_or_else(|_| "bucket".to_string());

    let provider = StaticProvider::new(minio_access_key, minio_secret_key, None);
    let minio = Minio::builder()
        .endpoint(&minio_endpoint)
        .provider(provider)
        .secure(false)
        .build()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name = "uploaded_file.mp4".to_string();

    while let Some(field) = multipart.next_field().await.unwrap() {
        if field.name() == Some("file") {
            file_name = field.file_name().unwrap_or(&file_name).to_string();
            let data = field.bytes().await.unwrap().to_vec();
            file_data = Some(data);
            break; // Assuming one file per upload
        }
    }

    let file_bytes = file_data.ok_or_else(|| (StatusCode::BAD_REQUEST, "No file provided".to_string()))?;

    minio
        .put_object(&minio_bucket, &file_name, file_bytes.into())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let query = "INSERT INTO videos (video_path) VALUES ($1) RETURNING video_id, video_path";
    let video = sqlx::query_as::<_, Video>(query)
        .bind(&file_name)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(json!({
        "status": true,
        "message": "File uploaded successfully",
        "video": video.video_id.unwrap().to_string()
    })))
}

async fn upload_raw_video(
    Path((bucket_name, object_name)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    body: Body,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let minio_endpoint = env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "minio:9000".to_string());
    let minio_access_key = env::var("MINIO_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string());
    let minio_secret_key = env::var("MINIO_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string());

    let client = Client::new();
    let content_length = headers
        .get(header::CONTENT_LENGTH)
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    let url = format!(
        "http://{}/{}/{}?accessKey={}&secretKey={}",
        minio_endpoint, bucket_name, object_name, minio_access_key, minio_secret_key
    );

    let stream = body.into_data_stream().map(|result| {
        result.map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send + Sync>)
    });

    let reqwest_body = reqwest::Body::wrap_stream(stream);

    let mut request_builder = client.put(&url).body(reqwest_body);

    if let Some(length) = content_length {
        request_builder = request_builder.header(reqwest::header::CONTENT_LENGTH, length);
    }

    let response = request_builder.send().await;

    match response {
        Ok(response) => {
            if response.status().is_success() {
                Ok((StatusCode::CREATED, "Object created".to_string()))
            } else {
                let status = response.status();
                let text = response.text().await.unwrap_or_else(|_| "<no response body>".to_string());
                eprintln!("MinIO upload failed with status: {} and body: {}", status, text);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("MinIO Error: {} - {}", status, text),
                ))
            }
        }
        Err(e) => {
            eprintln!("Error uploading to MinIO: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to upload file {}", e),
            ))
        }
    }
}

// --- Main Application ---

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://dbuser:p@localhost:1111/data".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(100)
        .connect(&db_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        // User routes
        .route("/users", post(add_user).get(get_users))
        .route("/users/id/:user_id", get(get_user_by_id))
        .route("/users/username", get(get_user_by_username))
        .route("/users/email", get(get_user_by_email))
        // Video routes
        .route("/videos", get(get_videos))
        .route("/videos/id", get(get_video_by_id))
        .route("/videos/path", get(get_video_by_path))
        .route("/upload-video", post(upload_video))
        .route("/upload-raw-video/:bucket_name/:object_name", post(upload_raw_video))
        // Middleware
        .layer(cors)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await?;
    println!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
