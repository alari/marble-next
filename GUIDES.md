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
- Create and maintain a `spec` folder with the following structure:
  * `spec/spec.md` - Project overview and index to other specs
  * `spec/crates/` - Directory for all crate-specific specs
  * `spec/domain/` - Directory for domain concept specs
  * `spec/handoffs/` - Directory for work-in-progress handoffs
- For every crate (both internal and external):
  * Create a dedicated spec file in `spec/crates/{crate_name}.md`
  * Document purpose, responsibilities, key interfaces, and usage patterns
  * For external crates, document version-specific features and limitations
- Keep specs concise and interconnected with clear navigation links
- For data structures, reference the actual code file if it's self-documenting
- Specs should capture insights that would be difficult to regain from reading code
- Use Mermaid diagrams when they clarify architecture or workflows
- Update affected spec files after EACH iteration of development
- After completing significant work:
  * Review all affected specs for soundness and validity
  * Remove obsolete information
  * Split overly long sections into separate files
  * Ensure specs are structured logically and contain sufficient context
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

## 12. Initial Session Workflow

- When beginning a new chat with these guidelines:
  * Read only the minimal set of files needed to understand the project basics:
    - README.md (for project overview)
    - Root Cargo.toml (for project structure)
    - spec/spec.md (if available, for domain context)
  * After reading these initial files, reach back to clarify next steps
  * Explicitly ask what to focus on next:
    - "Would you like me to focus on improving specifications?"
    - "Should I work on implementation of a specific feature?"
    - "Would you like me to help with testing or refactoring?"
  * Take no actions and make no changes until receiving explicit direction
  * When given direction, follow the principles in these guidelines
  * Prioritize understanding over action until the path forward is clear

## 13. Handoff Process

- Maintain handoff files in a dedicated `spec/handoffs/` directory
- In `spec/spec.md`, include a "Current Handoffs" section listing active handoff files:
  Current Handoffs

  [User Authentication](handoffs/user_auth.md) - Basic login flow implemented
  [Task Management](handoffs/task_management.md) - API design in progress
- Create individual handoff files for each feature/task in progress
- Name files descriptively: `spec/handoffs/feature_name.md`
- After significant changes, update the relevant handoff file with:
  * Current status summary (1-2 sentences)
  * What was accomplished in the current session
  * Key insights and decisions from our discussions
  * Architectural or design decisions made
  * Known issues, limitations, or challenges
  * Next steps to continue work
  * References to relevant spec files and code
  * Any blockers or questions that need resolution
- Completely rewrite each handoff file after significant changes
- At the beginning of a session:
  * Reference the handoff list from `spec/spec.md` without reading individual handoff files
  * Ask if any specific handoff should be read
  * Only read the requested handoff file(s)
  * Wait for explicit instructions before taking action, even after reading handoffs
- Link from handoff files to relevant specs and from specs to relevant handoffs when appropriate
- Include a "Last updated" timestamp at the top of each handoff file

## 14. Incremental Development (CRITICAL)

- Break implementation into small, focused iterations
- Each iteration should:
  * Implement one specific piece of functionality
  * Be small enough to complete in a single step (typically <100 lines of new code)
  * Compile successfully
  * Be testable whenever possible
- After EACH iteration:
  * Update the relevant handoff file
  * Review and update affected spec files
  * Verify compliance with all guidelines
- Propose the next increment explicitly before proceeding
- Never attempt to implement an entire feature in one step
- When describing the next increment, be specific about:
  * What will be implemented
  * Which files will be modified
  * Expected outcome
- If `cargo build` fails:
  * Report the error immediately
  * Propose a solution
  * Wait for explicit approval before implementing the fix
- Always verify dependencies before adding them:
  * Check that you're using current versions
  * Confirm they're added to the correct Cargo.toml location
  * Use .workspace = true syntax for workspace dependencies
- Remember: Many small working steps are better than one large implementation that might not compile

## 15. Dependency Management for Unknown Crates
- When encountering an unfamiliar dependency or version:
  1. First check `spec/crates/{dependency_name}.md` for existing documentation
  2. If no documentation exists, ask the user for information about:
     * The dependency's purpose and core functionality
     * Preferred usage patterns
     * Any specific version constraints or features to use
     * Alternatives considered and why this dependency was chosen
  3. Document this information in `spec/crates/{dependency_name}.md`
  4. Reference this spec when using the dependency in code
- Always verify dependencies have been correctly added to Cargo.toml files
- Use `.workspace = true` syntax for all workspace dependencies
