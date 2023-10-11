use std::collections::HashMap;
use std::error::Error;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use reqwest::RequestBuilder;
use serde_json;
use serde::{Serialize, Deserialize};

use crate::mazure::client_authentication::{ClientAuthenticator, AuthenticationError};

/// A library to get AAD application tokens.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AADCredentials {
    tenant_id: String,
    client_id: String,
    secret: String,
}

impl AADCredentials {
    pub fn from_file(path: impl AsRef<Path>) -> Result<AADCredentials, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        Ok(serde_json::from_reader(reader)?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AADTokenResponse {
    token_type: String,
    expires_on: String,
    access_token: String
}

impl AADTokenResponse {
    pub fn to_token(self: &Self) -> Result<AADToken, AuthenticationError> {
        let expires = self.expires_on.parse::<u64>()?;
        Ok(AADToken { token: self.access_token.clone(), expires })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AADToken {
    pub token: String,
    pub expires: u64
}

impl AADToken {
    pub fn is_expired(self: &Self) -> bool {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(ut_duration) => ut_duration.as_secs() >= self.expires,
            Err(_e) => true
        }
    }
}

#[derive(Debug, Clone)]
pub struct AADClient {
    http_client: reqwest::Client,
    credentials: AADCredentials,
    resource: String,
    oauth_endpoint: String,
    cached_token: Arc<Mutex<Option<AADToken>>>,
}

impl AADClient {

    pub fn new(
        http_client: reqwest::Client,
        credentials: AADCredentials,
        resource: impl Into<String>,
        oauth_endpoint: Option<&str>,
    ) -> Self {

        let ep = match oauth_endpoint {
            Some(ep) => ep.into(),
            None => "https://login.microsoftonline.com".into()
        };

        Self {
            http_client,
            credentials,
            resource: resource.into(),
            oauth_endpoint: ep,
            cached_token: Arc::new(Mutex::new(Option::None)),
        }
    }

    pub async fn get_token(self: &Self) -> Result<AADToken, AuthenticationError> {
        let url = format!("{}/{}/oauth2/token", self.oauth_endpoint, self.credentials.tenant_id);
        let mut params = HashMap::new();
        params.insert("grant_type", "client_credentials");
        params.insert("client_id", self.credentials.client_id.as_str());
        params.insert("client_secret", self.credentials.secret.as_str());
        params.insert("resource", self.resource.as_str());

        let res = self.http_client
            .get(url)
            .form(&params)
            .send()
            .await?;

        if res.status() != reqwest::StatusCode::OK {
            return Err(AuthenticationError::AuthenticationAcquisitionError(res.status().to_string()).into())
        }

        let token_response: AADTokenResponse = res.json().await?;
        Ok(token_response.to_token()?)
    }

    pub async fn get_cached_token(self: &Self) -> Result<AADToken, AuthenticationError> {
        let mut guard = self.cached_token.lock()?;

        if let Some(token) = &*guard {
            if !token.is_expired() {
                return Ok(token.clone());
            }
        }

        let token = self.get_token().await?;
        *guard = Some(token.clone());

        Ok(token)
    }
}

#[async_trait(?Send)]
impl ClientAuthenticator for Arc<AADClient> {
    async fn authenticate(&self, reqbuilder: RequestBuilder) -> Result<RequestBuilder, AuthenticationError> {
        let token = self.get_cached_token();
        let t = token.await?;

        Ok(reqbuilder.bearer_auth(&t.token))
    }
}
