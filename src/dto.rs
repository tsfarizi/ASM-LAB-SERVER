use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entities::{account, classroom, user};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct Judge0SubmissionRequest {
    pub source_code: String,
    pub language_id: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_output: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_time_limit: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiler_options: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_line_arguments: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccountRole {
    User,
    Admin,
}

impl AccountRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountRole::User => "user",
            AccountRole::Admin => "admin",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "user" | "USER" | "User" => Some(AccountRole::User),
            "admin" | "ADMIN" | "Admin" => Some(AccountRole::Admin),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    pub id: i32,
    pub npm: String,
    pub role: AccountRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountRequest {
    pub npm: String,
    pub role: AccountRole,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountRoleRequest {
    pub role: AccountRole,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub npm: String,
    #[serde(default)]
    pub as_admin: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginClassroomInfo {
    pub id: i32,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub programming_language: Option<String>,
    pub language_locked: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub account: AccountResponse,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classroom: Option<LoginClassroomInfo>,
    pub is_new: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminExistsResponse {
    pub exists: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub name: String,
    pub npm: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub npm: Option<String>,
    pub code: Option<String>,
}

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

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: i32,
    pub name: String,
    pub npm: String,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<user::Model> for UserResponse {
    fn from(model: user::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            npm: model.npm,
            code: model.code,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl ClassroomResponse {
    pub fn from_models(classroom: classroom::Model, users: Vec<user::Model>) -> Self {
        Self {
            id: classroom.id,
            name: classroom.name,
            programming_language: normalize_language(&classroom.programming_language),
            language_locked: classroom.language_locked,
            users: users.into_iter().map(UserResponse::from).collect(),
            created_at: classroom.created_at,
            updated_at: classroom.updated_at,
        }
    }
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

impl AccountResponse {
    pub fn from_model(model: account::Model) -> Self {
        let role = AccountRole::from_str(&model.role).unwrap_or(AccountRole::User);

        Self {
            id: model.id,
            npm: model.npm,
            role,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

fn normalize_language(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
