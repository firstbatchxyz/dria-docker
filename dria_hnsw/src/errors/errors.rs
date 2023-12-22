use std::fmt;
use std::fmt::{Formatter};
use actix_web::{
    http::{header::ContentType, StatusCode},
    App, HttpResponse, error::ResponseError
};
use derive_more::{Display, Error};

// Define an enum for your custom error types
#[derive(Debug)]
pub enum DeserializeError {
    MissingKey,
    InvalidForm,
    RedisConnectionError,
    DNSResolverError,
    ClusterConnectionError
}


// Implementing std::fmt::Display for DeserializeError
impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeserializeError::MissingKey => write!(f, "Key is missing in the response"),
            DeserializeError::InvalidForm => write!(f, "Value is not in the expected format"),
            DeserializeError::RedisConnectionError => write!(f, "Error connecting to Redis"), // Add this line
            DeserializeError::DNSResolverError => write!(f, "Error resolving DNS"), // Add this line
            DeserializeError::ClusterConnectionError => write!(f, "Error connecting to Cluster at init"), // Add this line
        }
    }
}

// Implementing std::error::Error for DeserializeError
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
