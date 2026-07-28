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
    /// A feature the caller asked for that this deployment has not been configured with, such as
    /// uploads without a bucket. A 501 rather than a 500: nothing is broken, it is simply absent.
    #[error("{0}")]
    NotConfigured(String),
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

    pub fn not_configured(message: impl Into<String>) -> Self {
        Self::NotConfigured(message.into())
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::NotConfigured(_) => StatusCode::NOT_IMPLEMENTED,
            Self::Database(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
            Self::Database(_) | Self::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Turns a unique-constraint violation into a 409 carrying a message the caller can act on,
/// instead of a 500 that says nothing. Racing two signups for the same address is a normal
/// thing for clients to do, not a server fault.
pub trait OnUniqueViolation<T> {
    fn on_unique_violation(self, message: &str) -> AppResult<T>;
}

impl<T> OnUniqueViolation<T> for AppResult<T> {
    fn on_unique_violation(self, message: &str) -> AppResult<T> {
        self.map_err(|error| match &error {
            AppError::Database(sqlx::Error::Database(db)) if db.is_unique_violation() => {
                AppError::conflict(message)
            }
            _ => error,
        })
    }
}

/// A problem detail as described by RFC 9457, served as `application/problem+json`. Public
/// so that the OpenAPI document can describe every failure with the shape it really has.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(title = "Problem")]
pub struct Problem {
    #[schema(example = 404)]
    pub status: u16,
    #[schema(example = "Not Found")]
    pub title: String,
    /// A human-readable explanation. Server faults always read "Internal server error", since
    /// their detail belongs in the logs rather than in a response.
    #[schema(example = "not found")]
    pub detail: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();

        // Internal failures are logged in full and reported as a bare 500, so that a database
        // error can never leak a table name or a connection string to a caller. A missing
        // feature is the exception: it is a 5xx by status but there is nothing to hide, and
        // "Internal server error" would send someone looking for a fault that does not exist.
        let detail = if matches!(self, Self::NotConfigured(_)) {
            self.to_string()
        } else if status.is_server_error() {
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

    /// A 500 must say nothing, but a 501 has to say what is missing, or the caller cannot tell
    /// an unconfigured feature from a broken one.
    #[tokio::test]
    async fn a_missing_feature_explains_itself_and_a_fault_does_not() {
        use axum::body::to_bytes;

        async fn detail(error: AppError) -> String {
            let response = error.into_response();
            let body = to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("a small body");

            serde_json::from_slice::<serde_json::Value>(&body).expect("problem json")["detail"]
                .as_str()
                .expect("a detail")
                .to_owned()
        }

        let missing = AppError::not_configured("uploads are not configured on this server");
        assert_eq!(
            detail(missing).await,
            "uploads are not configured on this server"
        );

        let broken = AppError::Unexpected(anyhow::anyhow!("the connection string is postgres://"));
        assert_eq!(detail(broken).await, "Internal server error");
    }
}
