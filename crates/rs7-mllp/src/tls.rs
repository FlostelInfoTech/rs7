//! TLS/mTLS configuration for secure MLLP communication
//!
//! This module provides utilities for configuring TLS and mutual TLS (mTLS)
//! for both MLLP clients and servers over TCP.

#[cfg(feature = "tls")]
use rs7_core::error::{Error, Result};

#[cfg(feature = "tls")]
use std::io::BufReader;
#[cfg(feature = "tls")]
use std::path::Path;
#[cfg(feature = "tls")]
use std::sync::Arc;

#[cfg(feature = "tls")]
use rustls::pki_types::CertificateDer;
#[cfg(feature = "tls")]
use rustls::{ClientConfig, RootCertStore, ServerConfig};
#[cfg(feature = "tls")]
use rustls_pemfile::{certs, private_key};

/// TLS configuration for MLLP client connections
#[cfg(feature = "tls")]
#[derive(Clone)]
pub struct TlsClientConfig {
    pub(crate) config: Arc<ClientConfig>,
}

#[cfg(feature = "tls")]
impl TlsClientConfig {
    /// Create a new TLS client configuration with system root certificates
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "tls")]
    /// # {
    /// use rs7_mllp::tls::TlsClientConfig;
    ///
    /// let config = TlsClientConfig::new().unwrap();
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        let mut root_store = RootCertStore::empty();

        // Add system root certificates
        for cert in webpki_roots::TLS_SERVER_ROOTS.iter() {
            root_store.roots.push(cert.clone());
        }

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Self {
            config: Arc::new(config),
        })
    }

    /// Create a TLS client configuration with custom root CA certificate
    ///
    /// # Arguments
    ///
    /// * `ca_cert_path` - Path to the PEM-encoded CA certificate file
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "tls")]
    /// # {
    /// use rs7_mllp::tls::TlsClientConfig;
    ///
    /// let config = TlsClientConfig::with_ca_cert("ca-cert.pem").unwrap();
    /// # }
    /// ```
    pub fn with_ca_cert(ca_cert_path: impl AsRef<Path>) -> Result<Self> {
        let mut root_store = RootCertStore::empty();

        // Load CA certificate
        let ca_cert_file = std::fs::File::open(ca_cert_path.as_ref())?;
        let mut reader = BufReader::new(ca_cert_file);

        let ca_certs = certs(&mut reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Network(format!("Failed to parse CA certificate: {}", e)))?;

        for cert in ca_certs {
            root_store
                .add(cert)
                .map_err(|e| Error::Network(format!("Failed to add CA certificate: {}", e)))?;
        }

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Self {
            config: Arc::new(config),
        })
    }

    /// Create a TLS client configuration with mutual TLS (client certificate)
    ///
    /// # Arguments
    ///
    /// * `ca_cert_path` - Path to the PEM-encoded CA certificate file
    /// * `client_cert_path` - Path to the PEM-encoded client certificate file
    /// * `client_key_path` - Path to the PEM-encoded client private key file
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "tls")]
    /// # {
    /// use rs7_mllp::tls::TlsClientConfig;
    ///
    /// let config = TlsClientConfig::with_mtls(
    ///     "ca-cert.pem",
    ///     "client-cert.pem",
    ///     "client-key.pem"
    /// ).unwrap();
    /// # }
    /// ```
    pub fn with_mtls(
        ca_cert_path: impl AsRef<Path>,
        client_cert_path: impl AsRef<Path>,
        client_key_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let mut root_store = RootCertStore::empty();

        // Load CA certificate
        let ca_cert_file = std::fs::File::open(ca_cert_path.as_ref())?;
        let mut reader = BufReader::new(ca_cert_file);

        let ca_certs = certs(&mut reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Network(format!("Failed to parse CA certificate: {}", e)))?;

        for cert in ca_certs {
            root_store
                .add(cert)
                .map_err(|e| Error::Network(format!("Failed to add CA certificate: {}", e)))?;
        }

        // Load client certificate
        let client_cert_file = std::fs::File::open(client_cert_path.as_ref())?;
        let mut cert_reader = BufReader::new(client_cert_file);

        let client_certs: Vec<CertificateDer> = certs(&mut cert_reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Network(format!("Failed to parse client certificate: {}", e)))?;

        // Load client private key
        let client_key_file = std::fs::File::open(client_key_path.as_ref())?;
        let mut key_reader = BufReader::new(client_key_file);

        let client_key = private_key(&mut key_reader)
            .map_err(|e| Error::Network(format!("Failed to parse private key: {}", e)))?
            .ok_or_else(|| Error::Network("No private key found in file".to_string()))?;

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(client_certs, client_key)
            .map_err(|e| Error::Network(format!("Failed to configure client auth: {}", e)))?;

        Ok(Self {
            config: Arc::new(config),
        })
    }
}

/// TLS configuration for MLLP server connections
#[cfg(feature = "tls")]
#[derive(Clone)]
pub struct TlsServerConfig {
    pub(crate) config: Arc<ServerConfig>,
}

#[cfg(feature = "tls")]
impl TlsServerConfig {
    /// Create a new TLS server configuration
    ///
    /// # Arguments
    ///
    /// * `cert_path` - Path to the PEM-encoded server certificate file
    /// * `key_path` - Path to the PEM-encoded server private key file
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "tls")]
    /// # {
    /// use rs7_mllp::tls::TlsServerConfig;
    ///
    /// let config = TlsServerConfig::new("server-cert.pem", "server-key.pem").unwrap();
    /// # }
    /// ```
    pub fn new(
        cert_path: impl AsRef<Path>,
        key_path: impl AsRef<Path>,
    ) -> Result<Self> {
        // Load server certificate
        let cert_file = std::fs::File::open(cert_path.as_ref())?;
        let mut cert_reader = BufReader::new(cert_file);

        let certs: Vec<CertificateDer> = certs(&mut cert_reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Network(format!("Failed to parse server certificate: {}", e)))?;

        // Load server private key
        let key_file = std::fs::File::open(key_path.as_ref())?;
        let mut key_reader = BufReader::new(key_file);

        let key = private_key(&mut key_reader)
            .map_err(|e| Error::Network(format!("Failed to parse private key: {}", e)))?
            .ok_or_else(|| Error::Network("No private key found in file".to_string()))?;

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| Error::Network(format!("Failed to configure server TLS: {}", e)))?;

        Ok(Self {
            config: Arc::new(config),
        })
    }

    /// Create a TLS server configuration with mutual TLS (client certificate verification)
    ///
    /// # Arguments
    ///
    /// * `cert_path` - Path to the PEM-encoded server certificate file
    /// * `key_path` - Path to the PEM-encoded server private key file
    /// * `ca_cert_path` - Path to the PEM-encoded CA certificate file for client verification
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "tls")]
    /// # {
    /// use rs7_mllp::tls::TlsServerConfig;
    ///
    /// let config = TlsServerConfig::with_mtls(
    ///     "server-cert.pem",
    ///     "server-key.pem",
    ///     "ca-cert.pem"
    /// ).unwrap();
    /// # }
    /// ```
    pub fn with_mtls(
        cert_path: impl AsRef<Path>,
        key_path: impl AsRef<Path>,
        ca_cert_path: impl AsRef<Path>,
    ) -> Result<Self> {
        // Load CA certificate for client verification
        let ca_cert_file = std::fs::File::open(ca_cert_path.as_ref())?;
        let mut ca_reader = BufReader::new(ca_cert_file);

        let mut client_auth_roots = RootCertStore::empty();
        let ca_certs = certs(&mut ca_reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Network(format!("Failed to parse CA certificate: {}", e)))?;

        for cert in ca_certs {
            client_auth_roots
                .add(cert)
                .map_err(|e| Error::Network(format!("Failed to add CA certificate: {}", e)))?;
        }

        // Load server certificate
        let cert_file = std::fs::File::open(cert_path.as_ref())?;
        let mut cert_reader = BufReader::new(cert_file);

        let certs: Vec<CertificateDer> = certs(&mut cert_reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Network(format!("Failed to parse server certificate: {}", e)))?;

        // Load server private key
        let key_file = std::fs::File::open(key_path.as_ref())?;
        let mut key_reader = BufReader::new(key_file);

        let key = private_key(&mut key_reader)
            .map_err(|e| Error::Network(format!("Failed to parse private key: {}", e)))?
            .ok_or_else(|| Error::Network("No private key found in file".to_string()))?;

        let client_cert_verifier = rustls::server::WebPkiClientVerifier::builder(Arc::new(client_auth_roots))
            .build()
            .map_err(|e| Error::Network(format!("Failed to build client verifier: {}", e)))?;

        let config = ServerConfig::builder()
            .with_client_cert_verifier(client_cert_verifier)
            .with_single_cert(certs, key)
            .map_err(|e| Error::Network(format!("Failed to configure server TLS: {}", e)))?;

        Ok(Self {
            config: Arc::new(config),
        })
    }
}

#[cfg(feature = "tls")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_new() {
        let config = TlsClientConfig::new();
        assert!(config.is_ok());
    }

    // Note: Additional tests would require test certificates
    // These should be added in integration tests
}
