# Authorized Client
[![Build Status][build-img]][build-url]
[![Documentation][docs-img]][docs-url]
## About
The goal of this library is to make extremely easy to use rest endpoints which are protected by oauth 2.0 client credentials authorization.
The client is based on the `Reqwest` and `Oauth2` library

For now this library only supports endpoints which return `json` bodies.

## Usage
Add this library as a dependency to your project.
```toml
[dependencies]
authorized_client = { git = "https://github.com/jeroenvervaeke/authorized_client.git" }
```

## Example code
```rust
use authorized_client::{AuthorizedClient, Settings};
use url::Url;

// Set up the client
let settings = Settings {
    client_id: "xxxxxxxxxx".to_string(),
    client_secret: "xxxxxxxxxx".to_string(),
    token_url: "https://authorization-server.com/token".to_string(),
    scopes: vec![ "profile".to_string(), "email".to_string() ]
};

// Create a new client, this immediately tries to connect to the auth server and get a bearer token.
// If this fails your settings are probably wrong.
let client = AuthorizedClient::connect(settings).await?;

// Call your desired endpoints
let repsonse: MyResponse = client.get(Url::parse("https://protected-endpoint.com/info")?).await?;
```

[build-img]: https://github.com/jeroenvervaeke/authorized_client/actions/workflows/rust.yml/badge.svg?branch=master
[build-url]: https://github.com/jeroenvervaeke/authorized_client/actions/workflows/rust.yml
[docs-img]: https://img.shields.io/badge/Docs-up%20to%20date-success
[docs-url]: https://jeroenvervaeke.github.io/authorized_client/authorized_client/index.html

