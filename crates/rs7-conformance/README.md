# rs7-conformance

Conformance profile validation for HL7 v2.x messages.

## Features

- XML conformance profile parsing
- Usage validation (R, RE, O, X, C)
- Cardinality validation
- Length constraints
- Conditional predicates
- Detailed validation reports

## Installation

```toml
[dependencies]
rs7-conformance = "0.19"
```

## Quick Example

```rust
use rs7_conformance::{ProfileParser, ConformanceValidator};

let profile = ProfileParser::from_file("adt_a01_profile.xml")?;
let validator = ConformanceValidator::new(profile);
let result = validator.validate(&message)?;

if !result.is_valid() {
    for error in result.errors {
        eprintln!("{}: {}", error.location, error.message);
    }
}
```

See [main README](../../README.md) for full documentation.

## License

Licensed under either of Apache License 2.0 or MIT license at your option.
