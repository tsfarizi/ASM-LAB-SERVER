use reqwest::Client;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub http_client: Client,
    pub judge0_base_url: String,
}
