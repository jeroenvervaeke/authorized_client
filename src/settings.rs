use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub client_id: String,
    pub client_secret: String,
    pub token_url: String,
    pub scopes: Vec<String>,
}
