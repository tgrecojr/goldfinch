# Goldfinch

A command-line tool for reading key-value pairs from an AWS Secrets Manager secret.

## Features

- **List all keys**: Display all keys within a secret's JSON object
- **Get by exact key name**: Retrieve a specific key's value
- **Search**: Find keys matching a substring pattern
- **Flexible output**: JSON format (default) or plain text
- **Read-only**: Safe operations with no ability to modify or create secrets
- **Flexible configuration**: Specify target secret via CLI argument or environment variable

## How it Works

Goldfinch targets a single AWS Secret that contains JSON key-value pairs. The commands operate on the keys and values within that secret.

For example, if your secret `my-app-secrets` contains:
```json
{
  "api_key": "abc123",
  "db_password": "secret123",
  "prod_db_url": "https://prod.example.com",
  "staging_db_url": "https://staging.example.com"
}
```

You can list all keys, retrieve specific values, or search for keys matching a pattern.

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

### Specifying the target secret

You can specify which AWS secret to target in two ways:

**Option 1: Command-line argument** (takes precedence)
```bash
goldfinch --secret my-app-secrets list
goldfinch -s my-app-secrets list
```

**Option 2: Environment variable**
```bash
export GOLDFINCH_SECRET=my-app-secrets
goldfinch list
```

The CLI argument will override the environment variable if both are provided.

### List all keys

```bash
# JSON format (default)
goldfinch --secret my-app-secrets list

# Plain text format
goldfinch --secret my-app-secrets list --format plain
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
goldfinch --secret my-app-secrets get db_password

# Plain text format (just the value)
goldfinch --secret my-app-secrets get db_password --format plain
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

Search uses substring matching - it will find any key whose name contains the pattern:

```bash
# Find all keys containing "db"
goldfinch --secret my-app-secrets search db

# Find all keys containing "url" in plain format
goldfinch --secret my-app-secrets search url --format plain
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

**Set default secret via environment variable:**
```bash
export GOLDFINCH_SECRET=my-app-secrets
goldfinch list
goldfinch get api_key
```

**Pipe secret value to another command:**
```bash
goldfinch -s my-app-secrets get db_password --format plain | some-other-command
```

**Search and process with jq:**
```bash
goldfinch -s my-app-secrets search prod | jq '.[] | .key'
```

**Export secret value to environment variable:**
```bash
export API_KEY=$(goldfinch -s my-app-secrets get api_key --format plain)
```

**Use different secrets without changing environment:**
```bash
goldfinch -s prod-secrets get db_password
goldfinch -s staging-secrets get db_password
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

- **No secret specified**: "Secret name is required. Provide it via --secret flag or GOLDFINCH_SECRET environment variable"
- **Secret not found**: "Failed to fetch secret 'name'"
- **Invalid JSON**: "Secret value is not valid JSON"
- **Not a JSON object**: "Secret value is not a JSON object with key-value pairs"
- **Key not found**: "Key 'key-name' not found in secret"
- **No search results**: "No keys found matching pattern 'pattern'"
- **Access denied**: "Not authorized to perform operation"

## Development

### Run without installing

```bash
# With environment variable
export GOLDFINCH_SECRET=my-app-secrets
cargo run -- list
cargo run -- get my-key
cargo run -- search pattern

# With CLI argument
cargo run -- --secret my-app-secrets list
cargo run -- -s my-app-secrets get my-key
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

## License

This project is open source and available under the MIT License.
