use crate::settings::Settings;
use anyhow::{bail, Context, Result};
use log::{debug, trace};
use oauth2::basic::BasicClient;
use oauth2::http::StatusCode;
use oauth2::reqwest::async_http_client;
use oauth2::{AuthUrl, ClientId, ClientSecret, Scope, TokenResponse, TokenUrl};
use reqwest::{Client, Method, Request};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{RwLock, RwLockWriteGuard};
use tokio::time::{sleep, Duration};
use url::Url;

#[derive(Clone)]
pub struct AuthorizedClient {
    credentials: Arc<RwLock<Credentials>>,
    http_client: Client,
    settings: Settings,
}

const MAX_RETRY_COUNT: u8 = 3;

impl AuthorizedClient {
    pub async fn connect(settings: Settings) -> Result<Self> {
        // Create the underlying http client, will be reused for every call
        let http_client = Client::new();

        trace!("Initial connect to '{}'", settings.token_url);
        // Fetch the bearer token for the first time
        let credentials = Arc::new(RwLock::new(Self::get_bearer_token(&settings).await?));
        trace!(
            "Successfully connected: Got bearer token from {}",
            settings.token_url
        );

        Ok(AuthorizedClient {
            credentials,
            http_client,
            settings,
        })
    }

    async fn get_bearer_token(settings: &Settings) -> Result<Credentials> {
        trace!("Preparing client credentials exchange");
        // Create a new oauth "client"
        let oauth_client = BasicClient::new(
            ClientId::new(settings.client_id.clone()),
            Some(ClientSecret::new(settings.client_secret.clone())),
            AuthUrl::new("http://unused".to_string())?,
            Some(TokenUrl::new(settings.token_url.clone())?),
        );

        // Build a client credentials request
        let mut exchange_request = oauth_client.exchange_client_credentials();

        // Add the requested scopes to the request
        for scope in settings.scopes.iter().cloned() {
            exchange_request = exchange_request.add_scope(Scope::new(scope));
        }

        // Exchange the client_id and client_secret for a bearer token
        let response = exchange_request.request_async(async_http_client).await?;

        trace!(
            "Successfully exchanged client_id and client_secret for a bearer token: {:?}",
            response
        );

        // Extract the required data
        let expires_at = Instant::now()
            .checked_add(
                response
                    .expires_in()
                    .context("Expires in is missing in token response")?,
            )
            .context("Duration was so long it caused an overflow")?;
        let access_token = response.access_token().secret().to_owned();

        // Return the fetched credentials
        Ok(Credentials {
            access_token,
            expires_at,
        })
    }

    pub async fn get<R>(&self, url: Url) -> Result<R>
    where
        R: for<'de> Deserialize<'de>,
    {
        self.request_json(|| Request::new(Method::GET, url.clone()))
            .await
    }

    async fn ensure_authenticated(&self) -> Result<()> {
        // Verify that the credentials are not expired yet
        // read lock: This will block until the write lock (if present) is released
        if self.credentials.read().await.expires_at < Instant::now() {
            trace!("Credentials appear to be expired, preparing to double check in a upgradable read lock and refresh if required");

            // Acquire a write lock, only one write lock can access the data at once
            let write_lock = self.credentials.write().await;

            // We make sure no other write lock has updated the credentials in the time we were waiting to acquire the write lock
            if write_lock.expires_at < Instant::now() {
                debug!("Credentials are expired, refreshing the authentication");
                self.refresh_authentication(write_lock).await?;
            }
        }

        Ok(())
    }

    async fn force_refresh_authentication(&self) -> Result<()> {
        trace!("Force refreshing bearer token");
        let write_lock = self.credentials.write().await;
        self.refresh_authentication(write_lock).await
    }

    async fn refresh_authentication(
        &self,
        mut write_lock: RwLockWriteGuard<'_, Credentials>,
    ) -> Result<()> {
        debug!("Refreshing bearer token");
        let result = Self::get_bearer_token(&self.settings).await?;

        write_lock.expires_at = result.expires_at;
        write_lock.access_token = result.access_token;

        debug!("Refreshed bearer token");
        Ok(())
    }

    async fn request_json<R>(&self, request_builder: impl Fn() -> Request) -> Result<R>
    where
        R: for<'de> Deserialize<'de>,
    {
        // Ensure we don't attempt to make a request with an expired access token
        self.ensure_authenticated().await?;

        // Number of times we received unauthorized for a certain request
        // When we reach MAX_RETRY_COUNT we stop trying
        let mut unauthorized_retries = 0;

        loop {
            // Build the request
            let mut request = request_builder();

            // Add the bearer token to the request headers
            let headers = request.headers_mut();
            headers.insert(
                "Authorization",
                format!("Bearer {}", self.credentials.read().await.access_token).parse()?,
            );

            // Execute the request
            let response = self.http_client.execute(request).await?;

            // When the server returns 200: return the deserialized json
            // When the server returns 401: refresh authentication and retry
            // In other cases, throw an error
            match response.status() {
                StatusCode::OK => return Ok(response.json().await?),
                StatusCode::UNAUTHORIZED => {
                    // When we reached the maximum amount of retries: bail
                    if unauthorized_retries == MAX_RETRY_COUNT {
                        bail!(format!(
                            "Failed to authenticate, retries = {} ",
                            MAX_RETRY_COUNT
                        ))
                    }

                    // Increase the retry counter
                    unauthorized_retries += 1;
                    trace!("Unauthorized retry: {}", unauthorized_retries);

                    // If we have already retried once add some sleep time in between retries, we don't want to DDOS the oauth server
                    if unauthorized_retries > 1 {
                        let sleep_duration = 500 * unauthorized_retries as u64;
                        trace!("Sleeping for {}ms before retrying", sleep_duration);
                        sleep(Duration::from_millis(sleep_duration)).await;
                    }

                    // Refresh the bearer token
                    self.force_refresh_authentication().await?;
                }
                status_code => {
                    bail!("Unsupported status code (CODE={})", status_code.as_u16())
                }
            }
        }
    }
}

#[derive(Clone)]
struct Credentials {
    access_token: String,
    expires_at: Instant,
}
