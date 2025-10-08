///! Performance optimizations for HL7 parsing
///!
///! This module contains optimized parsing functions that reduce allocations
///! and improve performance for high-throughput scenarios.

use rs7_core::{
    delimiters::Delimiters,
    encoding::Encoding,
    error::{Error, Result},
    field::{Component, Field, Repetition, SubComponent},
    segment::Segment,
};

/// Parse a field with pre-allocated capacity hints
#[inline]
pub(crate) fn parse_field_optimized(input: &str, delimiters: &Delimiters) -> Result<Field> {
    let mut field = Field::new();

    if input.is_empty() {
        field.add_repetition(Repetition::new());
        return Ok(field);
    }

    // Count repetitions first to pre-allocate
    let rep_count = if delimiters.repetition_separator == '~' {
        input.matches('~').count() + 1
    } else {
        input.matches(delimiters.repetition_separator).count() + 1
    };

    // Pre-allocate repetitions vector
    let mut repetitions = Vec::with_capacity(rep_count);

    for rep_str in input.split(delimiters.repetition_separator) {
        let repetition = parse_repetition_optimized(rep_str, delimiters)?;
        repetitions.push(repetition);
    }

    field.repetitions = repetitions;
    Ok(field)
}

/// Parse a repetition with optimized component handling
#[inline]
pub(crate) fn parse_repetition_optimized(input: &str, delimiters: &Delimiters) -> Result<Repetition> {
    let mut repetition = Repetition::new();

    if input.is_empty() {
        repetition.add_component(Component::new());
        return Ok(repetition);
    }

    // Count components for pre-allocation
    let comp_count = if delimiters.component_separator == '^' {
        input.matches('^').count() + 1
    } else {
        input.matches(delimiters.component_separator).count() + 1
    };

    // Pre-allocate components vector
    let mut components = Vec::with_capacity(comp_count);

    for comp_str in input.split(delimiters.component_separator) {
        let component = parse_component_optimized(comp_str, delimiters)?;
        components.push(component);
    }

    repetition.components = components;
    Ok(repetition)
}

/// Parse a component with optimized subcomponent handling
#[inline]
pub(crate) fn parse_component_optimized(input: &str, delimiters: &Delimiters) -> Result<Component> {
    let mut component = Component::new();

    if input.is_empty() {
        component.add_subcomponent(SubComponent::new(""));
        return Ok(component);
    }

    // Fast path: no subcomponents (most common case)
    if !input.contains(delimiters.subcomponent_separator) {
        let decoded = if input.contains(delimiters.escape_character) {
            Encoding::decode(input, delimiters)?
        } else {
            // No escape sequences - avoid decoding overhead
            input.to_string()
        };
        component.add_subcomponent(SubComponent::new(decoded));
        return Ok(component);
    }

    // Slow path: multiple subcomponents
    let sub_count = input.matches(delimiters.subcomponent_separator).count() + 1;
    let mut subcomponents = Vec::with_capacity(sub_count);

    for sub_str in input.split(delimiters.subcomponent_separator) {
        let decoded = if sub_str.contains(delimiters.escape_character) {
            Encoding::decode(sub_str, delimiters)?
        } else {
            sub_str.to_string()
        };
        subcomponents.push(SubComponent::new(decoded));
    }

    component.subcomponents = subcomponents;
    Ok(component)
}

/// Optimized segment parsing that minimizes allocations
pub(crate) fn parse_segment_optimized(input: &str, delimiters: &Delimiters) -> Result<Segment> {
    if input.len() < 3 {
        return Err(Error::parse("Segment too short"));
    }

    let segment_id = &input[0..3];
    let mut segment = Segment::new(segment_id);

    if input.len() <= 3 {
        return Ok(segment);
    }

    // Check for field separator
    if input.chars().nth(3) != Some(delimiters.field_separator) {
        return Err(Error::parse(format!(
            "Expected field separator after segment ID, got '{}'",
            input.chars().nth(3).unwrap_or(' ')
        )));
    }

    let rest = &input[4..];

    // Count fields for pre-allocation
    let field_count = rest.matches(delimiters.field_separator).count() + 1;
    let mut fields = Vec::with_capacity(field_count);

    // Parse fields
    for field_str in rest.split(delimiters.field_separator) {
        let field = parse_field_optimized(field_str, delimiters)?;
        fields.push(field);
    }

    segment.fields = fields;
    Ok(segment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_optimized_simple() {
        let delims = Delimiters::default();
        let field = parse_field_optimized("TEST", &delims).unwrap();
        assert_eq!(field.value(), Some("TEST"));
    }

    #[test]
    fn test_parse_field_optimized_components() {
        let delims = Delimiters::default();
        let field = parse_field_optimized("DOE^JOHN^A", &delims).unwrap();
        let rep = field.get_repetition(0).unwrap();
        assert_eq!(rep.get_component(0).unwrap().value(), Some("DOE"));
        assert_eq!(rep.get_component(1).unwrap().value(), Some("JOHN"));
        assert_eq!(rep.get_component(2).unwrap().value(), Some("A"));
    }

    #[test]
    fn test_parse_segment_optimized() {
        let delims = Delimiters::default();
        let segment = parse_segment_optimized("PID|1|12345|67890^^^MRN", &delims).unwrap();
        assert_eq!(segment.id, "PID");
        assert_eq!(segment.get_field_value(1), Some("1"));
        assert_eq!(segment.get_field_value(2), Some("12345"));
    }
}
