use axum::Router;
use axum::routing::{delete, get, post, put};

use crate::state::AppState;

pub mod account;
pub mod auth;
pub mod classroom;
pub mod judge;

pub fn classroom_router() -> Router<AppState> {
    Router::new()
        .route(
            "/classrooms",
            get(classroom::list_classrooms).post(classroom::create_classroom),
        )
        .route("/:id", delete(classroom::delete_classroom))
        .route("/:id/events", get(classroom::classroom_events))
        .route("/:id/finish", post(classroom::finish_exam))
        .route(
            "/classrooms/:id/users",
            get(classroom::list_classroom_users).post(classroom::add_user_to_classroom),
        )
        .route("/classrooms/:id/users/status", put(classroom::update_users_status))
        .route(
            "/classrooms/:classroom_id/users/:user_id",
            put(classroom::update_user_in_classroom).delete(classroom::delete_user_from_classroom),
        )
}

pub fn api_router() -> Router<AppState> {
    Router::new()
        .merge(classroom_router())
        .route("/judge0/submissions", post(judge::submit_code))
        .route(
            "/accounts",
            get(account::list_accounts).post(account::create_account),
        )
        .route(
            "/accounts/:id",
            get(account::get_account)
                .patch(account::update_account_role)
                .delete(account::delete_account),
        )
        .route("/auth/login", post(auth::login))
        .route("/auth/admin-exists", get(auth::admin_exists))
}
