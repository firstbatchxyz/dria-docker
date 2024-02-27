use actix_web::{
    error::ResponseError,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::{Display, Error};
use std::fmt;

#[derive(Debug)]
pub enum DeserializeError {
    MissingKey,
    InvalidForm,
    RocksDBConnectionError,
    RedisConnectionError,
    DNSResolverError,
    ClusterConnectionError,
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeserializeError::MissingKey => write!(f, "Key is missing in the response"),
            DeserializeError::InvalidForm => write!(f, "Value is not in the expected format"),
            DeserializeError::RocksDBConnectionError => write!(f, "Error connecting to RocksDB"),
            DeserializeError::RedisConnectionError => write!(f, "Error connecting to Redis"),
            DeserializeError::DNSResolverError => write!(f, "Error resolving DNS"),
            DeserializeError::ClusterConnectionError => {
                write!(f, "Error connecting to Cluster at init")
            }
        }
    }
}

impl std::error::Error for DeserializeError {}

#[derive(Debug, Display, Error)]
pub enum MiddlewareError {
    #[display(fmt = "internal error")]
    InternalError,

    #[display(fmt = "api key not found")]
    APIKeyError,

    #[display(fmt = "timeout")]
    Timeout,
}

impl ResponseError for MiddlewareError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            MiddlewareError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            MiddlewareError::APIKeyError => StatusCode::UNAUTHORIZED,
            MiddlewareError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}

#[derive(Debug)]
pub struct ValidationError(pub String);

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ValidationError {}
