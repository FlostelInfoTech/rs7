//! Conformance profile validator

pub mod result;

pub use result::{
    ConformanceErrorType, ConformanceValidationError, ConformanceValidationInfo,
    ConformanceValidationResult, ConformanceValidationWarning, Severity, ValidationLocation,
};

use crate::profile::{ConditionalUsage, ConformanceProfile, FieldProfile, SegmentProfile, Usage};
use rs7_core::{Message, Segment};

/// Conformance profile validator
pub struct ConformanceValidator {
    profile: ConformanceProfile,
}

impl ConformanceValidator {
    /// Create a new conformance validator
    pub fn new(profile: ConformanceProfile) -> Self {
        Self { profile }
    }

    /// Get the profile
    pub fn profile(&self) -> &ConformanceProfile {
        &self.profile
    }

    /// Validate a message against the conformance profile
    pub fn validate(&self, message: &Message) -> ConformanceValidationResult {
        let mut result = ConformanceValidationResult::new();

        // Validate message structure
        self.validate_message_structure(message, &mut result);

        // Validate each segment in the profile
        for segment_profile in &self.profile.message.segments {
            self.validate_segment(message, segment_profile, &mut result);
        }

        result
    }

    /// Validate overall message structure
    fn validate_message_structure(&self, _message: &Message, result: &mut ConformanceValidationResult) {
        // Check message type matches profile
        let _profile = &self.profile.message;

        result.add_info(ConformanceValidationInfo {
            location: None,
            message: format!(
                "Validating against profile: {} {}",
                self.profile.metadata.name, self.profile.metadata.version
            ),
        });

        // Note: In MVP, we're not strictly enforcing message type match
        // This would be added in a future enhancement
    }

    /// Validate a segment according to its profile
    fn validate_segment(
        &self,
        message: &Message,
        segment_profile: &SegmentProfile,
        result: &mut ConformanceValidationResult,
    ) {
        // Count occurrences of this segment
        let segments: Vec<&Segment> = message
            .segments
            .iter()
            .filter(|s| s.id == segment_profile.name)
            .collect();

        let occurrence_count = segments.len();

        // Validate segment usage
        self.validate_segment_usage(&segment_profile.name, occurrence_count, segment_profile, result);

        // Validate segment cardinality
        self.validate_segment_cardinality(
            &segment_profile.name,
            occurrence_count,
            segment_profile,
            result,
        );

        // Validate each occurrence of the segment
        for (index, segment) in segments.iter().enumerate() {
            self.validate_segment_fields(segment, index, segment_profile, message, result);
        }
    }

    /// Validate segment usage (R, RE, O, X)
    fn validate_segment_usage(
        &self,
        segment_name: &str,
        occurrence_count: usize,
        segment_profile: &SegmentProfile,
        result: &mut ConformanceValidationResult,
    ) {
        match segment_profile.usage {
            Usage::Required => {
                if occurrence_count == 0 {
                    result.add_error(
                        ConformanceValidationError::new(
                            ValidationLocation::segment(segment_name.to_string()),
                            ConformanceErrorType::RequiredElementMissing,
                            format!("Required segment {} is missing", segment_name),
                        )
                        .with_rule(format!("{} usage=R", segment_name)),
                    );
                }
            }
            Usage::RequiredIfKnown => {
                if occurrence_count == 0 {
                    result.add_warning(ConformanceValidationWarning {
                        location: ValidationLocation::segment(segment_name.to_string()),
                        message: format!(
                            "Required if known segment {} is missing",
                            segment_name
                        ),
                        rule: Some(format!("{} usage=RE", segment_name)),
                    });
                }
            }
            Usage::NotUsed => {
                if occurrence_count > 0 {
                    result.add_error(
                        ConformanceValidationError::new(
                            ValidationLocation::segment(segment_name.to_string()),
                            ConformanceErrorType::NotUsedElementPresent,
                            format!(
                                "Segment {} is marked as not used but is present ({} occurrence(s))",
                                segment_name, occurrence_count
                            ),
                        )
                        .with_rule(format!("{} usage=X", segment_name)),
                    );
                }
            }
            Usage::Optional => {
                // No validation needed for optional segments
            }
        }
    }

    /// Validate segment cardinality
    fn validate_segment_cardinality(
        &self,
        segment_name: &str,
        occurrence_count: usize,
        segment_profile: &SegmentProfile,
        result: &mut ConformanceValidationResult,
    ) {
        let cardinality = &segment_profile.cardinality;

        if !cardinality.satisfies(occurrence_count) {
            if occurrence_count < cardinality.min {
                result.add_error(
                    ConformanceValidationError::new(
                        ValidationLocation::segment(segment_name.to_string()),
                        ConformanceErrorType::BelowMinimumOccurrences,
                        format!(
                            "Segment {} has {} occurrence(s), but minimum is {}",
                            segment_name, occurrence_count, cardinality.min
                        ),
                    )
                    .with_rule(format!("{} cardinality={}", segment_name, cardinality.to_string())),
                );
            } else if let Some(max) = cardinality.max {
                result.add_error(
                    ConformanceValidationError::new(
                        ValidationLocation::segment(segment_name.to_string()),
                        ConformanceErrorType::ExceedsMaximumOccurrences,
                        format!(
                            "Segment {} has {} occurrence(s), but maximum is {}",
                            segment_name, occurrence_count, max
                        ),
                    )
                    .with_rule(format!("{} cardinality={}", segment_name, cardinality.to_string())),
                );
            }
        }
    }

    /// Validate fields within a segment
    fn validate_segment_fields(
        &self,
        segment: &Segment,
        segment_index: usize,
        segment_profile: &SegmentProfile,
        message: &Message,
        result: &mut ConformanceValidationResult,
    ) {
        for field_profile in &segment_profile.fields {
            self.validate_field(segment, segment_index, field_profile, message, result);
        }
    }

    /// Validate a specific field
    fn validate_field(
        &self,
        segment: &Segment,
        segment_index: usize,
        field_profile: &FieldProfile,
        message: &Message,
        result: &mut ConformanceValidationResult,
    ) {
        let field_position = field_profile.position;
        let field = segment.get_field(field_position);

        // Count field occurrences (for repeating fields)
        let occurrence_count = field.map(|f| f.repetitions.len()).unwrap_or(0);

        // Validate field usage
        self.validate_field_usage(
            &segment.id,
            segment_index,
            field_position,
            occurrence_count,
            field_profile,
            message,
            result,
        );

        // Validate field cardinality
        self.validate_field_cardinality(
            &segment.id,
            segment_index,
            field_position,
            occurrence_count,
            field_profile,
            result,
        );

        // Validate field length if present
        if let Some(field) = field {
            self.validate_field_length(
                &segment.id,
                segment_index,
                field_position,
                field,
                field_profile,
                result,
            );
        }
    }

    /// Validate field usage
    fn validate_field_usage(
        &self,
        segment_id: &str,
        segment_index: usize,
        field_position: usize,
        occurrence_count: usize,
        field_profile: &FieldProfile,
        message: &Message,
        result: &mut ConformanceValidationResult,
    ) {
        let location = if segment_index > 0 {
            ValidationLocation {
                segment: segment_id.to_string(),
                segment_index: Some(segment_index),
                field: Some(field_position),
                component: None,
            }
        } else {
            ValidationLocation::field(segment_id.to_string(), field_position)
        };

        match &field_profile.usage {
            ConditionalUsage::Required => {
                if occurrence_count == 0 {
                    let field_name = field_profile
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("Field {}", field_position));
                    result.add_error(
                        ConformanceValidationError::new(
                            location,
                            ConformanceErrorType::RequiredElementMissing,
                            format!("Required field {} is missing", field_name),
                        )
                        .with_rule(format!("{}-{} usage=R", segment_id, field_position)),
                    );
                }
            }
            ConditionalUsage::RequiredIfKnown => {
                if occurrence_count == 0 {
                    let field_name = field_profile
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("Field {}", field_position));
                    result.add_warning(ConformanceValidationWarning {
                        location,
                        message: format!("Required if known field {} is missing", field_name),
                        rule: Some(format!("{}-{} usage=RE", segment_id, field_position)),
                    });
                }
            }
            ConditionalUsage::NotUsed => {
                if occurrence_count > 0 {
                    let field_name = field_profile
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("Field {}", field_position));
                    result.add_error(
                        ConformanceValidationError::new(
                            location,
                            ConformanceErrorType::NotUsedElementPresent,
                            format!("Field {} is marked as not used but is present", field_name),
                        )
                        .with_rule(format!("{}-{} usage=X", segment_id, field_position)),
                    );
                }
            }
            ConditionalUsage::Optional => {
                // No validation needed
            }
            ConditionalUsage::Conditional(predicate) => {
                // Evaluate predicate to determine actual usage
                if let Ok(actual_usage) = crate::predicate::PredicateEvaluator::evaluate(predicate, message) {
                    // Recursively validate with the evaluated usage
                    let temp_profile = FieldProfile {
                        position: field_profile.position,
                        name: field_profile.name.clone(),
                        usage: ConditionalUsage::from_usage(actual_usage),
                        cardinality: field_profile.cardinality,
                        datatype: field_profile.datatype.clone(),
                        length: field_profile.length,
                        table_id: field_profile.table_id.clone(),
                        components: field_profile.components.clone(),
                        value_set: field_profile.value_set.clone(),
                    };
                    self.validate_field_usage(
                        segment_id,
                        segment_index,
                        field_position,
                        occurrence_count,
                        &temp_profile,
                        message,
                        result,
                    );
                }
            }
        }
    }

    /// Validate field cardinality
    fn validate_field_cardinality(
        &self,
        segment_id: &str,
        segment_index: usize,
        field_position: usize,
        occurrence_count: usize,
        field_profile: &FieldProfile,
        result: &mut ConformanceValidationResult,
    ) {
        let location = if segment_index > 0 {
            ValidationLocation {
                segment: segment_id.to_string(),
                segment_index: Some(segment_index),
                field: Some(field_position),
                component: None,
            }
        } else {
            ValidationLocation::field(segment_id.to_string(), field_position)
        };

        let cardinality = &field_profile.cardinality;

        if !cardinality.satisfies(occurrence_count) {
            if occurrence_count < cardinality.min {
                result.add_error(
                    ConformanceValidationError::new(
                        location,
                        ConformanceErrorType::BelowMinimumOccurrences,
                        format!(
                            "Field has {} occurrence(s), but minimum is {}",
                            occurrence_count, cardinality.min
                        ),
                    )
                    .with_rule(format!(
                        "{}-{} cardinality={}",
                        segment_id,
                        field_position,
                        cardinality.to_string()
                    )),
                );
            } else if let Some(max) = cardinality.max {
                result.add_error(
                    ConformanceValidationError::new(
                        location,
                        ConformanceErrorType::ExceedsMaximumOccurrences,
                        format!(
                            "Field has {} occurrence(s), but maximum is {}",
                            occurrence_count, max
                        ),
                    )
                    .with_rule(format!(
                        "{}-{} cardinality={}",
                        segment_id,
                        field_position,
                        cardinality.to_string()
                    )),
                );
            }
        }
    }

    /// Validate field length
    fn validate_field_length(
        &self,
        segment_id: &str,
        segment_index: usize,
        field_position: usize,
        field: &rs7_core::Field,
        field_profile: &FieldProfile,
        result: &mut ConformanceValidationResult,
    ) {
        if let Some(max_length) = field_profile.length {
            // Check each repetition
            for (rep_index, repetition) in field.repetitions.iter().enumerate() {
                // Get the encoded value of this repetition
                // Use default delimiters for length calculation
                let delimiters = rs7_core::Delimiters::default();
                let value = repetition.encode(&delimiters);
                let actual_length = value.len();

                if actual_length > max_length {
                    let location = if segment_index > 0 {
                        ValidationLocation {
                            segment: segment_id.to_string(),
                            segment_index: Some(segment_index),
                            field: Some(field_position),
                            component: None,
                        }
                    } else {
                        ValidationLocation::field(segment_id.to_string(), field_position)
                    };

                    let field_name = field_profile
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("Field {}", field_position));

                    result.add_error(
                        ConformanceValidationError::new(
                            location,
                            ConformanceErrorType::ExceedsMaxLength,
                            format!(
                                "Field {} repetition {} has length {}, but maximum is {}",
                                field_name,
                                rep_index + 1,
                                actual_length,
                                max_length
                            ),
                        )
                        .with_rule(format!(
                            "{}-{} length<={}",
                            segment_id, field_position, max_length
                        )),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{Cardinality, MessageProfile, ProfileMetadata};
    use rs7_core::Version;

    fn create_test_profile() -> ConformanceProfile {
        let metadata = ProfileMetadata::new(
            "Test Profile".to_string(),
            "1.0".to_string(),
            Version::V2_5,
        );

        let mut message = MessageProfile::new("ADT".to_string(), "A01".to_string());

        // MSH segment - required, exactly one
        let mut msh = SegmentProfile::new(
            "MSH".to_string(),
            Usage::Required,
            Cardinality::one(),
        );
        msh.add_field(FieldProfile::new(
            9,
            Usage::Required,
            Cardinality::one(),
        ));
        message.add_segment(msh);

        // PID segment - required, exactly one
        let mut pid = SegmentProfile::new(
            "PID".to_string(),
            Usage::Required,
            Cardinality::one(),
        );
        pid.add_field(FieldProfile::new(
            3,
            Usage::Required,
            Cardinality::one(),
        ));
        message.add_segment(pid);

        ConformanceProfile::new(metadata, message)
    }

    #[test]
    fn test_validator_creation() {
        let profile = create_test_profile();
        let validator = ConformanceValidator::new(profile);
        assert_eq!(validator.profile().metadata.name, "Test Profile");
    }

    // More tests would go here
}
