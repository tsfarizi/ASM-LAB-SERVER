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
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginClassroomInfo {
    pub id: i32,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub programming_language: Option<String>,
    pub language_locked: bool,
}

impl LoginClassroomInfo {
    pub fn from_model(classroom: classroom::Model) -> Self {
        Self {
            id: classroom.id,
            name: classroom.name,
            programming_language: normalize_language(&classroom.programming_language),
            language_locked: classroom.language_locked,
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
            tasks: Vec::new(),
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
