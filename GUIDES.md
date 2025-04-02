# Rust Codebase Collaboration Guidelines

## 0. Starting Workflow
- Begin by reading key context files in this order:
  * README.md for project overview
  * Root Cargo.toml to understand project organization and dependencies
  * spec/spec.md if it exists for domain context
- Only explore deeper when specific problems require it
- When deeper exploration is needed, prioritize:
  * Relevant spec files
  * Crate-specific Cargo.toml files
  * API trait/interface files

## 1. Crate Structure
- Use small, focused crates with clear, single responsibilities
- Each crate should represent a distinct domain or technical concern
- Favor many small crates over few large ones to simplify understanding and maintenance
- Aim for crates that can be explained in a single paragraph

## 2. API Interface Organization
- Each crate should have a clearly identifiable API entry point
- Implement using an `api.rs` file at the crate root or an `api` module containing public interfaces
- Public interfaces should be defined as traits when possible
- Example structure:
    my_crate/
    src/
    api.rs        # Contains public-facing traits and types
    impl/         # Implementation details
    model.rs      # Data structures
    lib.rs        # Re-exports from api.rs

## 3. Specs Management
- Create and maintain a `spec` folder in the root of the project
- Maintain a central `spec/spec.md` with project overview and index to other specs
- Create separate spec files for each crate and domain concept
- For every crate you generate or learn important details about, create a dedicated spec file named after the crate
- Keep specs concise and interconnected with clear navigation links
- For data structures, reference the actual code file if it's self-documenting
- Specs should capture insights that would be difficult to regain from reading code
- Use Mermaid diagrams when they clarify architecture or workflows
- Update specs whenever gaining insights that would be challenging to reconstruct later
- After completing significant work (feature acceptance, module decomposition, responsibility changes):
  * Review all affected specs for soundness and validity
  * Remove obsolete information
  * Split overly long sections into separate files
  * Ensure specs are structured logically and contain sufficient context
  * Verify that individual spec files remain concise and focused
- Prepare a commit after any significant reorganization of specs
- Treat specs as the project's memory - prioritize their quality and accuracy

## 4. Testing Approach
- Write unit tests for public interfaces
- Co-locate tests with code using `#[cfg(test)]` modules
- Focus on testing behavior, not implementation details
- Treat integration/end-to-end tests as separate initiatives to be explicitly proposed
- Use test-driven development (TDD) when appropriate for complex logic

## 5. Error Handling
- Use `thiserror` for defining error types in library crates
- Use `anyhow` for application code when appropriate
- Create domain-specific error types for each module
- Propagate errors when:
* The caller needs to make decisions based on error variants
* The error needs context only available at a higher level
- Handle errors locally when:
* They're expected parts of normal operation
* They can be meaningfully recovered from at the current level
- Provide helpful error messages that give context on what went wrong

## 6. Dependency Management
- Place all dependencies with explicit versions in the main Cargo.toml
- Register all workspace crates in the root Cargo.toml's [workspace.dependencies] section
- In individual crate Cargo.toml files, reference workspace dependencies using the `.workspace = true` syntax:
  ```toml
  [dependencies]
  some-crate.workspace = true
  tokio.workspace = true
  ```
- Avoid using path references like some-crate = {path = "../../some-crate"}
- Be selective with feature flags - only enable what's necessary
- Document dependency selection rationale in specs

## 7. Async Approach
- Use async/await by default with tokio
- Enable only necessary tokio features (typically "rt-multi-thread" and "macros")
- Use blocking functions only for operations that don't benefit from async

## 8. Configuration
- Use dotenv for environment-based configuration
- Implement a typed config structure for application settings
- Provide sensible defaults where appropriate
- Validate configurations at startup

## 9. Version Control & Changelog
- Use conventional commit messages with types and scopes:
  * `feat(scope): message` - for new features
  * `fix(scope): message` - for bug fixes
  * `refactor(scope): message` - for code changes that neither fix bugs nor add features
  * `docs(scope): message` - for documentation changes
  * `chore(scope): message` - for maintenance tasks
  * `test(scope): message` - for adding or fixing tests
- Keep commit messages concise and descriptive
- Commit after any significant documentation reorganization with `docs(specs): <description>`
- Update CHANGELOG.md after feature completion and user acceptance
- Format: `## [version] - YYYY-MM-DD` followed by bullet points
- Add changelog entries only after changes are tested and approved

## 10. Communication
- Ask before introducing new features or dependencies
- Request clarification for ambiguous requirements
- Present multiple options when appropriate, with clear trade-offs
- Ask for quick user input if it reduces research time
- Acknowledge we're collaborating as experienced Rust developers

## 11. Progress Tracking

- Maintain a `TASKS.md` file in the root directory to track work status
- Format tasks with clear status indicators:
  * `[TODO]` - Not yet started
  * `[WIP]` - Work in progress
  * `[REVIEW]` - Ready for review
  * `[DONE]` - Completed
- Include for each task:
  * Descriptive title
  * Link to relevant spec files
  * Next concrete steps (for in-progress tasks)
  * Completion criteria
- For specs that need more development, add a `## Future Work` section with:
  * Missing information that needs to be gathered
  * Areas that need deeper exploration
  * Questions that need answers
- Begin each session by reviewing `TASKS.md` to identify the current focus
- Update task status at the end of each significant action
- Each task should represent an achievable unit of work that can be completed in a reasonable timeframe
- Break down large features into smaller, trackable tasks
