//! Core trait and types for custom Z-segments

use crate::error::Result;
use rs7_core::Segment;
use chrono::{NaiveDateTime, NaiveDate, NaiveTime, DateTime, Utc};

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

// Implementations for i32
impl BuildableField for i32 {
    type Storage = Option<i32>;
    type Inner = i32;

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

impl BuildableField for Option<i32> {
    type Storage = Option<i32>;
    type Inner = i32;

    fn set_value(storage: &mut Self::Storage, value: Self::Inner) {
        *storage = Some(value);
    }

    fn build_value(storage: Self::Storage, _field_name: &str, _seg_id: &str) -> Result<Self> {
        Ok(storage)
    }
}

impl ParseSegmentField for i32 {
    fn parse_field(segment: &Segment, field_num: usize, seg_id: &str) -> Result<Self> {
        segment
            .get_field_value(field_num)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| {
                crate::error::CustomSegmentError::missing_field(
                    format!("{}-{}", seg_id, field_num),
                    seg_id,
                )
            })
    }
}

impl ParseSegmentField for Option<i32> {
    fn parse_field(segment: &Segment, field_num: usize, _seg_id: &str) -> Result<Self> {
        Ok(segment
            .get_field_value(field_num)
            .and_then(|s| s.parse::<i32>().ok()))
    }
}

impl SerializeSegmentField for i32 {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        let _ = segment.set_field_value(field_num, &self.to_string());
    }
}

impl SerializeSegmentField for Option<i32> {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        if let Some(v) = self {
            let _ = segment.set_field_value(field_num, &v.to_string());
        }
    }
}

impl BuilderFieldType for i32 {
    type Inner = i32;
}

impl BuilderFieldType for Option<i32> {
    type Inner = i32;
}

// Implementations for i64
impl BuildableField for i64 {
    type Storage = Option<i64>;
    type Inner = i64;

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

impl BuildableField for Option<i64> {
    type Storage = Option<i64>;
    type Inner = i64;

    fn set_value(storage: &mut Self::Storage, value: Self::Inner) {
        *storage = Some(value);
    }

    fn build_value(storage: Self::Storage, _field_name: &str, _seg_id: &str) -> Result<Self> {
        Ok(storage)
    }
}

impl ParseSegmentField for i64 {
    fn parse_field(segment: &Segment, field_num: usize, seg_id: &str) -> Result<Self> {
        segment
            .get_field_value(field_num)
            .and_then(|s| s.parse::<i64>().ok())
            .ok_or_else(|| {
                crate::error::CustomSegmentError::missing_field(
                    format!("{}-{}", seg_id, field_num),
                    seg_id,
                )
            })
    }
}

impl ParseSegmentField for Option<i64> {
    fn parse_field(segment: &Segment, field_num: usize, _seg_id: &str) -> Result<Self> {
        Ok(segment
            .get_field_value(field_num)
            .and_then(|s| s.parse::<i64>().ok()))
    }
}

impl SerializeSegmentField for i64 {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        let _ = segment.set_field_value(field_num, &self.to_string());
    }
}

impl SerializeSegmentField for Option<i64> {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        if let Some(v) = self {
            let _ = segment.set_field_value(field_num, &v.to_string());
        }
    }
}

impl BuilderFieldType for i64 {
    type Inner = i64;
}

impl BuilderFieldType for Option<i64> {
    type Inner = i64;
}

// Implementations for bool
impl BuildableField for bool {
    type Storage = Option<bool>;
    type Inner = bool;

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

impl BuildableField for Option<bool> {
    type Storage = Option<bool>;
    type Inner = bool;

    fn set_value(storage: &mut Self::Storage, value: Self::Inner) {
        *storage = Some(value);
    }

    fn build_value(storage: Self::Storage, _field_name: &str, _seg_id: &str) -> Result<Self> {
        Ok(storage)
    }
}

impl ParseSegmentField for bool {
    fn parse_field(segment: &Segment, field_num: usize, seg_id: &str) -> Result<Self> {
        segment
            .get_field_value(field_num)
            .and_then(|s| {
                // HL7 typically uses Y/N or 1/0 for booleans
                match s.to_uppercase().as_str() {
                    "Y" | "YES" | "T" | "TRUE" | "1" => Some(true),
                    "N" | "NO" | "F" | "FALSE" | "0" => Some(false),
                    _ => None,
                }
            })
            .ok_or_else(|| {
                crate::error::CustomSegmentError::missing_field(
                    format!("{}-{}", seg_id, field_num),
                    seg_id,
                )
            })
    }
}

impl ParseSegmentField for Option<bool> {
    fn parse_field(segment: &Segment, field_num: usize, _seg_id: &str) -> Result<Self> {
        Ok(segment.get_field_value(field_num).and_then(|s| {
            match s.to_uppercase().as_str() {
                "Y" | "YES" | "T" | "TRUE" | "1" => Some(true),
                "N" | "NO" | "F" | "FALSE" | "0" => Some(false),
                _ => None,
            }
        }))
    }
}

impl SerializeSegmentField for bool {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        let value = if *self { "Y" } else { "N" };
        let _ = segment.set_field_value(field_num, value);
    }
}

impl SerializeSegmentField for Option<bool> {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        if let Some(v) = self {
            let value = if *v { "Y" } else { "N" };
            let _ = segment.set_field_value(field_num, value);
        }
    }
}

impl BuilderFieldType for bool {
    type Inner = bool;
}

impl BuilderFieldType for Option<bool> {
    type Inner = bool;
}

// ============================================================================
// NaiveDateTime implementations (timestamp without timezone)
// ============================================================================

// BuildableField for required NaiveDateTime
impl BuildableField for NaiveDateTime {
    type Storage = Option<NaiveDateTime>;
    type Inner = NaiveDateTime;

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

// BuildableField for optional NaiveDateTime
impl BuildableField for Option<NaiveDateTime> {
    type Storage = Option<NaiveDateTime>;
    type Inner = NaiveDateTime;

    fn set_value(storage: &mut Self::Storage, value: Self::Inner) {
        *storage = Some(value);
    }

    fn build_value(storage: Self::Storage, _field_name: &str, _seg_id: &str) -> Result<Self> {
        Ok(storage)
    }
}

// ParseSegmentField for NaiveDateTime
// Supports HL7 datetime formats: YYYYMMDDHHMMSS[.SSSS]
impl ParseSegmentField for NaiveDateTime {
    fn parse_field(segment: &Segment, field_num: usize, seg_id: &str) -> Result<Self> {
        segment
            .get_field_value(field_num)
            .and_then(|s| parse_hl7_datetime(s))
            .ok_or_else(|| {
                crate::error::CustomSegmentError::missing_field(
                    format!("{}-{}", seg_id, field_num),
                    seg_id,
                )
            })
    }
}

// ParseSegmentField for Option<NaiveDateTime>
impl ParseSegmentField for Option<NaiveDateTime> {
    fn parse_field(segment: &Segment, field_num: usize, _seg_id: &str) -> Result<Self> {
        Ok(segment.get_field_value(field_num).and_then(parse_hl7_datetime))
    }
}

// SerializeSegmentField for NaiveDateTime
impl SerializeSegmentField for NaiveDateTime {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        let formatted = self.format("%Y%m%d%H%M%S").to_string();
        let _ = segment.set_field_value(field_num, &formatted);
    }
}

// SerializeSegmentField for Option<NaiveDateTime>
impl SerializeSegmentField for Option<NaiveDateTime> {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        if let Some(dt) = self {
            dt.set_field(segment, field_num);
        }
    }
}

// BuilderFieldType for NaiveDateTime
impl BuilderFieldType for NaiveDateTime {
    type Inner = NaiveDateTime;
}

// BuilderFieldType for Option<NaiveDateTime>
impl BuilderFieldType for Option<NaiveDateTime> {
    type Inner = NaiveDateTime;
}

// ============================================================================
// NaiveDate implementations (date without time)
// ============================================================================

// BuildableField for required NaiveDate
impl BuildableField for NaiveDate {
    type Storage = Option<NaiveDate>;
    type Inner = NaiveDate;

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

// BuildableField for optional NaiveDate
impl BuildableField for Option<NaiveDate> {
    type Storage = Option<NaiveDate>;
    type Inner = NaiveDate;

    fn set_value(storage: &mut Self::Storage, value: Self::Inner) {
        *storage = Some(value);
    }

    fn build_value(storage: Self::Storage, _field_name: &str, _seg_id: &str) -> Result<Self> {
        Ok(storage)
    }
}

// ParseSegmentField for NaiveDate
// Supports HL7 date formats: YYYYMMDD, YYYYMM, YYYY
impl ParseSegmentField for NaiveDate {
    fn parse_field(segment: &Segment, field_num: usize, seg_id: &str) -> Result<Self> {
        segment
            .get_field_value(field_num)
            .and_then(|s| parse_hl7_date(s))
            .ok_or_else(|| {
                crate::error::CustomSegmentError::missing_field(
                    format!("{}-{}", seg_id, field_num),
                    seg_id,
                )
            })
    }
}

// ParseSegmentField for Option<NaiveDate>
impl ParseSegmentField for Option<NaiveDate> {
    fn parse_field(segment: &Segment, field_num: usize, _seg_id: &str) -> Result<Self> {
        Ok(segment.get_field_value(field_num).and_then(parse_hl7_date))
    }
}

// SerializeSegmentField for NaiveDate
impl SerializeSegmentField for NaiveDate {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        let formatted = self.format("%Y%m%d").to_string();
        let _ = segment.set_field_value(field_num, &formatted);
    }
}

// SerializeSegmentField for Option<NaiveDate>
impl SerializeSegmentField for Option<NaiveDate> {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        if let Some(date) = self {
            date.set_field(segment, field_num);
        }
    }
}

// BuilderFieldType for NaiveDate
impl BuilderFieldType for NaiveDate {
    type Inner = NaiveDate;
}

// BuilderFieldType for Option<NaiveDate>
impl BuilderFieldType for Option<NaiveDate> {
    type Inner = NaiveDate;
}

// ============================================================================
// NaiveTime implementations (time without date)
// ============================================================================

// BuildableField for required NaiveTime
impl BuildableField for NaiveTime {
    type Storage = Option<NaiveTime>;
    type Inner = NaiveTime;

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

// BuildableField for optional NaiveTime
impl BuildableField for Option<NaiveTime> {
    type Storage = Option<NaiveTime>;
    type Inner = NaiveTime;

    fn set_value(storage: &mut Self::Storage, value: Self::Inner) {
        *storage = Some(value);
    }

    fn build_value(storage: Self::Storage, _field_name: &str, _seg_id: &str) -> Result<Self> {
        Ok(storage)
    }
}

// ParseSegmentField for NaiveTime
// Supports HL7 time formats: HHMMSS[.SSSS], HHMM
impl ParseSegmentField for NaiveTime {
    fn parse_field(segment: &Segment, field_num: usize, seg_id: &str) -> Result<Self> {
        segment
            .get_field_value(field_num)
            .and_then(|s| parse_hl7_time(s))
            .ok_or_else(|| {
                crate::error::CustomSegmentError::missing_field(
                    format!("{}-{}", seg_id, field_num),
                    seg_id,
                )
            })
    }
}

// ParseSegmentField for Option<NaiveTime>
impl ParseSegmentField for Option<NaiveTime> {
    fn parse_field(segment: &Segment, field_num: usize, _seg_id: &str) -> Result<Self> {
        Ok(segment.get_field_value(field_num).and_then(parse_hl7_time))
    }
}

// SerializeSegmentField for NaiveTime
impl SerializeSegmentField for NaiveTime {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        let formatted = self.format("%H%M%S").to_string();
        let _ = segment.set_field_value(field_num, &formatted);
    }
}

// SerializeSegmentField for Option<NaiveTime>
impl SerializeSegmentField for Option<NaiveTime> {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        if let Some(time) = self {
            time.set_field(segment, field_num);
        }
    }
}

// BuilderFieldType for NaiveTime
impl BuilderFieldType for NaiveTime {
    type Inner = NaiveTime;
}

// BuilderFieldType for Option<NaiveTime>
impl BuilderFieldType for Option<NaiveTime> {
    type Inner = NaiveTime;
}

// ============================================================================
// DateTime<Utc> implementations (timezone-aware timestamp)
// ============================================================================

// BuildableField for required DateTime<Utc>
impl BuildableField for DateTime<Utc> {
    type Storage = Option<DateTime<Utc>>;
    type Inner = DateTime<Utc>;

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

// BuildableField for optional DateTime<Utc>
impl BuildableField for Option<DateTime<Utc>> {
    type Storage = Option<DateTime<Utc>>;
    type Inner = DateTime<Utc>;

    fn set_value(storage: &mut Self::Storage, value: Self::Inner) {
        *storage = Some(value);
    }

    fn build_value(storage: Self::Storage, _field_name: &str, _seg_id: &str) -> Result<Self> {
        Ok(storage)
    }
}

// ParseSegmentField for DateTime<Utc>
// Supports HL7 datetime formats with timezone: YYYYMMDDHHMMSS[.SSSS][+/-ZZZZ]
impl ParseSegmentField for DateTime<Utc> {
    fn parse_field(segment: &Segment, field_num: usize, seg_id: &str) -> Result<Self> {
        segment
            .get_field_value(field_num)
            .and_then(|s| parse_hl7_datetime_utc(s))
            .ok_or_else(|| {
                crate::error::CustomSegmentError::missing_field(
                    format!("{}-{}", seg_id, field_num),
                    seg_id,
                )
            })
    }
}

// ParseSegmentField for Option<DateTime<Utc>>
impl ParseSegmentField for Option<DateTime<Utc>> {
    fn parse_field(segment: &Segment, field_num: usize, _seg_id: &str) -> Result<Self> {
        Ok(segment.get_field_value(field_num).and_then(parse_hl7_datetime_utc))
    }
}

// SerializeSegmentField for DateTime<Utc>
impl SerializeSegmentField for DateTime<Utc> {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        let formatted = self.format("%Y%m%d%H%M%S").to_string();
        let _ = segment.set_field_value(field_num, &formatted);
    }
}

// SerializeSegmentField for Option<DateTime<Utc>>
impl SerializeSegmentField for Option<DateTime<Utc>> {
    fn set_field(&self, segment: &mut Segment, field_num: usize) {
        if let Some(dt) = self {
            dt.set_field(segment, field_num);
        }
    }
}

// BuilderFieldType for DateTime<Utc>
impl BuilderFieldType for DateTime<Utc> {
    type Inner = DateTime<Utc>;
}

// BuilderFieldType for Option<DateTime<Utc>>
impl BuilderFieldType for Option<DateTime<Utc>> {
    type Inner = DateTime<Utc>;
}

// ============================================================================
// Helper functions for parsing HL7 date/time formats
// ============================================================================

/// Parse HL7 datetime format: YYYYMMDDHHMMSS[.SSSS]
fn parse_hl7_datetime(s: &str) -> Option<NaiveDateTime> {
    // Try full format with optional fractional seconds
    if s.len() >= 14 {
        let year = s[0..4].parse::<i32>().ok()?;
        let month = s[4..6].parse::<u32>().ok()?;
        let day = s[6..8].parse::<u32>().ok()?;
        let hour = s[8..10].parse::<u32>().ok()?;
        let minute = s[10..12].parse::<u32>().ok()?;
        let second = s[12..14].parse::<u32>().ok()?;

        let date = NaiveDate::from_ymd_opt(year, month, day)?;
        let time = NaiveTime::from_hms_opt(hour, minute, second)?;

        return Some(NaiveDateTime::new(date, time));
    }

    None
}

/// Parse HL7 date format: YYYYMMDD, YYYYMM, or YYYY
fn parse_hl7_date(s: &str) -> Option<NaiveDate> {
    match s.len() {
        8 => {
            // YYYYMMDD
            let year = s[0..4].parse::<i32>().ok()?;
            let month = s[4..6].parse::<u32>().ok()?;
            let day = s[6..8].parse::<u32>().ok()?;
            NaiveDate::from_ymd_opt(year, month, day)
        }
        6 => {
            // YYYYMM - default to first day of month
            let year = s[0..4].parse::<i32>().ok()?;
            let month = s[4..6].parse::<u32>().ok()?;
            NaiveDate::from_ymd_opt(year, month, 1)
        }
        4 => {
            // YYYY - default to January 1st
            let year = s[0..4].parse::<i32>().ok()?;
            NaiveDate::from_ymd_opt(year, 1, 1)
        }
        _ => None,
    }
}

/// Parse HL7 time format: HHMMSS[.SSSS] or HHMM
fn parse_hl7_time(s: &str) -> Option<NaiveTime> {
    if s.len() >= 6 {
        // HHMMSS format
        let hour = s[0..2].parse::<u32>().ok()?;
        let minute = s[2..4].parse::<u32>().ok()?;
        let second = s[4..6].parse::<u32>().ok()?;
        NaiveTime::from_hms_opt(hour, minute, second)
    } else if s.len() == 4 {
        // HHMM format
        let hour = s[0..2].parse::<u32>().ok()?;
        let minute = s[2..4].parse::<u32>().ok()?;
        NaiveTime::from_hms_opt(hour, minute, 0)
    } else {
        None
    }
}

/// Parse HL7 datetime with timezone: YYYYMMDDHHMMSS[+/-ZZZZ]
fn parse_hl7_datetime_utc(s: &str) -> Option<DateTime<Utc>> {
    use chrono::TimeZone;

    // For simplicity, parse as naive datetime and convert to UTC
    // Full timezone support would require parsing +/-ZZZZ offset
    let naive_dt = parse_hl7_datetime(s)?;
    Some(Utc.from_utc_datetime(&naive_dt))
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

    // Tests for i32 type
    #[test]
    fn test_i32_parse_positive() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "42").unwrap();

        let result = i32::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_i32_parse_negative() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "-123").unwrap();

        let result = i32::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, -123);
    }

    #[test]
    fn test_i32_parse_zero() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "0").unwrap();

        let result = i32::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_i32_serialize() {
        let mut segment = Segment::new("TEST");
        let value: i32 = -456;

        value.set_field(&mut segment, 1);
        assert_eq!(segment.get_field_value(1).unwrap(), "-456");
    }

    #[test]
    fn test_i32_roundtrip() {
        let mut segment = Segment::new("TEST");
        let original: i32 = 789;

        original.set_field(&mut segment, 1);
        let parsed = i32::parse_field(&segment, 1, "TEST").unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_option_i32_parse_some() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "100").unwrap();

        let result = Option::<i32>::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, Some(100));
    }

    #[test]
    fn test_option_i32_parse_none() {
        let segment = Segment::new("TEST");

        let result = Option::<i32>::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, None);
    }

    // Tests for i64 type
    #[test]
    fn test_i64_parse_large_positive() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "9223372036854775807").unwrap(); // i64::MAX

        let result = i64::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, 9223372036854775807);
    }

    #[test]
    fn test_i64_parse_large_negative() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "-9223372036854775808").unwrap(); // i64::MIN

        let result = i64::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, -9223372036854775808);
    }

    #[test]
    fn test_i64_serialize() {
        let mut segment = Segment::new("TEST");
        let value: i64 = 123456789012345;

        value.set_field(&mut segment, 1);
        assert_eq!(segment.get_field_value(1).unwrap(), "123456789012345");
    }

    #[test]
    fn test_i64_roundtrip() {
        let mut segment = Segment::new("TEST");
        let original: i64 = -987654321098765;

        original.set_field(&mut segment, 1);
        let parsed = i64::parse_field(&segment, 1, "TEST").unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_option_i64_parse_some() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "555555").unwrap();

        let result = Option::<i64>::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, Some(555555));
    }

    #[test]
    fn test_option_i64_parse_none() {
        let segment = Segment::new("TEST");

        let result = Option::<i64>::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, None);
    }

    // Tests for bool type - parsing all supported formats
    #[test]
    fn test_bool_parse_y() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "Y").unwrap();

        let result = bool::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_bool_parse_yes() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "YES").unwrap();

        let result = bool::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_bool_parse_t() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "T").unwrap();

        let result = bool::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_bool_parse_true() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "TRUE").unwrap();

        let result = bool::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_bool_parse_one() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "1").unwrap();

        let result = bool::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_bool_parse_n() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "N").unwrap();

        let result = bool::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_bool_parse_no() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "NO").unwrap();

        let result = bool::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_bool_parse_f() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "F").unwrap();

        let result = bool::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_bool_parse_false() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "FALSE").unwrap();

        let result = bool::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_bool_parse_zero() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "0").unwrap();

        let result = bool::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_bool_parse_case_insensitive() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "yes").unwrap();

        let result = bool::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, true);

        segment.set_field_value(1, "No").unwrap();
        let result = bool::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn test_bool_serialize_true() {
        let mut segment = Segment::new("TEST");
        let value = true;

        value.set_field(&mut segment, 1);
        assert_eq!(segment.get_field_value(1).unwrap(), "Y");
    }

    #[test]
    fn test_bool_serialize_false() {
        let mut segment = Segment::new("TEST");
        let value = false;

        value.set_field(&mut segment, 1);
        assert_eq!(segment.get_field_value(1).unwrap(), "N");
    }

    #[test]
    fn test_bool_roundtrip_true() {
        let mut segment = Segment::new("TEST");
        let original = true;

        original.set_field(&mut segment, 1);
        let parsed = bool::parse_field(&segment, 1, "TEST").unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_bool_roundtrip_false() {
        let mut segment = Segment::new("TEST");
        let original = false;

        original.set_field(&mut segment, 1);
        let parsed = bool::parse_field(&segment, 1, "TEST").unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_option_bool_parse_some() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "Y").unwrap();

        let result = Option::<bool>::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, Some(true));
    }

    #[test]
    fn test_option_bool_parse_none() {
        let segment = Segment::new("TEST");

        let result = Option::<bool>::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_bool_parse_invalid_value() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "INVALID").unwrap();

        let result = bool::parse_field(&segment, 1, "TEST");
        assert!(result.is_err());
    }

    // Tests for NaiveDateTime type
    #[test]
    fn test_naive_datetime_parse() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "20250119143000").unwrap(); // Jan 19, 2025 14:30:00

        let result = NaiveDateTime::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result.format("%Y%m%d%H%M%S").to_string(), "20250119143000");
    }

    #[test]
    fn test_naive_datetime_serialize() {
        use chrono::NaiveDate;
        let mut segment = Segment::new("TEST");
        let dt = NaiveDate::from_ymd_opt(2025, 1, 19)
            .unwrap()
            .and_hms_opt(14, 30, 0)
            .unwrap();

        dt.set_field(&mut segment, 1);
        assert_eq!(segment.get_field_value(1).unwrap(), "20250119143000");
    }

    #[test]
    fn test_naive_datetime_roundtrip() {
        use chrono::NaiveDate;
        let mut segment = Segment::new("TEST");
        let original = NaiveDate::from_ymd_opt(2025, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();

        original.set_field(&mut segment, 1);
        let parsed = NaiveDateTime::parse_field(&segment, 1, "TEST").unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_option_naive_datetime_parse_some() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "20250101000000").unwrap();

        let result = Option::<NaiveDateTime>::parse_field(&segment, 1, "TEST").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_option_naive_datetime_parse_none() {
        let segment = Segment::new("TEST");

        let result = Option::<NaiveDateTime>::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, None);
    }

    // Tests for NaiveDate type
    #[test]
    fn test_naive_date_parse_yyyymmdd() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "20250119").unwrap(); // Jan 19, 2025

        let result = NaiveDate::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result.format("%Y%m%d").to_string(), "20250119");
    }

    #[test]
    fn test_naive_date_parse_yyyymm() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "202501").unwrap(); // Jan 2025 (defaults to 1st)

        let result = NaiveDate::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result.format("%Y%m%d").to_string(), "20250101");
    }

    #[test]
    fn test_naive_date_parse_yyyy() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "2025").unwrap(); // 2025 (defaults to Jan 1st)

        let result = NaiveDate::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result.format("%Y%m%d").to_string(), "20250101");
    }

    #[test]
    fn test_naive_date_serialize() {
        let mut segment = Segment::new("TEST");
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();

        date.set_field(&mut segment, 1);
        assert_eq!(segment.get_field_value(1).unwrap(), "20250615");
    }

    #[test]
    fn test_naive_date_roundtrip() {
        let mut segment = Segment::new("TEST");
        let original = NaiveDate::from_ymd_opt(1980, 1, 15).unwrap();

        original.set_field(&mut segment, 1);
        let parsed = NaiveDate::parse_field(&segment, 1, "TEST").unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_option_naive_date_parse_some() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "20250119").unwrap();

        let result = Option::<NaiveDate>::parse_field(&segment, 1, "TEST").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_option_naive_date_parse_none() {
        let segment = Segment::new("TEST");

        let result = Option::<NaiveDate>::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, None);
    }

    // Tests for NaiveTime type
    #[test]
    fn test_naive_time_parse_hhmmss() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "143000").unwrap(); // 14:30:00

        let result = NaiveTime::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result.format("%H%M%S").to_string(), "143000");
    }

    #[test]
    fn test_naive_time_parse_hhmm() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "1430").unwrap(); // 14:30 (seconds default to 00)

        let result = NaiveTime::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result.format("%H%M%S").to_string(), "143000");
    }

    #[test]
    fn test_naive_time_serialize() {
        let mut segment = Segment::new("TEST");
        let time = NaiveTime::from_hms_opt(23, 59, 59).unwrap();

        time.set_field(&mut segment, 1);
        assert_eq!(segment.get_field_value(1).unwrap(), "235959");
    }

    #[test]
    fn test_naive_time_roundtrip() {
        let mut segment = Segment::new("TEST");
        let original = NaiveTime::from_hms_opt(8, 15, 30).unwrap();

        original.set_field(&mut segment, 1);
        let parsed = NaiveTime::parse_field(&segment, 1, "TEST").unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_option_naive_time_parse_some() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "120000").unwrap();

        let result = Option::<NaiveTime>::parse_field(&segment, 1, "TEST").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_option_naive_time_parse_none() {
        let segment = Segment::new("TEST");

        let result = Option::<NaiveTime>::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, None);
    }

    // Tests for DateTime<Utc> type
    #[test]
    fn test_datetime_utc_parse() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "20250119143000").unwrap();

        let result = DateTime::<Utc>::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result.format("%Y%m%d%H%M%S").to_string(), "20250119143000");
    }

    #[test]
    fn test_datetime_utc_serialize() {
        use chrono::TimeZone;
        let mut segment = Segment::new("TEST");
        let dt = Utc.with_ymd_and_hms(2025, 1, 19, 14, 30, 0).unwrap();

        dt.set_field(&mut segment, 1);
        assert_eq!(segment.get_field_value(1).unwrap(), "20250119143000");
    }

    #[test]
    fn test_datetime_utc_roundtrip() {
        use chrono::TimeZone;
        let mut segment = Segment::new("TEST");
        let original = Utc.with_ymd_and_hms(2025, 12, 31, 23, 59, 59).unwrap();

        original.set_field(&mut segment, 1);
        let parsed = DateTime::<Utc>::parse_field(&segment, 1, "TEST").unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_option_datetime_utc_parse_some() {
        let mut segment = Segment::new("TEST");
        segment.set_field_value(1, "20250101000000").unwrap();

        let result = Option::<DateTime<Utc>>::parse_field(&segment, 1, "TEST").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_option_datetime_utc_parse_none() {
        let segment = Segment::new("TEST");

        let result = Option::<DateTime<Utc>>::parse_field(&segment, 1, "TEST").unwrap();
        assert_eq!(result, None);
    }

    // Tests for HL7 datetime parsing helper functions
    #[test]
    fn test_parse_hl7_datetime_valid() {
        let result = parse_hl7_datetime("20250119143000").unwrap();
        assert_eq!(result.format("%Y%m%d%H%M%S").to_string(), "20250119143000");
    }

    #[test]
    fn test_parse_hl7_datetime_invalid() {
        assert!(parse_hl7_datetime("invalid").is_none());
        assert!(parse_hl7_datetime("2025").is_none());
        assert!(parse_hl7_datetime("20250119").is_none()); // Date only, not datetime
    }

    #[test]
    fn test_parse_hl7_date_formats() {
        // YYYYMMDD
        let date1 = parse_hl7_date("20250119").unwrap();
        assert_eq!(date1.format("%Y%m%d").to_string(), "20250119");

        // YYYYMM
        let date2 = parse_hl7_date("202501").unwrap();
        assert_eq!(date2.format("%Y%m%d").to_string(), "20250101");

        // YYYY
        let date3 = parse_hl7_date("2025").unwrap();
        assert_eq!(date3.format("%Y%m%d").to_string(), "20250101");
    }

    #[test]
    fn test_parse_hl7_time_formats() {
        // HHMMSS
        let time1 = parse_hl7_time("143000").unwrap();
        assert_eq!(time1.format("%H%M%S").to_string(), "143000");

        // HHMM
        let time2 = parse_hl7_time("1430").unwrap();
        assert_eq!(time2.format("%H%M%S").to_string(), "143000");
    }

    #[test]
    fn test_parse_hl7_time_invalid() {
        assert!(parse_hl7_time("invalid").is_none());
        assert!(parse_hl7_time("14").is_none());
        assert!(parse_hl7_time("999999").is_none()); // Invalid hour
    }
}
