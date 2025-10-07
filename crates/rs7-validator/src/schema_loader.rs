//! Schema loader for HL7 message definitions
//!
//! This module provides functionality to load message schemas from JSON files
//! embedded in the binary at compile time.

use crate::MessageSchema;
use rs7_core::{error::Result, Version};

/// Load a schema for a specific message type and version
pub fn load_schema(version: Version, message_type: &str, trigger_event: &str) -> Result<MessageSchema> {
    let version_str = match version {
        Version::V2_3 | Version::V2_3_1 => "v2_3",
        Version::V2_4 => "v2_4",
        Version::V2_5 | Version::V2_5_1 => "v2_5",
        Version::V2_6 => "v2_6",
        Version::V2_7 | Version::V2_7_1 => "v2_7",
    };

    // Construct the schema key
    let schema_key = if trigger_event.is_empty() {
        message_type.to_string()
    } else {
        format!("{}_{}", message_type, trigger_event)
    };

    // Try to load embedded schema
    match load_embedded_schema(version_str, &schema_key) {
        Some(schema) => Ok(schema),
        None => Err(rs7_core::error::Error::Validation(format!(
            "Schema not found for {} {} version {}",
            message_type, trigger_event, version.as_str()
        ))),
    }
}

/// Load an embedded schema from compile-time included JSON
fn load_embedded_schema(version: &str, schema_key: &str) -> Option<MessageSchema> {
    // Include schemas at compile time
    match (version, schema_key) {
        // V2.3 schemas - ADT messages
        ("v2_3", "ADT_A01") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A01.json")),
        ("v2_3", "ADT_A02") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A02.json")),
        ("v2_3", "ADT_A03") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A03.json")),
        ("v2_3", "ADT_A04") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A04.json")),
        ("v2_3", "ADT_A05") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A05.json")),
        ("v2_3", "ADT_A06") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A06.json")),
        ("v2_3", "ADT_A07") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A07.json")),
        ("v2_3", "ADT_A08") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A08.json")),
        ("v2_3", "ADT_A09") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A09.json")),
        ("v2_3", "ADT_A10") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A10.json")),
        ("v2_3", "ADT_A11") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A11.json")),
        ("v2_3", "ADT_A12") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A12.json")),
        ("v2_3", "ADT_A13") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A13.json")),
        ("v2_3", "ADT_A17") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A17.json")),
        ("v2_3", "ADT_A28") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A28.json")),
        ("v2_3", "ADT_A31") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A31.json")),
        ("v2_3", "ADT_A40") => parse_schema_json(include_str!("../schemas/v2_3/ADT_A40.json")),
        // V2.3 - Other message types
        ("v2_3", "ORU_R01") => parse_schema_json(include_str!("../schemas/v2_3/ORU_R01.json")),
        ("v2_3", "ORM_O01") => parse_schema_json(include_str!("../schemas/v2_3/ORM_O01.json")),
        ("v2_3", "ACK") => parse_schema_json(include_str!("../schemas/v2_3/ACK.json")),
        ("v2_3", "SIU_S12") => parse_schema_json(include_str!("../schemas/v2_3/SIU_S12.json")),
        ("v2_3", "SIU_S13") => parse_schema_json(include_str!("../schemas/v2_3/SIU_S13.json")),
        ("v2_3", "SIU_S14") => parse_schema_json(include_str!("../schemas/v2_3/SIU_S14.json")),
        ("v2_3", "SIU_S15") => parse_schema_json(include_str!("../schemas/v2_3/SIU_S15.json")),
        ("v2_3", "MDM_T01") => parse_schema_json(include_str!("../schemas/v2_3/MDM_T01.json")),
        ("v2_3", "MDM_T02") => parse_schema_json(include_str!("../schemas/v2_3/MDM_T02.json")),
        ("v2_3", "MDM_T04") => parse_schema_json(include_str!("../schemas/v2_3/MDM_T04.json")),
        ("v2_3", "DFT_P03") => parse_schema_json(include_str!("../schemas/v2_3/DFT_P03.json")),
        ("v2_3", "DFT_P11") => parse_schema_json(include_str!("../schemas/v2_3/DFT_P11.json")),
        ("v2_3", "QRY_A19") => parse_schema_json(include_str!("../schemas/v2_3/QRY_A19.json")),
        ("v2_3", "QRY_Q01") => parse_schema_json(include_str!("../schemas/v2_3/QRY_Q01.json")),
        ("v2_3", "QRY_Q02") => parse_schema_json(include_str!("../schemas/v2_3/QRY_Q02.json")),

        // V2.4 schemas - ADT messages
        ("v2_4", "ADT_A01") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A01.json")),
        ("v2_4", "ADT_A02") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A02.json")),
        ("v2_4", "ADT_A03") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A03.json")),
        ("v2_4", "ADT_A04") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A04.json")),
        ("v2_4", "ADT_A05") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A05.json")),
        ("v2_4", "ADT_A06") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A06.json")),
        ("v2_4", "ADT_A07") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A07.json")),
        ("v2_4", "ADT_A08") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A08.json")),
        ("v2_4", "ADT_A09") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A09.json")),
        ("v2_4", "ADT_A10") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A10.json")),
        ("v2_4", "ADT_A11") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A11.json")),
        ("v2_4", "ADT_A12") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A12.json")),
        ("v2_4", "ADT_A13") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A13.json")),
        ("v2_4", "ADT_A17") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A17.json")),
        ("v2_4", "ADT_A28") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A28.json")),
        ("v2_4", "ADT_A31") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A31.json")),
        ("v2_4", "ADT_A40") => parse_schema_json(include_str!("../schemas/v2_4/ADT_A40.json")),
        // V2.4 - Other message types
        ("v2_4", "ORU_R01") => parse_schema_json(include_str!("../schemas/v2_4/ORU_R01.json")),
        ("v2_4", "ORM_O01") => parse_schema_json(include_str!("../schemas/v2_4/ORM_O01.json")),
        ("v2_4", "ACK") => parse_schema_json(include_str!("../schemas/v2_4/ACK.json")),
        ("v2_4", "SIU_S12") => parse_schema_json(include_str!("../schemas/v2_4/SIU_S12.json")),
        ("v2_4", "SIU_S13") => parse_schema_json(include_str!("../schemas/v2_4/SIU_S13.json")),
        ("v2_4", "SIU_S14") => parse_schema_json(include_str!("../schemas/v2_4/SIU_S14.json")),
        ("v2_4", "SIU_S15") => parse_schema_json(include_str!("../schemas/v2_4/SIU_S15.json")),
        ("v2_4", "MDM_T01") => parse_schema_json(include_str!("../schemas/v2_4/MDM_T01.json")),
        ("v2_4", "MDM_T02") => parse_schema_json(include_str!("../schemas/v2_4/MDM_T02.json")),
        ("v2_4", "MDM_T04") => parse_schema_json(include_str!("../schemas/v2_4/MDM_T04.json")),
        ("v2_4", "DFT_P03") => parse_schema_json(include_str!("../schemas/v2_4/DFT_P03.json")),
        ("v2_4", "DFT_P11") => parse_schema_json(include_str!("../schemas/v2_4/DFT_P11.json")),
        ("v2_4", "QRY_A19") => parse_schema_json(include_str!("../schemas/v2_4/QRY_A19.json")),
        ("v2_4", "QRY_Q01") => parse_schema_json(include_str!("../schemas/v2_4/QRY_Q01.json")),
        ("v2_4", "QRY_Q02") => parse_schema_json(include_str!("../schemas/v2_4/QRY_Q02.json")),

        // V2.5 schemas - ADT messages
        ("v2_5", "ADT_A01") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A01.json")),
        ("v2_5", "ADT_A02") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A02.json")),
        ("v2_5", "ADT_A03") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A03.json")),
        ("v2_5", "ADT_A04") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A04.json")),
        ("v2_5", "ADT_A05") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A05.json")),
        ("v2_5", "ADT_A06") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A06.json")),
        ("v2_5", "ADT_A07") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A07.json")),
        ("v2_5", "ADT_A08") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A08.json")),
        ("v2_5", "ADT_A09") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A09.json")),
        ("v2_5", "ADT_A10") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A10.json")),
        ("v2_5", "ADT_A11") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A11.json")),
        ("v2_5", "ADT_A12") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A12.json")),
        ("v2_5", "ADT_A13") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A13.json")),
        ("v2_5", "ADT_A17") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A17.json")),
        ("v2_5", "ADT_A28") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A28.json")),
        ("v2_5", "ADT_A31") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A31.json")),
        ("v2_5", "ADT_A40") => parse_schema_json(include_str!("../schemas/v2_5/ADT_A40.json")),
        // V2.5 - Other message types
        ("v2_5", "ORU_R01") => parse_schema_json(include_str!("../schemas/v2_5/ORU_R01.json")),
        ("v2_5", "ORM_O01") => parse_schema_json(include_str!("../schemas/v2_5/ORM_O01.json")),
        ("v2_5", "ACK") => parse_schema_json(include_str!("../schemas/v2_5/ACK.json")),
        ("v2_5", "SIU_S12") => parse_schema_json(include_str!("../schemas/v2_5/SIU_S12.json")),
        ("v2_5", "SIU_S13") => parse_schema_json(include_str!("../schemas/v2_5/SIU_S13.json")),
        ("v2_5", "SIU_S14") => parse_schema_json(include_str!("../schemas/v2_5/SIU_S14.json")),
        ("v2_5", "SIU_S15") => parse_schema_json(include_str!("../schemas/v2_5/SIU_S15.json")),
        ("v2_5", "MDM_T01") => parse_schema_json(include_str!("../schemas/v2_5/MDM_T01.json")),
        ("v2_5", "MDM_T02") => parse_schema_json(include_str!("../schemas/v2_5/MDM_T02.json")),
        ("v2_5", "MDM_T04") => parse_schema_json(include_str!("../schemas/v2_5/MDM_T04.json")),
        ("v2_5", "DFT_P03") => parse_schema_json(include_str!("../schemas/v2_5/DFT_P03.json")),
        ("v2_5", "DFT_P11") => parse_schema_json(include_str!("../schemas/v2_5/DFT_P11.json")),
        ("v2_5", "QRY_A19") => parse_schema_json(include_str!("../schemas/v2_5/QRY_A19.json")),
        ("v2_5", "QRY_Q01") => parse_schema_json(include_str!("../schemas/v2_5/QRY_Q01.json")),
        ("v2_5", "QRY_Q02") => parse_schema_json(include_str!("../schemas/v2_5/QRY_Q02.json")),

        // V2.6 schemas - ADT messages
        ("v2_6", "ADT_A01") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A01.json")),
        ("v2_6", "ADT_A02") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A02.json")),
        ("v2_6", "ADT_A03") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A03.json")),
        ("v2_6", "ADT_A04") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A04.json")),
        ("v2_6", "ADT_A05") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A05.json")),
        ("v2_6", "ADT_A06") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A06.json")),
        ("v2_6", "ADT_A07") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A07.json")),
        ("v2_6", "ADT_A08") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A08.json")),
        ("v2_6", "ADT_A09") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A09.json")),
        ("v2_6", "ADT_A10") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A10.json")),
        ("v2_6", "ADT_A11") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A11.json")),
        ("v2_6", "ADT_A12") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A12.json")),
        ("v2_6", "ADT_A13") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A13.json")),
        ("v2_6", "ADT_A17") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A17.json")),
        ("v2_6", "ADT_A28") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A28.json")),
        ("v2_6", "ADT_A31") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A31.json")),
        ("v2_6", "ADT_A40") => parse_schema_json(include_str!("../schemas/v2_6/ADT_A40.json")),
        // V2.6 - Other message types
        ("v2_6", "ORU_R01") => parse_schema_json(include_str!("../schemas/v2_6/ORU_R01.json")),
        ("v2_6", "ORM_O01") => parse_schema_json(include_str!("../schemas/v2_6/ORM_O01.json")),
        ("v2_6", "ACK") => parse_schema_json(include_str!("../schemas/v2_6/ACK.json")),
        ("v2_6", "SIU_S12") => parse_schema_json(include_str!("../schemas/v2_6/SIU_S12.json")),
        ("v2_6", "SIU_S13") => parse_schema_json(include_str!("../schemas/v2_6/SIU_S13.json")),
        ("v2_6", "SIU_S14") => parse_schema_json(include_str!("../schemas/v2_6/SIU_S14.json")),
        ("v2_6", "SIU_S15") => parse_schema_json(include_str!("../schemas/v2_6/SIU_S15.json")),
        ("v2_6", "MDM_T01") => parse_schema_json(include_str!("../schemas/v2_6/MDM_T01.json")),
        ("v2_6", "MDM_T02") => parse_schema_json(include_str!("../schemas/v2_6/MDM_T02.json")),
        ("v2_6", "MDM_T04") => parse_schema_json(include_str!("../schemas/v2_6/MDM_T04.json")),
        ("v2_6", "DFT_P03") => parse_schema_json(include_str!("../schemas/v2_6/DFT_P03.json")),
        ("v2_6", "DFT_P11") => parse_schema_json(include_str!("../schemas/v2_6/DFT_P11.json")),
        ("v2_6", "QRY_A19") => parse_schema_json(include_str!("../schemas/v2_6/QRY_A19.json")),
        ("v2_6", "QRY_Q01") => parse_schema_json(include_str!("../schemas/v2_6/QRY_Q01.json")),
        ("v2_6", "QRY_Q02") => parse_schema_json(include_str!("../schemas/v2_6/QRY_Q02.json")),

        // V2.7 schemas - ADT messages
        ("v2_7", "ADT_A01") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A01.json")),
        ("v2_7", "ADT_A02") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A02.json")),
        ("v2_7", "ADT_A03") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A03.json")),
        ("v2_7", "ADT_A04") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A04.json")),
        ("v2_7", "ADT_A05") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A05.json")),
        ("v2_7", "ADT_A06") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A06.json")),
        ("v2_7", "ADT_A07") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A07.json")),
        ("v2_7", "ADT_A08") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A08.json")),
        ("v2_7", "ADT_A09") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A09.json")),
        ("v2_7", "ADT_A10") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A10.json")),
        ("v2_7", "ADT_A11") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A11.json")),
        ("v2_7", "ADT_A12") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A12.json")),
        ("v2_7", "ADT_A13") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A13.json")),
        ("v2_7", "ADT_A17") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A17.json")),
        ("v2_7", "ADT_A28") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A28.json")),
        ("v2_7", "ADT_A31") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A31.json")),
        ("v2_7", "ADT_A40") => parse_schema_json(include_str!("../schemas/v2_7/ADT_A40.json")),
        // V2.7 - Other message types
        ("v2_7", "ORU_R01") => parse_schema_json(include_str!("../schemas/v2_7/ORU_R01.json")),
        ("v2_7", "ORM_O01") => parse_schema_json(include_str!("../schemas/v2_7/ORM_O01.json")),
        ("v2_7", "ACK") => parse_schema_json(include_str!("../schemas/v2_7/ACK.json")),
        ("v2_7", "SIU_S12") => parse_schema_json(include_str!("../schemas/v2_7/SIU_S12.json")),
        ("v2_7", "SIU_S13") => parse_schema_json(include_str!("../schemas/v2_7/SIU_S13.json")),
        ("v2_7", "SIU_S14") => parse_schema_json(include_str!("../schemas/v2_7/SIU_S14.json")),
        ("v2_7", "SIU_S15") => parse_schema_json(include_str!("../schemas/v2_7/SIU_S15.json")),
        ("v2_7", "MDM_T01") => parse_schema_json(include_str!("../schemas/v2_7/MDM_T01.json")),
        ("v2_7", "MDM_T02") => parse_schema_json(include_str!("../schemas/v2_7/MDM_T02.json")),
        ("v2_7", "MDM_T04") => parse_schema_json(include_str!("../schemas/v2_7/MDM_T04.json")),
        ("v2_7", "DFT_P03") => parse_schema_json(include_str!("../schemas/v2_7/DFT_P03.json")),
        ("v2_7", "DFT_P11") => parse_schema_json(include_str!("../schemas/v2_7/DFT_P11.json")),
        ("v2_7", "QRY_A19") => parse_schema_json(include_str!("../schemas/v2_7/QRY_A19.json")),
        ("v2_7", "QRY_Q01") => parse_schema_json(include_str!("../schemas/v2_7/QRY_Q01.json")),
        ("v2_7", "QRY_Q02") => parse_schema_json(include_str!("../schemas/v2_7/QRY_Q02.json")),

        _ => None,
    }
}

/// Parse a JSON schema string into a MessageSchema
fn parse_schema_json(json: &str) -> Option<MessageSchema> {
    serde_json::from_str(json).ok()
}

/// Get a list of all available schemas
pub fn list_available_schemas(version: Version) -> Vec<String> {
    let schemas = vec![
        // ADT messages
        "ADT^A01".to_string(), "ADT^A02".to_string(), "ADT^A03".to_string(),
        "ADT^A04".to_string(), "ADT^A05".to_string(), "ADT^A06".to_string(),
        "ADT^A07".to_string(), "ADT^A08".to_string(), "ADT^A09".to_string(),
        "ADT^A10".to_string(), "ADT^A11".to_string(), "ADT^A12".to_string(),
        "ADT^A13".to_string(), "ADT^A17".to_string(), "ADT^A28".to_string(),
        "ADT^A31".to_string(), "ADT^A40".to_string(),
        // Order messages
        "ORU^R01".to_string(),
        "ORM^O01".to_string(),
        // Acknowledgment
        "ACK".to_string(),
        // Scheduling
        "SIU^S12".to_string(), "SIU^S13".to_string(), "SIU^S14".to_string(), "SIU^S15".to_string(),
        // Medical Document Management
        "MDM^T01".to_string(), "MDM^T02".to_string(), "MDM^T04".to_string(),
        // Financial
        "DFT^P03".to_string(), "DFT^P11".to_string(),
        // Query
        "QRY^A19".to_string(), "QRY^Q01".to_string(), "QRY^Q02".to_string(),
    ];

    // All versions currently have the same set of schemas
    match version {
        Version::V2_3 | Version::V2_3_1 => schemas,
        Version::V2_4 => schemas,
        Version::V2_5 | Version::V2_5_1 => schemas,
        Version::V2_6 => schemas,
        Version::V2_7 | Version::V2_7_1 => schemas,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_adt_a01_schema() {
        let schema = load_schema(Version::V2_5, "ADT", "A01").unwrap();
        assert_eq!(schema.message_type, "ADT");
        assert_eq!(schema.trigger_event, "A01");
        assert_eq!(schema.version, "2.5");
        assert!(schema.segments.contains_key("MSH"));
        assert!(schema.segments.contains_key("PID"));
    }

    #[test]
    fn test_load_oru_r01_schema() {
        let schema = load_schema(Version::V2_5, "ORU", "R01").unwrap();
        assert_eq!(schema.message_type, "ORU");
        assert!(schema.segments.contains_key("OBR"));
        assert!(schema.segments.contains_key("OBX"));
    }

    #[test]
    fn test_load_ack_schema() {
        let schema = load_schema(Version::V2_5, "ACK", "").unwrap();
        assert_eq!(schema.message_type, "ACK");
        assert!(schema.segments.contains_key("MSA"));
    }

    #[test]
    fn test_load_nonexistent_schema() {
        let result = load_schema(Version::V2_5, "INVALID", "XXX");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_available_schemas() {
        let schemas = list_available_schemas(Version::V2_5);
        assert!(schemas.contains(&"ADT^A01".to_string()));
        assert!(schemas.contains(&"ORU^R01".to_string()));
        assert!(schemas.len() >= 3);
    }
}
