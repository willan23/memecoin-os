use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("token not found: {0}")]
    TokenNotFound(String),
    #[error("invalid token definition: {0}")]
    InvalidDefinition(String),
    #[error("provider error ({provider}): {message}")]
    Provider { provider: String, message: String },
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("http: {0}")]
    Http(String),
    #[error("policy denied: {0}")]
    PolicyDenied(String),
}

pub type Result<T> = std::result::Result<T, Error>;
