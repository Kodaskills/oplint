default:
    @just --list

# Format Rust code
[group('quality')]
fmt:
    cargo fmt --all

# Lint Rust code
[group('quality')]
lint:
    cargo clippy --all-targets --all-features

# Check for outdated dependencies
[group('quality')]
outdated:
    cargo outdated --depth 1

# Check all quality groups
[group('quality')]
check: fmt lint test doc audit outdated

# Run tests
[group('tests')]
test:
    cargo test --all-targets --all-features

# Generate documentation
[group('doc')]
doc:
    cargo doc --no-deps --all-features

# Run security audit
[group('security')]
audit:
    cargo audit
