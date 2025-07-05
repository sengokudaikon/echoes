default:
    @just --list

# Run checks before committing
check: fmt clippy test

# Format code
fmt:
    cargo +nightly fmt --all

# Run clippy
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
    cargo nextest run

# Run tests with coverage
coverage:
    cargo tarpaulin --out Html --output-dir target/coverage

# Security audit
audit:
    cargo deny check

# Update dependencies
update:
    cargo update
    cargo outdated

# Build for all platforms
build-all:
    cargo build --release
    cross build --release --target x86_64-pc-windows-gnu
    cross build --release --target aarch64-unknown-linux-gnu

# Generate changelog
changelog:
    git cliff -o CHANGELOG.md

# Clean everything
clean:
    cargo clean
    rm -rf target/

# Run before release
release-prep: check changelog
    echo "Ready for release!"

# Watch for changes and run tests
watch:
    cargo watch -x test -x clippy

# Serve docs locally
docs:
    cargo doc --open --no-deps