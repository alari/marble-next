# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build, Lint, & Test Commands

- Build all crates: `cargo build`
- Run tests: `cargo test`
- Run single test: `cargo test test_name` or `cargo test path::to::test_function`
- Run specific crate tests: `cargo test -p crate-name`
- Test migrations: `./scripts/test_migrations.sh`
- Integration testing with PostgreSQL: `docker-compose -f docker-compose.test.yml up -d`

## Code Style Guidelines

- **Imports**: Group imports by source (std, external, internal) with blank lines between
- **Errors**: Use `thiserror` for error types, with descriptive error messages
- **Types**: Define public types at crate level and re-export them
- **Formatting**: Standard Rust formatting with 4 spaces indentation
- **Documentation**: Use doc comments (`//!` for module-level, `///` for item-level)
- **Naming**: Use snake_case for functions/variables, CamelCase for types
- **Error Handling**: Propagate errors with `?`, define custom error types per crate
- **Testing**: Write unit and integration tests; use `#[cfg(test)]` modules
- **Async**: Use `tokio` for async runtime with `async/await` syntax

## Development Workflow

- Follow the incremental development approach in GUIDES.md
- Update TASKS.md when completing or starting tasks
- Use conventional commit messages: feat/fix/docs/refactor/chore/test(scope): message
- Refer to GUIDES.md for complete collaboration guidelines