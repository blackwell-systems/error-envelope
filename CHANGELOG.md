# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2025-12-26

### Added
- `Error::with_cause_message()` method for fluent builder-style cause attachment
- Domain error mapping documentation in API.md (thiserror pattern with From trait)
- API.md: Complete API reference extracted from README (364 lines)
- ERROR_CODES.md: Complete error codes reference with all 18 codes (167 lines)
- ARCHITECTURE.md: Visual architecture guide with 4 mermaid diagrams (272 lines)
- Four new examples:
  - `domain_errors.rs` - thiserror → error-envelope mapping via From trait
  - `validation.rs` - Field-level validation with structured error details
  - `rate_limiting.rs` - Rate limiting with retry_after and time windows
  - `tracing.rs` - Trace ID propagation through middleware

### Changed
- README restructured as landing page (337→235 lines, 30% reduction)
- Hero example now compile-clean with struct definitions and proper HashMap types
- Hero example demonstrates validation errors with structured field details
- Quick Start reduced to minimal rate limiting example (demonstrates retry_after)
- Table of Contents now has clickable links to sections and external docs
- "Why error-envelope" section reframed around formal error contracts
- Overview section reordered to lead with anyhow and Axum integration
- anyhow integration description clarified: "at the HTTP boundary"
- Design Principles section condensed with link to ARCHITECTURE.md
- Stack architecture explicitly documented: anyhow → error-envelope → Axum
- Examples section reorganized to list all five examples with descriptions

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
