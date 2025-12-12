# Goldfinch

A command-line tool for reading key-value pairs from AWS Secrets Manager secrets.

## Features

- **List all keys**: Display all keys across all specified secrets
- **Get by exact key name**: Retrieve a specific key's value from any secret
- **Search**: Find keys matching a substring pattern across all secrets
- **Multiple secrets**: Work with one or more AWS secrets simultaneously
- **Flexible output**: JSON format (default) or plain text
- **Read-only**: Safe operations with no ability to modify or create secrets
- **Flexible configuration**: Specify target secrets via CLI argument or environment variable

## How it Works

Goldfinch can target one or more AWS Secrets that contain JSON key-value pairs. When multiple secrets are specified, their key-value pairs are merged together. The commands operate on the combined keys and values from all secrets.

For example, if you have two secrets:

**Secret 1: `my-app-config`**
```json
{
  "api_key": "abc123",
  "db_password": "secret123"
}
```

**Secret 2: `my-app-urls`**
```json
{
  "prod_db_url": "https://prod.example.com",
  "staging_db_url": "https://staging.example.com"
}
```

You can work with both secrets together, and Goldfinch will merge all keys from both secrets for listing, searching, and retrieval.

## Prerequisites

1. **Rust**: Install from [https://rustup.rs/](https://rustup.rs/)
2. **AWS Credentials**: Configure your AWS credentials using one of these methods:
   - AWS CLI configuration (`~/.aws/credentials`)
   - Environment variables (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`)
   - IAM role (if running on EC2/ECS)

## Installation

### Build from source

```bash
cargo build --release
```

The binary will be available at `target/release/goldfinch`.

### Install locally

```bash
cargo install --path .
```

This will install `goldfinch` to your Cargo bin directory (usually `~/.cargo/bin`).

## Usage

### Specifying target secrets

You can specify which AWS secrets to target in two ways:

**Option 1: Command-line argument** (takes precedence)
```bash
# Single secret
goldfinch --secrets my-app-secrets list
goldfinch -s my-app-secrets list

# Multiple secrets (comma-separated)
goldfinch --secrets my-app-config,my-app-urls list
goldfinch -s secret1,secret2,secret3 list
```

**Option 2: Environment variable**
```bash
# Single secret
export GOLDFINCH_SECRETS=my-app-secrets
goldfinch list

# Multiple secrets (comma-separated)
export GOLDFINCH_SECRETS=my-app-config,my-app-urls
goldfinch list
```

The CLI argument will override the environment variable if both are provided.

### List all keys

```bash
# JSON format (default) - single secret
goldfinch --secrets my-app-secrets list

# Plain text format - single secret
goldfinch --secrets my-app-secrets list --format plain

# Multiple secrets - merges all keys
goldfinch --secrets my-app-config,my-app-urls list
```

Output (JSON):
```json
[
  "api_key",
  "db_password",
  "prod_db_url",
  "staging_db_url"
]
```

Output (plain):
```
api_key
db_password
prod_db_url
staging_db_url
```

### Get a specific key's value

```bash
# JSON format with metadata
goldfinch --secrets my-app-secrets get db_password

# Plain text format (just the value)
goldfinch --secrets my-app-secrets get db_password --format plain

# Search across multiple secrets
goldfinch --secrets my-app-config,my-app-urls get prod_db_url
```

Output (JSON):
```json
{
  "key": "db_password",
  "value": "secret123"
}
```

Output (plain):
```
secret123
```

### Search for keys

Search uses substring matching - it will find any key whose name contains the pattern across all specified secrets:

```bash
# Find all keys containing "db" across all secrets
goldfinch --secrets my-app-config,my-app-urls search db

# Find all keys containing "url" in plain format
goldfinch --secrets my-app-config,my-app-urls search url --format plain
```

Output (JSON):
```json
[
  {
    "key": "db_password",
    "value": "secret123"
  },
  {
    "key": "prod_db_url",
    "value": "https://prod.example.com"
  },
  {
    "key": "staging_db_url",
    "value": "https://staging.example.com"
  }
]
```

Output (plain):
```
db_password: secret123
prod_db_url: https://prod.example.com
staging_db_url: https://staging.example.com
```

## Common Use Cases

**Set default secrets via environment variable:**
```bash
# Single secret
export GOLDFINCH_SECRETS=my-app-secrets
goldfinch list
goldfinch get api_key

# Multiple secrets
export GOLDFINCH_SECRETS=my-app-config,my-app-urls
goldfinch list
```

**Pipe secret value to another command:**
```bash
goldfinch -s my-app-secrets get db_password --format plain | some-other-command
```

**Search and process with jq:**
```bash
goldfinch -s my-app-config,my-app-urls search prod | jq '.[] | .key'
```

**Export secret value to environment variable:**
```bash
export API_KEY=$(goldfinch -s my-app-secrets get api_key --format plain)
```

**Work with multiple secrets from different environments:**
```bash
# Combine config and secrets from multiple sources
goldfinch -s app-base-config,prod-secrets get db_password
goldfinch -s app-base-config,staging-secrets get db_password
```

**Merge secrets for comprehensive key listing:**
```bash
# List all keys across multiple secret stores
goldfinch -s team-secrets,app-secrets,infra-secrets list
```

## Value Type Handling

Goldfinch handles different JSON value types appropriately:

- **Strings**: Returned as-is
- **Numbers**: Converted to string representation
- **Booleans**: Converted to "true" or "false"
- **Null**: Converted to "null"
- **Arrays/Objects**: Serialized to JSON string

## Required AWS Permissions

The application requires the `secretsmanager:GetSecretValue` permission to read secrets.

**Important:** The example below is a **generic policy** that grants broad access. You should **tailor this policy to your specific needs and environment**. As a security best practice, limit access to only the secrets your application needs to read.

### Generic Policy (NOT recommended for production)

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "secretsmanager:GetSecretValue"
      ],
      "Resource": "arn:aws:secretsmanager:*:*:secret:*"
    }
  ]
}
```

### Recommended: Restrict to Specific Secrets

Limit access to only the secrets you need:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "secretsmanager:GetSecretValue"
      ],
      "Resource": [
        "arn:aws:secretsmanager:us-east-1:123456789012:secret:my-app-secrets-*",
        "arn:aws:secretsmanager:us-east-1:123456789012:secret:prod-config-*"
      ]
    }
  ]
}
```

### Recommended: Restrict by Region and Account

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "secretsmanager:GetSecretValue"
      ],
      "Resource": "arn:aws:secretsmanager:us-east-1:123456789012:secret:*"
    }
  ]
}
```

### Recommended: Use Path-Based Naming with Wildcards

Organize secrets with a naming convention and restrict by prefix:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "secretsmanager:GetSecretValue"
      ],
      "Resource": "arn:aws:secretsmanager:us-east-1:123456789012:secret:/myapp/*"
    }
  ]
}
```

**Note:** AWS Secrets Manager automatically appends a 6-character suffix to secret ARNs (e.g., `-AbCdEf`), which is why wildcards with `-*` are used in the examples above.

## Error Handling

The application provides clear error messages for common issues:

- **No secrets specified**: "Secret names are required. Provide them via --secrets flag or GOLDFINCH_SECRETS environment variable"
- **Secret not found**: "Failed to fetch secret 'name'"
- **Invalid JSON**: "Secret value is not valid JSON"
- **Not a JSON object**: "Secret value is not a JSON object with key-value pairs"
- **Key not found**: "Key 'key-name' not found in secret" (searches across all specified secrets)
- **No search results**: "No keys found matching pattern 'pattern'" (searches across all specified secrets)
- **Access denied**: "Not authorized to perform operation"

## Development

### Run without installing

```bash
# With environment variable - single secret
export GOLDFINCH_SECRETS=my-app-secrets
cargo run -- list
cargo run -- get my-key
cargo run -- search pattern

# With environment variable - multiple secrets
export GOLDFINCH_SECRETS=my-app-config,my-app-urls
cargo run -- list

# With CLI argument - single secret
cargo run -- --secrets my-app-secrets list
cargo run -- -s my-app-secrets get my-key

# With CLI argument - multiple secrets
cargo run -- --secrets my-app-config,my-app-urls list
cargo run -- -s secret1,secret2,secret3 search pattern
```

### Run tests

The project includes comprehensive unit tests covering all core functionality:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_get_key_success
```

**Test Coverage:**
- Value type handling (strings, numbers, booleans, null, arrays, objects)
- Key listing functionality
- Getting specific keys by name
- Searching keys with substring matching
- Error cases (key not found, no matches)
- Edge cases (empty strings, unicode, special characters, long values)
- JSON parsing validation

All tests run locally without requiring AWS credentials or connectivity.

### Check for issues

```bash
cargo clippy
```

### Format code

```bash
cargo fmt
```

## Examples

**Example 1: Single Secret**

Given a secret named `app-config` containing:
```json
{
  "database_url": "postgresql://localhost/mydb",
  "redis_host": "localhost",
  "redis_port": "6379",
  "api_timeout_ms": "5000",
  "enable_cache": "true"
}
```

```bash
# List all configuration keys
goldfinch -s app-config list --format plain
# Output:
# api_timeout_ms
# database_url
# enable_cache
# redis_host
# redis_port

# Get database connection string
DB_URL=$(goldfinch -s app-config get database_url --format plain)

# Find all Redis-related configuration
goldfinch -s app-config search redis
# Output (JSON):
# [
#   {
#     "key": "redis_host",
#     "value": "localhost"
#   },
#   {
#     "key": "redis_port",
#     "value": "6379"
#   }
# ]
```

**Example 2: Multiple Secrets**

Given two secrets:

Secret `base-config`:
```json
{
  "app_name": "myapp",
  "log_level": "info"
}
```

Secret `env-config`:
```json
{
  "database_url": "postgresql://prod.db/myapp",
  "api_key": "prod-key-123"
}
```

```bash
# List all keys from both secrets
goldfinch -s base-config,env-config list --format plain
# Output:
# api_key
# app_name
# database_url
# log_level

# Get a key that exists in the second secret
goldfinch -s base-config,env-config get api_key --format plain
# Output:
# prod-key-123

# Search across both secrets
goldfinch -s base-config,env-config search app
# Output (JSON):
# [
#   {
#     "key": "app_name",
#     "value": "myapp"
#   }
# ]
```

## License

This project is open source and available under the MIT License.
