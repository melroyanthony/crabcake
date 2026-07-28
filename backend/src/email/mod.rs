pub mod mailer;
pub mod templates;

use serde::{Deserialize, Serialize};

pub use mailer::Mailer;

/// A rendered email, ready to send.
///
/// Rendering happens when the message is created rather than when it is sent, so that a broken
/// template fails in the request that caused it instead of in a worker minutes later, and so
/// the queue holds no reference to a template that may have changed since.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub to: String,
    pub subject: String,
    pub html: String,
    pub text: String,
}
