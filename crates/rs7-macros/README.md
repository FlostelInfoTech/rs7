# rs7-macros

Procedural macros for HL7 message processing (placeholder for future implementation).

## Status

This crate is reserved for future procedural macro implementations. Currently contains placeholder derive macros.

## Planned Features (Phase 5)

- `#[derive(Segment)]` - Type-safe segment definitions
- `#[derive(Message)]` - Complete message structures
- `#[hl7_type]` - Composite data type macros

## Current Status

All macros return empty token streams. Use the `rs7-custom` crate's `z_segment!` declarative macro for custom segment definitions in the meantime.

See [ROADMAP.md](../../ROADMAP.md) for planned enhancements.

## License

Licensed under either of Apache License 2.0 or MIT license at your option.
