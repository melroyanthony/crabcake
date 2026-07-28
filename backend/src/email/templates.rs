use std::sync::LazyLock;

use minijinja::{Environment, Value, context};

use crate::{AppError, AppResult, Config, email::Email};

/// Templates are compiled into the binary rather than read from disk, so the Docker image is
/// the binary alone and a missing file cannot turn into a runtime failure.
static TEMPLATES: LazyLock<Environment<'static>> = LazyLock::new(|| {
    let mut environment = Environment::new();

    for (name, source) in [
        (
            "layout.html",
            include_str!("../../templates/email/layout.html"),
        ),
        (
            "reset_password.html",
            include_str!("../../templates/email/reset_password.html"),
        ),
        (
            "reset_password.txt",
            include_str!("../../templates/email/reset_password.txt"),
        ),
        (
            "new_account.html",
            include_str!("../../templates/email/new_account.html"),
        ),
        (
            "new_account.txt",
            include_str!("../../templates/email/new_account.txt"),
        ),
    ] {
        environment
            .add_template(name, source)
            .unwrap_or_else(|error| panic!("email template {name} does not compile: {error}"));
    }

    environment
});

/// The email behind a "forgot my password" request.
pub fn reset_password(config: &Config, address: &str, token: &str) -> AppResult<Email> {
    // The link points at the frontend, which collects the new password and then calls the API.
    // Sending people straight to an API endpoint would show them raw JSON.
    let reset_url = format!(
        "{}/reset-password?token={token}",
        config.frontend_host.trim_end_matches('/')
    );

    let subject = format!("Reset your {} password", config.project_name);
    let values = context! {
        subject => subject,
        project_name => config.project_name,
        email => address,
        reset_url => safe_url(reset_url),
        expire_hours => config.password_reset_token_expire_hours,
    };

    render(address, subject.clone(), "reset_password", values)
}

/// The email a superuser's freshly created account gets.
pub fn new_account(config: &Config, address: &str) -> AppResult<Email> {
    let login_url = format!("{}/login", config.frontend_host.trim_end_matches('/'));
    let subject = format!("Your {} account is ready", config.project_name);

    let values = context! {
        subject => subject,
        project_name => config.project_name,
        email => address,
        login_url => safe_url(login_url),
    };

    render(address, subject.clone(), "new_account", values)
}

/// Exempts a URL from HTML escaping.
///
/// Autoescaping is left on, because names and addresses do reach these templates. It escapes
/// `/` as `&#x2f;`, which a browser decodes but which reads as gibberish in the link people are
/// invited to copy and paste. These URLs are built here from configuration and a random token,
/// with nothing a caller supplies, so there is nothing to escape.
fn safe_url(url: String) -> Value {
    Value::from_safe_string(url)
}

/// Renders the HTML and plain-text halves of one message. Both are sent, because a text part
/// is what clients that refuse HTML fall back to, and its absence is a spam signal.
fn render(to: &str, subject: String, template: &str, values: minijinja::Value) -> AppResult<Email> {
    let html = render_one(&format!("{template}.html"), &values)?;
    let text = render_one(&format!("{template}.txt"), &values)?;

    Ok(Email {
        to: to.to_owned(),
        subject,
        html,
        text,
    })
}

fn render_one(name: &str, values: &minijinja::Value) -> AppResult<String> {
    TEMPLATES
        .get_template(name)
        .and_then(|template| template.render(values))
        .map_err(|error| AppError::Unexpected(anyhow::anyhow!("could not render {name}: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config {
            project_name: "Crabcake".to_owned(),
            frontend_host: "https://example.com/".to_owned(),
            ..Config::for_tests()
        }
    }

    #[test]
    fn a_reset_email_carries_the_link_in_both_parts() {
        let email = reset_password(&config(), "ada@example.com", "the-token").unwrap();

        assert_eq!(email.to, "ada@example.com");
        assert!(email.subject.contains("Crabcake"));

        for part in [&email.html, &email.text] {
            assert!(
                part.contains("https://example.com/reset-password?token=the-token"),
                "missing link in {part}"
            );
        }
    }

    /// A trailing slash on FRONTEND_HOST is the kind of thing nobody notices until every link
    /// in every email has two.
    #[test]
    fn a_trailing_slash_on_the_host_does_not_double_up() {
        let email = new_account(&config(), "ada@example.com").unwrap();

        assert!(email.html.contains("https://example.com/login"));
        assert!(!email.html.contains("example.com//"));
    }

    #[test]
    fn the_layout_wraps_the_content() {
        let email = new_account(&config(), "ada@example.com").unwrap();

        assert!(email.html.contains("<!doctype html>"));
        assert!(email.html.contains("Your account is ready"));
    }

    #[test]
    fn the_text_part_has_no_markup() {
        let email = reset_password(&config(), "ada@example.com", "t").unwrap();

        assert!(!email.text.contains('<'), "got {}", email.text);
    }
}
