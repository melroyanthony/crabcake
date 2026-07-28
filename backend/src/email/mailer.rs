use lettre::{
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    message::{Mailbox, MultiPart},
    transport::smtp::authentication::Credentials,
};
use secrecy::ExposeSecret;

use crate::{AppError, AppResult, Config, email::Email};

/// Sends mail, or explains where it would have gone.
///
/// A transport is built once and reused: SMTP connection setup involves a TLS handshake and,
/// often, authentication, which is not something to repeat per message.
#[derive(Clone)]
pub struct Mailer {
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from: Mailbox,
}

impl Mailer {
    /// Builds a mailer, or a mailer that only logs when no SMTP server is configured. A fresh
    /// checkout should run and be explorable before anyone has thought about email.
    pub fn from_config(config: &Config) -> AppResult<Self> {
        if !config.emails_enabled() {
            tracing::warn!(
                "email is not configured; messages will be logged instead of sent. Set \
                 SMTP_HOST and EMAILS_FROM_EMAIL to send them."
            );

            return Ok(Self {
                transport: None,
                // Never used without a transport, and not worth failing startup over.
                from: Mailbox::new(
                    None,
                    "noreply@invalid".parse().expect("a literal address parses"),
                ),
            });
        }

        // Parsed before the transport is built, so that a typo in EMAILS_FROM_EMAIL is a plain
        // startup error rather than a connection pool that gets created and immediately dropped.
        let from = format!(
            "{} <{}>",
            config.emails_from_name(),
            config.emails_from_email
        )
        .parse::<Mailbox>()
        .map_err(|error| {
            AppError::Unexpected(anyhow::anyhow!(
                "EMAILS_FROM_EMAIL is not a valid address: {error}"
            ))
        })?;

        let mut builder = if config.smtp_tls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host).map_err(
                |error| AppError::Unexpected(anyhow::anyhow!("could not reach SMTP: {error}")),
            )?
        } else {
            // Plain SMTP, which is what Mailcatcher and friends speak. Fine locally, and the
            // template's own configuration only does this when ENVIRONMENT=local.
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.smtp_host)
        };

        builder = builder.port(config.smtp_port);

        if !config.smtp_user.is_empty() {
            builder = builder.credentials(Credentials::new(
                config.smtp_user.clone(),
                config.smtp_password.expose_secret().to_owned(),
            ));
        }

        Ok(Self {
            transport: Some(builder.build()),
            from,
        })
    }

    pub async fn send(&self, email: &Email) -> AppResult<()> {
        let Some(transport) = &self.transport else {
            tracing::info!(
                to = %email.to,
                subject = %email.subject,
                body = %email.text,
                "email not sent: no SMTP server configured"
            );

            return Ok(());
        };

        let message = lettre::Message::builder()
            .from(self.from.clone())
            .to(email.to.parse().map_err(|error| {
                AppError::validation(format!("not a valid email address: {error}"))
            })?)
            .subject(&email.subject)
            .multipart(MultiPart::alternative_plain_html(
                email.text.clone(),
                email.html.clone(),
            ))
            .map_err(|error| {
                AppError::Unexpected(anyhow::anyhow!("could not build the message: {error}"))
            })?;

        transport.send(message).await.map_err(|error| {
            AppError::Unexpected(anyhow::anyhow!("could not send the message: {error}"))
        })?;

        tracing::info!(to = %email.to, subject = %email.subject, "email sent");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mailer_without_smtp_configured_still_builds() {
        let mailer = Mailer::from_config(&Config::for_tests()).unwrap();

        assert!(mailer.transport.is_none());
    }

    #[tokio::test]
    async fn sending_without_smtp_configured_succeeds_quietly() {
        let mailer = Mailer::from_config(&Config::for_tests()).unwrap();
        let email = Email {
            to: "ada@example.com".to_owned(),
            subject: "Hello".to_owned(),
            html: "<p>Hello</p>".to_owned(),
            text: "Hello".to_owned(),
        };

        // A password reset that fails because nobody set up SMTP should not look to the caller
        // like a broken account.
        assert!(mailer.send(&email).await.is_ok());
    }

    #[test]
    fn a_malformed_from_address_fails_when_email_is_switched_on() {
        let config = Config {
            smtp_host: "localhost".to_owned(),
            emails_from_email: "not an address".to_owned(),
            ..Config::for_tests()
        };

        assert!(Mailer::from_config(&config).is_err());
    }
}
