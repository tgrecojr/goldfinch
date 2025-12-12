# Goldfinch

A command-line tool for reading key-value pairs from AWS Secrets Manager secrets.

## Features

- **List all secrets**: Display all top-level secret names in your AWS account
- **Get secret contents**: Retrieve all key-value pairs from a specific secret by name
- **Search**: Find secrets and keys matching a substring pattern (searches both secret names and key names)
- **Automatic discovery**: Automatically searches all AWS secrets in your account
- **Flexible output**: JSON format (default) or plain text
- **Read-only**: Safe operations with no ability to modify or create secrets

## How it Works

Goldfinch automatically discovers all AWS Secrets Manager secrets in your AWS account.

- **`list` command**: Shows all top-level secret names
- **`get <secret_name>` command**: Retrieves all key-value pairs from a specific secret
- **`search` command**: Searches both secret names and key names within secrets

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

- `list` will show: `my-app-config`, `my-app-urls`
- `search app` will find the secret names containing "app" plus any keys containing "app"
- `get my-app-config` will retrieve all key-value pairs from `my-app-config`

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

### List all secret names

```bash
# JSON format (default)
goldfinch list

# Plain text format
goldfinch list --format plain
```

Output (JSON):
```json
[
  "my-app-config",
  "my-app-urls"
]
```

Output (plain):
```
my-app-config
my-app-urls
```

### Get all key-value pairs from a secret

```bash
# JSON format (default)
goldfinch get my-app-config

# Plain text format
goldfinch get my-app-config --format plain
```

Output (JSON):
```json
{
  "api_key": "abc123",
  "db_password": "secret123"
}
```

Output (plain):
```
api_key: abc123
db_password: secret123
```

### Search for secrets and keys

Search uses substring matching - it will find:
1. Secret names containing the pattern (displayed with `[Secret]` prefix)
2. Key names within secrets containing the pattern (displayed as `secret-name/key-name`)

```bash
# Find all secrets and keys containing "app"
goldfinch search app

# Find all secrets and keys containing "url" in plain format
goldfinch search url --format plain
```

Output (JSON) for `goldfinch search app`:
```json
[
  {
    "key": "[Secret] my-app-config",
    "value": "2 keys"
  },
  {
    "key": "[Secret] my-app-urls",
    "value": "2 keys"
  },
  {
    "key": "my-app-config/api_key",
    "value": "abc123"
  }
]
```

Output (plain) for `goldfinch search url`:
```
[Secret] my-app-urls: 2 keys
my-app-urls/prod_db_url: https://prod.example.com
my-app-urls/staging_db_url: https://staging.example.com
```

## Common Use Cases

**List all available secrets:**
```bash
goldfinch list
```

**Get a secret in JSON format and pipe to jq:**
```bash
goldfinch get my-app-config | jq '.api_key'
```

**Search for secrets containing a pattern:**
```bash
goldfinch search prod
```

**Search and process with jq to get only key names:**
```bash
goldfinch search app | jq '.[] | .key'
```

**Extract a specific value from a secret:**
```bash
# Using jq to extract a specific key
export API_KEY=$(goldfinch get my-app-config | jq -r '.api_key')
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
- **No search results**: "No secrets or keys found matching pattern 'pattern'" (searches both secret names and keys)
- **Access denied**: "Not authorized to perform operation"

## Development

### Run without installing

```bash
cargo run -- list
cargo run -- get my-app-config
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
cargo test test_get_secret_success
```

**Test Coverage:**
- Value type handling (strings, numbers, booleans, null, arrays, objects)
- Secret listing functionality
- Getting all key-value pairs from a secret by name
- Searching secrets and keys with substring matching
- Error cases (no matches, invalid JSON)
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
# List all secrets in your account
goldfinch list --format plain
# Output:
# app-config
# env-config

# Get all values from app-config secret
goldfinch get app-config

# Find all secrets and keys related to Redis
goldfinch search redis
# Output (JSON):
# [
#   {
#     "key": "app-config/redis_host",
#     "value": "localhost"
#   },
#   {
#     "key": "app-config/redis_port",
#     "value": "6379"
#   }
# ]

# Search for secrets containing "app" in their name
goldfinch search app
# Output (JSON):
# [
#   {
#     "key": "[Secret] app-config",
#     "value": "5 keys"
#   },
#   {
#     "key": "env-config/app_name",
#     "value": "myapp"
#   }
# ]
```

## License

This project is open source and available under the MIT License.
