use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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
    #[serde(default, skip_serializing)]
    #[schema(example = "51422582")]
    pub npm: Option<String>,
}
