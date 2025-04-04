# Documentation Standards

## Status Indicators
Each specification document should include a status indicator:
- **[DRAFT]** - Initial documentation, may be incomplete or change significantly
- **[REVIEW]** - Ready for review, seeking feedback
- **[STABLE]** - Approved specification, only minor changes expected
- **[IMPLEMENTED]** - Specification has been implemented in code

## File Naming Conventions
- All filenames should use underscores instead of hyphens (e.g., `marble_core.md` not `marble-core.md`)
- Template files should be named `template.md` in each directory
- Domain concept files should be descriptive of the concept (e.g., `authentication.md`)

## Cross-References
- All specs should include a "Related Specifications" section
- Links should use relative paths (e.g., `../domain/concept.md`)
- Link text should include the document title

## Diagrams
- Use Mermaid diagrams for visualizations
- Include diagrams directly in Markdown using triple-backtick syntax
- Provide a text description of the diagram for accessibility
