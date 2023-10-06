use reqwest::Response;
use reqwest::header::ToStrError;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use thiserror::Error;
use uuid::Uuid;

use crate::mazure::aadclient::{AADClient, AADError};

pub static SERVICE_BUS_RESOURCE: &str = "https://servicebus.azure.net";

use std::sync::Arc;

#[derive(Error, Debug)]
pub enum AzureServiceBusError {
    #[error("Unable to authenticate: {0}")]
    AuthenticationError(String),

    #[error("Communication error: {0}")]
    CommunicationError(String),
    
    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("Request error: {0}")]
    RequestError(String),

    #[error("Service error: {0}")]
    ServiceError(String),
}

impl AzureServiceBusError {
    pub fn from(status: reqwest::StatusCode) -> AzureServiceBusError {
        let sc = status.as_u16();
        if sc >= 500 {
            AzureServiceBusError::ServiceError(sc.to_string())
        }
        else if sc >= 400 && sc < 500 {
            AzureServiceBusError::RequestError(sc.to_string())
        }
        else {
            AzureServiceBusError::CommunicationError(format!("Unexpected HTTP response: {}", sc))
        }
    }
}

impl From<AADError> for AzureServiceBusError {
    fn from(e: AADError) -> Self {
        AzureServiceBusError::AuthenticationError(e.to_string())
    }
}

impl From<reqwest::Error> for AzureServiceBusError {
    fn from(e: reqwest::Error) -> Self {
        AzureServiceBusError::CommunicationError(e.to_string())
    }
}

impl From<ToStrError> for AzureServiceBusError {
    fn from(e: ToStrError) -> Self {
        AzureServiceBusError::ConversionError(e.to_string())
    }
}

impl From<serde_json::Error> for AzureServiceBusError {
    fn from(e: serde_json::Error) -> Self {
        AzureServiceBusError::ConversionError(e.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrokerProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "CorrelationId")]
    correlation_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "DeliveryCount")]
    delivery_count: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "EnqueuedSequenceNumber")]
    enqueued_sequence_number: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "EnqueuedTimeUtc")]
    enqueued_time_utc: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Label")]
    label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "LockToken")]
    lock_token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "LockedUntilUtc")]
    locked_until_utc: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "MessageId")]
    message_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "SequenceNumber")]
    sequence_number: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "State")]
    state: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "TimeToLive")]
    time_to_live: Option<i64>,
}

impl BrokerProperties {
    pub fn new_empty() -> Self {
        BrokerProperties {
            correlation_id: None,
            delivery_count: None,
            enqueued_sequence_number: None,
            enqueued_time_utc: None,
            label: None,
            lock_token: None,
            locked_until_utc: None,
            message_id: None,
            sequence_number: None,
            state: None,
            time_to_live: None
        }
    }

    pub fn from_http_response(res: &Response) -> Result<BrokerProperties, AzureServiceBusError> {
        match res.headers().get("BrokerProperties") {
            None => {
                return Err(AzureServiceBusError::ConversionError("BrokerProperites header not present in response.".into()));
            }
            Some(props_text) => {
                Ok(serde_json::from_str(props_text.to_str()?)?)
            }
        }
    }

    pub fn to_json(self: &Self) -> Result<String, AzureServiceBusError> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub properties: BrokerProperties,
    pub content: Vec<u8>,
    pub content_type: String,
}

impl Message {
    pub fn json_into<T: DeserializeOwned>(self: &Self) -> Result<T, AzureServiceBusError> {
        Ok(serde_json::from_slice(&self.content)?)
    }

    pub fn new_json<T: Serialize>(body: &T) -> Result<Message, AzureServiceBusError> {
        let raw_bytes = serde_json::to_vec(body)?;
        Ok(Message {
            properties: BrokerProperties::new_empty(),
            content: raw_bytes,
            content_type: "text/json".into()
        })
    }
}

pub struct AzureServiceBusClient {
    aad_client: Arc<AADClient>,
    namespace: String,
    path: String,
}

impl AzureServiceBusClient {

    pub fn new(aad_client: Arc<AADClient>, namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            aad_client,
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    pub async fn send_json<T: Serialize>(self: &Self, body: &T) -> Result<String, AzureServiceBusError> {
        let msg = Message::new_json(body)?;
        self.send(&msg).await
    }

    pub async fn send(self: &Self, message: &Message) -> Result<String, AzureServiceBusError> {
        let url = format!(
            "https://{}.servicebus.windows.net/{}/messages",
            urlencoding::encode(self.namespace.as_str()),
            urlencoding::encode(self.path.as_str()));

        // this hacky bit is to set a correlation id to a random value if 1 wasn't specified.
        // The input will not be modified though.
        let correlation_id: String;
        let props_json: String;
        match &message.properties.correlation_id {
            Some(_correlation_id) => {
                correlation_id = _correlation_id.clone();
                props_json = message.properties.to_json()?;
            }
            None => {
                correlation_id = Uuid::new_v4().to_string();
                let mut new_props = message.properties.clone();
                new_props.correlation_id = Some(correlation_id.clone());
                props_json = new_props.to_json()?;
            }
        };

        let token = self.aad_client.get_cached_token().await?;
        let client = reqwest::Client::new();

        let res = client.post(url)
            .bearer_auth(token.token)
            .header("Content-Type", &message.content_type)
            .header("BrokerProperties", &props_json)
            .body(message.content.to_vec())
            .send()
            .await?;

        if res.status() != 201 {
            return Err(AzureServiceBusError::from(res.status()));
        }

        Ok(correlation_id)
    }

    pub async fn peek_lock(self: &Self) -> Result<Option<Message>, AzureServiceBusError> {
        let url = format!(
            "https://{}.servicebus.windows.net/{}/messages/head",
            urlencoding::encode(self.namespace.as_str()),
            urlencoding::encode(self.path.as_str()));

        let token = self.aad_client.get_cached_token().await?;
        let client = reqwest::Client::new();

        let res = client.post(url)
            .bearer_auth(token.token)
            .header("Content-Length", 0)
            .send()
            .await?;

        let status = res.status();

        if status == 201 {
            let content_type = match res.headers().get("Content-Type") {
                None => "".into(),
                Some(hv) => hv.to_str()?.into()
            };

            let properties = BrokerProperties::from_http_response(&res)?;
            let content = res.bytes().await?.to_vec();
            return Ok(Some(Message { properties, content, content_type }));
        }
        else if status == 204 {
            // No messages were found.
            return Ok(None);
        }
        else {
            return Err(AzureServiceBusError::from(status));
        }
    }

    pub async fn delete_message(self: &Self, message_properties: &BrokerProperties) -> Result<(), AzureServiceBusError> {
        let message_id = match &message_properties.message_id {
            None => Err(AzureServiceBusError::RequestError("No message id found in broker properties.".into())),
            Some(message_id) => Ok(message_id)
        }?;

        let lock_token = match &message_properties.lock_token {
            None => Err(AzureServiceBusError::RequestError("No lock token found in broker properties.".into())),
            Some(lock_token) => Ok(lock_token)
        }?;

        let url = format!(
            "https://{}.servicebus.windows.net/{}/messages/{}/{}",
            urlencoding::encode(self.namespace.as_str()),
            urlencoding::encode(self.path.as_str()),
            urlencoding::encode(message_id.as_str()),
            urlencoding::encode(lock_token));

        let token = self.aad_client.get_cached_token().await?;
        let client = reqwest::Client::new();

        let res = client.delete(url)
            .bearer_auth(token.token)
            .header("Content-Length", 0)
            .send()
            .await?;

        let status = res.status();
        if status != 200 {
            return Err(AzureServiceBusError::from(status));
        }

        Ok(())
    }
}