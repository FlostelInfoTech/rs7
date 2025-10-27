//! MLLP (Minimal Lower Layer Protocol) support
//!
//! MLLP is a simple framing protocol used for transmitting HL7 messages over TCP.
//! Format: \<VT\>message\<FS\>\<CR\>
//! - VT (Vertical Tab): 0x0B - Start of block
//! - FS (File Separator): 0x1C - End of block
//! - CR (Carriage Return): 0x0D - End of message

use rs7_core::{
    error::{Error, Result},
    message::Message,
};
use rs7_parser::parse_message;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

/// MLLP frame markers
pub const START_OF_BLOCK: u8 = 0x0B; // Vertical Tab (VT)
pub const END_OF_BLOCK: u8 = 0x1C; // File Separator (FS)
pub const CARRIAGE_RETURN: u8 = 0x0D; // Carriage Return (CR)

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
    stream: TcpStream,
}

impl MllpClient {
    /// Connect to an MLLP server
    pub async fn connect(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| Error::Network(format!("Failed to connect: {}", e)))?;

        Ok(Self { stream })
    }

    /// Send a message and wait for acknowledgment
    pub async fn send_message(&mut self, message: &Message) -> Result<Message> {
        // Encode message
        let hl7_text = message.encode();

        // Wrap in MLLP frame
        let framed = MllpFrame::wrap(&hl7_text);

        // Send
        self.stream
            .write_all(&framed)
            .await
            .map_err(|e| Error::Network(format!("Failed to send: {}", e)))?;

        // Receive acknowledgment
        self.receive_message().await
    }

    /// Receive a message
    pub async fn receive_message(&mut self) -> Result<Message> {
        let mut buffer = Vec::new();
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
}

impl MllpServer {
    /// Bind to an address
    pub async fn bind(addr: &str) -> Result<Self> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| Error::Network(format!("Failed to bind: {}", e)))?;

        Ok(Self { listener })
    }

    /// Accept a connection
    pub async fn accept(&self) -> Result<MllpConnection> {
        let (stream, _addr) = self
            .listener
            .accept()
            .await
            .map_err(|e| Error::Network(format!("Failed to accept: {}", e)))?;

        Ok(MllpConnection { stream })
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
    stream: TcpStream,
}

impl MllpConnection {
    /// Receive a message
    pub async fn receive_message(&mut self) -> Result<Message> {
        let mut buffer = Vec::new();
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

    /// Send a message
    pub async fn send_message(&mut self, message: &Message) -> Result<()> {
        let hl7_text = message.encode();
        let framed = MllpFrame::wrap(&hl7_text);

        self.stream
            .write_all(&framed)
            .await
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
