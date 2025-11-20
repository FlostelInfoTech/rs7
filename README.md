# rs7 - HL7 v2.x Library for Rust

A comprehensive Rust library for parsing, validating, and creating HL7 v2.x healthcare messages, inspired by the Java HAPI library.

## Features

- **✅ Parsing and Serialization**: Parse HL7 pipe-delimited messages into structured data and serialize back
- **✅ Multiple HL7 Versions**: Support for HL7 v2.3, v2.4, v2.5, v2.6, v2.7, and v2.7.1
- **✅ Message Validation**: Validate messages against HL7 standards with detailed error reporting
- **✅ Schema-Based Validation**: Comprehensive schemas for all HL7 versions (2.3-2.7)
- **✅ Data Type Validation**: Format checking for all HL7 data types (dates, times, numerics, coded values, etc.)
- **✅ Vocabulary Validation**: Code set validation against HL7 standard tables (gender, patient class, processing ID, etc.)
- **✅ Conformance Profile Validation**: Validate messages against HL7 v2 conformance profiles (XML-based)
- **✅ Terser API**: Easy field access using path notation (e.g., `PID-5-1`, `OBX(2)-5`)
- **✅ Encoding/Escaping**: Proper handling of HL7 escape sequences
- **✅ Message Builders**: Fluent API for creating messages (ADT A01-A13/A17/A28/A31/A40, ORU, ORM, OUL, OML, RDE, RAS, RDS, RGV, RRA, RRD, SIU, MDM, DFT, QRY)
- **✅ Complex Field Builders**: Builder patterns for composite data types (XPN, XAD, XTN, CX, XCN)
- **✅ Message Types**: Support for ADT (A01-A40), SIU (S12-S15), MDM (T01-T04), DFT (P03, P11), QRY (A19, Q01-Q02), BAR (P01-P02), Pharmacy (RDE, RAS, RDS, RGV, RRD, RRA), Laboratory (OUL, OML), MFN, ORM, ORU, ACK, and other message types
- **✅ ACK Generation**: Automatic acknowledgment message creation
- **✅ MLLP Support**: Network transmission using Minimal Lower Layer Protocol (intra-organization)
- **✅ HTTP Transport**: HL7-over-HTTP support for inter-organization communication
- **✅ FHIR Conversion**: Convert HL7 v2 messages to FHIR R4 resources (Patient, Observation, Encounter, DiagnosticReport, etc.)
- **✅ Custom Z-Segments**: Type-safe framework for defining and parsing custom organization-specific Z-segments
- **🚀 Fast and Safe**: Built with Rust for performance and memory safety
- **📦 Modular Design**: Use only the components you need

## Architecture

```
rs7/
├── rs7-core        - Core data structures (Message, Segment, Field)
├── rs7-parser      - HL7 message parser using nom
├── rs7-validator   - Message validation against HL7 standards
├── rs7-conformance - Conformance profile validation (XML-based)
├── rs7-terser      - Path-based field access API
├── rs7-custom      - Type-safe custom Z-segment framework
├── rs7-mllp        - MLLP protocol for network transmission (intra-organization)
├── rs7-http        - HTTP transport for inter-organization communication
├── rs7-fhir        - HL7 v2 to FHIR R4 conversion
├── rs7-wasm        - WebAssembly bindings for JavaScript/TypeScript
├── rs7-cli         - Command-line interface for message analysis
└── rs7-macros      - Derive macros for message types
```

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
rs7-core = "0.9"
rs7-parser = "0.9"
rs7-terser = "0.9"
rs7-validator = "0.9"
rs7-conformance = "0.9"  # Optional: for conformance profile validation
rs7-custom = "0.9"       # Optional: for custom Z-segment support
rs7-mllp = "0.9"         # Optional: for MLLP network support (intra-organization)
rs7-http = "0.9"         # Optional: for HTTP transport (inter-organization)
rs7-fhir = "0.9"         # Optional: for FHIR conversion
```

### Parsing a Message

```rust
use rs7_parser::parse_message;
use rs7_terser::Terser;

let hl7 = r"MSH|^~\&|SendApp|SendFac|RecApp|RecFac|20240315||ADT^A01|12345|P|2.5
PID|1|12345|67890^^^MRN|DOE^JOHN^A||19800101|M";

// Parse the message
let message = parse_message(hl7)?;

// Access fields using Terser
let terser = Terser::new(&message);
let patient_name = terser.get("PID-5-1")?;  // "DOE"
let given_name = terser.get("PID-5-2")?;    // "JOHN"
let dob = terser.get("PID-7")?;             // "19800101"
```

### Creating a Message with Builders

```rust
use rs7_core::builders::adt::AdtBuilder;
use rs7_core::Version;

// Use the fluent builder API
let message = AdtBuilder::a01(Version::V2_5)
    .sending_application("MyApp")
    .sending_facility("MyFacility")
    .receiving_application("RecApp")
    .receiving_facility("RecFacility")
    .patient_id("12345")
    .patient_name("DOE", "JOHN")
    .date_of_birth("19800101")
    .sex("M")
    .patient_class("I")
    .assigned_location("ER^101^1")
    .build()?;

// Encode to HL7
let hl7_string = message.encode();
```

### Creating a Message Manually

```rust
use rs7_core::{Message, Segment, Field, Delimiters};
use rs7_terser::TerserMut;

let mut message = Message::new();

// Build MSH segment
let mut msh = Segment::new("MSH");
let delims = Delimiters::default();
msh.add_field(Field::from_value("|"));
msh.add_field(Field::from_value("^~\\&"));
msh.set_field_value(3, "MyApp")?;
msh.set_field_value(9, "ADT^A01")?;
message.add_segment(msh);

// Use Terser to set values
let mut terser = TerserMut::new(&mut message);
terser.set("PID-5-1", "SMITH")?;
terser.set("PID-5-2", "JOHN")?;
terser.set("PID-8", "M")?;

// Encode to HL7
let hl7_string = message.encode();
```

### Validation

```rust
use rs7_validator::Validator;
use rs7_core::Version;

// Basic validation
let validator = Validator::new(Version::V2_5);
let result = validator.validate(&message);

// Schema-based validation with data type checking
let validator = Validator::for_message_type(Version::V2_5, "ADT", "A01")?;
let result = validator.validate(&message);

if result.is_valid() {
    println!("Message is valid!");
} else {
    for error in &result.errors {
        println!("Error at {}: {}", error.location, error.message);
    }
}

// Data type validation
use rs7_core::types::DataType;
use rs7_validator::validate_data_type;

let validation = validate_data_type("20240315", DataType::DT);
if validation.is_valid() {
    println!("Valid date!");
}

// Vocabulary validation
use rs7_validator::TableRegistry;

let registry = TableRegistry::new();
let vocab_result = registry.validate("0001", "M"); // Table 0001: Administrative Sex
if vocab_result.is_valid() {
    println!("Valid gender code!");
}

// Conformance profile validation
use rs7_conformance::{ProfileParser, ConformanceValidator};

let profile = ProfileParser::parse_file("profiles/adt_a01.xml")?;
let conformance_validator = ConformanceValidator::new(profile);
let conformance_result = conformance_validator.validate(&message);

if !conformance_result.is_valid() {
    for error in &conformance_result.errors {
        println!("Conformance error at {}: {}", error.location, error.message);
    }
}
```

### MLLP Server

```rust
use rs7_mllp::MllpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = MllpServer::bind("127.0.0.1:2575").await?;

    loop {
        let mut conn = server.accept().await?;

        tokio::spawn(async move {
            let message = conn.receive_message().await?;
            // Process message...
            let ack = create_ack(&message)?;
            conn.send_message(&ack).await?;
        });
    }
}
```

### MLLP Client

```rust
use rs7_mllp::MllpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MllpClient::connect("127.0.0.1:2575").await?;

    let ack = client.send_message(&message).await?;
    println!("Received ACK: {}", ack.get_control_id().unwrap());

    client.close().await?;
}
```

### HTTP Server

```rust
use rs7_http::HttpServer;
use rs7_core::Message;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = HttpServer::new()
        .with_handler(Arc::new(|message: Message| {
            println!("Received: {:?}", message.get_message_type());
            // Create and return ACK
            Ok(message) // Simplified for example
        }));
        // Optional: .with_auth("username".into(), "password".into());

    server.serve("127.0.0.1:8080").await?;
}
```

### HTTP Client

```rust
use rs7_http::HttpClient;
use rs7_core::builders::adt::AdtBuilder;
use rs7_core::Version;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = HttpClient::new("http://example.com/hl7")?
        .with_timeout(Duration::from_secs(30))?;
        // Optional: .with_auth("username".into(), "password".into());

    let message = AdtBuilder::a01(Version::V2_5)
        .patient_id("12345")
        .patient_name("DOE", "JOHN")
        .build()?;

    let ack = client.send_message(&message).await?;
    println!("ACK received: {:?}", ack.get_control_id());
}
```

### FHIR Conversion

```rust
use rs7_fhir::prelude::*;
use rs7_parser::parse_message;

let hl7 = r"MSH|^~\&|SendApp|SendFac|RecApp|RecFac|20240315||ADT^A01|12345|P|2.5
PID|1||67890^^^MRN||DOE^JOHN^A||19800101|M|||123 Main St^^Boston^MA^02101||555-1234
PV1||I|ER^101^1||||12345^SMITH^JANE^^^MD";

let message = parse_message(hl7)?;

// Convert to FHIR resources
let patient = PatientConverter::convert(&message)?;
let encounter = EncounterConverter::convert(&message)?;
let practitioner = PractitionerConverter::convert_attending_doctor(&message)?;

// Serialize to JSON
let json = serde_json::to_string_pretty(&patient)?;
println!("{}", json);
```

**Available Converters:**
- Patient (PID → Patient)
- Observation (OBX → Observation)
- Practitioner (PV1/ORC → Practitioner)
- Encounter (PV1 → Encounter)
- DiagnosticReport (OBR → DiagnosticReport)
- AllergyIntolerance (AL1 → AllergyIntolerance)
- MedicationAdministration (RXA → MedicationAdministration)
- Condition (PRB/DG1 → Condition)
- Procedure (PR1 → Procedure)

See [rs7-fhir/README.md](crates/rs7-fhir/README.md) for complete documentation.

### Custom Z-Segments

RS7 provides a type-safe framework for working with custom organization-specific Z-segments:

```rust
use rs7_custom::{z_segment, MessageExt};
use rs7_parser::parse_message;

// Define a custom Z-segment
z_segment! {
    ZPV,  // Patient Visit Extension
    id = "ZPV",
    fields = {
        1 => visit_type: String,
        2 => visit_number: String,
        3 => patient_class: Option<String>,
        4 => department_code: Option<String>,
    }
}

// Parse a message containing the Z-segment
let hl7 = r"MSH|^~\&|SendApp|SendFac|RecApp|RecFac|20240315||ADT^A01|12345|P|2.5
PID|1|12345|67890^^^MRN|DOE^JOHN^A||19800101|M
ZPV|OUTPATIENT|V12345|O|CARDIO";

let message = parse_message(hl7)?;

// Extract the custom segment
if let Some(zpv) = message.get_custom_segment::<ZPV>()? {
    println!("Visit Type: {}", zpv.visit_type);
    println!("Visit Number: {}", zpv.visit_number);
}

// Build a Z-segment programmatically
let new_zpv = ZPV::builder()
    .visit_type("EMERGENCY")
    .visit_number("V99999")
    .patient_class("E")
    .build()?;

// Add to message
message.add_custom_segment(new_zpv);
```

**Features:**
- Type-safe segment definitions with compile-time validation
- Fluent builder API for ergonomic segment creation
- Custom validation hooks for business rules
- Support for primitive types, DateTime (chrono), Option<T>, Vec<T> (repeating fields), tuple types (components), and Vec<Tuple> (repeating components)
- Zero overhead for standard HL7 segments

See [rs7-custom/README.md](crates/rs7-custom/README.md) for complete documentation and examples.

### WebAssembly (JavaScript/TypeScript)

RS7 can be used in browsers and Node.js via WebAssembly:

```bash
npm install rs7-wasm
```

```javascript
import init, { parseMessage, getTerserValue } from 'rs7-wasm';

await init();

const hl7 = `MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315||ADT^A01|12345|P|2.5
PID|1||MRN123||DOE^JOHN||19800101|M`;

const message = parseMessage(hl7);
const patientName = getTerserValue(message, "PID-5");
console.log(patientName); // "DOE^JOHN"
```

**Features:**
- Parse and validate HL7 messages in the browser
- Full TypeScript type definitions
- Zero-copy parsing for maximum performance
- Works with all modern browsers and Node.js

See [rs7-wasm/README.md](crates/rs7-wasm/README.md) for complete documentation and examples.

### Command-Line Interface

RS7 provides a powerful CLI tool for parsing, validating, and analyzing HL7 messages:

```bash
# Install the CLI
cargo install --path crates/rs7-cli

# Parse and display message structure
rs7 parse message.hl7 --format pretty

# Validate against HL7 standards
rs7 validate message.hl7

# Extract specific fields using Terser paths
rs7 extract message.hl7 PID-5 PID-7 PID-8

# Convert to JSON
rs7 convert message.hl7 --to json --pretty

# Display comprehensive message info
rs7 info message.hl7
```

**Commands:**
- `parse` - Parse and display HL7 message structure (text, JSON, pretty formats)
- `validate` - Validate messages against HL7 standards with detailed error reports
- `extract` - Extract field values using Terser paths (supports indexing like `OBX(1)-5`)
- `convert` - Convert to JSON or FHIR R4 format
- `info` - Display comprehensive message information and statistics

**Features:**
- Colored terminal output for enhanced readability
- Support for stdin and file input
- JSON output for programmatic processing
- Batch processing capabilities
- FHIR conversion (with `--features fhir`)

See [rs7-cli/README.md](crates/rs7-cli/README.md) for complete documentation and examples.

## Examples

The `examples/` directory contains complete working examples:

- `parse_adt.rs` - Parse and analyze an ADT^A01 message
- `create_message.rs` - Build messages programmatically (manual)
- `message_builders.rs` - Build messages using the builder API
- `complex_fields.rs` - Build complex composite fields (XPN, XAD, XTN, CX, XCN)
- `schema_validation.rs` - Validate messages using schemas
- `datatype_validation.rs` - Data type format validation examples
- `enhanced_validation.rs` - Complete validation with data type checking
- `vocabulary_validation.rs` - HL7 table/code set validation examples
- `complete_validation.rs` - Full validation with data types and vocabulary
- `mllp_server.rs` - MLLP server that receives messages and sends ACKs
- `mllp_client.rs` - MLLP client that sends messages
- `http_server.rs` (rs7-http) - HTTP server that receives HL7 messages over HTTP
- `http_client.rs` (rs7-http) - HTTP client that sends HL7 messages over HTTP
- `convert_adt.rs` (rs7-fhir) - Convert ADT^A01 to FHIR Patient/Encounter
- `convert_oru.rs` (rs7-fhir) - Convert ORU^R01 to FHIR Observation/DiagnosticReport

Run examples:

```bash
cargo run --example parse_adt
cargo run --example create_message
cargo run --example message_builders
cargo run --example complex_fields
cargo run --example schema_validation
cargo run --example datatype_validation
cargo run --example enhanced_validation
cargo run --example vocabulary_validation
cargo run --example complete_validation
cargo run --example mllp_server
cargo run --example mllp_client  # In another terminal

# HTTP transport examples
cargo run --example http_server -p rs7-http
cargo run --example http_client -p rs7-http  # In another terminal

# FHIR conversion examples
cargo run --example convert_adt -p rs7-fhir
cargo run --example convert_oru -p rs7-fhir
```

## Terser Path Notation

The Terser API uses a simple path notation for accessing fields:

| Path | Description |
|------|-------------|
| `PID-5` | PID segment, field 5 |
| `PID-5-1` | PID segment, field 5, component 1 |
| `PID-5-1-2` | PID segment, field 5, component 1, subcomponent 2 |
| `OBX(2)-5` | Second OBX segment (1-indexed), field 5 |
| `PID-11(1)-1` | PID segment, field 11, second repetition, component 1 |

## HL7 Message Hierarchy

```
Message
  └─ Segment (MSH, PID, OBX, etc.)
       └─ Field (separated by |)
            └─ Repetition (separated by ~)
                 └─ Component (separated by ^)
                      └─ Subcomponent (separated by &)
```

## Supported HL7 Versions

- HL7 v2.3
- HL7 v2.3.1
- HL7 v2.4
- HL7 v2.5
- HL7 v2.5.1
- HL7 v2.6
- HL7 v2.7
- HL7 v2.7.1

## Supported Message Types

### ADT - Admit/Discharge/Transfer
- **A01** - Admit/Visit Notification
- **A02** - Transfer a Patient
- **A03** - Discharge/End Visit
- **A04** - Register a Patient
- **A05** - Pre-admit a Patient
- **A06** - Change Outpatient to Inpatient
- **A07** - Change Inpatient to Outpatient
- **A08** - Update Patient Information
- **A09** - Patient Departing - Tracking
- **A10** - Patient Arriving - Tracking
- **A11** - Cancel Admit/Visit Notification
- **A12** - Cancel Transfer
- **A13** - Cancel Discharge/End Visit
- **A17** - Swap Patients
- **A28** - Add Person Information
- **A31** - Update Person Information
- **A40** - Merge Patient - Patient Identifier List

### SIU - Scheduling Information Unsolicited
- **S12** - Notification of New Appointment Booking
- **S13** - Notification of Appointment Rescheduling
- **S14** - Notification of Appointment Modification
- **S15** - Notification of Appointment Cancellation

### MDM - Medical Document Management
- **T01** - Original Document Notification
- **T02** - Original Document Notification and Content
- **T04** - Document Status Change Notification

### DFT - Detailed Financial Transaction
- **P03** - Post Detail Financial Transaction
- **P11** - Post Detail Financial Transactions - Expanded

### QRY - Query Messages
- **A19** - Patient Query
- **Q01** - Query Sent for Immediate Response
- **Q02** - Query Sent for Deferred Response

### BAR - Billing Account Record
- **P01** - Add Patient Account
- **P02** - Purge Patient Accounts

### RDE - Pharmacy/Treatment Encoded Order
- **O11** - Pharmacy/Treatment Encoded Order

### RAS - Pharmacy/Treatment Administration
- **O17** - Pharmacy/Treatment Administration

### RDS - Pharmacy/Treatment Dispense
- **O13** - Pharmacy/Treatment Dispense

### RGV - Pharmacy/Treatment Give
- **O15** - Pharmacy/Treatment Give

### RRD - Pharmacy/Treatment Dispense Information
- **O14** - Pharmacy/Treatment Dispense Information

### RRA - Pharmacy/Treatment Administration Acknowledgment
- **O18** - Pharmacy/Treatment Administration Acknowledgment

### OUL - Unsolicited Laboratory Observation
- **R21** - Unsolicited Laboratory Observation

### OML - Laboratory Order
- **O21** - Laboratory Order

### MFN - Master File Notification
- **M01** - Master File Not Otherwise Specified

### Other Message Types
- **ORM** - Order Messages (Pharmacy, Lab, etc.)
- **ORU** - Observation Results (Lab results, etc.)
- **ACK** - General Acknowledgment

## Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p rs7-parser

# Run with output
cargo test -- --nocapture
```

## Performance

RS7 is optimized for high-throughput message processing:

- **Zero-copy parsing** with minimal allocations
- **Cached Terser** for 5-10x faster repeated field access
- **Optimized parsers** with pre-allocation for common patterns
- **Benchmarking suite** for performance validation

```bash
# Run benchmarks
cargo bench --workspace

# Parser benchmarks only
cargo bench -p rs7-parser

# Terser benchmarks only
cargo bench -p rs7-terser
```

**Typical Performance:**
- Small messages (3 segments): 2-5 µs parse time (~40,000 msg/sec)
- Medium messages (8 segments): 8-12 µs parse time (~100,000 msg/sec)
- Terser cached access: 50-100 ns (5-10x faster than first access)

See [PERFORMANCE.md](PERFORMANCE.md) for detailed optimization guide.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## Roadmap

- [x] Complete schema definitions for all HL7 versions (v2.3-2.7) ✅
- [x] Additional message type schemas (ADT A02-A40, SIU, MDM, DFT, QRY) ✅
- [x] Message builders (ADT, ORM, ORU, SIU, MDM, DFT, QRY) ✅
- [x] Enhanced data type validation (format checking) ✅
- [x] Vocabulary/code set validation ✅
- [x] More message schemas (BAR, RAS, RDE, RDS, MFN) ✅
- [x] Additional pharmacy schemas (RGV, RRD, RRA) ✅
- [x] Laboratory message schemas (OUL, OML) ✅
- [x] Additional ADT builder variants (A05-A07, A09-A13, A17, A28, A31, A40) ✅
- [x] Laboratory message builders (OUL, OML) ✅
- [x] Pharmacy message builders (RDE, RAS, RDS, RGV, RRD, RRA) ✅
- [x] Complex field builder methods (XPN, XAD, XTN, CX, XCN) ✅
- [x] HL7 FHIR conversion utilities ✅ (9 converters complete - see rs7-fhir/README.md)
- [x] Performance optimizations ✅ (Cached Terser, optimized parsers, benchmarking suite)
- [x] WebAssembly support ✅ (Full JavaScript/TypeScript bindings - see rs7-wasm/README.md)
- [x] CLI tool for message analysis ✅ (5 commands: parse, validate, extract, convert, info - see rs7-cli/README.md)
- [x] HTTP transport support ✅ (HL7-over-HTTP for inter-organization communication - see rs7-http/README.md)
- [x] Custom Z-segment framework ✅ (Type-safe custom segment support with validation - see rs7-custom/README.md)
- [x] Conformance profile validation ✅ (XML-based conformance profiles with usage, cardinality, and length validation - see rs7-conformance crate)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Comparison with HAPI

| Feature | rs7 | HAPI (Java) |
|---------|-----|-------------|
| Language | Rust | Java |
| Memory Safety | Compile-time guaranteed | Runtime checked |
| Parser | nom (zero-copy) | Custom |
| Async I/O | Tokio | Blocking/NIO |
| Terser API | ✅ | ✅ |
| Validation | ✅ | ✅ |
| Conformance Profiles | ✅ | ✅ |
| MLLP | ✅ | ✅ |
| HTTP Transport | ✅ | ✅ |
| Message Types | In progress | Comprehensive |
| HL7 FHIR | ✅ (9 converters) | ✅ |

## Acknowledgments

Inspired by the excellent [HAPI library](https://hapifhir.github.io/hapi-hl7v2/) for Java.

## Resources

- [HL7 International](https://www.hl7.org/)
- [HL7 v2.x Documentation](https://www.hl7.org/implement/standards/product_brief.cfm?product_id=185)
- [HAPI Documentation](https://hapifhir.github.io/hapi-hl7v2/)
