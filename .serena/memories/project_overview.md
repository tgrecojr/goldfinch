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
- `list` - Display all keys across all secrets
- `get <KEY>` - Retrieve a specific key's value from any secret
- `search <PATTERN>` - Find keys matching a substring pattern across all secrets

Configuration:
- No configuration required - automatically discovers and operates on all secrets in the AWS account

## Automatic Secret Discovery
The tool automatically discovers all AWS Secrets Manager secrets in your account using the `ListSecrets` API. All secrets containing JSON key-value pairs are merged together, and commands operate on the combined data from all secrets.
