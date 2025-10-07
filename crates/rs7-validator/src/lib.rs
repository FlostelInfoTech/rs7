//! Validation for HL7 messages
//!
//! This crate provides validation functionality for HL7 v2.x messages
//! against their respective standards (v2.3, v2.4, v2.5, v2.6, v2.7).
//!
//! ## Supported Message Schemas
//!
//! The validator includes 32 message schemas across 5 HL7 versions:
//! - **ADT** (17 schemas): A01-A13, A17, A28, A31, A40
//! - **SIU** (4 schemas): S12-S15 (Scheduling)
//! - **MDM** (3 schemas): T01, T02, T04 (Medical Documents)
//! - **DFT** (2 schemas): P03, P11 (Financial Transactions)
//! - **QRY** (3 schemas): A19, Q01, Q02 (Query Messages)
//! - **ORU** (1 schema): R01 (Observation Results)
//! - **ORM** (1 schema): O01 (Orders)
//! - **ACK** (1 schema): General Acknowledgment

pub mod schema_loader;

use rs7_core::{
    error::Result,
    message::Message,
    segment::Segment,
    Version,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use schema_loader::{load_schema, list_available_schemas};

/// Validation result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an error
    pub fn add_error(&mut self, error: ValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub location: String,
    pub message: String,
    pub error_type: ValidationErrorType,
}

impl ValidationError {
    pub fn new(location: String, message: String, error_type: ValidationErrorType) -> Self {
        Self {
            location,
            message,
            error_type,
        }
    }
}

/// Type of validation error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationErrorType {
    MissingRequiredField,
    InvalidDataType,
    InvalidLength,
    InvalidCardinality,
    InvalidValue,
    StructuralError,
}

/// Validation warning
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationWarning {
    pub location: String,
    pub message: String,
}

impl ValidationWarning {
    pub fn new(location: String, message: String) -> Self {
        Self { location, message }
    }
}

/// HL7 message validator
pub struct Validator {
    version: Version,
    schema: Option<MessageSchema>,
}

impl Validator {
    /// Create a new validator for the given version
    pub fn new(version: Version) -> Self {
        Self {
            version,
            schema: None,
        }
    }

    /// Create a validator with a custom schema
    pub fn with_schema(version: Version, schema: MessageSchema) -> Self {
        Self {
            version,
            schema: Some(schema),
        }
    }

    /// Create a validator with auto-loaded schema for a specific message type
    pub fn for_message_type(version: Version, message_type: &str, trigger_event: &str) -> Result<Self> {
        let schema = load_schema(version, message_type, trigger_event)?;
        Ok(Self {
            version,
            schema: Some(schema),
        })
    }

    /// Validate a message
    pub fn validate(&self, message: &Message) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Basic structure validation
        self.validate_structure(message, &mut result);

        // Schema-based validation (if schema is available)
        if let Some(schema) = &self.schema {
            self.validate_against_schema(message, schema, &mut result);
        }

        result
    }

    /// Validate basic message structure
    fn validate_structure(&self, message: &Message, result: &mut ValidationResult) {
        // Must have at least one segment (MSH)
        if message.segments.is_empty() {
            result.add_error(ValidationError::new(
                "Message".to_string(),
                "Message must contain at least one segment".to_string(),
                ValidationErrorType::StructuralError,
            ));
            return;
        }

        // First segment must be MSH
        if message.segments[0].id != "MSH" {
            result.add_error(ValidationError::new(
                "Segment[0]".to_string(),
                "First segment must be MSH".to_string(),
                ValidationErrorType::StructuralError,
            ));
        }

        // Validate segment IDs
        for (i, segment) in message.segments.iter().enumerate() {
            if let Err(e) = segment.validate_id() {
                result.add_error(ValidationError::new(
                    format!("Segment[{}]", i),
                    e.to_string(),
                    ValidationErrorType::StructuralError,
                ));
            }
        }

        // Check version matches
        if let Some(msg_version) = message.get_version() {
            if msg_version != self.version {
                result.add_warning(ValidationWarning::new(
                    "MSH-12".to_string(),
                    format!(
                        "Message version ({}) differs from validator version ({})",
                        msg_version.as_str(),
                        self.version.as_str()
                    ),
                ));
            }
        }
    }

    /// Validate against schema
    fn validate_against_schema(
        &self,
        message: &Message,
        schema: &MessageSchema,
        result: &mut ValidationResult,
    ) {
        // Validate each segment against schema
        for (i, segment) in message.segments.iter().enumerate() {
            if let Some(seg_def) = schema.segments.get(&segment.id) {
                self.validate_segment(segment, seg_def, i, result);
            }
        }

        // Check for required segments
        for (seg_id, seg_def) in &schema.segments {
            if seg_def.required {
                let found = message.segments.iter().any(|s| &s.id == seg_id);
                if !found {
                    result.add_error(ValidationError::new(
                        "Message".to_string(),
                        format!("Required segment {} is missing", seg_id),
                        ValidationErrorType::MissingRequiredField,
                    ));
                }
            }
        }
    }

    /// Validate a segment
    fn validate_segment(
        &self,
        segment: &Segment,
        definition: &SegmentDefinition,
        index: usize,
        result: &mut ValidationResult,
    ) {
        let location_prefix = format!("{}[{}]", segment.id, index);

        // Validate each field
        for (field_idx, field_def) in &definition.fields {
            let field = segment.get_field(*field_idx);

            if field_def.required && (field.is_none() || field.unwrap().is_empty()) {
                result.add_error(ValidationError::new(
                    format!("{}-{}", location_prefix, field_idx),
                    format!("Required field {} is missing or empty", field_idx),
                    ValidationErrorType::MissingRequiredField,
                ));
            }

            // Validate max length if field exists
            if let Some(f) = field {
                if let Some(max_len) = field_def.max_length {
                    if let Some(value) = f.value() {
                        if value.len() > max_len {
                            result.add_error(ValidationError::new(
                                format!("{}-{}", location_prefix, field_idx),
                                format!(
                                    "Field exceeds maximum length ({} > {})",
                                    value.len(),
                                    max_len
                                ),
                                ValidationErrorType::InvalidLength,
                            ));
                        }
                    }
                }
            }
        }
    }
}

/// Message schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSchema {
    pub message_type: String,
    pub trigger_event: String,
    pub version: String,
    pub segments: HashMap<String, SegmentDefinition>,
}

/// Segment definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentDefinition {
    pub name: String,
    pub required: bool,
    pub repeating: bool,
    pub fields: HashMap<usize, FieldDefinition>,
}

/// Field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub data_type: String,
    pub required: bool,
    pub repeating: bool,
    pub max_length: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rs7_core::{field::Field, segment::Segment};

    fn create_test_message() -> Message {
        let mut msg = Message::new();

        let mut msh = Segment::new("MSH");
        msh.add_field(Field::from_value("^~\\&"));
        msh.add_field(Field::from_value(""));
        msh.add_field(Field::from_value("SendApp"));
        msg.add_segment(msh);

        let mut pid = Segment::new("PID");
        pid.add_field(Field::from_value("1"));
        msg.add_segment(pid);

        msg
    }

    #[test]
    fn test_validate_valid_message() {
        let msg = create_test_message();
        let validator = Validator::new(Version::V2_5);
        let result = validator.validate(&msg);

        assert!(result.is_valid());
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_empty_message() {
        let msg = Message::new();
        let validator = Validator::new(Version::V2_5);
        let result = validator.validate(&msg);

        assert!(!result.is_valid());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_missing_msh() {
        let mut msg = Message::new();
        msg.add_segment(Segment::new("PID"));

        let validator = Validator::new(Version::V2_5);
        let result = validator.validate(&msg);

        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.message.contains("MSH")));
    }

    #[test]
    fn test_validate_invalid_segment_id() {
        let mut msg = Message::new();
        msg.add_segment(Segment::new("MSH"));
        msg.add_segment(Segment::new("X")); // Invalid ID (too short)

        let validator = Validator::new(Version::V2_5);
        let result = validator.validate(&msg);

        assert!(!result.is_valid());
    }
}
