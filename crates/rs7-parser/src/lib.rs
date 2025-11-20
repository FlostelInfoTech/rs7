//! HL7 message parser using nom
//!
//! This crate provides parsing functionality for HL7 v2.x messages.

mod optimized;

// nom parser combinators (for future enhancements)
use rs7_core::{
    batch::{Batch, BatchHeader, BatchTrailer, File, FileHeader, FileTrailer},
    delimiters::Delimiters,
    encoding::Encoding,
    error::{Error, Result},
    field::{Component, Field, Repetition, SubComponent},
    message::Message,
    segment::Segment,
};
use chrono::NaiveDateTime;

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

/// Parse a complete HL7 batch message
///
/// A batch message consists of:
/// - BHS (Batch Header Segment)
/// - One or more complete messages (each starting with MSH)
/// - BTS (Batch Trailer Segment)
pub fn parse_batch(input: &str) -> Result<Batch> {
    let input = input.trim();

    if !input.starts_with("BHS") {
        return Err(Error::parse("Batch must start with BHS segment"));
    }

    // Split into segments
    let segment_strings: Vec<&str> = input
        .split('\r')
        .flat_map(|s| s.split('\n'))
        .filter(|s| !s.is_empty())
        .collect();

    if segment_strings.len() < 2 {
        return Err(Error::parse("Batch must have at least BHS and BTS segments"));
    }

    // Extract delimiters from BHS segment (same format as MSH)
    let delimiters = extract_delimiters_from_bhs(segment_strings[0])?;

    // Parse BHS (first segment)
    let header = parse_bhs_segment(segment_strings[0], &delimiters)?;

    // Find BTS segment (last segment)
    let last_idx = segment_strings.len() - 1;
    if !segment_strings[last_idx].starts_with("BTS") {
        return Err(Error::parse("Batch must end with BTS segment"));
    }
    let trailer = parse_bts_segment(segment_strings[last_idx], &delimiters)?;

    // Parse messages between BHS and BTS
    let mut messages = Vec::new();
    let mut current_message_lines = Vec::new();

    for line in &segment_strings[1..last_idx] {
        if line.starts_with("MSH") {
            // Start of a new message - save previous if exists
            if !current_message_lines.is_empty() {
                let msg_text = current_message_lines.join("\r");
                messages.push(parse_message(&msg_text)?);
                current_message_lines.clear();
            }
        }
        current_message_lines.push(*line);
    }

    // Don't forget the last message
    if !current_message_lines.is_empty() {
        let msg_text = current_message_lines.join("\r");
        messages.push(parse_message(&msg_text)?);
    }

    let batch = Batch {
        header,
        messages,
        trailer,
    };

    // Validate the batch
    batch.validate()?;

    Ok(batch)
}

/// Parse a complete HL7 file message
///
/// A file message consists of:
/// - FHS (File Header Segment)
/// - One or more batches (each with BHS...BTS)
/// - FTS (File Trailer Segment)
pub fn parse_file(input: &str) -> Result<File> {
    let input = input.trim();

    if !input.starts_with("FHS") {
        return Err(Error::parse("File must start with FHS segment"));
    }

    // Split into segments
    let segment_strings: Vec<&str> = input
        .split('\r')
        .flat_map(|s| s.split('\n'))
        .filter(|s| !s.is_empty())
        .collect();

    if segment_strings.len() < 2 {
        return Err(Error::parse("File must have at least FHS and FTS segments"));
    }

    // Extract delimiters from FHS segment
    let delimiters = extract_delimiters_from_fhs(segment_strings[0])?;

    // Parse FHS (first segment)
    let header = parse_fhs_segment(segment_strings[0], &delimiters)?;

    // Find FTS segment (last segment)
    let last_idx = segment_strings.len() - 1;
    if !segment_strings[last_idx].starts_with("FTS") {
        return Err(Error::parse("File must end with FTS segment"));
    }
    let trailer = parse_fts_segment(segment_strings[last_idx], &delimiters)?;

    // Parse batches between FHS and FTS
    let mut batches = Vec::new();
    let mut current_batch_lines = Vec::new();

    for line in &segment_strings[1..last_idx] {
        if line.starts_with("BHS") {
            // Start of a new batch - save previous if exists
            if !current_batch_lines.is_empty() {
                let batch_text = current_batch_lines.join("\r");
                batches.push(parse_batch(&batch_text)?);
                current_batch_lines.clear();
            }
        }
        current_batch_lines.push(*line);
    }

    // Don't forget the last batch
    if !current_batch_lines.is_empty() {
        let batch_text = current_batch_lines.join("\r");
        batches.push(parse_batch(&batch_text)?);
    }

    let file = File {
        header,
        batches,
        trailer,
    };

    // Validate the file
    file.validate()?;

    Ok(file)
}

/// Extract delimiters from BHS segment (same format as MSH)
fn extract_delimiters_from_bhs(input: &str) -> Result<Delimiters> {
    if !input.starts_with("BHS") {
        return Err(Error::parse("Expected BHS segment"));
    }

    if input.len() < 8 {
        return Err(Error::parse("BHS segment too short"));
    }

    let field_sep = input.chars().nth(3).ok_or_else(|| {
        Error::parse("Cannot extract field separator from BHS")
    })?;

    let encoding_chars: String = input.chars().skip(4).take(4).collect();

    Delimiters::from_encoding_characters(field_sep, &encoding_chars)
}

/// Extract delimiters from FHS segment (same format as MSH)
fn extract_delimiters_from_fhs(input: &str) -> Result<Delimiters> {
    if !input.starts_with("FHS") {
        return Err(Error::parse("Expected FHS segment"));
    }

    if input.len() < 8 {
        return Err(Error::parse("FHS segment too short"));
    }

    let field_sep = input.chars().nth(3).ok_or_else(|| {
        Error::parse("Cannot extract field separator from FHS")
    })?;

    let encoding_chars: String = input.chars().skip(4).take(4).collect();

    Delimiters::from_encoding_characters(field_sep, &encoding_chars)
}

/// Parse FHS (File Header Segment)
fn parse_fhs_segment(input: &str, delimiters: &Delimiters) -> Result<FileHeader> {
    let segment = parse_segment_like_msh("FHS", input, delimiters)?;

    Ok(FileHeader {
        field_separator: delimiters.field_separator,
        encoding_characters: delimiters.encoding_characters(),
        sending_application: segment.get_field_value(3).map(String::from),
        sending_facility: segment.get_field_value(4).map(String::from),
        receiving_application: segment.get_field_value(5).map(String::from),
        receiving_facility: segment.get_field_value(6).map(String::from),
        creation_datetime: parse_datetime_field(segment.get_field_value(7)),
        security: segment.get_field_value(8).map(String::from),
        file_name_id: segment.get_field_value(9).map(String::from),
        comment: segment.get_field_value(10).map(String::from),
        control_id: segment.get_field_value(11).map(String::from),
        reference_control_id: segment.get_field_value(12).map(String::from),
        sending_network_address: segment.get_field_value(13).map(String::from),
        receiving_network_address: segment.get_field_value(14).map(String::from),
    })
}

/// Parse BHS (Batch Header Segment)
fn parse_bhs_segment(input: &str, delimiters: &Delimiters) -> Result<BatchHeader> {
    let segment = parse_segment_like_msh("BHS", input, delimiters)?;

    Ok(BatchHeader {
        field_separator: delimiters.field_separator,
        encoding_characters: delimiters.encoding_characters(),
        sending_application: segment.get_field_value(3).map(String::from),
        sending_facility: segment.get_field_value(4).map(String::from),
        receiving_application: segment.get_field_value(5).map(String::from),
        receiving_facility: segment.get_field_value(6).map(String::from),
        creation_datetime: parse_datetime_field(segment.get_field_value(7)),
        security: segment.get_field_value(8).map(String::from),
        batch_name_id_type: segment.get_field_value(9).map(String::from),
        comment: segment.get_field_value(10).map(String::from),
        control_id: segment.get_field_value(11).map(String::from),
        reference_control_id: segment.get_field_value(12).map(String::from),
        sending_network_address: segment.get_field_value(13).map(String::from),
        receiving_network_address: segment.get_field_value(14).map(String::from),
    })
}

/// Parse FTS (File Trailer Segment)
fn parse_fts_segment(input: &str, delimiters: &Delimiters) -> Result<FileTrailer> {
    let segment = parse_segment(input, delimiters)?;

    if segment.id != "FTS" {
        return Err(Error::parse("Expected FTS segment"));
    }

    let batch_count = segment.get_field_value(1)
        .and_then(|s| s.parse::<usize>().ok());

    let comment = segment.get_field_value(2).map(String::from);

    Ok(FileTrailer {
        batch_count,
        comment,
    })
}

/// Parse BTS (Batch Trailer Segment)
fn parse_bts_segment(input: &str, delimiters: &Delimiters) -> Result<BatchTrailer> {
    let segment = parse_segment(input, delimiters)?;

    if segment.id != "BTS" {
        return Err(Error::parse("Expected BTS segment"));
    }

    let message_count = segment.get_field_value(1)
        .and_then(|s| s.parse::<usize>().ok());

    let comment = segment.get_field_value(2).map(String::from);

    let totals = if let Some(totals_str) = segment.get_field_value(3) {
        totals_str
            .split(delimiters.repetition_separator)
            .filter_map(|s| s.parse::<f64>().ok())
            .collect()
    } else {
        Vec::new()
    };

    Ok(BatchTrailer {
        message_count,
        comment,
        totals,
    })
}

/// Helper to parse segments with MSH-like structure (FHS/BHS)
///
/// These segments have the same special structure as MSH where field 1 is the field separator
/// and field 2 is the encoding characters
fn parse_segment_like_msh(segment_type: &str, input: &str, delimiters: &Delimiters) -> Result<Segment> {
    if !input.starts_with(segment_type) {
        return Err(Error::parse(format!("{} segment must start with '{}'", segment_type, segment_type)));
    }

    let mut segment = Segment::new(segment_type);

    // Add field 1 (field separator)
    segment.add_field(Field::from_value(delimiters.field_separator.to_string()));

    // Add field 2 (encoding characters)
    segment.add_field(Field::from_value(delimiters.encoding_characters()));

    // Parse the rest of the fields starting after "XXX|^~\&"
    let field_start = 9; // "XXX|^~\&|".len()
    if input.len() <= field_start {
        return Ok(segment);
    }

    let rest = &input[field_start..];
    let field_strings: Vec<&str> = rest.split(delimiters.field_separator).collect();

    // Add remaining fields
    for field_str in field_strings {
        let field = parse_field(field_str, delimiters)?;
        segment.add_field(field);
    }

    Ok(segment)
}

/// Parse an HL7 datetime field into NaiveDateTime
///
/// HL7 datetime formats:
/// - TS (v2.3-v2.5): YYYYMMDDHHMMSS[.SSSS][+/-ZZZZ]
/// - DTM (v2.6+): YYYY[MM[DD[HH[MM[SS[.S[S[S[S]]]]]]]]][+/-ZZZZ]
fn parse_datetime_field(value: Option<&str>) -> Option<NaiveDateTime> {
    value.and_then(|s| {
        if s.is_empty() {
            return None;
        }

        // Remove timezone offset if present (we use NaiveDateTime)
        let datetime_str = s.split('+').next()
            .and_then(|s| s.split('-').next())
            .unwrap_or(s);

        // Try various datetime formats
        // Full: YYYYMMDDHHMMSS
        if datetime_str.len() >= 14 {
            if let Ok(dt) = NaiveDateTime::parse_from_str(&datetime_str[..14], "%Y%m%d%H%M%S") {
                return Some(dt);
            }
        }

        // Date only: YYYYMMDD
        if datetime_str.len() >= 8 {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(&datetime_str[..8], "%Y%m%d") {
                return Some(date.and_hms_opt(0, 0, 0)?);
            }
        }

        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

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

    #[test]
    fn test_parse_bhs_segment() {
        let bhs = "BHS|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143000||BATCH001||B12345";
        let delims = Delimiters::default();
        let header = parse_bhs_segment(bhs, &delims).unwrap();

        assert_eq!(header.sending_application, Some("SendApp".to_string()));
        assert_eq!(header.sending_facility, Some("SendFac".to_string()));
        assert_eq!(header.receiving_application, Some("RecApp".to_string()));
        assert_eq!(header.receiving_facility, Some("RecFac".to_string()));
        assert_eq!(header.batch_name_id_type, Some("BATCH001".to_string()));
        assert_eq!(header.control_id, Some("B12345".to_string()));
        assert!(header.creation_datetime.is_some());
    }

    #[test]
    fn test_parse_fhs_segment() {
        let fhs = "FHS|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143000||FILE001||F12345";
        let delims = Delimiters::default();
        let header = parse_fhs_segment(fhs, &delims).unwrap();

        assert_eq!(header.sending_application, Some("SendApp".to_string()));
        assert_eq!(header.sending_facility, Some("SendFac".to_string()));
        assert_eq!(header.file_name_id, Some("FILE001".to_string()));
        assert_eq!(header.control_id, Some("F12345".to_string()));
        assert!(header.creation_datetime.is_some());
    }

    #[test]
    fn test_parse_bts_segment() {
        let bts = "BTS|2|Batch complete";
        let delims = Delimiters::default();
        let trailer = parse_bts_segment(bts, &delims).unwrap();

        assert_eq!(trailer.message_count, Some(2));
        assert_eq!(trailer.comment, Some("Batch complete".to_string()));
    }

    #[test]
    fn test_parse_fts_segment() {
        let fts = "FTS|1|File complete";
        let delims = Delimiters::default();
        let trailer = parse_fts_segment(fts, &delims).unwrap();

        assert_eq!(trailer.batch_count, Some(1));
        assert_eq!(trailer.comment, Some("File complete".to_string()));
    }

    #[test]
    fn test_parse_batch() {
        let batch_text = "\
BHS|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143000||BATCH001||B12345
MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143000||ADT^A01|MSG001|P|2.5
PID|1|12345|67890^^^MRN|DOE^JOHN^A||19800101|M
MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143100||ADT^A01|MSG002|P|2.5
PID|1|54321|09876^^^MRN|SMITH^JANE^B||19900202|F
BTS|2";

        let batch = parse_batch(batch_text).unwrap();

        assert_eq!(batch.header.sending_application, Some("SendApp".to_string()));
        assert_eq!(batch.header.control_id, Some("B12345".to_string()));
        assert_eq!(batch.messages.len(), 2);
        assert_eq!(batch.trailer.message_count, Some(2));

        // Check first message
        assert_eq!(batch.messages[0].get_sending_application(), Some("SendApp"));
        assert_eq!(batch.messages[0].segments.len(), 2); // MSH + PID

        // Check second message
        assert_eq!(batch.messages[1].segments.len(), 2); // MSH + PID
    }

    #[test]
    fn test_parse_batch_with_validation_error() {
        // BTS says 5 messages but only 2 are present
        let batch_text = "\
BHS|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143000||BATCH001||B12345
MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143000||ADT^A01|MSG001|P|2.5
PID|1|12345|67890^^^MRN|DOE^JOHN^A||19800101|M
MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143100||ADT^A01|MSG002|P|2.5
PID|1|54321|09876^^^MRN|SMITH^JANE^B||19900202|F
BTS|5";

        let result = parse_batch(batch_text);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("message count mismatch"));
    }

    #[test]
    fn test_parse_file() {
        let file_text = "\
FHS|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143000||FILE001||F12345
BHS|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143000||BATCH001||B12345
MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143000||ADT^A01|MSG001|P|2.5
PID|1|12345|67890^^^MRN|DOE^JOHN^A||19800101|M
BTS|1
BHS|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143100||BATCH002||B12346
MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143100||ADT^A01|MSG002|P|2.5
PID|1|54321|09876^^^MRN|SMITH^JANE^B||19900202|F
BTS|1
FTS|2";

        let file = parse_file(file_text).unwrap();

        assert_eq!(file.header.sending_application, Some("SendApp".to_string()));
        assert_eq!(file.header.control_id, Some("F12345".to_string()));
        assert_eq!(file.batches.len(), 2);
        assert_eq!(file.trailer.batch_count, Some(2));

        // Check first batch
        assert_eq!(file.batches[0].header.control_id, Some("B12345".to_string()));
        assert_eq!(file.batches[0].messages.len(), 1);

        // Check second batch
        assert_eq!(file.batches[1].header.control_id, Some("B12346".to_string()));
        assert_eq!(file.batches[1].messages.len(), 1);
    }

    #[test]
    fn test_parse_file_with_validation_error() {
        // FTS says 5 batches but only 2 are present
        let file_text = "\
FHS|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143000||FILE001||F12345
BHS|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143000||BATCH001||B12345
MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143000||ADT^A01|MSG001|P|2.5
PID|1|12345|67890^^^MRN|DOE^JOHN^A||19800101|M
BTS|1
BHS|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143100||BATCH002||B12346
MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315143100||ADT^A01|MSG002|P|2.5
PID|1|54321|09876^^^MRN|SMITH^JANE^B||19900202|F
BTS|1
FTS|5";

        let result = parse_file(file_text);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("batch count mismatch"));
    }

    #[test]
    fn test_parse_datetime_field() {
        // Full timestamp
        let dt1 = parse_datetime_field(Some("20240315143000"));
        assert!(dt1.is_some());
        let dt1 = dt1.unwrap();
        assert_eq!(dt1.year(), 2024);
        assert_eq!(dt1.month(), 3);
        assert_eq!(dt1.day(), 15);
        assert_eq!(dt1.hour(), 14);
        assert_eq!(dt1.minute(), 30);
        assert_eq!(dt1.second(), 0);

        // Date only
        let dt2 = parse_datetime_field(Some("20240315"));
        assert!(dt2.is_some());
        let dt2 = dt2.unwrap();
        assert_eq!(dt2.year(), 2024);
        assert_eq!(dt2.month(), 3);
        assert_eq!(dt2.day(), 15);

        // Empty
        let dt3 = parse_datetime_field(Some(""));
        assert!(dt3.is_none());

        // None
        let dt4 = parse_datetime_field(None);
        assert!(dt4.is_none());
    }

    #[test]
    fn test_parse_batch_single_message() {
        let batch_text = "\
BHS|^~\\&|APP|FAC|||20240315||BATCH||B001
MSH|^~\\&|APP|FAC|SYS|HOSP|20240315||ADT^A01|MSG001|P|2.5
PID|1|12345
BTS|1";

        let batch = parse_batch(batch_text).unwrap();
        assert_eq!(batch.messages.len(), 1);
        assert_eq!(batch.trailer.message_count, Some(1));
    }

    #[test]
    fn test_extract_delimiters_from_bhs() {
        let bhs = "BHS|^~\\&|SendApp|SendFac";
        let delims = extract_delimiters_from_bhs(bhs).unwrap();

        assert_eq!(delims.field_separator, '|');
        assert_eq!(delims.component_separator, '^');
        assert_eq!(delims.repetition_separator, '~');
        assert_eq!(delims.escape_character, '\\');
        assert_eq!(delims.subcomponent_separator, '&');
    }

    #[test]
    fn test_extract_delimiters_from_fhs() {
        let fhs = "FHS|^~\\&|SendApp|SendFac";
        let delims = extract_delimiters_from_fhs(fhs).unwrap();

        assert_eq!(delims.field_separator, '|');
        assert_eq!(delims.component_separator, '^');
        assert_eq!(delims.repetition_separator, '~');
        assert_eq!(delims.escape_character, '\\');
        assert_eq!(delims.subcomponent_separator, '&');
    }
}
