# RS7 Enhancement Roadmap

This document outlines the identified gaps between RS7 and leading HL7 libraries (HAPI, HL7apy, python-hl7) and the implementation plan to address them.

## Gap Analysis Summary

| Category | Gap | Priority | Status |
|----------|-----|----------|--------|
| Parser | XML Encoding Support | High | Planned |
| Parser | Streaming Parser | Medium | Planned |
| Parser | Lenient/Strict Modes | Medium | Planned |
| Versions | v2.1, v2.2 Support | Medium | Planned |
| Versions | v2.8.x Support | High | Planned |
| ACK/NAK | Automatic ACK Generation | High | Planned |
| Terser | Segment Group Navigation | Medium | Planned |
| Terser | Wildcard Pattern Matching | Low | Planned |
| Macros | Derive Macro Implementation | High | Planned |
| Validation | Conformance Profile Completion | Medium | Planned |
| Network | Connection Pooling | Medium | Planned |
| Network | Application Router | High | Planned |
| Network | Two-Port MLLP | Low | Planned |
| FHIR | Bidirectional Conversion | High | Planned |
| FHIR | Bundle Support | Medium | Planned |
| DX | Enhanced Error Messages | Medium | Planned |

## Detailed Gap Descriptions

### 1. ACK/NAK Generation (High Priority)

**Current State**: Limited ACK support. No automatic ACK generation from incoming messages.

**Target State**: Full ACK/NAK builder with:
- Automatic MSA segment generation from incoming MSH
- Support for Original Mode (AA, AE, AR) and Enhanced Mode (CA, CE, CR)
- ERR segment generation for error details
- MSH-15/MSH-16 acknowledgment condition handling

**Implementation**: `crates/rs7-core/src/builders/ack.rs`

### 2. Derive Macros (High Priority)

**Current State**: `rs7-macros` contains placeholder implementations returning empty `TokenStream`.

**Target State**: Working derive macros for:
- `#[derive(Segment)]` - Type-safe segment definitions
- `#[derive(Message)]` - Type-safe message structures
- `#[hl7_type]` - Custom data type definitions

**Implementation**: `crates/rs7-macros/src/lib.rs`

### 3. XML Encoding Support (High Priority)

**Current State**: Only ER7 (pipe-delimited) format supported.

**Target State**: New `rs7-xml` crate providing:
- Parse HL7 v2.x XML format
- Encode messages to XML
- Bidirectional ER7 ↔ XML conversion
- XML schema validation

**Implementation**: `crates/rs7-xml/` (new crate)

### 4. Extended Version Support (High Priority)

**Current State**: v2.3 through v2.7.1 supported.

**Target State**: Support for:
- v2.1, v2.2 (legacy systems)
- v2.8, v2.8.1, v2.8.2 (latest standard)

**Implementation**: `crates/rs7-core/src/lib.rs`, schema additions

### 5. Application Router (High Priority)

**Current State**: Basic MLLP server accepts connections but no message routing.

**Target State**: Route messages to handlers based on message type/trigger:
```rust
server.route("ADT", "*", handle_adt);
server.route("ORU", "R01", handle_lab_result);
server.route_default(handle_unknown);
```

**Implementation**: `crates/rs7-mllp/src/router.rs`

### 6. Streaming Parser (Medium Priority)

**Current State**: Parser loads entire message into memory.

**Target State**: Streaming parser for large messages:
```rust
pub fn parse_streaming<R: Read>(reader: R) -> impl Iterator<Item = Result<Segment>>
```

**Implementation**: `crates/rs7-parser/src/streaming.rs`

### 7. Lenient/Strict Parsing Modes (Medium Priority)

**Current State**: Single parsing mode with fixed strictness.

**Target State**: Configurable parsing:
```rust
ParserConfig::new()
    .strict_segment_ids(false)
    .allow_trailing_delimiters(true)
    .recover_from_errors(true)
```

**Implementation**: `crates/rs7-parser/src/config.rs`

### 8. Segment Group Navigation in Terser (Medium Priority)

**Current State**: Terser uses flat segment indexing.

**Target State**: HAPI-style group paths:
```rust
terser.get("/PATIENT_RESULT(0)/ORDER_OBSERVATION(1)/OBX-5")
terser.get_all("OBX(*)-5")
```

**Implementation**: `crates/rs7-terser/src/path.rs`

### 9. Connection Pooling (Medium Priority)

**Current State**: Each client creates a new connection.

**Target State**: Connection pool for MLLP:
```rust
let pool = MllpConnectionPool::builder()
    .max_connections(10)
    .idle_timeout(Duration::from_secs(300))
    .build("host:port").await?;
```

**Implementation**: `crates/rs7-mllp/src/pool.rs`

### 10. Bidirectional FHIR Conversion (High Priority)

**Current State**: Only HL7 v2 → FHIR conversion.

**Target State**: FHIR → HL7 v2 conversion for all resource types.

**Implementation**: `crates/rs7-fhir/src/converters/reverse/`

### 11. Enhanced Error Messages (Medium Priority)

**Current State**: Basic error messages without location context.

**Target State**: Rich error context:
```rust
Error::ParseError {
    message: "Invalid date format",
    segment: "PID",
    field: 7,
    value: "invalid-date",
    line: 3,
}
```

**Implementation**: `crates/rs7-core/src/error.rs`

## Implementation Order

1. ACK/NAK Generation - Foundation for all network communication
2. Extended Version Support - Enables broader compatibility
3. Derive Macros - Improves developer experience significantly
4. Application Router - Essential for production MLLP servers
5. Lenient/Strict Parsing - Handles real-world malformed messages
6. Enhanced Error Messages - Improves debugging experience
7. Segment Group Navigation - Advanced Terser features
8. Streaming Parser - Large message support
9. Connection Pooling - Production scalability
10. XML Encoding Support - Standard compliance
11. Bidirectional FHIR - Complete interoperability

## References

- [HAPI HL7 v2](https://hapifhir.github.io/hapi-hl7v2/)
- [HL7apy](https://crs4.github.io/hl7apy/)
- [NIST HL7 v2 Testing Tools](https://www.nist.gov/itl/products-and-services/healthcare-standards-testing/testing-tools/hl7-v2-conformance-testing)
- [HL7 ACK Best Practices](https://datica-2019.netlify.app/academy/hl7-202-the-hl7-ack-acknowledgement-message/)
- [HL7 XML Encoding Rules](http://v2plus.hl7.org/2021Jan/xml-encoding-rules.html)
