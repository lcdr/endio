
# Changelog

## [0.2.0]

### Added

- Derive macros to automatically implement `Serialize` and `Deserialize` for common cases. Can be disabled by not setting feature "derive".

- In addition to the serialization impls for primitive types, impls for references to these types have been added.

- De-/serialize impl for Ipv4Addr.

- Shorthand aliases for endiannesses.

### Changed

- The slice and vec impls have been generalized to work with types other than u8.
