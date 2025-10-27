//! HTTP transport for HL7 v2.x messages
//!
//! This crate implements the HAPI HL7-over-HTTP specification for sending HL7 v2.x
//! messages over HTTP/HTTPS. This is ideal for inter-organization communication where
//! MLLP (used for intra-organization) is not suitable.
//!
//! # Features
//!
//! - **HTTP Client**: Send HL7 messages via HTTP POST and receive ACK responses
//! - **HTTP Server**: Receive HL7 messages and send ACK responses
//! - **Content Types**: Support for `x-application/hl7-v2+er7` (pipe-delimited)
//! - **Authentication**: HTTP Basic Auth (with `auth` feature)
//! - **TLS/HTTPS**: Secure connections (with `tls` feature)
//! - **Async**: Built on Tokio for high performance
//!
//! # Quick Start
//!
//! ## Server
//!
//! ```no_run
//! use rs7_http::HttpServer;
//! use rs7_core::Message;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let server = HttpServer::new()
//!     .with_handler(Arc::new(|message: Message| {
//!         println!("Received: {:?}", message.get_message_type());
//!         // Return ACK
//!         Ok(message) // Simplified for example
//!     }));
//!
//! server.serve("127.0.0.1:8080").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Client
//!
//! ```no_run
//! use rs7_http::HttpClient;
//! use rs7_core::builders::adt::AdtBuilder;
//! use rs7_core::Version;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = HttpClient::new("http://example.com/hl7")?;
//!
//! let message = AdtBuilder::a01(Version::V2_5)
//!     .patient_id("12345")
//!     .patient_name("DOE", "JOHN")
//!     .build()?;
//!
//! let ack = client.send_message(&message).await?;
//! println!("ACK received: {:?}", ack.get_control_id());
//! # Ok(())
//! # }
//! ```
//!
//! # Comparison: MLLP vs HTTP
//!
//! | Feature | MLLP (rs7-mllp) | HTTP (rs7-http) |
//! |---------|-----------------|-----------------|
//! | Use Case | Intra-organization | Inter-organization |
//! | Protocol | TCP + framing | HTTP/HTTPS |
//! | Connection | Persistent | Stateless |
//! | Security | VPN typically | TLS + Auth |
//! | Firewall | Requires configuration | Works through proxies |
//! | Cloud/SaaS | Limited | Native support |
//!
//! # Specification
//!
//! This implementation follows the HAPI HL7-over-HTTP specification:
//! <https://hapifhir.github.io/hapi-hl7v2/hapi-hl7overhttp/specification.html>

pub mod auth;
pub mod client;
pub mod error;
pub mod server;

#[cfg(feature = "tls")]
pub mod tls;

#[cfg(feature = "logging")]
pub mod logging;

#[cfg(feature = "metrics")]
pub mod metrics;

#[cfg(feature = "ratelimit")]
pub mod ratelimit;

#[cfg(feature = "queue")]
pub mod queue;

#[cfg(feature = "batch")]
pub mod batch;

#[cfg(feature = "routing")]
pub mod router;

#[cfg(feature = "webhooks")]
pub mod webhook;

#[cfg(feature = "retry")]
pub mod retry;

#[cfg(feature = "auth")]
pub mod middleware;

#[cfg(feature = "websocket")]
pub mod websocket;

pub use client::HttpClient;
pub use error::{Error, Result};
pub use server::{HttpServer, MessageHandler};

/// Content type for HL7 v2.x messages in pipe-delimited (ER7) format
///
/// This is the standard content type defined in the HAPI specification
/// for vertical bar encoding.
pub const CONTENT_TYPE_HL7_ER7: &str = "x-application/hl7-v2+er7";

/// Content type for HL7 v2.x messages in XML format
///
/// This content type is used for XML-encoded HL7 v2.x messages.
pub const CONTENT_TYPE_HL7_XML: &str = "x-application/hl7-v2+xml";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_constants() {
        assert_eq!(CONTENT_TYPE_HL7_ER7, "x-application/hl7-v2+er7");
        assert_eq!(CONTENT_TYPE_HL7_XML, "x-application/hl7-v2+xml");
    }
}
