use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entities::{classroom, user};

use super::user::{CreateUserRequest, UserResponse};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateClassroomRequest {
    pub name: String,
    #[serde(default)]
    pub programming_language: Option<String>,
    #[serde(default)]
    pub lock_language: Option<bool>,
    #[serde(default)]
    pub users: Vec<CreateUserRequest>,
    #[serde(default)]
    pub tasks: Vec<String>,
    #[serde(default)]
    pub is_exam: Option<bool>,
    #[serde(default)]
    pub test_code: Option<String>,
    #[serde(default)]
    pub time_limit: Option<i64>,
    #[serde(default)]
    pub presetup_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClassroomRequest {
    pub name: Option<String>,
    pub programming_language: Option<String>,
    #[serde(default)]
    pub lock_language: Option<bool>,
    #[serde(default)]
    pub users: Option<Vec<CreateUserRequest>>,
    pub tasks: Option<Vec<String>>,
    #[serde(default)]
    pub is_exam: Option<bool>,
    #[serde(default)]
    pub test_code: Option<String>,
    #[serde(default)]
    pub time_limit: Option<i64>,
    #[serde(default)]
    pub presetup_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginClassroomInfo {
    pub id: i32,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub programming_language: Option<String>,
    pub language_locked: bool,
    pub is_exam: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_limit: Option<i64>,
    pub presetup_code: String,
}

impl LoginClassroomInfo {
    pub fn from_model(classroom: classroom::Model) -> Self {
        Self {
            id: classroom.id,
            name: classroom.name,
            programming_language: normalize_language(&classroom.programming_language),
            language_locked: classroom.language_locked,
            is_exam: classroom.is_exam,
            time_limit: if classroom.is_exam {
                Some(classroom.time_limit)
            } else {
                None
            },
            presetup_code: classroom.presetup_code,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClassroomResponse {
    pub id: i32,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub programming_language: Option<String>,
    pub language_locked: bool,
    pub users: Vec<UserResponse>,
    #[serde(default)]
    pub tasks: Vec<String>,
    pub is_exam: bool,
    pub test_code: String,
    pub time_limit: i64,
    pub presetup_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ClassroomResponse {
    pub fn from_models(classroom: classroom::Model, users: Vec<user::Model>) -> Self {
        Self {
            id: classroom.id,
            name: classroom.name,
            programming_language: normalize_language(&classroom.programming_language),
            language_locked: classroom.language_locked,
            users: users.into_iter().map(UserResponse::from).collect(),
            tasks: deserialize_tasks(&classroom.tasks),
            is_exam: classroom.is_exam,
            test_code: classroom.test_code,
            time_limit: classroom.time_limit,
            presetup_code: classroom.presetup_code,
            created_at: classroom.created_at,
            updated_at: classroom.updated_at,
        }
    }
}

pub(crate) fn normalize_language(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(crate) fn serialize_tasks(tasks: &[String]) -> String {
    serde_json::to_string(tasks).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn deserialize_tasks(value: &str) -> Vec<String> {
    serde_json::from_str(value).unwrap_or_default()
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FinishExamRequest {
    pub npm: String,
    pub code: String,
    pub language_id: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUsersStatusRequest {
    pub user_ids: Vec<i32>,
    pub active: bool,
}
