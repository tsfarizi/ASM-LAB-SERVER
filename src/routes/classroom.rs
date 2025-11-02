use axum::{
    response::sse::{Event, Sse},
    Json,
    extract::{Path, State, Query},
    http::StatusCode,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, TransactionTrait,
};
use utoipa::IntoParams;
use serde::Deserialize;
use futures_util::stream::{Stream};
use std::time::Duration;


use crate::{
    dto::{
        ClassroomResponse, CreateClassroomRequest, CreateUserRequest, UpdateClassroomRequest,
        UpdateUserRequest, UserResponse, classroom::serialize_tasks, FinishExamRequest, Judge0SubmissionRequest, Judge0SubmissionResponse, UpdateUsersStatusRequest,
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

#[derive(Deserialize)]
pub struct EventsParams {
    npm: String,
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
#[allow(dead_code)]
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
        tasks,
        is_exam,
        test_code,
        time_limit,
        presetup_code,
    } = payload;

    let programming_language = programming_language.unwrap_or_default().trim().to_string();
    let tasks = serialize_tasks(&tasks);

    let classroom_model = classroom::ActiveModel {
        name: sea_orm::ActiveValue::Set(name),
        programming_language: sea_orm::ActiveValue::Set(programming_language),
        language_locked: sea_orm::ActiveValue::Set(lock_language.unwrap_or(false)),
        tasks: sea_orm::ActiveValue::Set(tasks),
        is_exam: sea_orm::ActiveValue::Set(is_exam.unwrap_or(false)),
        test_code: sea_orm::ActiveValue::Set(test_code.unwrap_or_default()),
        time_limit: sea_orm::ActiveValue::Set(time_limit.unwrap_or(0)),
        presetup_code: sea_orm::ActiveValue::Set(presetup_code.unwrap_or_default()),
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
#[allow(dead_code)]
pub async fn update_classroom(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateClassroomRequest>,
) -> Result<Json<ClassroomResponse>, AppError> {
    let (classroom_model, _users) = load_classroom_with_users(&state, id).await?;
    let txn = state.db.begin().await?;
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
    if let Some(tasks) = payload.tasks {
        classroom_am.tasks = sea_orm::ActiveValue::Set(serialize_tasks(&tasks));
    }
    if let Some(is_exam) = payload.is_exam {
        classroom_am.is_exam = sea_orm::ActiveValue::Set(is_exam);
    }
    if let Some(test_code) = payload.test_code {
        classroom_am.test_code = sea_orm::ActiveValue::Set(test_code);
    }
    if let Some(time_limit) = payload.time_limit {
        classroom_am.time_limit = sea_orm::ActiveValue::Set(time_limit);
    }
    if let Some(presetup_code) = payload.presetup_code {
        classroom_am.presetup_code = sea_orm::ActiveValue::Set(presetup_code);
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
    if let Some(active) = payload.active {
        user_am.active = sea_orm::ActiveValue::Set(active);
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

#[utoipa::path(
    get,
    path = "/api/classrooms/{id}/events",
    params(ClassroomPath, ("npm" = String, Query, description = "User NPM")),
    tag = "Classrooms",
    responses(
        (status = 200, description = "Subscribe to classroom events"),
    )
)]
pub async fn classroom_events(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Query(params): Query<EventsParams>,
) -> Result<Sse<impl Stream<Item = Result<Event, AppError>>>, AppError> {
    let (classroom, user) = find_classroom_and_user(&state.db, id, &params.npm).await?;

    if !classroom.is_exam {
        return Err(AppError::BadRequest("Not an exam classroom".into()));
    }

    let exam_started_at = user.exam_started_at.ok_or_else(|| AppError::BadRequest("Exam not started".into()))?;
    let time_limit = Duration::from_secs(classroom.time_limit as u64 * 60);
    let end_time = exam_started_at + time_limit;

    let stream = async_stream::stream! {
        loop {
            let now = Utc::now();
            if now >= end_time {
                yield Ok(Event::default().data("timeup"));
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    };

    Ok(Sse::new(stream))
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

async fn find_classroom_and_user(db: &DatabaseConnection, classroom_id: i32, npm: &str) -> Result<(classroom::Model, user::Model), AppError> {
    let user = user::Entity::find()
        .filter(user::Column::Npm.eq(npm))
        .filter(user::Column::ClassroomId.eq(classroom_id))
        .one(db)
        .await?
        .ok_or(AppError::UserNotFound)?;

    let classroom = classroom::Entity::find_by_id(classroom_id)
        .one(db)
        .await?
        .ok_or(AppError::ClassroomNotFound)?;

    Ok((classroom, user))
}

#[utoipa::path(
    post,
    path = "/api/classrooms/{id}/finish",
    params(ClassroomPath),
    tag = "Classrooms",
    request_body = FinishExamRequest,
    responses(
        (status = 200, description = "Exam finished, code executed", body = Judge0SubmissionResponse),
        (status = 404, description = "Classroom or user not found")
    )
)]
pub async fn finish_exam(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<FinishExamRequest>,
) -> Result<Json<Judge0SubmissionResponse>, AppError> {
    let user_model = user::Entity::find()
        .filter(user::Column::ClassroomId.eq(id))
        .filter(user::Column::Npm.eq(&payload.npm))
        .one(&state.db)
        .await?
        .ok_or(AppError::UserNotFound)?;

    let mut user_am = user_model.into_active_model();
    user_am.active = sea_orm::ActiveValue::Set(false);
    user_am.code = sea_orm::ActiveValue::Set(payload.code.clone());
    user_am.update(&state.db).await?;

    let submission_payload = Judge0SubmissionRequest {
        source_code: payload.code,
        language_id: payload.language_id,
        npm: Some(payload.npm),
        stdin: None,
        expected_output: None,
        cpu_time_limit: None,
        memory_limit: None,
        compiler_options: None,
        command_line_arguments: None,
    };

    let endpoint = format!(
        "{}/submissions?base64_encoded=false&wait=true",
        state.judge0_base_url
    );

    let response = state
        .http_client
        .post(endpoint)
        .json(&submission_payload)
        .send()
        .await?;

    let status = response.status();

    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(AppError::External(format!(
            "status {} dari Judge0: {}",
            status.as_u16(),
            error_body
        )));
    }

    let result = response.json::<Judge0SubmissionResponse>().await?;
    Ok(Json(result))
}

#[utoipa::path(
    put,
    path = "/api/classrooms/{id}/users/status",
    params(ClassroomPath),
    tag = "Users",
    request_body = UpdateUsersStatusRequest,
    responses(
        (status = 204, description = "Users status updated"),
        (status = 404, description = "Classroom not found")
    )
)]
pub async fn update_users_status(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateUsersStatusRequest>,
) -> Result<StatusCode, AppError> {
    ensure_classroom_exists(&state, id).await?;

    user::Entity::update_many()
        .col_expr(user::Column::Active, payload.active.into())
        .filter(user::Column::Id.is_in(payload.user_ids))
        .filter(user::Column::ClassroomId.eq(id))
        .exec(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}