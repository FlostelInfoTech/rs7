//! HL7 message parser using nom
//!
//! This crate provides parsing functionality for HL7 v2.x messages.

// nom parser combinators (for future enhancements)
use rs7_core::{
    delimiters::Delimiters,
    encoding::Encoding,
    error::{Error, Result},
    field::{Component, Field, Repetition, SubComponent},
    message::Message,
    segment::Segment,
};

/// Parse a complete HL7 message
pub fn parse_message(input: &str) -> Result<Message> {
    let input = input.trim();

    // Extract delimiters from MSH segment
    let delimiters = extract_delimiters(input)?;

    // Split message into segments (by \r or \n)
    let segment_strings: Vec<&str> = input
        .split('\r')
        .flat_map(|s| s.split('\n'))
        .filter(|s| !s.is_empty())
        .collect();

    if segment_strings.is_empty() {
        return Err(Error::parse("Empty message"));
    }

    let mut message = Message::with_delimiters(delimiters);

    for (idx, seg_str) in segment_strings.iter().enumerate() {
        let segment = if idx == 0 {
            parse_msh_segment(seg_str, &delimiters)?
        } else {
            parse_segment(seg_str, &delimiters)?
        };
        message.add_segment(segment);
    }

    Ok(message)
}

/// Extract delimiters from MSH segment
///
/// MSH format: MSH|^~\&|...
/// Position 3 is field separator (|)
/// Positions 4-7 are encoding characters (^~\&)
fn extract_delimiters(input: &str) -> Result<Delimiters> {
    if !input.starts_with("MSH") {
        return Err(Error::parse("Message must start with MSH segment"));
    }

    if input.len() < 8 {
        return Err(Error::parse("MSH segment too short"));
    }

    let field_sep = input.chars().nth(3).ok_or_else(|| {
        Error::parse("Cannot extract field separator")
    })?;

    let encoding_chars: String = input.chars().skip(4).take(4).collect();

    Delimiters::from_encoding_characters(field_sep, &encoding_chars)
}

/// Parse MSH segment (special handling)
fn parse_msh_segment(input: &str, delimiters: &Delimiters) -> Result<Segment> {
    if !input.starts_with("MSH") {
        return Err(Error::parse("MSH segment must start with 'MSH'"));
    }

    let mut segment = Segment::new("MSH");

    // Add MSH-1 (field separator - not actually in the message text)
    segment.add_field(Field::from_value(delimiters.field_separator.to_string()));

    // Add MSH-2 (encoding characters - appears after MSH|)
    segment.add_field(Field::from_value(delimiters.encoding_characters()));

    // Parse the rest of the fields starting after "MSH|^~\&"
    // Note: the character at position 8 is the field separator before MSH-3
    let field_start = 9; // "MSH|^~\&|".len() - we want to start after the | following encoding chars
    if input.len() <= field_start {
        return Ok(segment);
    }

    let rest = &input[field_start..];
    let field_strings: Vec<&str> = rest.split(delimiters.field_separator).collect();

    // Add MSH-3 onwards
    for field_str in field_strings {
        let field = parse_field(field_str, delimiters)?;
        segment.add_field(field);
    }

    Ok(segment)
}

/// Parse a regular segment
fn parse_segment(input: &str, delimiters: &Delimiters) -> Result<Segment> {
    if input.len() < 3 {
        return Err(Error::parse("Segment too short"));
    }

    let segment_id = &input[0..3];
    let mut segment = Segment::new(segment_id);

    if input.len() <= 3 {
        return Ok(segment);
    }

    // Check for field separator after segment ID
    if input.chars().nth(3) != Some(delimiters.field_separator) {
        return Err(Error::parse(format!(
            "Expected field separator after segment ID, got '{}'",
            input.chars().nth(3).unwrap_or(' ')
        )));
    }

    let rest = &input[4..];
    let field_strings: Vec<&str> = rest.split(delimiters.field_separator).collect();

    for field_str in field_strings {
        let field = parse_field(field_str, delimiters)?;
        segment.add_field(field);
    }

    Ok(segment)
}

/// Parse a field (can contain repetitions)
fn parse_field(input: &str, delimiters: &Delimiters) -> Result<Field> {
    let mut field = Field::new();

    // Even empty fields should have one (empty) repetition
    let repetition_strings: Vec<&str> = if input.is_empty() {
        vec![""]
    } else {
        input.split(delimiters.repetition_separator).collect()
    };

    for rep_str in repetition_strings {
        let repetition = parse_repetition(rep_str, delimiters)?;
        field.add_repetition(repetition);
    }

    Ok(field)
}

/// Parse a repetition (can contain components)
fn parse_repetition(input: &str, delimiters: &Delimiters) -> Result<Repetition> {
    let mut repetition = Repetition::new();

    // Even empty repetitions should have one (empty) component
    let component_strings: Vec<&str> = if input.is_empty() {
        vec![""]
    } else {
        input.split(delimiters.component_separator).collect()
    };

    for comp_str in component_strings {
        let component = parse_component(comp_str, delimiters)?;
        repetition.add_component(component);
    }

    Ok(repetition)
}

/// Parse a component (can contain subcomponents)
fn parse_component(input: &str, delimiters: &Delimiters) -> Result<Component> {
    let mut component = Component::new();

    // Even empty components should have one (empty) subcomponent
    let subcomponent_strings: Vec<&str> = if input.is_empty() {
        vec![""]
    } else {
        input.split(delimiters.subcomponent_separator).collect()
    };

    for sub_str in subcomponent_strings {
        let subcomponent = parse_subcomponent(sub_str, delimiters)?;
        component.add_subcomponent(subcomponent);
    }

    Ok(component)
}

/// Parse a subcomponent (decode escape sequences)
fn parse_subcomponent(input: &str, delimiters: &Delimiters) -> Result<SubComponent> {
    if input.is_empty() {
        return Ok(SubComponent::new(""));
    }

    let decoded = Encoding::decode(input, delimiters)?;
    Ok(SubComponent::new(decoded))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_delimiters() {
        let msh = "MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315||ADT^A01|12345|P|2.5";
        let delims = extract_delimiters(msh).unwrap();

        assert_eq!(delims.field_separator, '|');
        assert_eq!(delims.component_separator, '^');
        assert_eq!(delims.repetition_separator, '~');
        assert_eq!(delims.escape_character, '\\');
        assert_eq!(delims.subcomponent_separator, '&');
    }

    #[test]
    fn test_parse_msh_segment() {
        let msh = "MSH|^~\\&|SendApp|SendFac|RecApp|RecFac";
        let delims = Delimiters::default();
        let segment = parse_msh_segment(msh, &delims).unwrap();

        assert_eq!(segment.id, "MSH");
        assert_eq!(segment.get_field_value(3), Some("SendApp"));
        assert_eq!(segment.get_field_value(4), Some("SendFac"));
    }

    #[test]
    fn test_parse_segment() {
        let delims = Delimiters::default();
        let pid = "PID|1|12345|67890^^^MRN|DOE^JOHN^A|";
        let segment = parse_segment(pid, &delims).unwrap();

        assert_eq!(segment.id, "PID");
        assert_eq!(segment.get_field_value(1), Some("1"));
        assert_eq!(segment.get_field_value(2), Some("12345"));
    }

    #[test]
    fn test_parse_field_with_components() {
        let delims = Delimiters::default();
        let field_str = "DOE^JOHN^A";
        let field = parse_field(field_str, &delims).unwrap();

        let rep = field.get_repetition(0).unwrap();
        assert_eq!(rep.get_component(0).unwrap().value(), Some("DOE"));
        assert_eq!(rep.get_component(1).unwrap().value(), Some("JOHN"));
        assert_eq!(rep.get_component(2).unwrap().value(), Some("A"));
    }

    #[test]
    fn test_parse_field_with_subcomponents() {
        let delims = Delimiters::default();
        let field_str = "ID1&AssignAuth^ID2";
        let field = parse_field(field_str, &delims).unwrap();

        let rep = field.get_repetition(0).unwrap();
        let comp0 = rep.get_component(0).unwrap();
        assert_eq!(comp0.get_subcomponent(0).unwrap().as_str(), "ID1");
        assert_eq!(comp0.get_subcomponent(1).unwrap().as_str(), "AssignAuth");
    }

    #[test]
    fn test_parse_field_with_repetitions() {
        let delims = Delimiters::default();
        let field_str = "Value1~Value2~Value3";
        let field = parse_field(field_str, &delims).unwrap();

        assert_eq!(field.repetitions.len(), 3);
        assert_eq!(field.get_repetition(0).unwrap().value(), Some("Value1"));
        assert_eq!(field.get_repetition(1).unwrap().value(), Some("Value2"));
        assert_eq!(field.get_repetition(2).unwrap().value(), Some("Value3"));
    }

    #[test]
    fn test_parse_complete_message() {
        let msg = "MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143000||ADT^A01|12345|P|2.5\r\
                   PID|1|12345|67890^^^MRN|DOE^JOHN^A||19800101|M\r\
                   PV1|1|I|Ward^Room^Bed";

        let parsed = parse_message(msg).unwrap();

        assert_eq!(parsed.segments.len(), 3);
        assert_eq!(parsed.get_msh().unwrap().id, "MSH");
        assert_eq!(parsed.get_sending_application(), Some("SendApp"));

        let pid = &parsed.segments[1];
        assert_eq!(pid.id, "PID");
        assert_eq!(pid.get_field_value(2), Some("12345"));
    }

    #[test]
    fn test_parse_with_escape_sequences() {
        let delims = Delimiters::default();
        let field_str = "Test\\F\\Value";
        let field = parse_field(field_str, &delims).unwrap();

        assert_eq!(field.value(), Some("Test|Value"));
    }

    #[test]
    fn test_parse_empty_fields() {
        let delims = Delimiters::default();
        let segment = "PID|1||3|4|5";
        let parsed = parse_segment(segment, &delims).unwrap();

        assert_eq!(parsed.get_field_value(1), Some("1"));
        assert_eq!(parsed.get_field_value(2), Some(""));
        assert_eq!(parsed.get_field_value(3), Some("3"));
    }
}
