# rs7-templates

Template system for creating and validating HL7 messages from reusable patterns.

## Features

- Reusable message templates with variable placeholders
- Template inheritance and resolution
- YAML/JSON configuration
- Standard template library for common messages
- Template validation

## Installation

```toml
[dependencies]
rs7-templates = "0.19"
```

## Quick Example

```rust
use rs7_templates::{TemplateEngine, TemplateLibrary};

let library = TemplateLibrary::new();
let template = library.get("ADT_A01").unwrap();

let engine = TemplateEngine::new()
    .set_variable("patient_id", "12345")
    .set_variable("patient_name", "Doe^John");

let message = engine.create_message(template)?;
```

See [main README](../../README.md) for full documentation.

## License

Licensed under either of Apache License 2.0 or MIT license at your option.
