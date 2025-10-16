use axum::{Json, extract::State};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter,
};

use crate::{
    dto::{
        AccountResponse, AccountRole, AdminExistsResponse, LoginClassroomInfo, LoginRequest,
        LoginResponse,
    },
    entities::{account, classroom, user},
    error::AppError,
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login berhasil", body = LoginResponse),
        (status = 400, description = "Permintaan tidak valid")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let npm = payload.npm.trim();

    if npm.is_empty() {
        return Err(AppError::BadRequest("NPM wajib diisi".into()));
    }

    let existing = account::Entity::find()
        .filter(account::Column::Npm.eq(npm))
        .one(&state.db)
        .await?;

    if let Some(model) = existing {
        let classroom = find_classroom_for_npm(&state.db, npm).await?;
        return Ok(Json(LoginResponse {
            account: AccountResponse::from_model(model),
            classroom,
            is_new: false,
        }));
    }

    let admin_exists = account::Entity::find()
        .filter(account::Column::Role.eq(AccountRole::Admin.as_str()))
        .count(&state.db)
        .await?
        > 0;

    if payload.as_admin && admin_exists {
        return Err(AppError::BadRequest(
            "Admin sudah terdaftar, silakan hubungi admin yang ada.".into(),
        ));
    }

    let role = if payload.as_admin && !admin_exists {
        AccountRole::Admin
    } else {
        AccountRole::User
    };

    let now = Utc::now();
    let account = account::ActiveModel {
        npm: Set(npm.to_owned()),
        role: Set(role.as_str().to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    let classroom = find_classroom_for_npm(&state.db, npm).await?;

    Ok(Json(LoginResponse {
        account: AccountResponse::from_model(account),
        classroom,
        is_new: true,
    }))
}

#[utoipa::path(
    get,
    path = "/api/auth/admin-exists",
    tag = "Auth",
    responses(
        (status = 200, description = "Status ketersediaan admin", body = AdminExistsResponse)
    )
)]
pub async fn admin_exists(
    State(state): State<AppState>,
) -> Result<Json<AdminExistsResponse>, AppError> {
    let exists = account::Entity::find()
        .filter(account::Column::Role.eq(AccountRole::Admin.as_str()))
        .count(&state.db)
        .await?
        > 0;

    Ok(Json(AdminExistsResponse { exists }))
}

async fn find_classroom_for_npm(
    db: &DatabaseConnection,
    npm: &str,
) -> Result<Option<LoginClassroomInfo>, AppError> {
    let record = user::Entity::find()
        .filter(user::Column::Npm.eq(npm))
        .find_also_related(classroom::Entity)
        .one(db)
        .await?;

    if let Some((_user, Some(classroom_model))) = record {
        Ok(Some(LoginClassroomInfo::from_model(classroom_model)))
    } else {
        Ok(None)
    }
}
