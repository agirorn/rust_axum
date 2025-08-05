#![allow(unused)]
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;
use serde_json::json;
// use tokio_postgres::Error as TokioPostgresError;

use deadpool_postgres::PoolError;
// use deadpool::managed::errors::PoolError;
// use tokio_postgres::Error as TokioPostgresError;

use std::env;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, From)]
pub enum Error {
    // // -- String errors
    #[from]
    Message(String),
    #[from]
    TokioPostgresError(tokio_postgres::Error),
    #[from]
    PoolError(PoolError),
    // PoolError::<TokioPostgresError>,
    // #[from]
    // JsonError(serde_json::Error),
    // #[from]
    // Ed25519Pkcs8SpkiError(ed25519_dalek::pkcs8::spki::Error),
    // #[from]
    // Ed25519Pkcs8Error(ed25519_dalek::pkcs8::Error),
    // #[from]
    // Ed25519Error(ed25519_dalek::ed25519::Error),
    // #[from]
    // Base64DecodeError(base64::DecodeError),
    // #[from]
    // StringFromUtf8Error(std::string::FromUtf8Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

// Converting string into our Error by calling .into()
impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Message(value.to_string())
    }
}

/// Check if this app is running in development.
///
/// This requires that the RUST_ENV be set in production to something other than DEVELOPMENT
fn is_dev() -> bool {
    let rust_env = env::var("RUST_ENV")
        .unwrap_or("DEVELOPMENT".to_string())
        .to_uppercase();
    rust_env == "DEVELOPMENT"
}

// Implement HTTP response to make error handling easier in the Axum web-server
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("{self:#?}");
        match self {
            Error::TokioPostgresError(err) => {
                let body = if is_dev() {
                    Json(json!({
                        "error": "TokioPostgresError",
                        "message":  err.to_string(),
                    }))
                } else {
                    Json(json!({
                        "error":  "INTERNAL_SERVER_ERROR"
                    }))
                };
                (StatusCode::INTERNAL_SERVER_ERROR, body)
            }
            Error::PoolError(err) => {
                let body = if is_dev() {
                    Json(json!({
                    "error": "Tokio PoolError",
                        "message":  err.to_string(),
                    }))
                } else {
                    Json(json!({
                        "error":  "INTERNAL_SERVER_ERROR"
                    }))
                };
                (StatusCode::INTERNAL_SERVER_ERROR, body)
            }
            err => {
                // Here we could also be using the debug output
                // let body = Json(json!({
                //     "error": format!("{:?}", err)
                // }));
                // Using the Display string to presene the error
                let body = Json(json!({
                    "message": err.to_string(),
                }));

                (StatusCode::INTERNAL_SERVER_ERROR, body)
            }
        }
        .into_response()
    }
}
