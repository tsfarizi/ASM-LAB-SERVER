mod db;
mod dto;
mod entities;
mod error;
mod routes;
mod state;

use std::net::SocketAddr;

use anyhow::Result;
use axum::{
    Router,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
};
use reqwest::Client;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::classroom::list_classrooms,
        routes::classroom::get_classroom,
        routes::classroom::create_classroom,
        routes::classroom::update_classroom,
        routes::classroom::delete_classroom,
        routes::classroom::deactivate_users_post_exam,
        routes::classroom::list_classroom_users,
        routes::classroom::add_user_to_classroom,
        routes::classroom::update_user_in_classroom,
        routes::classroom::delete_user_from_classroom,
        routes::judge::submit_code,
        routes::account::list_accounts,
        routes::account::get_account,
        routes::account::create_account,
        routes::account::update_account_role,
        routes::account::delete_account,
        routes::auth::login,
        routes::auth::admin_exists
    ),
    components(
        schemas(
            dto::ClassroomResponse,
            dto::UserResponse,
            dto::CreateClassroomRequest,
            dto::UpdateClassroomRequest,
            dto::CreateUserRequest,
            dto::UpdateUserRequest,
            dto::Judge0SubmissionRequest,
            dto::AccountResponse,
            dto::CreateAccountRequest,
            dto::UpdateAccountRoleRequest,
            dto::AccountRole,
            dto::LoginRequest,
            dto::LoginResponse,
            dto::AdminExistsResponse
        )
    ),
    tags(
        (name = "Classrooms", description = "Manajemen entitas kelas"),
        (name = "Users", description = "Pengelolaan user di dalam kelas"),
        (name = "Executor", description = "Proxy eksekusi kode ke Judge0"),
        (name = "Accounts", description = "Manajemen akun login"),
        (name = "Auth", description = "Autentikasi pengguna")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://asm_lab.db?mode=rwc".into());

    let db = db::connect(&database_url).await?;
    db::init(&db).await?;

    let http_client = Client::builder().build()?;
    let judge0_base_url =
        std::env::var("JUDGE0_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:2358".into());

    let state = AppState {
        db,
        http_client,
        judge0_base_url,
    };

    let api_router = routes::api_router();

    let allowed_origins = AllowOrigin::list([
        HeaderValue::from_static("http://localhost:5173"),
        HeaderValue::from_static("https://tsfarizi.github.io"),
    ]);

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([ACCEPT, AUTHORIZATION, CONTENT_TYPE]);

    let app = Router::new()
        .nest("/api", api_router)
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .layer(cors)
        .with_state(state);

    let addr: SocketAddr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()
        .expect("SERVER_ADDR harus dalam format host:port");

    tracing::info!("Server running on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
