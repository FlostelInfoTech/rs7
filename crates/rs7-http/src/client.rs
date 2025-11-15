//! HTTP client for sending HL7 messages

use crate::{Error, Result, CONTENT_TYPE_HL7_ER7};
use reqwest::{Client, ClientBuilder};
use rs7_core::Message;
use rs7_parser::parse_message;
use std::time::Duration;

#[cfg(feature = "tls")]
use crate::tls::TlsClientConfig;

/// HTTP client for sending HL7 v2.x messages
///
/// # Example
///
/// ```no_run
/// use rs7_http::HttpClient;
/// use rs7_core::builders::adt::AdtBuilder;
/// use rs7_core::Version;
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = HttpClient::new("http://example.com/hl7")?
///     .with_auth("username".into(), "password".into())
///     .with_timeout(Duration::from_secs(30))?;
///
/// let message = AdtBuilder::a01(Version::V2_5)
///     .patient_id("12345")
///     .build()?;
///
/// let ack = client.send_message(&message).await?;
/// # Ok(())
/// # }
/// ```
pub struct HttpClient {
    endpoint: String,
    client: Client,
    auth: Option<(String, String)>,
    #[cfg(feature = "tls")]
    tls_config: Option<TlsClientConfig>,
    http2_only: bool,
}

impl HttpClient {
    /// Create a new HTTP client
    ///
    /// # Arguments
    /// * `endpoint` - The HTTP endpoint URL (e.g., "https://api.example.com/hl7")
    ///
    /// # Errors
    /// Returns an error if the URL is invalid or the HTTP client cannot be created
    pub fn new(endpoint: impl Into<String>) -> Result<Self> {
        let endpoint = endpoint.into();

        // Validate URL
        if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
            return Err(Error::InvalidUrl(
                "URL must start with http:// or https://".to_string(),
            ));
        }

        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            endpoint,
            client,
            auth: None,
            #[cfg(feature = "tls")]
            tls_config: None,
            http2_only: false,
        })
    }

    /// Enable HTTP/2 only mode
    ///
    /// When enabled, the client will only use HTTP/2 protocol.
    /// By default, the client uses HTTP/1.1 with HTTP/2 upgrade.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rs7_http::HttpClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = HttpClient::new("https://example.com/hl7")?
    ///     .with_http2_only()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_http2_only(mut self) -> Result<Self> {
        // Rebuild client with HTTP/2 only
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .http2_prior_knowledge()
            .build()?;

        self.client = client;
        self.http2_only = true;
        Ok(self)
    }

    /// Configure TLS/mTLS for the HTTP client
    ///
    /// # Arguments
    /// * `tls_config` - TLS configuration including optional client certificates
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "tls")]
    /// # {
    /// use rs7_http::{HttpClient, tls::TlsClientConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tls_config = TlsClientConfig::with_mtls(
    ///     "ca-cert.pem",
    ///     "client-cert.pem",
    ///     "client-key.pem"
    /// )?;
    ///
    /// let client = HttpClient::new("https://example.com/hl7")?
    ///     .with_tls(tls_config)?;
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    #[cfg(feature = "tls")]
    pub fn with_tls(mut self, tls_config: TlsClientConfig) -> Result<Self> {
        // Rebuild client with TLS configuration
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .use_preconfigured_tls(tls_config.config.clone())
            .build()?;

        self.client = client;
        self.tls_config = Some(tls_config);
        Ok(self)
    }

    /// Set HTTP Basic Authentication credentials
    ///
    /// # Arguments
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    pub fn with_auth(mut self, username: String, password: String) -> Self {
        self.auth = Some((username, password));
        self
    }

    /// Set request timeout
    ///
    /// # Arguments
    /// * `timeout` - Timeout duration for HTTP requests
    ///
    /// # Note
    /// This method rebuilds the HTTP client while preserving existing configuration
    /// (TLS, HTTP/2, etc.) to avoid losing settings.
    pub fn with_timeout(mut self, timeout: Duration) -> Result<Self> {
        let mut builder = ClientBuilder::new().timeout(timeout);

        // Preserve HTTP/2 setting
        if self.http2_only {
            builder = builder.http2_prior_knowledge();
        }

        // Preserve TLS configuration
        #[cfg(feature = "tls")]
        if let Some(ref tls_config) = self.tls_config {
            builder = builder.use_preconfigured_tls(tls_config.config.clone());
        }

        self.client = builder.build()?;
        Ok(self)
    }

    /// Send an HL7 message and receive the response
    ///
    /// # Arguments
    /// * `message` - The HL7 message to send
    ///
    /// # Returns
    /// The response message (typically an ACK)
    ///
    /// # Errors
    /// Returns an error if:
    /// - The network request fails
    /// - The server returns a non-2xx status code
    /// - The response content type is invalid
    /// - The response cannot be parsed as an HL7 message
    pub async fn send_message(&self, message: &Message) -> Result<Message> {
        let hl7_text = message.encode();
        let ack_text = self.send_message_raw(&hl7_text).await?;
        Ok(parse_message(&ack_text)?)
    }

    /// Send an HL7 message as a raw string
    ///
    /// This is a lower-level method that sends the HL7 message text directly
    /// and returns the raw response text.
    ///
    /// # Arguments
    /// * `hl7_text` - The HL7 message as a string
    ///
    /// # Returns
    /// The response message text
    ///
    /// # Errors
    /// Returns an error if the request fails or the response is invalid
    pub async fn send_message_raw(&self, hl7_text: &str) -> Result<String> {
        // Build request
        let mut request = self
            .client
            .post(&self.endpoint)
            .header("Content-Type", CONTENT_TYPE_HL7_ER7)
            .header("Date", chrono::Utc::now().to_rfc2822())
            .body(hl7_text.to_string());

        // Add authentication if configured
        if let Some((username, password)) = &self.auth {
            request = request.basic_auth(username, Some(password));
        }

        // Send request
        let response = request.send().await?;

        // Check status code
        let status = response.status();
        if !status.is_success() {
            return Err(Error::Http(format!(
                "HTTP {} {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            )));
        }

        // Verify content type
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !content_type.contains("hl7") {
            return Err(Error::ContentType {
                expected: CONTENT_TYPE_HL7_ER7.to_string(),
                actual: content_type.to_string(),
            });
        }

        // Read response body
        Ok(response.text().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client_valid_url() {
        assert!(HttpClient::new("http://example.com").is_ok());
        assert!(HttpClient::new("https://example.com").is_ok());
        assert!(HttpClient::new("https://example.com/path").is_ok());
    }

    #[test]
    fn test_new_client_invalid_url() {
        assert!(HttpClient::new("ftp://example.com").is_err());
        assert!(HttpClient::new("example.com").is_err());
        assert!(HttpClient::new("").is_err());
    }

    #[test]
    fn test_with_auth() {
        let client = HttpClient::new("http://example.com")
            .unwrap()
            .with_auth("user".to_string(), "pass".to_string());

        assert!(client.auth.is_some());
        assert_eq!(client.auth.unwrap(), ("user".to_string(), "pass".to_string()));
    }
}
