use std::num::ParseIntError;
use std::sync::PoisonError;

use async_trait::async_trait;
use reqwest::RequestBuilder;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("Unable to obtain authentication credentials: {0}")]
    AuthenticationAcquisitionError(String),

    #[error("Error converting, parsing, or formatting information for authentication: {0}")]
    ParseError(String),

    #[error("Communication error for authentication: {0}")]
    CommunicationError(String),

    #[error("General error for authentication: {0}")]
    GeneralError(String),
}

// Provide some basic conversions.

impl From<ParseIntError> for AuthenticationError {
    fn from(e: ParseIntError) -> Self {
        AuthenticationError::ParseError(e.to_string())
    }
}

impl From<reqwest::Error> for AuthenticationError {
    fn from(e: reqwest::Error) -> Self {
        AuthenticationError::CommunicationError(e.to_string())
    }
}

impl<T> From<PoisonError<T>> for AuthenticationError {
    fn from(e: PoisonError<T>) -> Self {
        AuthenticationError::GeneralError(format!("Concurrency error: {}", e))
    }
}

#[async_trait(?Send)]
pub trait ClientAuthenticator {
    async fn authenticate(&self, reqbuilder: RequestBuilder) -> Result<RequestBuilder, AuthenticationError>;
}