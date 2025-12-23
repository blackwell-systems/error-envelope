# Contributing to error-envelope

Thank you for your interest in contributing! This is a reference implementation designed to be minimal and stable.

## Development Setup

```bash
# Clone the repository
git clone https://github.com/blackwell-systems/error-envelope
cd error-envelope

# Run tests
cargo test --all-features

# Run lints
cargo fmt --all -- --check
cargo clippy --all-features -- -D warnings
```

## Testing

All changes must include tests and pass existing tests:

```bash
cargo test --all-features
```

## Code Style

- Use `cargo fmt` for formatting
- Fix all clippy warnings: `cargo clippy --all-features -- -D warnings`
- Follow Rust API guidelines
- Keep dependencies minimal

## Pull Requests

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Ensure CI passes (fmt, clippy, tests)
5. Submit a pull request

## Versioning

We follow [Semantic Versioning](https://semver.org/):

- **Breaking changes**: Increment major version
- **New features**: Increment minor version  
- **Bug fixes**: Increment patch version

## Error Codes

Error codes are **stable** and must never change. Adding new codes is acceptable, but existing codes must remain backward compatible.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
