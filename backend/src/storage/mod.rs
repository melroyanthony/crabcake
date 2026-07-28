pub mod keys;

use std::time::Duration;

use aws_credential_types::Credentials;
use aws_sdk_s3::{
    Client,
    config::{BehaviorVersion, Region},
    presigning::PresigningConfig,
};
use aws_smithy_http_client::tls::{self, rustls_provider::CryptoMode};
use secrecy::ExposeSecret;

use crate::{AppError, AppResult, Config};

/// Object storage, or an explanation of why there is none.
///
/// Files never pass through the API: the client is handed a presigned URL and talks to the
/// bucket directly. That keeps large uploads off the API's memory, off its request timeout and
/// out of its body limit, and it means adding a CDN later changes nothing here.
#[derive(Clone)]
pub struct Storage {
    client: Option<Client>,
    bucket: String,
    expires_in: Duration,
}

impl Storage {
    /// Builds a client when a bucket is configured, and a working "switched off" state otherwise,
    /// so that a project without uploads runs unchanged.
    pub async fn from_config(config: &Config) -> Self {
        if !config.uploads_enabled() {
            tracing::info!("uploads are off; set S3_BUCKET to switch them on");

            return Self {
                client: None,
                bucket: String::new(),
                expires_in: Duration::from_secs(config.upload_url_expire_seconds),
            };
        }

        // Supplied rather than left to the SDK's default, which would be a second TLS
        // implementation alongside the one the database and the mailer already use.
        let https = aws_smithy_http_client::Builder::new()
            .tls_provider(tls::Provider::Rustls(CryptoMode::Ring))
            .build_https();

        let mut loader = aws_config::defaults(BehaviorVersion::latest())
            .http_client(https)
            .region(Region::new(config.s3_region.clone()));

        // Explicit keys when they are given, and the usual AWS chain otherwise, which is what
        // makes instance roles and IRSA work in production without any keys in the environment.
        if !config.s3_access_key_id.is_empty() {
            loader = loader.credentials_provider(Credentials::new(
                config.s3_access_key_id.clone(),
                config.s3_secret_access_key.expose_secret().to_owned(),
                None,
                None,
                "config",
            ));
        }

        let shared = loader.load().await;
        let mut builder = aws_sdk_s3::config::Builder::from(&shared);

        if !config.s3_endpoint.is_empty() {
            builder = builder.endpoint_url(&config.s3_endpoint);
        }

        // MinIO serves buckets as a path, not as a subdomain. Real S3 prefers the subdomain and
        // rejects path style for some bucket names, which is why this is configurable rather
        // than always on.
        if config.s3_force_path_style {
            builder = builder.force_path_style(true);
        }

        tracing::info!(
            bucket = %config.s3_bucket,
            endpoint = %if config.s3_endpoint.is_empty() { "aws" } else { &config.s3_endpoint },
            "uploads are on"
        );

        Self {
            client: Some(Client::from_conf(builder.build())),
            bucket: config.s3_bucket.clone(),
            expires_in: Duration::from_secs(config.upload_url_expire_seconds),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.client.is_some()
    }

    pub fn expires_in(&self) -> Duration {
        self.expires_in
    }

    /// A URL the client can `PUT` a file to.
    ///
    /// The content type is part of what is signed, so a caller cannot announce an image and then
    /// store something else under that name.
    pub async fn presigned_put(&self, key: &str, content_type: &str) -> AppResult<String> {
        let request = self
            .client()?
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .presigned(self.presigning_config()?)
            .await
            .map_err(|error| {
                AppError::Unexpected(anyhow::anyhow!("could not sign an upload URL: {error}"))
            })?;

        Ok(request.uri().to_owned())
    }

    /// A URL the client can `GET` a file from, for a bucket that is not public.
    pub async fn presigned_get(&self, key: &str) -> AppResult<String> {
        let request = self
            .client()?
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(self.presigning_config()?)
            .await
            .map_err(|error| {
                AppError::Unexpected(anyhow::anyhow!("could not sign a download URL: {error}"))
            })?;

        Ok(request.uri().to_owned())
    }

    /// Whether an object exists, used to answer 404 rather than hand out a link to nothing.
    pub async fn exists(&self, key: &str) -> AppResult<bool> {
        match self
            .client()?
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(error) if error.as_service_error().is_some_and(|e| e.is_not_found()) => Ok(false),
            Err(error) => Err(AppError::Unexpected(anyhow::anyhow!(
                "could not check the object: {error}"
            ))),
        }
    }

    pub async fn delete(&self, key: &str) -> AppResult<()> {
        self.client()?
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|error| {
                AppError::Unexpected(anyhow::anyhow!("could not delete the object: {error}"))
            })?;

        Ok(())
    }

    fn client(&self) -> AppResult<&Client> {
        self.client
            .as_ref()
            .ok_or_else(|| AppError::not_configured("uploads are not configured on this server"))
    }

    fn presigning_config(&self) -> AppResult<PresigningConfig> {
        PresigningConfig::expires_in(self.expires_in).map_err(|error| {
            AppError::Unexpected(anyhow::anyhow!("invalid link lifetime: {error}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn no_bucket_means_uploads_are_off() {
        let storage = Storage::from_config(&Config::for_tests()).await;

        assert!(!storage.is_enabled());
    }

    /// Asking for a link on a server without uploads should say so, not fail as though something
    /// had broken.
    #[tokio::test]
    async fn asking_for_a_link_without_a_bucket_is_a_501() {
        let storage = Storage::from_config(&Config::for_tests()).await;

        let error = storage
            .presigned_put("anything", "text/plain")
            .await
            .expect_err("there is nowhere to upload to");

        assert!(matches!(error, AppError::NotConfigured(_)));
    }
}
