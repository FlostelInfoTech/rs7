//! Core data structures and types for HL7 v2.x message processing
//!
//! This crate provides the foundational types used throughout the rs7 library:
//! - Message structure hierarchy (Message, Segment, Field, Component, Subcomponent)
//! - Message builders for creating HL7 messages programmatically
//! - Encoding characters and delimiters
//! - Error types
//! - Common traits

pub mod builders;
pub mod delimiters;
pub mod encoding;
pub mod error;
pub mod field;
pub mod message;
pub mod segment;
pub mod types;

pub use delimiters::Delimiters;
pub use encoding::Encoding;
pub use error::{Error, Result};
pub use field::{Component, Field, Repetition, SubComponent};
pub use message::Message;
pub use segment::Segment;

use std::str::FromStr;

/// HL7 version enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Version {
    V2_3,
    V2_3_1,
    V2_4,
    V2_5,
    V2_5_1,
    V2_6,
    V2_7,
    V2_7_1,
}

impl Version {
    /// Parse version from string (e.g., "2.5" or "2.5.1")
    ///
    /// Note: This method is kept for backward compatibility.
    /// Consider using the `FromStr` trait implementation instead.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "2.3" => Some(Version::V2_3),
            "2.3.1" => Some(Version::V2_3_1),
            "2.4" => Some(Version::V2_4),
            "2.5" => Some(Version::V2_5),
            "2.5.1" => Some(Version::V2_5_1),
            "2.6" => Some(Version::V2_6),
            "2.7" => Some(Version::V2_7),
            "2.7.1" => Some(Version::V2_7_1),
            _ => None,
        }
    }

    /// Get version as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Version::V2_3 => "2.3",
            Version::V2_3_1 => "2.3.1",
            Version::V2_4 => "2.4",
            Version::V2_5 => "2.5",
            Version::V2_5_1 => "2.5.1",
            Version::V2_6 => "2.6",
            Version::V2_7 => "2.7",
            Version::V2_7_1 => "2.7.1",
        }
    }
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_str(s).ok_or_else(|| Error::UnsupportedVersion(format!("Unknown HL7 version: {}", s)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        assert_eq!(Version::from_str("2.5"), Some(Version::V2_5));
        assert_eq!(Version::from_str("2.7.1"), Some(Version::V2_7_1));
        assert_eq!(Version::from_str("invalid"), None);
    }

    #[test]
    fn test_version_as_str() {
        assert_eq!(Version::V2_5.as_str(), "2.5");
        assert_eq!(Version::V2_7_1.as_str(), "2.7.1");
    }
}
