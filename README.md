# Goldfinch

A command-line tool for reading key-value pairs from AWS Secrets Manager secrets.

## Features

- **List all keys**: Display all keys across all secrets in your AWS account
- **Get by exact key name**: Retrieve a specific key's value from any secret
- **Search**: Find keys matching a substring pattern across all secrets
- **Automatic discovery**: Automatically searches all AWS secrets in your account
- **Flexible output**: JSON format (default) or plain text
- **Read-only**: Safe operations with no ability to modify or create secrets

## How it Works

Goldfinch automatically discovers and reads all AWS Secrets Manager secrets in your AWS account that contain JSON key-value pairs. The key-value pairs from all secrets are merged together, and commands operate on the combined keys and values.

For example, if you have two secrets in your account:

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

Goldfinch will automatically merge all keys from both secrets for listing, searching, and retrieval operations.

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

Goldfinch automatically operates on all secrets in your AWS account. You don't need to specify which secrets to use.

### List all keys

```bash
# JSON format (default)
goldfinch list

# Plain text format
goldfinch list --format plain
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
goldfinch get db_password

# Plain text format (just the value)
goldfinch get db_password --format plain
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

Search uses substring matching - it will find any key whose name contains the pattern across all secrets:

```bash
# Find all keys containing "db" across all secrets
goldfinch search db

# Find all keys containing "url" in plain format
goldfinch search url --format plain
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

**List all available keys:**
```bash
goldfinch list
```

**Pipe secret value to another command:**
```bash
goldfinch get db_password --format plain | some-other-command
```

**Search and process with jq:**
```bash
goldfinch search prod | jq '.[] | .key'
```

**Export secret value to environment variable:**
```bash
export API_KEY=$(goldfinch get api_key --format plain)
```

## Value Type Handling

Goldfinch handles different JSON value types appropriately:

- **Strings**: Returned as-is
- **Numbers**: Converted to string representation
- **Booleans**: Converted to "true" or "false"
- **Null**: Converted to "null"
- **Arrays/Objects**: Serialized to JSON string

## Required AWS Permissions

The application requires the following AWS Secrets Manager permissions:

- `secretsmanager:ListSecrets` - To discover all secrets in your account
- `secretsmanager:GetSecretValue` - To read secret values

**Important:** The example below is a **generic policy** that grants broad access. You should **tailor this policy to your specific needs and environment**. As a security best practice, consider limiting access to specific regions or using resource tags.

### Generic Policy

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "secretsmanager:ListSecrets",
        "secretsmanager:GetSecretValue"
      ],
      "Resource": "*"
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
        "secretsmanager:ListSecrets"
      ],
      "Resource": "*"
    },
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

**Note:** The `ListSecrets` action doesn't support resource-level permissions, so it must use `"Resource": "*"`. However, you can restrict `GetSecretValue` to specific resources as shown above.

## Error Handling

The application provides clear error messages for common issues:

- **Failed to list secrets**: "Failed to list secrets" (if unable to discover secrets in your account)
- **Secret not found**: "Failed to fetch secret 'name'"
- **Invalid JSON**: "Secret value is not valid JSON"
- **Not a JSON object**: "Secret value is not a JSON object with key-value pairs"
- **Key not found**: "Key 'key-name' not found in secret" (searches across all secrets)
- **No search results**: "No keys found matching pattern 'pattern'" (searches across all secrets)
- **Access denied**: "Not authorized to perform operation"

## Development

### Run without installing

```bash
cargo run -- list
cargo run -- get my-key
cargo run -- search pattern
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

**Example: Working with Multiple Secrets**

Given two secrets in your AWS account:

Secret `app-config`:
```json
{
  "database_url": "postgresql://localhost/mydb",
  "redis_host": "localhost",
  "redis_port": "6379",
  "api_timeout_ms": "5000",
  "enable_cache": "true"
}
```

Secret `env-config`:
```json
{
  "app_name": "myapp",
  "log_level": "info",
  "api_key": "prod-key-123"
}
```

```bash
# List all keys from all secrets in your account
goldfinch list --format plain
# Output:
# api_key
# api_timeout_ms
# app_name
# database_url
# enable_cache
# log_level
# redis_host
# redis_port

# Get database connection string
DB_URL=$(goldfinch get database_url --format plain)

# Find all Redis-related configuration
goldfinch search redis
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

# Search across all secrets
goldfinch search app
# Output (JSON):
# [
#   {
#     "key": "app_name",
#     "value": "myapp"
#   },
#   {
#     "key": "api_timeout_ms",
#     "value": "5000"
#   }
# ]
```

## License

This project is open source and available under the MIT License.
