# HL7 Message Schemas

This directory contains JSON schema definitions for HL7 v2.x messages across different versions.

## Structure

```
schemas/
├── v2_3/     # HL7 version 2.3 & 2.3.1
├── v2_4/     # HL7 version 2.4
├── v2_5/     # HL7 version 2.5 & 2.5.1
├── v2_6/     # HL7 version 2.6
└── v2_7/     # HL7 version 2.7 & 2.7.1
```

## Schema Format

Each schema is a JSON file with the following structure:

```json
{
  "message_type": "ADT",
  "trigger_event": "A01",
  "version": "2.5",
  "description": "Admit/Visit Notification",
  "segments": {
    "MSH": {
      "name": "Message Header",
      "required": true,
      "repeating": false,
      "fields": {
        "1": {
          "name": "Field Separator",
          "data_type": "ST",
          "required": true,
          "repeating": false,
          "max_length": 1
        },
        ...
      }
    },
    ...
  }
}
```

## Available Schemas

All schemas are available for **HL7 v2.3, v2.4, v2.5, v2.6, and v2.7**.

### ADT (Admit/Discharge/Transfer) - 17 Schemas
- ✅ ADT^A01 - Admit/Visit Notification
- ✅ ADT^A02 - Transfer a Patient
- ✅ ADT^A03 - Discharge/End Visit
- ✅ ADT^A04 - Register a Patient
- ✅ ADT^A05 - Pre-admit a Patient
- ✅ ADT^A06 - Change Outpatient to Inpatient
- ✅ ADT^A07 - Change Inpatient to Outpatient
- ✅ ADT^A08 - Update Patient Information
- ✅ ADT^A09 - Patient Departing - Tracking
- ✅ ADT^A10 - Patient Arriving - Tracking
- ✅ ADT^A11 - Cancel Admit/Visit Notification
- ✅ ADT^A12 - Cancel Transfer
- ✅ ADT^A13 - Cancel Discharge/End Visit
- ✅ ADT^A17 - Swap Patients
- ✅ ADT^A28 - Add Person Information
- ✅ ADT^A31 - Update Person Information
- ✅ ADT^A40 - Merge Patient - Patient Identifier List

### SIU (Scheduling Information Unsolicited) - 4 Schemas
- ✅ SIU^S12 - Notification of New Appointment Booking
- ✅ SIU^S13 - Notification of Appointment Rescheduling
- ✅ SIU^S14 - Notification of Appointment Modification
- ✅ SIU^S15 - Notification of Appointment Cancellation

### MDM (Medical Document Management) - 3 Schemas
- ✅ MDM^T01 - Original Document Notification
- ✅ MDM^T02 - Original Document Notification and Content
- ✅ MDM^T04 - Document Status Change Notification

### DFT (Detailed Financial Transaction) - 2 Schemas
- ✅ DFT^P03 - Post Detail Financial Transaction
- ✅ DFT^P11 - Post Detail Financial Transactions - Expanded

### QRY (Query Messages) - 3 Schemas
- ✅ QRY^A19 - Patient Query
- ✅ QRY^Q01 - Query Sent for Immediate Response
- ✅ QRY^Q02 - Query Sent for Deferred Response

### ORU (Observation Result) - 1 Schema
- ✅ ORU^R01 - Unsolicited Observation Message

### ORM (Order Message) - 1 Schema
- ✅ ORM^O01 - General Order Message

### ACK (Acknowledgment) - 1 Schema
- ✅ ACK - General Acknowledgment

**Total: 32 message schemas across 5 HL7 versions (160 schema files)**

### Version-Specific Differences
- **V2.3**: Simpler data types, fewer optional fields
- **V2.4**: Added support for more complex data types
- **V2.5**: Enhanced security and internationalization
- **V2.6**: Improved vocabulary support
- **V2.7**: Enhanced conformance and implementation guides

## Data Type Definitions

Common HL7 data types used in schemas:

- **ST**: String
- **NM**: Numeric
- **TS**: Timestamp
- **DT**: Date
- **TM**: Time
- **ID**: Coded Identifier
- **IS**: Coded Value
- **CE**: Coded Element
- **CWE**: Coded With Exceptions
- **XPN**: Extended Person Name
- **XAD**: Extended Address
- **XTN**: Extended Telecommunication Number
- **CX**: Extended Composite ID
- **EI**: Entity Identifier
- **HD**: Hierarchic Designator
- **MSG**: Message Type
- **PT**: Processing Type
- **PL**: Person Location
- **XCN**: Extended Composite ID and Name

## Adding New Schemas

To add a new schema:

1. Create a JSON file following the structure above
2. Place it in the appropriate version directory
3. Update `schema_loader.rs` to include the new schema
4. Add the schema to the version's list in `list_available_schemas()`

Example:
```rust
("v2_5", "ADT_A04") => {
    let json = include_str!("../schemas/v2_5/ADT_A04.json");
    parse_schema_json(json)
}
```

## Schema Validation

All schemas are validated at compile time by including them in the binary.
Invalid JSON schemas will cause compilation errors.

## References

- HL7 v2.x Standard: https://www.hl7.org/implement/standards/product_brief.cfm?product_id=185
- HL7 v2.5 Specification: Official HL7 documentation
- HL7 v2.3-2.7 Implementation Guides

## Notes

- Schemas are embedded in the binary at compile time for zero-cost runtime access
- Field numbering follows HL7 convention (1-based indexing)
- MSH segment has special field numbering (MSH-1 is field separator, MSH-2 is encoding characters)
- Some fields marked as "Varies" for OBX-5 (observation value) which can be any data type
