use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{account::AccountResponse, classroom::LoginClassroomInfo};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub npm: String,
    #[serde(default)]
    pub as_admin: bool,
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
