use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("not authenticated")]
    Unauthorized,
    #[error("not enough permissions")]
    Forbidden,
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Validation(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    /// Anything the caller can neither cause nor fix. Never rendered to the client.
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl AppError {
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Database(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
            Self::Database(_) | Self::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// A problem detail as described by RFC 9457, served as `application/problem+json`.
#[derive(Debug, Serialize)]
struct Problem {
    status: u16,
    title: String,
    detail: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();

        // Internal failures are logged in full and reported as a bare 500, so that a database
        // error can never leak a table name or a connection string to a caller.
        let detail = if status.is_server_error() {
            tracing::error!(error = ?self, "request failed");
            "Internal server error".to_owned()
        } else {
            self.to_string()
        };

        let problem = Problem {
            status: status.as_u16(),
            title: status.canonical_reason().unwrap_or("Error").to_owned(),
            detail,
        };

        let mut response = (status, Json(problem)).into_response();
        response.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("application/problem+json"),
        );
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_errors_keep_their_message() {
        let error = AppError::validation("email is not valid");
        assert_eq!(error.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(error.to_string(), "email is not valid");
    }

    #[test]
    fn a_missing_row_is_a_404_not_a_500() {
        let error = AppError::Database(sqlx::Error::RowNotFound);
        assert_eq!(error.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn unexpected_errors_are_server_errors() {
        let error = AppError::Unexpected(anyhow::anyhow!("the disk caught fire"));
        assert_eq!(error.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
