# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
