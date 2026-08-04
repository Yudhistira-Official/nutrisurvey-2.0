use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Database(String),
    #[error("{0}")]
    Import(String),
    #[error("{0}")]
    Ai(String),
    #[error("{0}")]
    Export(String),
    #[error("{0}")]
    Io(String),
}

#[derive(Debug, Serialize)]
struct ErrorPayload {
    kind: &'static str,
    message: String,
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let kind = match self {
            Self::Validation(_) => "Validation",
            Self::Database(_) => "Database",
            Self::Import(_) => "Import",
            Self::Ai(_) => "Ai",
            Self::Export(_) => "Export",
            Self::Io(_) => "Io",
        };
        ErrorPayload {
            kind,
            message: self.to_string(),
        }
        .serialize(serializer)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}
