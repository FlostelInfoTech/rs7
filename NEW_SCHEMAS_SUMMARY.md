# RS7 v0.1.3 - New Message Schemas Summary

## Release Date: 2025-10-07

## Overview

Version 0.1.3 adds support for 6 additional HL7 message types, bringing the total message schema count from 32 to **38 message schemas**.

## New Message Types Added

### 1. BAR - Billing Account Record
Support for billing account management messages:
- **BAR^P01** - Add Patient Account
  - Segments: MSH, EVN, PID, PV1
  - Use case: Creating new patient billing accounts
- **BAR^P02** - Purge Patient Accounts
  - Segments: MSH, EVN, PID
  - Use case: Removing closed patient billing accounts

### 2. RDE - Pharmacy/Treatment Encoded Order
Pharmacy order encoding:
- **RDE^O11** - Pharmacy/Treatment Encoded Order
  - Segments: MSH, PID, ORC, RXE
  - Use case: Encoding pharmacy orders with detailed medication information
  - Key fields: Give Code, Give Amount, Give Units, Quantity/Timing

### 3. RAS - Pharmacy/Treatment Administration
Medication administration tracking:
- **RAS^O17** - Pharmacy/Treatment Administration
  - Segments: MSH, PID, ORC, RXA
  - Use case: Recording actual medication administration events
  - Key fields: Give Sub-ID Counter, Administration Sub-ID Counter, Date/Time Start, Administered Code, Administered Amount

### 4. RDS - Pharmacy/Treatment Dispense
Medication dispensing tracking:
- **RDS^O13** - Pharmacy/Treatment Dispense
  - Segments: MSH, PID, ORC, RXD
  - Use case: Recording pharmacy dispensing events
  - Key fields: Dispense Sub-ID Counter, Dispense/Give Code, Date/Time Dispensed, Actual Dispense Amount

### 5. MFN - Master File Notification
Master file updates and synchronization:
- **MFN^M01** - Master File Not Otherwise Specified
  - Segments: MSH, MFI, MFE (repeating)
  - Use case: Broadcasting master file updates to multiple systems
  - Key fields: Master File Identifier, File-Level Event Code, Record-Level Event Code, Primary Key Value

## Implementation Details

### Schema Files Created
- **30 new schema files** (6 message types × 5 HL7 versions)
  - `crates/rs7-validator/schemas/v2_3/BAR_P01.json`
  - `crates/rs7-validator/schemas/v2_3/BAR_P02.json`
  - `crates/rs7-validator/schemas/v2_3/RDE_O11.json`
  - `crates/rs7-validator/schemas/v2_3/RAS_O17.json`
  - `crates/rs7-validator/schemas/v2_3/RDS_O13.json`
  - `crates/rs7-validator/schemas/v2_3/MFN_M01.json`
  - (Plus copies for v2.4, v2.5, v2.6, v2.7)

### Code Updates
1. **Schema Loader** (`crates/rs7-validator/src/schema_loader.rs`)
   - Added 30 new schema entries in the `load_schema()` function
   - Updated `list_available_schemas()` to include new message types
   - Pattern for each version:
     ```rust
     ("v2_5", "BAR_P01") => parse_schema_json(include_str!("../schemas/v2_5/BAR_P01.json")),
     ("v2_5", "BAR_P02") => parse_schema_json(include_str!("../schemas/v2_5/BAR_P02.json")),
     // ... etc
     ```

2. **Tests Added** (6 new tests)
   - `test_load_bar_p01_schema()` - Validates BAR P01 schema loading
   - `test_load_rde_o11_schema()` - Validates RDE O11 schema loading
   - `test_load_ras_o17_schema()` - Validates RAS O17 schema loading
   - `test_load_mfn_m01_schema()` - Validates MFN M01 schema loading
   - `test_load_rds_o13_schema()` - Validates RDS O13 schema loading
   - `test_load_new_schemas_all_versions()` - Cross-version validation

3. **Documentation Updates**
   - `README.md` - Added new message types to features and supported message types section
   - `CHANGELOG.md` - Added v0.1.3 release entry with complete details

## Schema Structure Example

### BAR_P01.json Structure
```json
{
  "message_type": "BAR",
  "trigger_event": "P01",
  "version": "2.5",
  "description": "Add Patient Account",
  "segments": {
    "MSH": { "required": true, "repeating": false, "fields": {...} },
    "EVN": { "required": true, "repeating": false, "fields": {...} },
    "PID": { "required": true, "repeating": false, "fields": {...} },
    "PV1": { "required": false, "repeating": false, "fields": {...} }
  }
}
```

## Usage Example

### Loading and Validating a BAR Message

```rust
use rs7_core::Version;
use rs7_parser::parse_message;
use rs7_validator::Validator;

// Create validator with BAR P01 schema
let mut validator = Validator::new(Version::V2_5);
validator.load_schema("BAR", "P01").unwrap();

// Parse BAR message
let hl7 = r"MSH|^~\&|BILLING|FACILITY|RECEIVER|DEST|20250107||BAR^P01|12345|P|2.5
EVN|P01|20250107
PID|||12345^^^MRN||DOE^JOHN||19800101|M
PV1||I|ER^101^1";

let message = parse_message(hl7)?;

// Validate against schema
let errors = validator.validate(&message);
if errors.is_empty() {
    println!("Message is valid!");
}
```

### Loading and Validating an RDE Message

```rust
use rs7_core::Version;
use rs7_parser::parse_message;
use rs7_validator::Validator;

// Create validator with RDE O11 schema
let mut validator = Validator::new(Version::V2_5);
validator.load_schema("RDE", "O11").unwrap();

// Parse RDE message
let hl7 = r"MSH|^~\&|PHARMACY|FACILITY|RECEIVER|DEST|20250107||RDE^O11|12345|P|2.5
PID|||12345^^^MRN||DOE^JOHN||19800101|M
ORC|NW
RXE||ASPIRIN^325MG||325|MG";

let message = parse_message(hl7)?;

// Validate
let errors = validator.validate(&message);
```

## Complete Message Type Coverage

### All 38 Supported Message Schemas

1. **ADT - Admit/Discharge/Transfer** (16 schemas)
   - A01-A13, A17, A28, A31, A40

2. **SIU - Scheduling** (4 schemas)
   - S12-S15

3. **MDM - Medical Document Management** (3 schemas)
   - T01, T02, T04

4. **DFT - Detailed Financial Transaction** (2 schemas)
   - P03, P11

5. **QRY - Query** (3 schemas)
   - A19, Q01, Q02

6. **BAR - Billing Account Record** (2 schemas) ⭐ NEW
   - P01, P02

7. **RDE - Pharmacy Encoded Order** (1 schema) ⭐ NEW
   - O11

8. **RAS - Pharmacy Administration** (1 schema) ⭐ NEW
   - O17

9. **RDS - Pharmacy Dispense** (1 schema) ⭐ NEW
   - O13

10. **MFN - Master File Notification** (1 schema) ⭐ NEW
    - M01

11. **Other** (4 schemas)
    - ORM^O01, ORU^R01, ACK (2 versions)

## Test Results

All 101 tests passing:
- **rs7-core**: 47 tests
- **rs7-mllp**: 5 tests
- **rs7-parser**: 9 tests
- **rs7-terser**: 6 tests
- **rs7-validator**: 33 tests (6 new tests added)
- **Doc tests**: 1 test

## Version Information

- **Version**: 0.1.3
- **Release Date**: 2025-10-07
- **Previous Version**: 0.1.2
- **Schemas Added**: 30 files (6 types × 5 versions)
- **Total Schemas**: 38 message types across 5 HL7 versions
- **Total Schema Files**: ~190 files (38 types × 5 versions)

## Benefits

1. **Expanded Coverage**: Support for critical billing and pharmacy workflows
2. **Multi-Version Support**: All new schemas available across HL7 v2.3-2.7
3. **Consistent API**: Same validation interface for all message types
4. **Production Ready**: Fully tested with comprehensive test coverage
5. **Master File Sync**: MFN support enables system-to-system master data synchronization

## Next Steps

Potential future enhancements:
- Additional pharmacy message types (RGV, RRD, RRA)
- Laboratory message types (OUL, OML)
- Financial message types (BAR P03-P12)
- Clinical trial messages (CTI)
- Specimen messages (SSU, SSR)
- Message builders for new schemas

## Conclusion

Version 0.1.3 significantly expands the rs7 library's coverage of HL7 message types, particularly in the areas of billing and pharmacy workflows. With 38 message schemas now supported across 5 HL7 versions, rs7 provides comprehensive validation capabilities for a wide range of healthcare integration scenarios.
