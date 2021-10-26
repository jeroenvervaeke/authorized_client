use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct AuthorizedClientSettings {
    pub client_id: String,
    pub client_secret: String,
    pub token_url: String,
    pub scopes: Vec<String>,
}
