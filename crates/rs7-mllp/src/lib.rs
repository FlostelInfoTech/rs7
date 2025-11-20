//! MLLP (Minimal Lower Layer Protocol) support
//!
//! MLLP is a simple framing protocol used for transmitting HL7 messages over TCP.
//! Format: \<VT\>message\<FS\>\<CR\>
//! - VT (Vertical Tab): 0x0B - Start of block
//! - FS (File Separator): 0x1C - End of block
//! - CR (Carriage Return): 0x0D - End of message

// TLS/mTLS support (optional)
#[cfg(feature = "tls")]
pub mod tls;

use rs7_core::{
    error::{Error, Result},
    message::Message,
};
use rs7_parser::parse_message;
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[cfg(feature = "tls")]
use tokio_rustls::{TlsConnector, TlsAcceptor};
#[cfg(feature = "tls")]
use rustls::pki_types::ServerName;

// Stream wrapper to support both plain TCP and TLS
enum MllpStream {
    Plain(TcpStream),
    #[cfg(feature = "tls")]
    TlsClient(tokio_rustls::client::TlsStream<TcpStream>),
    #[cfg(feature = "tls")]
    TlsServer(tokio_rustls::server::TlsStream<TcpStream>),
}

impl MllpStream {
    async fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            MllpStream::Plain(stream) => {
                stream.read_exact(buf).await?;
                Ok(buf.len())
            }
            #[cfg(feature = "tls")]
            MllpStream::TlsClient(stream) => {
                stream.read_exact(buf).await?;
                Ok(buf.len())
            }
            #[cfg(feature = "tls")]
            MllpStream::TlsServer(stream) => {
                stream.read_exact(buf).await?;
                Ok(buf.len())
            }
        }
    }

    async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self {
            MllpStream::Plain(stream) => stream.write_all(buf).await,
            #[cfg(feature = "tls")]
            MllpStream::TlsClient(stream) => stream.write_all(buf).await,
            #[cfg(feature = "tls")]
            MllpStream::TlsServer(stream) => stream.write_all(buf).await,
        }
    }

    async fn shutdown(&mut self) -> std::io::Result<()> {
        match self {
            MllpStream::Plain(stream) => stream.shutdown().await,
            #[cfg(feature = "tls")]
            MllpStream::TlsClient(stream) => stream.shutdown().await,
            #[cfg(feature = "tls")]
            MllpStream::TlsServer(stream) => stream.shutdown().await,
        }
    }
}

/// MLLP frame markers
pub const START_OF_BLOCK: u8 = 0x0B; // Vertical Tab (VT)
pub const END_OF_BLOCK: u8 = 0x1C; // File Separator (FS)
pub const CARRIAGE_RETURN: u8 = 0x0D; // Carriage Return (CR)

/// Default maximum message size (10 MB)
/// This prevents DoS attacks via unbounded buffer growth
pub const DEFAULT_MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

/// Default read timeout (30 seconds)
pub const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Default write timeout (30 seconds)
pub const DEFAULT_WRITE_TIMEOUT: Duration = Duration::from_secs(30);

/// Default connection timeout (10 seconds)
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Configuration for MLLP client and server
#[derive(Debug, Clone)]
pub struct MllpConfig {
    /// Maximum message size in bytes (default: 10 MB)
    pub max_message_size: usize,
    /// Read timeout (default: 30 seconds)
    pub read_timeout: Duration,
    /// Write timeout (default: 30 seconds)
    pub write_timeout: Duration,
    /// Connection timeout (default: 10 seconds)
    pub connect_timeout: Duration,
}

impl Default for MllpConfig {
    fn default() -> Self {
        Self {
            max_message_size: DEFAULT_MAX_MESSAGE_SIZE,
            read_timeout: DEFAULT_READ_TIMEOUT,
            write_timeout: DEFAULT_WRITE_TIMEOUT,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
        }
    }
}

impl MllpConfig {
    /// Create a new configuration with custom settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum message size
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// Set the read timeout
    pub fn with_read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = timeout;
        self
    }

    /// Set the write timeout
    pub fn with_write_timeout(mut self, timeout: Duration) -> Self {
        self.write_timeout = timeout;
        self
    }

    /// Set the connection timeout
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }
}

/// MLLP message framing
pub struct MllpFrame;

impl MllpFrame {
    /// Wrap an HL7 message in MLLP framing
    pub fn wrap(message: &str) -> Vec<u8> {
        let mut framed = Vec::with_capacity(message.len() + 3);
        framed.push(START_OF_BLOCK);
        framed.extend_from_slice(message.as_bytes());
        framed.push(END_OF_BLOCK);
        framed.push(CARRIAGE_RETURN);
        framed
    }

    /// Unwrap an MLLP frame to get the HL7 message
    pub fn unwrap(framed: &[u8]) -> Result<String> {
        if framed.len() < 3 {
            return Err(Error::Mllp("Frame too short".to_string()));
        }

        if framed[0] != START_OF_BLOCK {
            return Err(Error::Mllp("Missing start-of-block marker".to_string()));
        }

        let end_pos = framed.len() - 2;
        if framed[end_pos] != END_OF_BLOCK {
            return Err(Error::Mllp("Missing end-of-block marker".to_string()));
        }

        if framed[framed.len() - 1] != CARRIAGE_RETURN {
            return Err(Error::Mllp("Missing carriage return".to_string()));
        }

        let message_bytes = &framed[1..end_pos];
        String::from_utf8(message_bytes.to_vec())
            .map_err(|e| Error::Mllp(format!("Invalid UTF-8: {}", e)))
    }
}

/// MLLP client for sending messages
pub struct MllpClient {
    stream: MllpStream,
    max_message_size: usize,
    read_timeout: Duration,
    write_timeout: Duration,
}

impl MllpClient {
    /// Connect to an MLLP server with default settings
    pub async fn connect(addr: &str) -> Result<Self> {
        Self::connect_with_config(addr, MllpConfig::default()).await
    }

    /// Connect to an MLLP server with custom configuration
    pub async fn connect_with_config(addr: &str, config: MllpConfig) -> Result<Self> {
        let tcp_stream = tokio::time::timeout(
            config.connect_timeout,
            TcpStream::connect(addr)
        )
        .await
        .map_err(|_| Error::Network(format!("Connection timeout after {:?}", config.connect_timeout)))?
        .map_err(|e| Error::Network(format!("Failed to connect: {}", e)))?;

        Ok(Self {
            stream: MllpStream::Plain(tcp_stream),
            max_message_size: config.max_message_size,
            read_timeout: config.read_timeout,
            write_timeout: config.write_timeout,
        })
    }

    /// Connect to an MLLP server with TLS encryption
    ///
    /// # Arguments
    ///
    /// * `addr` - Server address (e.g., "localhost:2575")
    /// * `server_name` - Server name for SNI (e.g., "localhost" or "example.com")
    /// * `tls_config` - TLS configuration (basic TLS or mTLS)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "tls")]
    /// # async fn example() -> rs7_core::error::Result<()> {
    /// use rs7_mllp::{MllpClient, tls::TlsClientConfig};
    ///
    /// // Basic TLS
    /// let tls_config = TlsClientConfig::new()?;
    /// let mut client = MllpClient::connect_tls(
    ///     "localhost:2575",
    ///     "localhost",
    ///     tls_config
    /// ).await?;
    ///
    /// // Or with mTLS
    /// let mtls_config = TlsClientConfig::with_mtls(
    ///     "ca-cert.pem",
    ///     "client-cert.pem",
    ///     "client-key.pem"
    /// )?;
    /// let mut client = MllpClient::connect_tls(
    ///     "localhost:2575",
    ///     "localhost",
    ///     mtls_config
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "tls")]
    pub async fn connect_tls(
        addr: &str,
        server_name: &str,
        tls_config: crate::tls::TlsClientConfig,
    ) -> Result<Self> {
        Self::connect_tls_with_config(addr, server_name, tls_config, MllpConfig::default()).await
    }

    /// Connect to an MLLP server with TLS encryption and custom configuration
    ///
    /// # Arguments
    ///
    /// * `addr` - Server address (e.g., "localhost:2575")
    /// * `server_name` - Server name for SNI (e.g., "localhost" or "example.com")
    /// * `tls_config` - TLS configuration (basic TLS or mTLS)
    /// * `config` - MLLP configuration (timeouts, buffer sizes)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "tls")]
    /// # async fn example() -> rs7_core::error::Result<()> {
    /// use rs7_mllp::{MllpClient, MllpConfig, tls::TlsClientConfig};
    /// use std::time::Duration;
    ///
    /// let tls_config = TlsClientConfig::new()?;
    /// let mllp_config = MllpConfig::new()
    ///     .with_read_timeout(Duration::from_secs(60))
    ///     .with_write_timeout(Duration::from_secs(60));
    ///
    /// let mut client = MllpClient::connect_tls_with_config(
    ///     "localhost:2575",
    ///     "localhost",
    ///     tls_config,
    ///     mllp_config
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "tls")]
    pub async fn connect_tls_with_config(
        addr: &str,
        server_name: &str,
        tls_config: crate::tls::TlsClientConfig,
        config: MllpConfig,
    ) -> Result<Self> {
        // Establish TCP connection first
        let tcp_stream = tokio::time::timeout(
            config.connect_timeout,
            TcpStream::connect(addr)
        )
        .await
        .map_err(|_| Error::Network(format!("Connection timeout after {:?}", config.connect_timeout)))?
        .map_err(|e| Error::Network(format!("Failed to connect: {}", e)))?;

        // Perform TLS handshake
        let connector = TlsConnector::from(tls_config.config.clone());
        let server_name = ServerName::try_from(server_name.to_string())
            .map_err(|e| Error::Network(format!("Invalid server name: {}", e)))?;

        let tls_stream = connector
            .connect(server_name, tcp_stream)
            .await
            .map_err(|e| Error::Network(format!("TLS handshake failed: {}", e)))?;

        Ok(Self {
            stream: MllpStream::TlsClient(tls_stream),
            max_message_size: config.max_message_size,
            read_timeout: config.read_timeout,
            write_timeout: config.write_timeout,
        })
    }

    /// Set the maximum message size
    pub fn set_max_message_size(&mut self, size: usize) {
        self.max_message_size = size;
    }

    /// Set the read timeout
    pub fn set_read_timeout(&mut self, timeout: Duration) {
        self.read_timeout = timeout;
    }

    /// Set the write timeout
    pub fn set_write_timeout(&mut self, timeout: Duration) {
        self.write_timeout = timeout;
    }

    /// Send a message and wait for acknowledgment
    pub async fn send_message(&mut self, message: &Message) -> Result<Message> {
        // Encode message
        let hl7_text = message.encode();

        // Wrap in MLLP frame
        let framed = MllpFrame::wrap(&hl7_text);

        // Send with timeout
        tokio::time::timeout(
            self.write_timeout,
            self.stream.write_all(&framed)
        )
        .await
        .map_err(|_| Error::Network(format!("Write timeout after {:?}", self.write_timeout)))?
        .map_err(|e| Error::Network(format!("Failed to send: {}", e)))?;

        // Receive acknowledgment
        self.receive_message().await
    }

    /// Receive a message with timeout and size limit
    pub async fn receive_message(&mut self) -> Result<Message> {
        tokio::time::timeout(
            self.read_timeout,
            self.receive_message_internal()
        )
        .await
        .map_err(|_| Error::Network(format!("Read timeout after {:?}", self.read_timeout)))?
    }

    /// Internal method to receive a message with buffer size protection
    async fn receive_message_internal(&mut self) -> Result<Message> {
        let mut buffer = Vec::with_capacity(8192); // Pre-allocate reasonable size
        let mut byte = [0u8; 1];

        // Read until we find the start marker
        loop {
            self.stream
                .read_exact(&mut byte)
                .await
                .map_err(|e| Error::Network(format!("Failed to read: {}", e)))?;

            if byte[0] == START_OF_BLOCK {
                buffer.push(byte[0]);
                break;
            }
        }

        // Read until we find the end markers
        let mut found_end = false;
        while !found_end {
            // Check buffer size limit
            if buffer.len() >= self.max_message_size {
                return Err(Error::Mllp(format!(
                    "Message exceeds maximum size of {} bytes",
                    self.max_message_size
                )));
            }

            self.stream
                .read_exact(&mut byte)
                .await
                .map_err(|e| Error::Network(format!("Failed to read: {}", e)))?;

            buffer.push(byte[0]);

            // Check for end sequence (FS CR)
            if buffer.len() >= 2 {
                let len = buffer.len();
                if buffer[len - 2] == END_OF_BLOCK && buffer[len - 1] == CARRIAGE_RETURN {
                    found_end = true;
                }
            }
        }

        // Unwrap frame
        let hl7_text = MllpFrame::unwrap(&buffer)?;

        // Parse message
        parse_message(&hl7_text)
    }

    /// Close the connection
    pub async fn close(mut self) -> Result<()> {
        self.stream
            .shutdown()
            .await
            .map_err(|e| Error::Network(format!("Failed to close: {}", e)))
    }
}

/// MLLP server for receiving messages
pub struct MllpServer {
    listener: TcpListener,
    config: MllpConfig,
    #[cfg(feature = "tls")]
    tls_acceptor: Option<TlsAcceptor>,
}

impl MllpServer {
    /// Bind to an address with default configuration
    pub async fn bind(addr: &str) -> Result<Self> {
        Self::bind_with_config(addr, MllpConfig::default()).await
    }

    /// Bind to an address with custom configuration
    pub async fn bind_with_config(addr: &str, config: MllpConfig) -> Result<Self> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| Error::Network(format!("Failed to bind: {}", e)))?;

        Ok(Self {
            listener,
            config,
            #[cfg(feature = "tls")]
            tls_acceptor: None,
        })
    }

    /// Bind to an address with TLS encryption
    ///
    /// # Arguments
    ///
    /// * `addr` - Server address (e.g., "0.0.0.0:2575")
    /// * `tls_config` - TLS configuration (basic TLS or mTLS)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "tls")]
    /// # async fn example() -> rs7_core::error::Result<()> {
    /// use rs7_mllp::{MllpServer, tls::TlsServerConfig};
    ///
    /// // Basic TLS
    /// let tls_config = TlsServerConfig::new("server-cert.pem", "server-key.pem")?;
    /// let server = MllpServer::bind_tls("0.0.0.0:2575", tls_config).await?;
    ///
    /// // Or with mTLS (client certificate verification)
    /// let mtls_config = TlsServerConfig::with_mtls(
    ///     "server-cert.pem",
    ///     "server-key.pem",
    ///     "ca-cert.pem"
    /// )?;
    /// let server = MllpServer::bind_tls("0.0.0.0:2575", mtls_config).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "tls")]
    pub async fn bind_tls(
        addr: &str,
        tls_config: crate::tls::TlsServerConfig,
    ) -> Result<Self> {
        Self::bind_tls_with_config(addr, tls_config, MllpConfig::default()).await
    }

    /// Bind to an address with TLS encryption and custom configuration
    ///
    /// # Arguments
    ///
    /// * `addr` - Server address (e.g., "0.0.0.0:2575")
    /// * `tls_config` - TLS configuration (basic TLS or mTLS)
    /// * `config` - MLLP configuration (timeouts, buffer sizes)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "tls")]
    /// # async fn example() -> rs7_core::error::Result<()> {
    /// use rs7_mllp::{MllpServer, MllpConfig, tls::TlsServerConfig};
    /// use std::time::Duration;
    ///
    /// let tls_config = TlsServerConfig::new("server-cert.pem", "server-key.pem")?;
    /// let mllp_config = MllpConfig::new()
    ///     .with_read_timeout(Duration::from_secs(60))
    ///     .with_write_timeout(Duration::from_secs(60));
    ///
    /// let server = MllpServer::bind_tls_with_config(
    ///     "0.0.0.0:2575",
    ///     tls_config,
    ///     mllp_config
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "tls")]
    pub async fn bind_tls_with_config(
        addr: &str,
        tls_config: crate::tls::TlsServerConfig,
        config: MllpConfig,
    ) -> Result<Self> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| Error::Network(format!("Failed to bind: {}", e)))?;

        let tls_acceptor = TlsAcceptor::from(tls_config.config.clone());

        Ok(Self {
            listener,
            config,
            tls_acceptor: Some(tls_acceptor),
        })
    }

    /// Accept a connection and return an MllpConnection with the server's configuration
    ///
    /// If the server was created with `bind_tls()` or `bind_tls_with_config()`, this will
    /// perform TLS handshake on the accepted connection.
    pub async fn accept(&self) -> Result<MllpConnection> {
        let (tcp_stream, _addr) = self
            .listener
            .accept()
            .await
            .map_err(|e| Error::Network(format!("Failed to accept: {}", e)))?;

        #[cfg(feature = "tls")]
        let stream = if let Some(ref acceptor) = self.tls_acceptor {
            // Perform TLS handshake
            let tls_stream = acceptor
                .accept(tcp_stream)
                .await
                .map_err(|e| Error::Network(format!("TLS handshake failed: {}", e)))?;
            MllpStream::TlsServer(tls_stream)
        } else {
            MllpStream::Plain(tcp_stream)
        };

        #[cfg(not(feature = "tls"))]
        let stream = MllpStream::Plain(tcp_stream);

        Ok(MllpConnection {
            stream,
            max_message_size: self.config.max_message_size,
            read_timeout: self.config.read_timeout,
            write_timeout: self.config.write_timeout,
        })
    }

    /// Get the local address
    pub fn local_addr(&self) -> Result<std::net::SocketAddr> {
        self.listener
            .local_addr()
            .map_err(|e| Error::Network(format!("Failed to get local addr: {}", e)))
    }
}

/// An MLLP connection
pub struct MllpConnection {
    stream: MllpStream,
    max_message_size: usize,
    read_timeout: Duration,
    write_timeout: Duration,
}

impl MllpConnection {
    /// Receive a message with timeout and size limit
    pub async fn receive_message(&mut self) -> Result<Message> {
        tokio::time::timeout(
            self.read_timeout,
            self.receive_message_internal()
        )
        .await
        .map_err(|_| Error::Network(format!("Read timeout after {:?}", self.read_timeout)))?
    }

    /// Internal method to receive a message with buffer size protection
    async fn receive_message_internal(&mut self) -> Result<Message> {
        let mut buffer = Vec::with_capacity(8192); // Pre-allocate reasonable size
        let mut byte = [0u8; 1];

        // Read until we find the start marker
        loop {
            self.stream
                .read_exact(&mut byte)
                .await
                .map_err(|e| Error::Network(format!("Failed to read: {}", e)))?;

            if byte[0] == START_OF_BLOCK {
                buffer.push(byte[0]);
                break;
            }
        }

        // Read until we find the end markers
        let mut found_end = false;
        while !found_end {
            // Check buffer size limit
            if buffer.len() >= self.max_message_size {
                return Err(Error::Mllp(format!(
                    "Message exceeds maximum size of {} bytes",
                    self.max_message_size
                )));
            }

            self.stream
                .read_exact(&mut byte)
                .await
                .map_err(|e| Error::Network(format!("Failed to read: {}", e)))?;

            buffer.push(byte[0]);

            // Check for end sequence (FS CR)
            if buffer.len() >= 2 {
                let len = buffer.len();
                if buffer[len - 2] == END_OF_BLOCK && buffer[len - 1] == CARRIAGE_RETURN {
                    found_end = true;
                }
            }
        }

        // Unwrap frame
        let hl7_text = MllpFrame::unwrap(&buffer)?;

        // Parse message
        parse_message(&hl7_text)
    }

    /// Send a message with timeout
    pub async fn send_message(&mut self, message: &Message) -> Result<()> {
        let hl7_text = message.encode();
        let framed = MllpFrame::wrap(&hl7_text);

        tokio::time::timeout(
            self.write_timeout,
            self.stream.write_all(&framed)
        )
        .await
        .map_err(|_| Error::Network(format!("Write timeout after {:?}", self.write_timeout)))?
        .map_err(|e| Error::Network(format!("Failed to send: {}", e)))
    }

    /// Close the connection
    pub async fn close(mut self) -> Result<()> {
        self.stream
            .shutdown()
            .await
            .map_err(|e| Error::Network(format!("Failed to close: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_frame() {
        let message = "MSH|^~\\&|TEST";
        let framed = MllpFrame::wrap(message);

        assert_eq!(framed[0], START_OF_BLOCK);
        assert_eq!(framed[framed.len() - 2], END_OF_BLOCK);
        assert_eq!(framed[framed.len() - 1], CARRIAGE_RETURN);
    }

    #[test]
    fn test_unwrap_frame() {
        let message = "MSH|^~\\&|TEST";
        let framed = MllpFrame::wrap(message);
        let unwrapped = MllpFrame::unwrap(&framed).unwrap();

        assert_eq!(unwrapped, message);
    }

    #[test]
    fn test_unwrap_invalid_frame() {
        let invalid = vec![0x00, 0x01, 0x02];
        assert!(MllpFrame::unwrap(&invalid).is_err());
    }

    #[test]
    fn test_unwrap_missing_start() {
        let mut framed = MllpFrame::wrap("TEST");
        framed[0] = 0x00; // Corrupt start marker
        assert!(MllpFrame::unwrap(&framed).is_err());
    }

    #[test]
    fn test_unwrap_missing_end() {
        let mut framed = MllpFrame::wrap("TEST");
        let len = framed.len();
        framed[len - 2] = 0x00; // Corrupt end marker
        assert!(MllpFrame::unwrap(&framed).is_err());
    }
}
