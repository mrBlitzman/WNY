use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error("settings: {0}")]
    Settings(String),
    #[error("policy: {0}")]
    Policy(String),
    #[error("engine {id}: {detail}")]
    Engine { id: String, detail: String },
    #[error("integrity: {0}")]
    Integrity(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

impl Error {
    pub fn msg(m: impl Into<String>) -> Self {
        Self::Message(m.into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
