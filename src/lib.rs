//! # Authorized Client
//! The goal of this library is to make extremely easy to use rest endpoints which are protected by oauth 2.0 client credentials authorization.
//! The client is based on the `Reqwest` and `Oauth2` library
//!
//! For now this library only supports endpoints which return `json` bodies.
//!
//! ## Usage
//! Add this library as a dependency to your project.
//! ```toml
//! [dependencies]
//! authorized_client = { git = "ssh://git@github.com/jeroenvervaeke/authorized_client.git" }
//! ```
//!
//! ## Example code
//! ```no_run
//!# async fn doc_test() -> anyhow::Result<()> {
//!# use serde::Deserialize;
//!# #[derive(Deserialize)]
//!# struct MyResponse {}
//! use authorized_client::{AuthorizedClient, Settings};
//! use url::Url;
//!
//! // Set up the client
//! let settings = Settings {
//!     client_id: "xxxxxxxxxx".to_string(),
//!     client_secret: "xxxxxxxxxx".to_string(),
//!     token_url: "https://authorization-server.com/token".to_string(),
//!     scopes: vec![ "profile".to_string(), "email".to_string() ]
//! };
//!
//! // Create a new client, this immediately tries to connect to the auth server and get a bearer token.
//! // If this fails your settings are probably wrong.
//! let client = AuthorizedClient::connect(settings).await?;
//!
//! let repsonse: MyResponse = client.get(Url::parse("https://protected-endpoint.com/info")?).await?;
//!
//!# Ok(())
//!# }
//! ```
mod authorized_client;
mod settings;

pub use crate::authorized_client::AuthorizedClient;
pub use crate::settings::Settings;
