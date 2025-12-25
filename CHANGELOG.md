# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] - 2025-12-25

### Changed
- Add comprehensive documentation coverage to all public items (100% coverage)
- Document all Code enum variants with descriptions and HTTP status codes
- Document all Error struct public fields

## [0.2.1] - 2025-12-23

### Changed
- Improve README formatting and structure

## [0.2.0] - 2025-12-23

### Added
- New `anyhow-support` feature for seamless anyhow integration
- `From<anyhow::Error>` implementation for automatic error conversion
- Enhanced README with TL;DR, Table of Contents, and improved structure
- Real-world Axum handler examples in documentation
- Framework Integration section with detailed Axum usage

### Changed
- Improved documentation structure with better navigation
- Updated installation examples to show feature combinations

## [0.1.0] - 2025-12-23

### Added
- Initial release with full feature parity to Go err-envelope v1.1.0
- Core error struct with code, message, details, trace_id, retryable fields
- Type-safe error codes enum with 18 variants
- Builder pattern with immutable `with_*` methods
- Helper constructors (internal, not_found, unauthorized, etc.)
- Formatted constructors using `format!()` macro
- Custom JSON serialization for `retry_after` as human-readable duration
- `std::error::Error` trait implementation
- Axum `IntoResponse` integration (optional feature)
- `from()` for mapping arbitrary errors to Error
- `validation()` for field-level validation errors
- `is()` for checking error codes
- Comprehensive test suite (17 tests passing)
- Full documentation with examples

[Unreleased]: https://github.com/blackwell-systems/error-envelope/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/blackwell-systems/error-envelope/releases/tag/v0.1.0
