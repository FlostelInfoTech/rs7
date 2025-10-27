//! Terser API for HL7 messages
//!
//! The Terser provides a convenient API for accessing HL7 message fields
//! using path notation, similar to HAPI's Terser in Java.
//!
//! Path format examples:
//! - `PID-5-1` - PID segment, field 5, component 1
//! - `PID-5-1-2` - PID segment, field 5, component 1, subcomponent 2
//! - `OBX(2)-5` - Second OBX segment, field 5
//! - `PID-11(2)-1` - PID segment, field 11, second repetition, component 1

mod path;
pub mod cache;

use rs7_core::{
    error::{Error, Result},
    field::Field,
    message::Message,
    segment::Segment,
};

pub use cache::CachedTerser;
use path::TerserPath;

/// Terser for accessing HL7 message fields using path notation
pub struct Terser<'a> {
    message: &'a Message,
}

impl<'a> Terser<'a> {
    /// Create a new Terser for the given message
    pub fn new(message: &'a Message) -> Self {
        Self { message }
    }

    /// Get a value using path notation
    ///
    /// Examples:
    /// - `get("PID-5-1")` - Get PID field 5, component 1
    /// - `get("OBX(2)-5")` - Get second OBX segment, field 5
    /// - `get("PID-11(2)-1")` - Get PID field 11, repetition 2, component 1
    pub fn get(&self, path: &str) -> Result<Option<&str>> {
        let parsed_path = TerserPath::parse(path)?;

        // Find the segment
        let segments = self.message.get_segments_by_id(&parsed_path.segment_id);

        if segments.is_empty() {
            return Ok(None);
        }

        let segment = segments.get(parsed_path.segment_index)
            .ok_or_else(|| Error::terser_path(format!(
                "Segment index {} out of bounds for {}",
                parsed_path.segment_index, parsed_path.segment_id
            )))?;

        // Get the field
        let field = match segment.get_field(parsed_path.field_index) {
            Some(f) => f,
            None => return Ok(None),
        };

        // Navigate to the value
        let value = self.get_field_value(
            field,
            parsed_path.repetition_index,
            parsed_path.component_index,
            parsed_path.subcomponent_index,
        );

        Ok(value)
    }

    /// Get a field value at the specified indices
    fn get_field_value<'b>(
        &self,
        field: &'b Field,
        rep_idx: usize,
        comp_idx: Option<usize>,
        sub_idx: Option<usize>,
    ) -> Option<&'b str> {
        let repetition = field.get_repetition(rep_idx)?;

        match (comp_idx, sub_idx) {
            (None, None) => {
                // Just the field value
                repetition.value()
            }
            (Some(c_idx), None) => {
                // Component value
                repetition.get_component(c_idx)?.value()
            }
            (Some(c_idx), Some(s_idx)) => {
                // Subcomponent value
                repetition
                    .get_component(c_idx)?
                    .get_subcomponent(s_idx)?
                    .as_str()
                    .into()
            }
            (None, Some(_)) => {
                // Invalid: can't have subcomponent without component
                None
            }
        }
    }
}

/// Mutable terser for modifying message values
pub struct TerserMut<'a> {
    message: &'a mut Message,
}

impl<'a> TerserMut<'a> {
    /// Create a new mutable Terser
    pub fn new(message: &'a mut Message) -> Self {
        Self { message }
    }

    /// Set a value using path notation
    pub fn set(&mut self, path: &str, value: &str) -> Result<()> {
        let parsed_path = TerserPath::parse(path)?;

        // Find or create the segment
        let segment_index = self.ensure_segment(&parsed_path.segment_id, parsed_path.segment_index)?;

        // Ensure field exists and set the value
        {
            let segment = &mut self.message.segments[segment_index];

            // Ensure field exists
            while segment.fields.len() < parsed_path.field_index {
                segment.add_field(Field::new());
            }

            let field = segment.get_field_mut(parsed_path.field_index)
                .ok_or_else(|| Error::terser_path("Failed to get field"))?;

            // Set the value
            Self::set_field_value_static(
                field,
                value,
                parsed_path.repetition_index,
                parsed_path.component_index,
                parsed_path.subcomponent_index,
            )?;
        }

        Ok(())
    }

    /// Ensure segment exists at the given index
    fn ensure_segment(&mut self, segment_id: &str, index: usize) -> Result<usize> {
        let mut current_index = 0;
        let mut actual_index = None;

        for (i, seg) in self.message.segments.iter().enumerate() {
            if seg.id == segment_id {
                if current_index == index {
                    actual_index = Some(i);
                    break;
                }
                current_index += 1;
            }
        }

        if let Some(idx) = actual_index {
            Ok(idx)
        } else {
            // Create new segment
            let new_segment = Segment::new(segment_id);
            self.message.add_segment(new_segment);
            Ok(self.message.segments.len() - 1)
        }
    }

    /// Set a field value at the specified indices
    fn set_field_value_static(
        field: &mut Field,
        value: &str,
        rep_idx: usize,
        comp_idx: Option<usize>,
        sub_idx: Option<usize>,
    ) -> Result<()> {
        use rs7_core::field::{Component, Repetition, SubComponent};

        // Ensure repetition exists
        while field.repetitions.len() <= rep_idx {
            field.add_repetition(Repetition::new());
        }

        let repetition = field.get_repetition_mut(rep_idx)
            .ok_or_else(|| Error::terser_path("Failed to get repetition"))?;

        match (comp_idx, sub_idx) {
            (None, None) => {
                // Set entire field value
                if repetition.components.is_empty() {
                    repetition.add_component(Component::from_value(value));
                } else {
                    repetition.components[0] = Component::from_value(value);
                }
            }
            (Some(c_idx), None) => {
                // Set component value
                while repetition.components.len() <= c_idx {
                    repetition.add_component(Component::new());
                }

                repetition.components[c_idx] = Component::from_value(value);
            }
            (Some(c_idx), Some(s_idx)) => {
                // Set subcomponent value
                while repetition.components.len() <= c_idx {
                    repetition.add_component(Component::new());
                }

                let component = &mut repetition.components[c_idx];

                while component.subcomponents.len() <= s_idx {
                    component.add_subcomponent(SubComponent::new(""));
                }

                component.subcomponents[s_idx] = SubComponent::new(value);
            }
            (None, Some(_)) => {
                return Err(Error::terser_path(
                    "Cannot set subcomponent without component index",
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path_with_segment_index() {
        let path = TerserPath::parse("OBX(2)-5").unwrap();
        assert_eq!(path.segment_id, "OBX");
        assert_eq!(path.segment_index, 2);
        assert_eq!(path.field_index, 5);
    }

    #[test]
    fn test_parse_path_with_repetition() {
        let path = TerserPath::parse("PID-11(1)-1").unwrap();
        assert_eq!(path.field_index, 11);
        assert_eq!(path.repetition_index, 1);
        assert_eq!(path.component_index, Some(1));
    }

    #[test]
    fn test_parse_invalid_path() {
        assert!(TerserPath::parse("").is_err());
        assert!(TerserPath::parse("PID").is_err());
        assert!(TerserPath::parse("PID-").is_err());
    }
}
