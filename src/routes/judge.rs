use axum::{Json, extract::State};
use serde_json::Value;

use crate::{dto::Judge0SubmissionRequest, error::AppError, state::AppState};

#[utoipa::path(
    post,
    path = "/api/judge0/submissions",
    tag = "Executor",
    request_body = Judge0SubmissionRequest,
    responses(
        (status = 200, description = "Hasil eksekusi dari Judge0", body = serde_json::Value),
        (status = 502, description = "Permintaan ke Judge0 gagal"),
    )
)]
pub async fn submit_code(
    State(state): State<AppState>,
    Json(payload): Json<Judge0SubmissionRequest>,
) -> Result<Json<Value>, AppError> {
    let endpoint = format!(
        "{}/submissions?base64_encoded=false&wait=true",
        state.judge0_base_url
    );

    let response = state
        .http_client
        .post(endpoint)
        .json(&payload)
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

    let result = response.json::<Value>().await?;
    Ok(Json(result))
}
