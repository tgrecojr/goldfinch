# Code Style & Conventions

## Rust Style
- Follow standard Rust conventions (enforced by `cargo fmt`)
- Use `rustfmt` for automatic formatting
- Use `clippy` for linting

## Naming Conventions
- **Functions**: snake_case (e.g., `fetch_secret`, `value_to_string`)
- **Structs**: PascalCase (e.g., `OutputFormat`, `Commands`)
- **Enums**: PascalCase with PascalCase variants (e.g., `OutputFormat::Json`)
- **Constants**: SCREAMING_SNAKE_CASE

## Code Organization
- Single file architecture (src/main.rs)
- Functions organized logically: AWS operations, data processing, output formatting
- Unit tests in `#[cfg(test)]` module at bottom of main.rs
- Integration tests in separate `tests/` directory

## Error Handling
- Use `anyhow::Result` for error propagation
- Provide user-friendly error messages
- Use `?` operator for error propagation

## Testing Conventions
- Unit tests cover individual functions with mock data
- Integration tests use `assert_cmd` to test CLI behavior
- Test names follow pattern: `test_<function>_<scenario>`
- Use descriptive test names and assertions
