# RS7 CLI - Command-Line Interface for HL7 v2.x Messages

A powerful command-line tool for parsing, validating, and analyzing HL7 v2.x messages.

## Installation

```bash
cargo install --path crates/rs7-cli

# Or with FHIR conversion support:
cargo install --path crates/rs7-cli --features fhir
```

## Usage

```bash
rs7 <COMMAND> [OPTIONS]
```

## Commands

### Parse - Parse and display HL7 messages

Parse an HL7 message and display its structure:

```bash
# Parse from file (text format)
rs7 parse message.hl7

# Parse from stdin
cat message.hl7 | rs7 parse -

# JSON output
rs7 parse message.hl7 --format json

# Pretty formatted output with colors
rs7 parse message.hl7 --format pretty

# Show detailed segment information
rs7 parse message.hl7 --detailed
```

**Example Output (text format):**
```
HL7 Message Parsed Successfully
Version: V2_5
Message Type: ADT^A01
Control ID: MSG123
Segments: 5
```

**Example Output (pretty format):**
```
Message Structure:
  Version: V2_5
  Type: ADT^A01
  Control ID: MSG123
  Segments: 5

  1. MSH (9 fields)
      [0]: |
      [1]: ^~\&
      [2]: SendApp
      ...
```

### Validate - Validate messages against HL7 standards

Validate an HL7 message:

```bash
# Validate with auto-detected version
rs7 validate message.hl7

# Validate with specific version
rs7 validate message.hl7 --version 2.5

# JSON output
rs7 validate message.hl7 --format json
```

**Example Output:**
```
✓ Message is valid

No issues found.
```

**With errors:**
```
✗ Message validation failed

Errors:
  • MSH-9 - Missing required field: Message Type

Warnings:
  • PID-8 - Deprecated code used in Administrative Sex
```

### Extract - Extract field values using Terser paths

Extract specific fields from a message:

```bash
# Extract single field
rs7 extract message.hl7 PID-5

# Extract multiple fields
rs7 extract message.hl7 PID-5 PID-3 MSH-10

# Extract with component indexing
rs7 extract message.hl7 "PID-5-0" "PID-5-1"

# Extract from indexed segments
rs7 extract message.hl7 "OBX(0)-5" "OBX(1)-5"

# JSON output
rs7 extract message.hl7 PID-5 PID-3 --format json
```

**Example Output:**
```
PID-5: DOE^JOHN
PID-3: MRN123
MSH-10: MSG123
```

**Terser Path Syntax:**
- `SEG-field`: Field in first occurrence of segment (e.g., `PID-5`)
- `SEG(index)-field`: Field in indexed segment (e.g., `OBX(1)-5`)
- `SEG-field-component`: Component within field (e.g., `PID-5-0` for family name)
- `SEG-field-component-subcomponent`: Subcomponent (e.g., `PID-11-0-0`)

**Note:** Component indexing is 0-based:
- `PID-5-0` = Family name
- `PID-5-1` = Given name
- `PID-5-2` = Middle name

### Convert - Convert messages to different formats

Convert HL7 messages to JSON or FHIR:

```bash
# Convert to JSON
rs7 convert message.hl7 --to json

# Convert to JSON (pretty-printed)
rs7 convert message.hl7 --to json --pretty

# Convert to FHIR R4 (requires 'fhir' feature)
rs7 convert message.hl7 --to fhir --pretty
```

**Example JSON Output:**
```json
{
  "version": "V2_5",
  "message_type": "ADT^A01",
  "control_id": "MSG123",
  "sending_application": "SendApp",
  "segments": [
    {
      "id": "MSH",
      "fields": ["|", "^~\\&", "SendApp", ...]
    },
    ...
  ]
}
```

### Info - Display comprehensive message information

Show detailed message information:

```bash
rs7 info message.hl7
```

**Example Output:**
```
HL7 Message Information
==================================================

Header Information:
  Version:              V2_5
  Message Type:         ADT^A01
  Control ID:           MSG123
  Sending Application:  SendApp
  Sending Facility:     Hospital
  Receiving Application: RecvApp
  Receiving Facility:   Lab

Message Structure:
  Total Segments:       8
  Segment Types:        5

  Segment Breakdown:
    1 x MSH
    1 x EVN
    1 x PID
    2 x OBX
    1 x PV1

Size Information:
  Encoded Size:         1024 bytes
  Average Segment Size: 128 bytes
```

## Input Methods

All commands support reading from files or standard input:

```bash
# From file
rs7 parse message.hl7

# From stdin
cat message.hl7 | rs7 parse -
echo "MSH|^~\&|..." | rs7 parse -

# Pipe from other commands
curl http://example.com/message.hl7 | rs7 validate -
```

## Output Formats

### Text Format (default)
Human-readable output with colors (when terminal supports it)

### JSON Format
Machine-readable JSON output for programmatic processing:

```bash
rs7 parse message.hl7 --format json | jq '.segment_count'
rs7 validate message.hl7 --format json | jq '.errors[]'
rs7 extract message.hl7 PID-5 --format json
```

### Pretty Format
Enhanced text output with colors and formatting (parse command only):

```bash
rs7 parse message.hl7 --format pretty --detailed
```

## Examples

### Basic Workflow

```bash
# 1. Parse and check structure
rs7 parse sample.hl7 --format pretty

# 2. Validate against standards
rs7 validate sample.hl7

# 3. Extract patient demographics
rs7 extract sample.hl7 PID-5 PID-7 PID-8

# 4. Get full message info
rs7 info sample.hl7
```

### Processing Multiple Messages

```bash
# Validate all messages in a directory
for f in messages/*.hl7; do
  echo "Validating $f"
  rs7 validate "$f"
done

# Extract patient names from all messages
for f in messages/*.hl7; do
  rs7 extract "$f" PID-5 --format json
done | jq -s '.'
```

### Integration with Other Tools

```bash
# Use with jq for JSON processing
rs7 convert message.hl7 --to json --pretty | jq '.segments[] | select(.id == "PID")'

# Count segment types
rs7 parse message.hl7 --format json | jq '.segments[].id' | sort | uniq -c

# Find all observation results
rs7 extract message.hl7 "OBX(0)-5" "OBX(1)-5" "OBX(2)-5"
```

### FHIR Conversion

```bash
# Convert ADT message to FHIR
rs7 convert adt_a01.hl7 --to fhir --pretty > patient.json

# Convert ORU message to FHIR
rs7 convert oru_r01.hl7 --to fhir --pretty > observations.json
```

## Sample HL7 Message

Create a test message:

```bash
cat > sample.hl7 << 'EOF'
MSH|^~\&|SendApp|SendFac|RecvApp|RecvFac|20250108120000||ADT^A01|MSG123|P|2.5
EVN|A01|20250108120000
PID|1||MRN123||DOE^JOHN^A||19800101|M|||123 MAIN ST^^CITY^ST^12345||555-1234
PV1|1|I|WARD^101^1
EOF

rs7 parse sample.hl7 --format pretty
```

## Error Handling

The CLI returns appropriate exit codes:
- `0` - Success
- `1` - Error (parse failure, validation failure, file not found, etc.)

```bash
rs7 validate message.hl7
if [ $? -eq 0 ]; then
  echo "Valid message"
else
  echo "Invalid message"
fi
```

## Building from Source

```bash
# Standard build
cargo build --release --package rs7-cli

# With FHIR support
cargo build --release --package rs7-cli --features fhir

# The binary will be at: target/release/rs7
```

## Features

- **Default**: Core parsing, validation, and extraction
- **fhir**: FHIR R4 conversion support

## Performance

The CLI is optimized for speed:
- Parses typical messages in < 10 microseconds
- Validates in < 100 microseconds
- Minimal memory footprint
- Suitable for high-throughput batch processing

## License

MIT OR Apache-2.0

## Related Projects

- [rs7](https://gitlab.flostel.com/alexshao/rs7) - The core HL7 v2.x library for Rust
- [rs7-wasm](../rs7-wasm) - WebAssembly bindings for browser/Node.js
