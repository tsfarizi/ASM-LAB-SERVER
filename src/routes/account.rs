use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};

use crate::{
    dto::{AccountResponse, AccountRole, CreateAccountRequest, UpdateAccountRoleRequest},
    entities::account,
    error::AppError,
    state::AppState,
};

fn validate_role(role: AccountRole) -> Result<AccountRole, AppError> {
    match role {
        AccountRole::User | AccountRole::Admin => Ok(role),
    }
}

#[utoipa::path(
    get,
    path = "/api/accounts",
    tag = "Accounts",
    responses(
        (status = 200, description = "Daftar akun", body = [AccountResponse])
    )
)]
pub async fn list_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountResponse>>, AppError> {
    let accounts = account::Entity::find()
        .all(&state.db)
        .await?
        .into_iter()
        .map(AccountResponse::from_model)
        .collect();

    Ok(Json(accounts))
}

#[utoipa::path(
    get,
    path = "/api/accounts/{id}",
    params(("id" = i32, Path, description = "ID akun")),
    tag = "Accounts",
    responses(
        (status = 200, description = "Detail akun", body = AccountResponse),
        (status = 404, description = "Akun tidak ditemukan")
    )
)]
pub async fn get_account(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<AccountResponse>, AppError> {
    let account = account::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::BadRequest("Akun tidak ditemukan".into()))?;

    Ok(Json(AccountResponse::from_model(account)))
}

#[utoipa::path(
    post,
    path = "/api/accounts",
    tag = "Accounts",
    request_body = CreateAccountRequest,
    responses(
        (status = 201, description = "Akun dibuat", body = AccountResponse),
        (status = 400, description = "Permintaan tidak valid")
    )
)]
pub async fn create_account(
    State(state): State<AppState>,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<AccountResponse>), AppError> {
    let npm = payload.npm.trim();
    if npm.is_empty() {
        return Err(AppError::BadRequest("NPM wajib diisi".into()));
    }

    let role = validate_role(payload.role)?;

    let existing = account::Entity::find()
        .filter(account::Column::Npm.eq(npm))
        .one(&state.db)
        .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest("NPM sudah terdaftar.".into()));
    }

    let now = Utc::now();
    let model = account::ActiveModel {
        npm: Set(npm.to_owned()),
        role: Set(role.as_str().to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(AccountResponse::from_model(model)),
    ))
}

#[utoipa::path(
    patch,
    path = "/api/accounts/{id}",
    params(("id" = i32, Path, description = "ID akun")),
    tag = "Accounts",
    request_body = UpdateAccountRoleRequest,
    responses(
        (status = 200, description = "Akun diperbarui", body = AccountResponse),
        (status = 404, description = "Akun tidak ditemukan")
    )
)]
pub async fn update_account_role(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateAccountRoleRequest>,
) -> Result<Json<AccountResponse>, AppError> {
    let role = validate_role(payload.role)?;

    let account_model = account::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or(AppError::BadRequest("Akun tidak ditemukan".into()))?;

    let mut active_model = account_model.into_active_model();
    active_model.role = Set(role.as_str().to_owned());
    active_model.updated_at = Set(Utc::now());

    let updated = active_model.update(&state.db).await?;

    Ok(Json(AccountResponse::from_model(updated)))
}

#[utoipa::path(
    delete,
    path = "/api/accounts/{id}",
    params(("id" = i32, Path, description = "ID akun")),
    tag = "Accounts",
    responses(
        (status = 204, description = "Akun dihapus"),
        (status = 404, description = "Akun tidak ditemukan")
    )
)]
pub async fn delete_account(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let result = account::Entity::delete_by_id(id).exec(&state.db).await?;

    if result.rows_affected == 0 {
        return Err(AppError::BadRequest("Akun tidak ditemukan".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}
