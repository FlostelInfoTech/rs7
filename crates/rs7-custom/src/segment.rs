//! Core trait and types for custom Z-segments

use crate::error::Result;
use rs7_core::Segment;

/// Trait for custom Z-segments
///
/// This trait must be implemented by all custom Z-segment types. It provides
/// methods for parsing segments from HL7 structures, encoding them back,
/// validation, and documentation.
///
/// # Example
///
/// ```rust,ignore
/// use rs7_custom::CustomSegment;
/// use rs7_core::Segment;
///
/// struct ZPV {
///     visit_type: String,
///     visit_number: String,
/// }
///
/// impl CustomSegment for ZPV {
///     fn segment_id() -> &'static str {
///         "ZPV"
///     }
///
///     fn from_segment(segment: &Segment) -> Result<Self> {
///         Ok(ZPV {
///             visit_type: segment.get_field(1)?.to_string(),
///             visit_number: segment.get_field(2)?.to_string(),
///         })
///     }
///
///     fn to_segment(&self) -> Segment {
///         let mut segment = Segment::new("ZPV");
///         segment.set_field_value(1, &self.visit_type)?;
///         segment.set_field_value(2, &self.visit_number)?;
///         segment
///     }
/// }
/// ```
pub trait CustomSegment: Send + Sync {
    /// Get the segment ID (e.g., "ZPV", "ZCU")
    ///
    /// This must start with 'Z' as per HL7 convention for custom segments.
    fn segment_id() -> &'static str
    where
        Self: Sized;

    /// Parse a custom segment from an HL7 segment structure
    ///
    /// # Arguments
    ///
    /// * `segment` - The HL7 segment to parse
    ///
    /// # Returns
    ///
    /// The parsed custom segment or an error if parsing fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required fields are missing
    /// - Field values are invalid
    /// - Segment structure doesn't match expected format
    fn from_segment(segment: &Segment) -> Result<Self>
    where
        Self: Sized;

    /// Convert the custom segment to an HL7 segment structure
    ///
    /// # Returns
    ///
    /// An HL7 segment that can be encoded to ER7 format
    fn to_segment(&self) -> Segment;

    /// Validate the segment's business rules
    ///
    /// This is called after parsing and can be used to implement custom
    /// validation logic beyond basic field presence checks.
    ///
    /// # Returns
    ///
    /// Ok(()) if validation passes, or an error describing what failed
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// fn validate(&self) -> Result<()> {
    ///     if self.balance.unwrap_or(0.0) < 0.0 {
    ///         return Err(CustomSegmentError::validation_failed(
    ///             "ZCU",
    ///             "Balance cannot be negative"
    ///         ));
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn validate(&self) -> Result<()> {
        Ok(()) // Default: no validation
    }

    /// Get field definitions for documentation purposes
    ///
    /// This is optional but recommended for generating documentation
    /// and supporting tools.
    ///
    /// # Returns
    ///
    /// A vector of field definitions describing each field in the segment
    fn field_definitions() -> Vec<FieldDefinition>
    where
        Self: Sized,
    {
        Vec::new() // Default: no definitions
    }

    /// Get the segment type name for debugging
    ///
    /// Returns the Rust type name by default, can be overridden for
    /// custom display.
    fn type_name() -> &'static str
    where
        Self: Sized,
    {
        std::any::type_name::<Self>()
    }
}

/// Definition of a field in a custom segment
///
/// This is used for documentation, validation, and tooling support.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinition {
    /// Field number (1-based)
    pub number: usize,

    /// Human-readable field name
    pub name: String,

    /// HL7 data type (e.g., "ST", "NM", "CWE")
    pub data_type: String,

    /// Whether this field is required
    pub required: bool,

    /// Whether this field can repeat
    pub repeatable: bool,

    /// Maximum length (None for unlimited)
    pub max_length: Option<usize>,

    /// Human-readable description
    pub description: String,
}

impl FieldDefinition {
    /// Create a new field definition builder
    pub fn builder() -> FieldDefinitionBuilder {
        FieldDefinitionBuilder::default()
    }
}

/// Builder for creating field definitions
#[derive(Default)]
pub struct FieldDefinitionBuilder {
    number: Option<usize>,
    name: Option<String>,
    data_type: Option<String>,
    required: bool,
    repeatable: bool,
    max_length: Option<usize>,
    description: Option<String>,
}

impl FieldDefinitionBuilder {
    /// Set the field number
    pub fn number(mut self, number: usize) -> Self {
        self.number = Some(number);
        self
    }

    /// Set the field name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the HL7 data type
    pub fn data_type(mut self, data_type: impl Into<String>) -> Self {
        self.data_type = Some(data_type.into());
        self
    }

    /// Mark field as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Mark field as optional
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Mark field as repeatable
    pub fn repeatable(mut self) -> Self {
        self.repeatable = true;
        self
    }

    /// Set maximum length
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Build the field definition
    pub fn build(self) -> FieldDefinition {
        FieldDefinition {
            number: self.number.expect("Field number is required"),
            name: self.name.expect("Field name is required"),
            data_type: self.data_type.expect("Data type is required"),
            required: self.required,
            repeatable: self.repeatable,
            max_length: self.max_length,
            description: self.description.unwrap_or_default(),
        }
    }
}

/// Trait for parsing field values from segments
pub trait ParseSegmentField: Sized {
    fn parse_field(segment: &Segment, field_num: usize, seg_id: &str) -> Result<Self>;
}

/// Trait for serializing field values to segments
pub trait SerializeSegmentField {
    fn set_field(&self, segment: &mut Segment, field_num: usize);
}

/// Trait to determine the inner type for builder fields
pub trait BuilderFieldType {
    type Inner;
}

/// Wrapper for builder fields
#[derive(Debug, Default)]
pub struct BuilderField<T: BuildableField> {
    value: T::Storage,
}

/// Trait for types that can be built from builder fields
pub trait BuildableField: Sized {
    type Storage: Default;
    fn set_value(storage: &mut Self::Storage, value: Self::Inner);
    fn build_value(storage: Self::Storage, field_name: &str, seg_id: &str) -> Result<Self>;
    type Inner;
}

impl<T: BuildableField> BuilderField<T> {
    pub fn set(&mut self, value: T::Inner) {
        T::set_value(&mut self.value, value);
    }

    pub fn build(self, field_name: &str, seg_id: &str) -> Result<T> {
        T::build_value(self.value, field_name, seg_id)
    }
}

// Implement BuildableField for required types
impl BuildableField for String {
    type Storage = Option<String>;
    type Inner = String;

    fn set_value(storage: &mut Self::Storage, value: Self::Inner) {
        *storage = Some(value);
    }

    fn build_value(storage: Self::Storage, field_name: &str, seg_id: &str) -> Result<Self> {
        storage.ok_or_else(|| {
            crate::error::CustomSegmentError::missing_field(
                format!("{}-{}", seg_id, field_name),
                seg_id,
            )
        })
    }
}

impl BuildableField for f64 {
    type Storage = Option<f64>;
    type Inner = f64;

    fn set_value(storage: &mut Self::Storage, value: Self::Inner) {
        *storage = Some(value);
    }

    fn build_value(storage: Self::Storage, field_name: &str, seg_id: &str) -> Result<Self> {
        storage.ok_or_else(|| {
            crate::error::CustomSegmentError::missing_field(
                format!("{}-{}", seg_id, field_name),
                seg_id,
            )
        })
    }
}

impl BuildableField for u32 {
    type Storage = Option<u32>;
    type Inner = u32;

    fn set_value(storage: &mut Self::Storage, value: Self::Inner) {
        *storage = Some(value);
    }

    fn build_value(storage: Self::Storage, field_name: &str, seg_id: &str) -> Result<Self> {
        storage.ok_or_else(|| {
            crate::error::CustomSegmentError::missing_field(
                format!("{}-{}", seg_id, field_name),
                seg_id,
            )
        })
    }
}

// Implement BuildableField for optional types
impl BuildableField for Option<String> {
    type Storage = Option<String>;
    type Inner = String;

    fn set_value(storage: &mut Self::Storage, value: Self::Inner) {
        *storage = Some(value);
    }

    fn build_value(storage: Self::Storage, _field_name: &str, _seg_id: &str) -> Result<Self> {
        Ok(storage)
    }
}

impl BuildableField for Option<f64> {
    type Storage = Option<f64>;
    type Inner = f64;

    fn set_value(storage: &mut Self::Storage, value: Self::Inner) {
        *storage = Some(value);
    }

    fn build_value(storage: Self::Storage, _field_name: &str, _seg_id: &str) -> Result<Self> {
        Ok(storage)
    }
}

impl BuildableField for Option<u32> {
    type Storage = Option<u32>;
    type Inner = u32;

    fn set_value(storage: &mut Self::Storage, value: Self::Inner) {
        *storage = Some(value);
    }

    fn build_value(storage: Self::Storage, _field_name: &str, _seg_id: &str) -> Result<Self> {
        Ok(storage)
    }
}

// Implementations for String
impl ParseSegmentField for String {
    fn parse_field(segment: &Segment, field_num: usize, seg_id: &str) -> Result<Self> {
        segment
            .get_field_value(field_num)
            .map(|s| s.to_string())
            .ok_or_else(|| {
                crate::error::CustomSegmentError::missing_field(
                    format!("{}-{}", seg_id, field_num),
                    seg_id,
                )
            })
    }
}

impl ParseSegmentField for Option<String> {
    fn parse_field(segment: &Segment, field_num: usize, _seg_id: &str) -> Result<Self> {
        Ok(segment.get_field_value(field_num).map(|s| s.to_string()))
    }
}

impl SerializeSegmentField for String {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        let _ = segment.set_field_value(field_num, self);
    }
}

impl SerializeSegmentField for Option<String> {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        if let Some(v) = self {
            let _ = segment.set_field_value(field_num, v);
        }
    }
}

impl BuilderFieldType for String {
    type Inner = String;
}

impl BuilderFieldType for Option<String> {
    type Inner = String;
}

// Implementations for f64
impl ParseSegmentField for f64 {
    fn parse_field(segment: &Segment, field_num: usize, seg_id: &str) -> Result<Self> {
        segment
            .get_field_value(field_num)
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| {
                crate::error::CustomSegmentError::missing_field(
                    format!("{}-{}", seg_id, field_num),
                    seg_id,
                )
            })
    }
}

impl ParseSegmentField for Option<f64> {
    fn parse_field(segment: &Segment, field_num: usize, _seg_id: &str) -> Result<Self> {
        Ok(segment
            .get_field_value(field_num)
            .and_then(|s| s.parse::<f64>().ok()))
    }
}

impl SerializeSegmentField for f64 {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        let _ = segment.set_field_value(field_num, &self.to_string());
    }
}

impl SerializeSegmentField for Option<f64> {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        if let Some(v) = self {
            let _ = segment.set_field_value(field_num, &v.to_string());
        }
    }
}

impl BuilderFieldType for f64 {
    type Inner = f64;
}

impl BuilderFieldType for Option<f64> {
    type Inner = f64;
}

// Implementations for u32
impl ParseSegmentField for u32 {
    fn parse_field(segment: &Segment, field_num: usize, seg_id: &str) -> Result<Self> {
        segment
            .get_field_value(field_num)
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| {
                crate::error::CustomSegmentError::missing_field(
                    format!("{}-{}", seg_id, field_num),
                    seg_id,
                )
            })
    }
}

impl ParseSegmentField for Option<u32> {
    fn parse_field(segment: &Segment, field_num: usize, _seg_id: &str) -> Result<Self> {
        Ok(segment
            .get_field_value(field_num)
            .and_then(|s| s.parse::<u32>().ok()))
    }
}

impl SerializeSegmentField for u32 {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        let _ = segment.set_field_value(field_num, &self.to_string());
    }
}

impl SerializeSegmentField for Option<u32> {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        if let Some(v) = self {
            let _ = segment.set_field_value(field_num, &v.to_string());
        }
    }
}

impl BuilderFieldType for u32 {
    type Inner = u32;
}

impl BuilderFieldType for Option<u32> {
    type Inner = u32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_definition_builder() {
        let field = FieldDefinition::builder()
            .number(1)
            .name("Visit Type")
            .data_type("ST")
            .required()
            .max_length(20)
            .description("Type of patient visit")
            .build();

        assert_eq!(field.number, 1);
        assert_eq!(field.name, "Visit Type");
        assert_eq!(field.data_type, "ST");
        assert!(field.required);
        assert!(!field.repeatable);
        assert_eq!(field.max_length, Some(20));
        assert_eq!(field.description, "Type of patient visit");
    }

    #[test]
    fn test_field_definition_optional() {
        let field = FieldDefinition::builder()
            .number(3)
            .name("Patient Class")
            .data_type("ST")
            .optional()
            .build();

        assert!(!field.required);
    }
}
