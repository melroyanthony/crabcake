use utoipa::{
    Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};

/// The document itself. Paths are not listed here: they are collected from the routers by
/// `utoipa-axum`, so a route cannot be added to the API and forgotten in the documentation.
#[derive(OpenApi)]
#[openapi(
    modifiers(&BearerAuth),
    tags(
        (name = "health", description = "Liveness and readiness probes."),
        (name = "login", description = "Signing in, refreshing, and signing out."),
        (name = "password", description = "Resetting a forgotten password by email."),
        (name = "users", description = "Accounts, both your own and, for superusers, everyone else's."),
        (name = "items", description = "The example domain. Replace it with yours."),
        (name = "uploads", description = "Files, exchanged directly with object storage through signed links."),
    ),
    info(
        title = "API",
        description = "\
Every failure is an RFC 9457 problem document served as `application/problem+json`.

Collections are paged with `skip` and `limit`, and answer with `data` alongside a `count` of \
every matching row, not just the ones returned.

Send the access token as `Authorization: Bearer <token>`. It is short-lived; exchange the \
refresh token at `/api/v1/login/refresh` to get another. Refresh tokens are single-use, so \
each exchange returns a new one and invalidates the one presented.",
    )
)]
pub struct ApiDoc;

/// Declared once here rather than repeated on every protected path.
pub struct BearerAuth;

impl Modify for BearerAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // Present whenever the derive has produced a components section, which it always has
        // by the time a modifier runs.
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer",
                SecurityScheme::Http(
                    Http::builder()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("An access token from `/api/v1/login/access-token`."))
                        .build(),
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    fn document() -> Value {
        serde_json::to_value(crate::api::openapi()).expect("the document should serialise")
    }

    /// Collects every `$ref` in the document, however deeply nested.
    fn references(value: &Value, found: &mut Vec<String>) {
        match value {
            Value::Object(fields) => {
                for (key, value) in fields {
                    if key == "$ref"
                        && let Some(reference) = value.as_str()
                    {
                        found.push(reference.to_owned());
                    }

                    references(value, found);
                }
            }
            Value::Array(values) => values.iter().for_each(|value| references(value, found)),
            _ => {}
        }
    }

    /// A reference to a schema that was never registered produces a document that looks fine
    /// until the client generator chokes on it, so it is worth failing here instead.
    #[test]
    fn every_reference_resolves() {
        let document = document();
        let schemas = &document["components"]["schemas"];

        let mut found = Vec::new();
        references(&document, &mut found);
        assert!(
            !found.is_empty(),
            "expected the document to reference something"
        );

        for reference in found {
            let name = reference
                .strip_prefix("#/components/schemas/")
                .unwrap_or_else(|| panic!("unexpected reference form: {reference}"));

            assert!(
                !schemas[name].is_null(),
                "{name} is referenced but not defined"
            );
        }
    }

    #[test]
    fn every_operation_is_documented() {
        let document = document();
        let paths = document["paths"].as_object().expect("paths");

        for (path, operations) in paths {
            for (method, operation) in operations.as_object().expect("operations") {
                assert!(
                    operation["summary"].is_string(),
                    "{method} {path} has no summary"
                );
                assert!(
                    operation["responses"]
                        .as_object()
                        .is_some_and(|r| !r.is_empty()),
                    "{method} {path} documents no responses"
                );
            }
        }
    }

    /// Trailing slashes are normalised away before routing, so a documented path that ends in
    /// one would send generated clients to an address the router never sees.
    #[test]
    fn no_path_has_a_trailing_slash() {
        let document = document();

        for path in document["paths"].as_object().expect("paths").keys() {
            assert!(
                path == "/" || !path.ends_with('/'),
                "{path} ends with a slash"
            );
        }
    }

    #[test]
    fn protected_operations_can_name_the_scheme_they_use() {
        let document = document();
        let schemes = &document["components"]["securitySchemes"];

        assert_eq!(schemes["bearer"]["scheme"], "bearer");
        assert_eq!(schemes["bearer"]["bearerFormat"], "JWT");
    }

    #[test]
    fn signing_in_is_open_and_reading_yourself_is_not() {
        let document = document();

        assert!(
            document["paths"]["/api/v1/login/access-token"]["post"]["security"].is_null(),
            "signing in cannot require a token"
        );
        assert!(
            !document["paths"]["/api/v1/users/me"]["get"]["security"].is_null(),
            "reading your own account should require a token"
        );
    }
}
