use std::io;

use anyhow;
use serde_json;
use storage_proofs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{}", _0)]    
    Io(#[from] io::Error),
    #[error("{}", _0)]
    Proof(#[from] storage_proofs::error::Error),
    #[error("{}", _0)]
    Strconv(#[from] std::string::FromUtf8Error),
    #[error("{}", _0)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;