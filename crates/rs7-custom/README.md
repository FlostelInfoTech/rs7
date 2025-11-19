# rs7-custom

Type-safe custom Z-segment support for RS7 HL7 v2.x parser.

## Overview

`rs7-custom` provides a framework for defining, parsing, and manipulating custom Z-segments in HL7 v2.x messages. Z-segments are organization-specific extensions to the HL7 standard that allow transmitting site-specific data.

## Features

- **Type-safe segment definitions** - Define custom segments with compile-time type checking
- **Declarative macro** - Easy segment definition using the `z_segment!` macro
- **Fluent builders** - Ergonomic API for creating segment instances
- **Validation hooks** - Custom business rule validation
- **Message integration** - Extension trait for seamless message manipulation
- **Zero overhead** - No impact on standard HL7 segment parsing

## Quick Start

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
rs7-custom = "0.8.0"
rs7-parser = "0.8.0"
```

Define a custom Z-segment:

```rust
use rs7_custom::{z_segment, MessageExt};
use rs7_parser::parse_message;

// Define ZPV - Patient Visit Extension
z_segment! {
    ZPV,
    id = "ZPV",
    fields = {
        1 => visit_type: String,           // Required field
        2 => visit_number: String,          // Required field
        3 => patient_class: Option<String>, // Optional field
        4 => department_code: Option<String>,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a message containing ZPV
    let hl7_message = "MSH|^~\\&|SendApp|SendFac|RecApp|RecFac|20240315||ADT^A01|MSG001|P|2.5\r\
                       PID|1|12345|67890^^^MRN|DOE^JOHN^A||19800101|M\r\
                       ZPV|OUTPATIENT|V12345|O|CARDIO";

    let message = parse_message(hl7_message)?;

    // Extract the ZPV segment
    if let Some(zpv) = message.get_custom_segment::<ZPV>()? {
        println!("Visit Type: {}", zpv.visit_type);
        println!("Visit Number: {}", zpv.visit_number);
    }

    Ok(())
}
```

## Creating Z-Segments

### Basic Segment

```rust
z_segment! {
    ZLO,  // Segment name (becomes struct name)
    id = "ZLO",  // HL7 segment ID
    fields = {
        1 => building_code: String,
        2 => floor_number: u32,
        3 => wing: Option<String>,
    }
}
```

### With Validation

Add custom validation logic to ensure data integrity:

```rust
z_segment! {
    ZCU,
    id = "ZCU",
    fields = {
        1 => customer_id: String,
        2 => balance: Option<f64>,
    },
    validate = |s: &ZCU| {
        if let Some(balance) = s.balance {
            if balance < 0.0 {
                return Err(CustomSegmentError::validation_failed(
                    "ZCU",
                    "Balance cannot be negative"
                ));
            }
        }
        Ok(())
    }
}
```

## Using Segments

### Building Segments

Use the fluent builder API:

```rust
let zpv = ZPV::builder()
    .visit_type("OUTPATIENT")
    .visit_number("V12345")
    .patient_class("O")
    .department_code("CARDIO")
    .build()?;
```

Required fields must be set, optional fields can be omitted:

```rust
let zpv = ZPV::builder()
    .visit_type("EMERGENCY")
    .visit_number("V99999")
    // patient_class and department_code omitted
    .build()?;
```

### Extracting from Messages

```rust
use rs7_custom::MessageExt;

// Get the first occurrence
if let Some(zpv) = message.get_custom_segment::<ZPV>()? {
    println!("Found: {}", zpv.visit_type);
}

// Get all occurrences
let all_zpvs = message.get_custom_segments::<ZPV>()?;
for zpv in all_zpvs {
    println!("Visit: {}", zpv.visit_number);
}

// Check existence
if message.has_custom_segment::<ZPV>() {
    println!("Message contains ZPV");
}
```

### Manipulating Messages

```rust
// Add a segment
let zpv = ZPV::builder()
    .visit_type("INPATIENT")
    .visit_number("V001")
    .build()?;
message.add_custom_segment(zpv);

// Replace a segment (replaces first occurrence)
let new_zpv = ZPV::builder()
    .visit_type("OUTPATIENT")
    .visit_number("V002")
    .build()?;
message.set_custom_segment(new_zpv)?;

// Remove all segments of a type
let removed = message.remove_custom_segments::<ZPV>();
println!("Removed {} segments", removed);
```

## Supported Field Types

The following field types are supported out of the box:

### Primitive Types
- `String` - Text fields (required)
- `Option<String>` - Optional text fields
- `u32` - Unsigned 32-bit integers (0 to 4,294,967,295)
- `Option<u32>` - Optional unsigned integers
- `i32` - Signed 32-bit integers (-2,147,483,648 to 2,147,483,647)
- `Option<i32>` - Optional signed integers
- `i64` - Signed 64-bit integers (large numbers)
- `Option<i64>` - Optional large integers
- `f64` - Floating point numbers
- `Option<f64>` - Optional floating point numbers
- `bool` - Boolean flags (true/false)
- `Option<bool>` - Optional boolean flags

### Date/Time Types (via chrono)
- `NaiveDateTime` - Timestamp without timezone
- `Option<NaiveDateTime>` - Optional timestamp
- `NaiveDate` - Date only (no time component)
- `Option<NaiveDate>` - Optional date
- `NaiveTime` - Time only (no date component)
- `Option<NaiveTime>` - Optional time
- `DateTime<Utc>` - UTC timestamp (timezone-aware)
- `Option<DateTime<Utc>>` - Optional UTC timestamp

### Repeating Fields (Vec<T>)

Repeating fields allow multiple values for a single field, following the HL7 v2.x specification. The values are separated by the repetition separator `~` (tilde) in HL7 encoding.

Supported repeating field types:
- `Vec<String>` - Multiple text values (e.g., phone numbers, email addresses)
- `Vec<u32>` - Multiple unsigned integers
- `Vec<i32>` - Multiple signed integers
- `Vec<i64>` - Multiple large integers
- `Vec<f64>` - Multiple floating point numbers
- `Vec<bool>` - Multiple boolean flags

Example with repeating fields:

```rust
z_segment! {
    ZCT,
    id = "ZCT",
    fields = {
        1 => patient_id: String,
        2 => phone_numbers: Vec<String>,      // Multiple phone numbers
        3 => emergency_contacts: Vec<u32>,    // Multiple contact IDs
        4 => verified_flags: Vec<bool>,       // Multiple flags
    }
}

// Building with multiple values
let zct = ZCT::builder()
    .patient_id("PAT-12345")
    .phone_numbers(vec![
        "555-1234".to_string(),
        "555-5678".to_string(),
        "555-9999".to_string(),
    ])
    .emergency_contacts(vec![101, 102, 103])
    .verified_flags(vec![true, false, true])
    .build()?;

// HL7 encoding uses ~ separator
// ZCT|PAT-12345|555-1234~555-5678~555-9999|101~102~103|Y~N~Y
```

**HL7 Encoding**:
- **Parsing**: `"value1~value2~value3"` → `vec!["value1", "value2", "value3"]`
- **Serialization**: `vec!["a", "b", "c"]` → `"a~b~c"`
- **Empty Vec**: `vec![]` → `""` (empty field)

### Boolean Field Parsing

Boolean fields support multiple HL7 conventions when parsing:
- **True values**: `Y`, `YES`, `T`, `TRUE`, `1` (case-insensitive)
- **False values**: `N`, `NO`, `F`, `FALSE`, `0` (case-insensitive)
- **Serialization**: Always outputs `Y` for true, `N` for false

### Date/Time Field Formats

Date/time fields use standard HL7 formats:

**NaiveDateTime** (YYYYMMDDHHMMSS):
- **Parsing**: `"20250119143000"` → Jan 19, 2025 14:30:00
- **Serialization**: `20250119143000`

**NaiveDate** (YYYYMMDD, YYYYMM, or YYYY):
- **Parsing**: `"20250119"` → Jan 19, 2025
- **Parsing**: `"202501"` → Jan 1, 2025 (defaults to first day of month)
- **Parsing**: `"2025"` → Jan 1, 2025 (defaults to first day of year)
- **Serialization**: Always `YYYYMMDD` format

**NaiveTime** (HHMMSS or HHMM):
- **Parsing**: `"143000"` → 14:30:00
- **Parsing**: `"1430"` → 14:30:00 (defaults seconds to 00)
- **Serialization**: Always `HHMMSS` format

**DateTime<Utc>** (YYYYMMDDHHMMSS):
- **Parsing**: `"20250119143000"` → 2025-01-19 14:30:00 UTC
- **Serialization**: `20250119143000` (in UTC)

Example with different types:

```rust
use chrono::{NaiveDateTime, NaiveDate, NaiveTime, DateTime, Utc};

z_segment! {
    ZMX,
    id = "ZMX",
    fields = {
        1 => id: String,                      // Required text
        2 => count: u32,                      // Required unsigned integer
        3 => temperature_delta: i32,          // Required signed integer (can be negative)
        4 => account_balance: i64,            // Required large integer
        5 => amount: Option<f64>,             // Optional decimal
        6 => is_active: bool,                 // Required boolean
        7 => verified: Option<bool>,          // Optional boolean
        8 => created_at: NaiveDateTime,       // Required timestamp
        9 => birth_date: NaiveDate,           // Required date
        10 => appointment_time: NaiveTime,    // Required time
        11 => last_updated: DateTime<Utc>,    // Required UTC timestamp
        12 => discharged_at: Option<NaiveDateTime>, // Optional timestamp
        13 => notes: Option<String>,          // Optional text
    }
}

let created = NaiveDate::from_ymd_opt(2025, 1, 19)
    .unwrap()
    .and_hms_opt(14, 30, 0)
    .unwrap();

let zmx = ZMX::builder()
    .id("MX001")
    .count(42u32)
    .temperature_delta(-5)                     // Signed integer (negative)
    .account_balance(1000000i64)               // Large integer
    .amount(123.45)                            // Optional float
    .is_active(true)                           // Boolean
    .verified(false)                           // Optional boolean
    .created_at(created)                       // DateTime
    .birth_date(NaiveDate::from_ymd_opt(1980, 6, 15).unwrap())  // Date
    .appointment_time(NaiveTime::from_hms_opt(10, 30, 0).unwrap()) // Time
    .last_updated(Utc::now())                  // UTC timestamp
    .discharged_at(created + chrono::Duration::days(3)) // Optional DateTime (3 days later)
    .notes("Sample notes")                     // Optional string
    .build()?;
```

## Message Extension Trait

The `MessageExt` trait extends `rs7_core::Message` with custom segment operations:

```rust
pub trait MessageExt {
    fn get_custom_segment<T: CustomSegment>(&self) -> Result<Option<T>>;
    fn get_custom_segments<T: CustomSegment>(&self) -> Result<Vec<T>>;
    fn has_custom_segment<T: CustomSegment>(&self) -> bool;
    fn set_custom_segment<T: CustomSegment>(&mut self, segment: T) -> Result<()>;
    fn add_custom_segment<T: CustomSegment>(&mut self, segment: T);
    fn remove_custom_segments<T: CustomSegment>(&mut self) -> usize;
}
```

Import it to access these methods on any `Message`:

```rust
use rs7_custom::MessageExt;
```

## Registry (Advanced)

The `CustomSegmentRegistry` allows dynamic registration and parsing of custom segments. This is useful for plugin systems or when segment definitions aren't known at compile time.

```rust
use rs7_custom::CustomSegmentRegistry;

// Register at application startup
CustomSegmentRegistry::global()
    .register::<ZPV>()?
    .register::<ZCU>()?;

// Check registration
if CustomSegmentRegistry::global().is_registered("ZPV") {
    println!("ZPV is registered");
}

// List all registered IDs
let ids = CustomSegmentRegistry::global().registered_ids();
```

## Error Handling

The crate provides a comprehensive error type:

```rust
use rs7_custom::CustomSegmentError;

match result {
    Err(CustomSegmentError::MissingField { field, segment }) => {
        eprintln!("Missing required field {} in {}", field, segment);
    }
    Err(CustomSegmentError::ValidationFailed { segment, reason }) => {
        eprintln!("Validation failed for {}: {}", segment, reason);
    }
    Err(CustomSegmentError::InvalidFieldValue { field, segment, value }) => {
        eprintln!("Invalid value '{}' for {}.{}", value, segment, field);
    }
    Ok(segment) => { /* use segment */ }
}
```

## Examples

See the `examples/` directory for complete working examples:

- `zpv_visit_segment.rs` - Basic Z-segment usage
- `zcu_customer_segment.rs` - Validation and error handling
- `message_manipulation.rs` - Comprehensive message operations
- `field_types.rs` - Demonstrating primitive field types (String, u32, i32, i64, f64, bool)
- `datetime_fields.rs` - Demonstrating date/time field types (NaiveDateTime, NaiveDate, NaiveTime, DateTime<Utc>)
- `repeating_fields.rs` - Demonstrating repeating fields (Vec<String>, Vec<u32>, Vec<i32>, Vec<i64>, Vec<f64>, Vec<bool>)

Run examples with:

```bash
cargo run --example zpv_visit_segment
cargo run --example zcu_customer_segment
cargo run --example message_manipulation
cargo run --example field_types
cargo run --example datetime_fields
cargo run --example repeating_fields
```

## Testing

The crate includes comprehensive unit and integration tests:

```bash
cargo test -p rs7-custom
```

## Best Practices

1. **Use validation** - Add validation rules to catch data errors early
2. **Document fields** - Use comments to explain field meanings
3. **Type appropriately** - Choose the right type (String, u32, f64) for each field
4. **Handle optionals** - Make fields `Option<T>` when they're truly optional
5. **Register early** - Register custom segments at application startup
6. **Error handling** - Always handle `Result` values from parsing operations

## Performance

The framework is designed for zero overhead:

- Macro-generated code is equivalent to hand-written implementations
- No runtime reflection or dynamic dispatch (except when using the registry)
- Validation only runs when explicitly called
- Type conversions are optimized by the compiler

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT License ([LICENSE-MIT](../../LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! Please see the main RS7 repository for contribution guidelines.
