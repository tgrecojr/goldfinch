# Goldfinch Project Overview

## Purpose
Goldfinch is a CLI tool for reading key-value pairs from AWS Secrets Manager secrets. It provides read-only operations: list keys, get specific values, and search keys by pattern. Supports JSON and plain text output formats.

## Tech Stack
- **Language**: Rust (edition 2021)
- **AWS SDK**: aws-sdk-secretsmanager (v1.13), aws-config (v1.1)
- **CLI**: clap (v4.5) with derive features
- **Async Runtime**: tokio (v1.35) with full features
- **Serialization**: serde (v1.0), serde_json (v1.0)
- **Error Handling**: anyhow (v1.0)
- **Testing**: assert_cmd (v2.0), predicates (v3.0)

## Project Structure
```
goldfinch/
├── src/
│   └── main.rs          # All source code (single file)
├── tests/
│   └── cli_tests.rs     # Integration tests (13 tests)
├── Cargo.toml           # Package configuration
└── README.md            # Comprehensive documentation
```

## Commands
The tool provides three main commands:
- `list` - Display all keys in the secret
- `get <KEY>` - Retrieve a specific key's value
- `search <PATTERN>` - Find keys matching a substring pattern

Configuration via:
- `--secrets` or `-s` CLI flag (comma-separated for multiple secrets)
- `GOLDFINCH_SECRETS` environment variable (comma-separated for multiple secrets)

## Multi-Secret Support
The tool can work with one or more AWS secrets simultaneously. When multiple secrets are specified, their key-value pairs are merged together, and all commands (list, get, search) operate on the combined data.
