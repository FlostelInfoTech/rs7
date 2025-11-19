//! Conformance profile data structures

pub mod parser;

use crate::error::Result;
use rs7_core::Version;

/// Root conformance profile structure
#[derive(Debug, Clone)]
pub struct ConformanceProfile {
    /// Profile metadata
    pub metadata: ProfileMetadata,
    /// Message profile (segments and fields)
    pub message: MessageProfile,
}

impl ConformanceProfile {
    /// Create a new conformance profile
    pub fn new(metadata: ProfileMetadata, message: MessageProfile) -> Self {
        Self { metadata, message }
    }

    /// Validate the profile structure itself
    pub fn validate(&self) -> Result<()> {
        // TODO: Implement profile validation
        Ok(())
    }
}

/// Profile metadata
#[derive(Debug, Clone)]
pub struct ProfileMetadata {
    /// Profile name
    pub name: String,
    /// Profile version
    pub version: String,
    /// Organization name
    pub organization: Option<String>,
    /// HL7 version this profile is based on
    pub hl7_version: Version,
    /// Creation/modification date
    pub date: Option<String>,
    /// Profile description
    pub description: Option<String>,
}

impl ProfileMetadata {
    /// Create new profile metadata
    pub fn new(name: String, version: String, hl7_version: Version) -> Self {
        Self {
            name,
            version,
            organization: None,
            hl7_version,
            date: None,
            description: None,
        }
    }
}

/// Message profile with segment structure
#[derive(Debug, Clone)]
pub struct MessageProfile {
    /// Message type (e.g., "ADT")
    pub message_type: String,
    /// Trigger event (e.g., "A01")
    pub trigger_event: String,
    /// Segment profiles in order
    pub segments: Vec<SegmentProfile>,
}

impl MessageProfile {
    /// Create new message profile
    pub fn new(message_type: String, trigger_event: String) -> Self {
        Self {
            message_type,
            trigger_event,
            segments: Vec::new(),
        }
    }

    /// Add a segment profile
    pub fn add_segment(&mut self, segment: SegmentProfile) {
        self.segments.push(segment);
    }
}

/// Segment profile with field constraints
#[derive(Debug, Clone)]
pub struct SegmentProfile {
    /// Segment ID (e.g., "MSH", "PID")
    pub name: String,
    /// Long name/description
    pub long_name: Option<String>,
    /// Usage code (R, RE, O, X)
    pub usage: Usage,
    /// Cardinality (min/max occurrences)
    pub cardinality: Cardinality,
    /// Field profiles
    pub fields: Vec<FieldProfile>,
}

impl SegmentProfile {
    /// Create new segment profile
    pub fn new(name: String, usage: Usage, cardinality: Cardinality) -> Self {
        Self {
            name,
            long_name: None,
            usage,
            cardinality,
            fields: Vec::new(),
        }
    }

    /// Add a field profile
    pub fn add_field(&mut self, field: FieldProfile) {
        self.fields.push(field);
    }

    /// Get field by position
    pub fn get_field(&self, position: usize) -> Option<&FieldProfile> {
        self.fields.iter().find(|f| f.position == position)
    }
}

/// Field profile with constraints
#[derive(Debug, Clone)]
pub struct FieldProfile {
    /// Field position (1-based)
    pub position: usize,
    /// Field name
    pub name: Option<String>,
    /// Usage code (R, RE, O, X)
    pub usage: Usage,
    /// Cardinality (min/max occurrences for repeating fields)
    pub cardinality: Cardinality,
    /// Data type (e.g., "ST", "CX", "TS")
    pub datatype: Option<String>,
    /// Maximum length
    pub length: Option<usize>,
    /// HL7 table ID (e.g., "0001")
    pub table_id: Option<String>,
}

impl FieldProfile {
    /// Create new field profile
    pub fn new(position: usize, usage: Usage, cardinality: Cardinality) -> Self {
        Self {
            position,
            name: None,
            usage,
            cardinality,
            datatype: None,
            length: None,
            table_id: None,
        }
    }
}

/// Usage codes (governs expected application behavior)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Usage {
    /// R - Required: Must be present, error if missing
    Required,
    /// RE - Required if Known: Must send if known, no error if unknown
    RequiredIfKnown,
    /// O - Optional: May be present
    Optional,
    /// X - Not Used: Must not be present
    NotUsed,
}

impl Usage {
    /// Parse usage code from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "R" => Ok(Usage::Required),
            "RE" => Ok(Usage::RequiredIfKnown),
            "O" => Ok(Usage::Optional),
            "X" => Ok(Usage::NotUsed),
            _ => Err(crate::error::ConformanceError::InvalidUsage(s.to_string())),
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Usage::Required => "R",
            Usage::RequiredIfKnown => "RE",
            Usage::Optional => "O",
            Usage::NotUsed => "X",
        }
    }

    /// Check if the element is required (R or RE)
    pub fn is_required(&self) -> bool {
        matches!(self, Usage::Required | Usage::RequiredIfKnown)
    }

    /// Check if the element must not be present
    pub fn is_not_used(&self) -> bool {
        matches!(self, Usage::NotUsed)
    }
}

/// Cardinality specification (min/max occurrences)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cardinality {
    /// Minimum occurrences
    pub min: usize,
    /// Maximum occurrences (None = unbounded)
    pub max: Option<usize>,
}

impl Cardinality {
    /// Create new cardinality
    pub fn new(min: usize, max: Option<usize>) -> Result<Self> {
        if let Some(max_val) = max {
            if min > max_val {
                return Err(crate::error::ConformanceError::InvalidCardinality(
                    format!("min ({}) > max ({})", min, max_val),
                ));
            }
        }
        Ok(Self { min, max })
    }

    /// Create cardinality from [min..max] notation
    pub fn from_range(min: usize, max: Option<usize>) -> Result<Self> {
        Self::new(min, max)
    }

    /// Exactly one occurrence [1..1]
    pub fn one() -> Self {
        Self { min: 1, max: Some(1) }
    }

    /// Zero or one occurrence [0..1]
    pub fn zero_or_one() -> Self {
        Self { min: 0, max: Some(1) }
    }

    /// Zero or more occurrences [0..*]
    pub fn zero_or_more() -> Self {
        Self { min: 0, max: None }
    }

    /// One or more occurrences [1..*]
    pub fn one_or_more() -> Self {
        Self { min: 1, max: None }
    }

    /// Check if a count satisfies this cardinality
    pub fn satisfies(&self, count: usize) -> bool {
        if count < self.min {
            return false;
        }
        if let Some(max) = self.max {
            if count > max {
                return false;
            }
        }
        true
    }

    /// Format as string (e.g., "[1..1]", "[0..*]")
    pub fn to_string(&self) -> String {
        match self.max {
            Some(max) => format!("[{}..{}]", self.min, max),
            None => format!("[{}..*]", self.min),
        }
    }
}

impl Default for Cardinality {
    fn default() -> Self {
        Self::one()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_from_str() {
        assert_eq!(Usage::from_str("R").unwrap(), Usage::Required);
        assert_eq!(Usage::from_str("RE").unwrap(), Usage::RequiredIfKnown);
        assert_eq!(Usage::from_str("O").unwrap(), Usage::Optional);
        assert_eq!(Usage::from_str("X").unwrap(), Usage::NotUsed);
        assert_eq!(Usage::from_str("r").unwrap(), Usage::Required); // case insensitive
        assert!(Usage::from_str("INVALID").is_err());
    }

    #[test]
    fn test_usage_as_str() {
        assert_eq!(Usage::Required.as_str(), "R");
        assert_eq!(Usage::RequiredIfKnown.as_str(), "RE");
        assert_eq!(Usage::Optional.as_str(), "O");
        assert_eq!(Usage::NotUsed.as_str(), "X");
    }

    #[test]
    fn test_usage_is_required() {
        assert!(Usage::Required.is_required());
        assert!(Usage::RequiredIfKnown.is_required());
        assert!(!Usage::Optional.is_required());
        assert!(!Usage::NotUsed.is_required());
    }

    #[test]
    fn test_cardinality_creation() {
        let card = Cardinality::new(1, Some(1)).unwrap();
        assert_eq!(card.min, 1);
        assert_eq!(card.max, Some(1));

        let card = Cardinality::one();
        assert_eq!(card.min, 1);
        assert_eq!(card.max, Some(1));

        let card = Cardinality::zero_or_more();
        assert_eq!(card.min, 0);
        assert_eq!(card.max, None);
    }

    #[test]
    fn test_cardinality_satisfies() {
        let one = Cardinality::one();
        assert!(!one.satisfies(0));
        assert!(one.satisfies(1));
        assert!(!one.satisfies(2));

        let zero_or_more = Cardinality::zero_or_more();
        assert!(zero_or_more.satisfies(0));
        assert!(zero_or_more.satisfies(1));
        assert!(zero_or_more.satisfies(100));

        let range = Cardinality::new(2, Some(5)).unwrap();
        assert!(!range.satisfies(1));
        assert!(range.satisfies(2));
        assert!(range.satisfies(5));
        assert!(!range.satisfies(6));
    }

    #[test]
    fn test_cardinality_invalid() {
        let result = Cardinality::new(5, Some(3));
        assert!(result.is_err());
    }
}
