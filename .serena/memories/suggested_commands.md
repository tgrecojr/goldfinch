# Suggested Development Commands

## Testing
```bash
# Run all tests (unit + integration)
cargo test

# Run with output visible
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Generate coverage report (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

## Code Quality
```bash
# Format code
cargo fmt

# Check for issues
cargo clippy

# Check compilation without building
cargo check
```

## Building & Running
```bash
# Build debug version
cargo build

# Build release version
cargo build --release

# Run without installing
export GOLDFINCH_SECRET=my-secret
cargo run -- list
cargo run -- get my-key
cargo run -- search pattern

# Install locally
cargo install --path .
```

## System Utilities (macOS/Darwin)
- `git` - version control
- `ls`, `cd`, `pwd` - file navigation
- `grep`, `find` - text/file search
- Standard Unix commands available on macOS
