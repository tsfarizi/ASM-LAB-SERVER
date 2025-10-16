use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, TransactionTrait,
};
use utoipa::IntoParams;

use crate::{
    dto::{
        ClassroomResponse, CreateClassroomRequest, CreateUserRequest, UpdateClassroomRequest,
        UpdateUserRequest, UserResponse,
    },
    entities::{classroom, user},
    error::AppError,
    state::AppState,
};

#[allow(dead_code)]
#[derive(Debug, IntoParams)]
pub struct ClassroomPath {
    pub id: i32,
}

#[allow(dead_code)]
#[derive(Debug, IntoParams)]
pub struct ClassroomUserPath {
    pub classroom_id: i32,
    pub user_id: i32,
}

#[utoipa::path(
    get,
    path = "/api/classrooms",
    tag = "Classrooms",
    responses(
        (status = 200, description = "List all classrooms", body = [ClassroomResponse])
    )
)]
pub async fn list_classrooms(
    State(state): State<AppState>,
) -> Result<Json<Vec<ClassroomResponse>>, AppError> {
    let data = classroom::Entity::find()
        .order_by_asc(classroom::Column::Id)
        .find_with_related(user::Entity)
        .all(&state.db)
        .await?;

    let payload = data
        .into_iter()
        .map(|(classroom, users)| ClassroomResponse::from_models(classroom, users))
        .collect();

    Ok(Json(payload))
}

#[utoipa::path(
    get,
    path = "/api/classrooms/{id}",
    params(ClassroomPath),
    tag = "Classrooms",
    responses(
        (status = 200, description = "Get classroom by id", body = ClassroomResponse),
        (status = 404, description = "Classroom not found")
    )
)]
pub async fn get_classroom(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ClassroomResponse>, AppError> {
    let (classroom, users) = load_classroom_with_users(&state, id).await?;
    Ok(Json(ClassroomResponse::from_models(classroom, users)))
}

#[utoipa::path(
    post,
    path = "/api/classrooms",
    tag = "Classrooms",
    request_body = CreateClassroomRequest,
    responses(
        (status = 201, description = "Classroom created", body = ClassroomResponse),
        (status = 400, description = "Invalid payload")
    )
)]
pub async fn create_classroom(
    State(state): State<AppState>,
    Json(payload): Json<CreateClassroomRequest>,
) -> Result<(StatusCode, Json<ClassroomResponse>), AppError> {
    let txn = state.db.begin().await?;
    let now = Utc::now();

    let CreateClassroomRequest {
        name,
        programming_language,
        lock_language,
        users,
    } = payload;

    let programming_language = programming_language.unwrap_or_default().trim().to_string();

    let classroom_model = classroom::ActiveModel {
        name: sea_orm::ActiveValue::Set(name),
        programming_language: sea_orm::ActiveValue::Set(programming_language),
        language_locked: sea_orm::ActiveValue::Set(lock_language.unwrap_or(false)),
        created_at: sea_orm::ActiveValue::Set(now),
        updated_at: sea_orm::ActiveValue::Set(now),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    insert_users(&txn, classroom_model.id, users).await?;
    txn.commit().await?;

    let response = load_classroom_with_users(&state, classroom_model.id).await?;
    Ok((
        StatusCode::CREATED,
        Json(ClassroomResponse::from_models(response.0, response.1)),
    ))
}

#[utoipa::path(
    put,
    path = "/api/classrooms/{id}",
    params(ClassroomPath),
    tag = "Classrooms",
    request_body = UpdateClassroomRequest,
    responses(
        (status = 200, description = "Classroom updated", body = ClassroomResponse),
        (status = 404, description = "Classroom not found")
    )
)]
pub async fn update_classroom(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateClassroomRequest>,
) -> Result<Json<ClassroomResponse>, AppError> {
    let txn = state.db.begin().await?;

    let (classroom_model, _users) = load_classroom_with_users(&state, id).await?;
    let mut classroom_am: classroom::ActiveModel = classroom_model.into_active_model();

    if let Some(name) = payload.name {
        classroom_am.name = sea_orm::ActiveValue::Set(name);
    }
    if let Some(programming_language) = payload.programming_language {
        let programming_language = programming_language.trim().to_string();
        classroom_am.programming_language = sea_orm::ActiveValue::Set(programming_language);
    }
    if let Some(lock_language) = payload.lock_language {
        classroom_am.language_locked = sea_orm::ActiveValue::Set(lock_language);
    }
    classroom_am.updated_at = sea_orm::ActiveValue::Set(Utc::now());

    let updated_classroom = classroom_am.update(&txn).await?;

    if let Some(users) = payload.users {
        user::Entity::delete_many()
            .filter(user::Column::ClassroomId.eq(id))
            .exec(&txn)
            .await?;
        insert_users(&txn, id, users).await?;
    }

    txn.commit().await?;

    let response = load_classroom_with_users(&state, updated_classroom.id).await?;

    Ok(Json(ClassroomResponse::from_models(response.0, response.1)))
}

#[utoipa::path(
    delete,
    path = "/api/classrooms/{id}",
    params(ClassroomPath),
    tag = "Classrooms",
    responses(
        (status = 204, description = "Classroom deleted"),
        (status = 404, description = "Classroom not found")
    )
)]
pub async fn delete_classroom(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let result = classroom::Entity::delete_by_id(id).exec(&state.db).await?;

    if result.rows_affected == 0 {
        return Err(AppError::ClassroomNotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/classrooms/{id}/users",
    params(ClassroomPath),
    tag = "Users",
    responses(
        (status = 200, description = "List users for classroom", body = [UserResponse]),
        (status = 404, description = "Classroom not found")
    )
)]
pub async fn list_classroom_users(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    ensure_classroom_exists(&state, id).await?;

    let users = user::Entity::find()
        .filter(user::Column::ClassroomId.eq(id))
        .order_by_asc(user::Column::Id)
        .all(&state.db)
        .await?;

    Ok(Json(users.into_iter().map(UserResponse::from).collect()))
}

#[utoipa::path(
    post,
    path = "/api/classrooms/{id}/users",
    params(ClassroomPath),
    tag = "Users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User added to classroom", body = UserResponse),
        (status = 404, description = "Classroom not found")
    )
)]
pub async fn add_user_to_classroom(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    ensure_classroom_exists(&state, id).await?;

    let now = Utc::now();
    let user_model = user::ActiveModel {
        classroom_id: sea_orm::ActiveValue::Set(id),
        name: sea_orm::ActiveValue::Set(payload.name),
        npm: sea_orm::ActiveValue::Set(payload.npm),
        code: sea_orm::ActiveValue::Set(payload.code),
        created_at: sea_orm::ActiveValue::Set(now),
        updated_at: sea_orm::ActiveValue::Set(now),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(UserResponse::from(user_model))))
}

#[utoipa::path(
    put,
    path = "/api/classrooms/{classroom_id}/users/{user_id}",
    params(ClassroomUserPath),
    tag = "Users",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = UserResponse),
        (status = 404, description = "Classroom or user not found")
    )
)]
pub async fn update_user_in_classroom(
    State(state): State<AppState>,
    Path((classroom_id, user_id)): Path<(i32, i32)>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    ensure_classroom_exists(&state, classroom_id).await?;

    let user_model = user::Entity::find_by_id(user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::UserNotFound)?;

    if user_model.classroom_id != classroom_id {
        return Err(AppError::UserNotFound);
    }

    let mut user_am = user_model.into_active_model();
    if let Some(name) = payload.name {
        user_am.name = sea_orm::ActiveValue::Set(name);
    }
    if let Some(npm) = payload.npm {
        user_am.npm = sea_orm::ActiveValue::Set(npm);
    }
    if let Some(code) = payload.code {
        user_am.code = sea_orm::ActiveValue::Set(code);
    }
    user_am.updated_at = sea_orm::ActiveValue::Set(Utc::now());

    let updated_user = user_am.update(&state.db).await?;

    Ok(Json(UserResponse::from(updated_user)))
}

#[utoipa::path(
    delete,
    path = "/api/classrooms/{classroom_id}/users/{user_id}",
    params(ClassroomUserPath),
    tag = "Users",
    responses(
        (status = 204, description = "User deleted"),
        (status = 404, description = "Classroom or user not found")
    )
)]
pub async fn delete_user_from_classroom(
    State(state): State<AppState>,
    Path((classroom_id, user_id)): Path<(i32, i32)>,
) -> Result<StatusCode, AppError> {
    ensure_classroom_exists(&state, classroom_id).await?;

    let user_model = user::Entity::find_by_id(user_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::UserNotFound)?;

    if user_model.classroom_id != classroom_id {
        return Err(AppError::UserNotFound);
    }

    let result = user::Entity::delete_by_id(user_id).exec(&state.db).await?;
    if result.rows_affected == 0 {
        return Err(AppError::UserNotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn ensure_classroom_exists(state: &AppState, id: i32) -> Result<(), AppError> {
    let exists = classroom::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .is_some();
    if exists {
        Ok(())
    } else {
        Err(AppError::ClassroomNotFound)
    }
}

async fn load_classroom_with_users(
    state: &AppState,
    id: i32,
) -> Result<(classroom::Model, Vec<user::Model>), AppError> {
    let result = classroom::Entity::find_by_id(id)
        .find_with_related(user::Entity)
        .all(&state.db)
        .await?;

    match result.into_iter().next() {
        Some(data) => Ok(data),
        None => Err(AppError::ClassroomNotFound),
    }
}

async fn insert_users(
    txn: &DatabaseTransaction,
    classroom_id: i32,
    users: Vec<CreateUserRequest>,
) -> Result<(), AppError> {
    if users.is_empty() {
        return Ok(());
    }

    for payload in users.into_iter().filter(|user| !user.npm.trim().is_empty()) {
        let now = Utc::now();
        user::ActiveModel {
            classroom_id: sea_orm::ActiveValue::Set(classroom_id),
            name: sea_orm::ActiveValue::Set(payload.name),
            npm: sea_orm::ActiveValue::Set(payload.npm),
            code: sea_orm::ActiveValue::Set(payload.code),
            created_at: sea_orm::ActiveValue::Set(now),
            updated_at: sea_orm::ActiveValue::Set(now),
            ..Default::default()
        }
        .insert(txn)
        .await?;
    }

    Ok(())
}
