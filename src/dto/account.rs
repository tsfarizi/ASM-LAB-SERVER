use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entities::account;

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
