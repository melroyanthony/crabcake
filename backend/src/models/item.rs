use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Item {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ItemCreate {
    #[validate(length(min = 1, max = 255, message = "must be between 1 and 255 characters"))]
    pub title: String,
    #[validate(length(max = 4096, message = "must be at most 4096 characters"))]
    pub description: Option<String>,
}

/// Every field is optional, so a client can send only what changed. `None` means "leave it
/// alone" rather than "set it to null".
#[derive(Debug, Deserialize, Validate)]
pub struct ItemUpdate {
    #[validate(length(min = 1, max = 255, message = "must be between 1 and 255 characters"))]
    pub title: Option<String>,
    #[validate(length(max = 4096, message = "must be at most 4096 characters"))]
    pub description: Option<String>,
}
