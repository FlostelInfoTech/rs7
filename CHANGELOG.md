# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2025-10-07

### Added
- **Additional Message Schemas** - Support for 6 new message types across all HL7 versions (2.3-2.7):
  - BAR (Billing Account Record): P01, P02
  - RDE (Pharmacy/Treatment Encoded Order): O11
  - RAS (Pharmacy/Treatment Administration): O17
  - RDS (Pharmacy/Treatment Dispense): O13
  - MFN (Master File Notification): M01
  - Total of 30 new schema files (6 types × 5 versions)
  - 6 new schema loader tests
  - Updated `list_available_schemas()` function
  - Total message schema count: 38 types (was 32)

- **Vocabulary/Code Set Validation** - Validation against HL7 standard tables:
  - TableRegistry with 13 built-in HL7 tables
  - Table 0001: Administrative Sex (M, F, O, U, etc.)
  - Table 0002: Marital Status
  - Table 0004: Patient Class (I, O, E, etc.)
  - Table 0007: Admission Type
  - Table 0061: Check Digit Scheme
  - Table 0063: Relationship
  - Table 0078: Interpretation Codes
  - Table 0085: Observation Result Status
  - Table 0103: Processing ID (P, D, T)
  - Table 0119: Order Control Codes (NW, CA, OK, etc.)
  - Table 0201: Telecommunication Use Code
  - Table 0203: Identifier Type (MR, SS, DL, etc.)
  - Table 0301: Universal ID Type
  - Support for custom/local tables
  - Deprecated code detection
  - Integration with schema-based validation
  - Field-to-table mapping via schema table_id field
- New examples: `vocabulary_validation.rs` and `complete_validation.rs`
- 8 new tests for vocabulary validation (total: 101 tests across all crates)

- **Data Type Validation** - Format validation for all HL7 data types:
  - Date/Time types (DT, TM, DTM, TS) with format verification
  - Numeric types (NM, SI) with range and format validation
  - String types (ST, TX, FT) with basic validation
  - Identifier types (ID, EI, CX, HD) with format rules
  - Coded elements (CE, CWE, CNE) with component structure validation
  - Composite types (XPN, XAD, XTN) for names, addresses, and telecom
  - Message type (MSG) and processing type (PT) validation
  - Numeric array (NA) validation
- Integrated data type validation into the schema-based validator
- New examples: `datatype_validation.rs` and `enhanced_validation.rs`
- Comprehensive test suite for data type validation (19 new tests)

## [0.1.1] - 2025-10-07

### Added
- **Message Builders** - Fluent builder API for creating HL7 messages programmatically:
  - `AdtBuilder` with support for A01, A02, A03, A04, A08
  - `OruR01Builder` for observation results
  - `OrmO01Builder` for orders
  - `SiuS12Builder` for scheduling
  - `MdmT01Builder` for medical documents
  - `DftP03Builder` for financial transactions
  - `QryA19Builder`, `QryQ01Builder`, `QryQ02Builder` for query messages
- **29 new message schemas** across all HL7 versions (v2.3-v2.7):
  - **ADT messages**: A02, A03, A04, A05, A06, A07, A08, A09, A10, A11, A12, A13, A17, A28, A31, A40
  - **SIU messages**: S12, S13, S14, S15 (Scheduling Information)
  - **MDM messages**: T01, T02, T04 (Medical Document Management)
  - **DFT messages**: P03, P11 (Detailed Financial Transactions)
  - **QRY messages**: A19, Q01, Q02 (Query Messages)
- Expanded trigger event constants in `message::trigger_events` module with all new message types
- Updated `list_available_schemas()` to include all 32 message schemas
- New example: `message_builders.rs` demonstrating builder API usage
- Comprehensive documentation updates in README.md and schemas/README.md

### Changed
- Schema loader now supports 32 total message schemas (up from 4)
- Total of 160 schema files across all HL7 versions (32 schemas × 5 versions)

## [0.1.0] - Initial Release

### Added
- Core HL7 v2.x data structures (Message, Segment, Field, Component, Subcomponent)
- Parser using nom for zero-copy parsing
- Support for HL7 v2.3, v2.3.1, v2.4, v2.5, v2.5.1, v2.6, v2.7, v2.7.1
- Terser API for path-based field access
- Message validation against HL7 standards
- Schema-based validation with initial schemas:
  - ADT^A01 (Admit/Visit Notification)
  - ORU^R01 (Observation Result)
  - ORM^O01 (Order Message)
  - ACK (Acknowledgment)
- MLLP protocol support for network transmission
- HL7 encoding and escape sequence handling
- ACK message generation
- Comprehensive test coverage
- Documentation and examples

[Unreleased]: https://github.com/yourusername/rs7/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/yourusername/rs7/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/yourusername/rs7/releases/tag/v0.1.0
