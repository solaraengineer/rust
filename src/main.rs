use axum::{
    Router,
    routing::{get, post},
    extract::State,
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use serde::{Deserialize, Serialize};
use dotenvy::dotenv;
use std::env;

#[derive(Deserialize)]
struct CreateUser {
    username: Option<String>,
    password: Option<String>,
    Email: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    id: i32,
}

#[derive(Serialize)]
struct SuccessResponse {
    message: String,
    user_id: i32,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

        let app = Router::new()
            .route("/users", get(get_users))
            .route("/saving", post(save_user))
            .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn save_user(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {

    let username = match payload.username {
        Some(u) if !u.is_empty() => u,
        _ => return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Username is required".into() })
        ).into_response(),
    };

    let password = match payload.password {
        Some(p) if !p.is_empty() => p,
        _ => return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Password is required".into() })
        ).into_response(),
    };


    let result: (i32,) = sqlx::query_as(
        "INSERT INTO users (username, password) VALUES ($1, $2) RETURNING id"
    )
        .bind(&username)
        .bind(&password)
        .fetch_one(&pool)
        .await
        .unwrap();
    (
        StatusCode::CREATED,
        Json(SuccessResponse {
            message: "User created".into(),
            user_id: result.0
        })
    ).into_response()
}

async fn get_users(State(pool): State<PgPool>) -> String {
    let rows: Vec<(i32, String)> = sqlx::query_as("SELECT id, username FROM users")
        .fetch_all(&pool)
        .await
        .unwrap();

    format!("{:?}", rows)
}