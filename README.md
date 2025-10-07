# rs7 - HL7 v2.x Library for Rust

A comprehensive Rust library for parsing, validating, and creating HL7 v2.x healthcare messages, inspired by the Java HAPI library.

## Features

- **✅ Parsing and Serialization**: Parse HL7 pipe-delimited messages into structured data and serialize back
- **✅ Multiple HL7 Versions**: Support for HL7 v2.3, v2.4, v2.5, v2.6, v2.7, and v2.7.1
- **✅ Message Validation**: Validate messages against HL7 standards with detailed error reporting
- **✅ Schema-Based Validation**: Comprehensive schemas for all HL7 versions (2.3-2.7)
- **✅ Data Type Validation**: Format checking for all HL7 data types (dates, times, numerics, coded values, etc.)
- **✅ Vocabulary Validation**: Code set validation against HL7 standard tables (gender, patient class, processing ID, etc.)
- **✅ Terser API**: Easy field access using path notation (e.g., `PID-5-1`, `OBX(2)-5`)
- **✅ Encoding/Escaping**: Proper handling of HL7 escape sequences
- **✅ Message Builders**: Fluent API for creating messages (ADT A01-A13/A17/A28/A31/A40, ORU, ORM, OUL, OML, RDE, RAS, RDS, RGV, RRA, RRD, SIU, MDM, DFT, QRY)
- **✅ Complex Field Builders**: Builder patterns for composite data types (XPN, XAD, XTN, CX, XCN)
- **✅ Message Types**: Support for ADT (A01-A40), SIU (S12-S15), MDM (T01-T04), DFT (P03, P11), QRY (A19, Q01-Q02), BAR (P01-P02), Pharmacy (RDE, RAS, RDS, RGV, RRD, RRA), Laboratory (OUL, OML), MFN, ORM, ORU, ACK, and other message types
- **✅ ACK Generation**: Automatic acknowledgment message creation
- **✅ MLLP Support**: Network transmission using Minimal Lower Layer Protocol
- **🚀 Fast and Safe**: Built with Rust for performance and memory safety
- **📦 Modular Design**: Use only the components you need

## Architecture

```
rs7/
├── rs7-core      - Core data structures (Message, Segment, Field)
├── rs7-parser    - HL7 message parser using nom
├── rs7-validator - Message validation against HL7 standards
├── rs7-terser    - Path-based field access API
├── rs7-mllp      - MLLP protocol for network transmission
└── rs7-macros    - Derive macros for message types
```

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
rs7-core = "0.1"
rs7-parser = "0.1"
rs7-terser = "0.1"
rs7-validator = "0.1"
rs7-mllp = "0.1"  # Optional: for network support
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
```

## Terser Path Notation

The Terser API uses a simple path notation for accessing fields:

| Path | Description |
|------|-------------|
| `PID-5` | PID segment, field 5 |
| `PID-5-1` | PID segment, field 5, component 1 |
| `PID-5-1-2` | PID segment, field 5, component 1, subcomponent 2 |
| `OBX(2)-5` | Third OBX segment (0-indexed), field 5 |
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

## Benchmarks

```bash
cargo bench
```

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
- [ ] HL7 FHIR conversion utilities
- [ ] Performance optimizations
- [ ] WebAssembly support
- [ ] CLI tool for message analysis

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
| MLLP | ✅ | ✅ |
| Message Types | In progress | Comprehensive |
| HL7 FHIR | Planned | ✅ |

## Acknowledgments

Inspired by the excellent [HAPI library](https://hapifhir.github.io/hapi-hl7v2/) for Java.

## Resources

- [HL7 International](https://www.hl7.org/)
- [HL7 v2.x Documentation](https://www.hl7.org/implement/standards/product_brief.cfm?product_id=185)
- [HAPI Documentation](https://hapifhir.github.io/hapi-hl7v2/)
