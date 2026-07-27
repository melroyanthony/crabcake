use serde::{Deserialize, Serialize};
use validator::Validate;

/// Offset pagination. `count` in the response is the total matching rows, not the page size,
/// so a client can render "showing 1-20 of 340" without a second request.
#[derive(Debug, Deserialize, Validate)]
pub struct Pagination {
    #[serde(default)]
    #[validate(range(min = 0, message = "cannot be negative"))]
    pub skip: i64,
    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 200, message = "must be between 1 and 200"))]
    pub limit: i64,
}

fn default_limit() -> i64 {
    20
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            skip: 0,
            limit: default_limit(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub count: i64,
}

impl<T> Page<T> {
    pub fn new(data: Vec<T>, count: i64) -> Self {
        Self { data, count }
    }
}

#[derive(Debug, Serialize)]
pub struct Message {
    pub message: String,
}

impl Message {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_absent_query_string_gives_a_first_page() {
        let page: Pagination = serde_urlencoded::from_str("").unwrap();

        assert_eq!(page.skip, 0);
        assert_eq!(page.limit, default_limit());
        assert!(page.validate().is_ok());
    }

    #[test]
    fn an_unbounded_limit_is_refused() {
        let page: Pagination = serde_urlencoded::from_str("limit=100000").unwrap();
        let errors = page.validate().unwrap_err();

        assert!(errors.errors().contains_key("limit"));
    }

    #[test]
    fn paging_backwards_is_refused() {
        let page: Pagination = serde_urlencoded::from_str("skip=-1").unwrap();

        assert!(page.validate().is_err());
    }
}
