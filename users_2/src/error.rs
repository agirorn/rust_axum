#![allow(unused)]
use derive_more::From;
pub type Result<T> = core::result::Result<T, Error>;
// use deadpool_postgres::PoolError;
use serde_json::json;
use std::env;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, From)]
pub enum Error {
    #[from]
    Message(String),
    #[from]
    BcryptError(bcrypt::BcryptError),
    // #[from]
    // PoolError(deadpool_postgres::PoolError),
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
