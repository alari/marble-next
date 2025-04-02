# Marble Core Specification

## Overview

The `marble_core` crate provides shared functionality, types, and interfaces used by both the Write Side and Read Side components of Marble. It establishes the foundational data model and common utilities needed across the system.

## Responsibilities

- Define core domain types for Obsidian content
- Provide authentication and user management interfaces
- Establish common error handling patterns
- Define shared configuration and environment handling
- Implement tenant isolation primitives

## Content Model

The core content types in Marble are:
1. **Markdown Documents** - Obsidian-compatible markdown files with frontmatter
2. **Canvas Documents** - Obsidian canvas visualization files
3. **Media Objects** - Images and other media files

### Frontmatter Structure
Markdown files can contain YAML frontmatter with the following structure:

```rust
pub struct FrontMatter {
    // Obsidian basics
    pub tags: Option<Vec<String>>,        // Content categorization
    pub aliases: Option<Vec<String>>,     // Alternative names for reference resolution
    
    // Publishing control
    pub publish: bool,                    // Whether content should be published
    pub permalink: Option<String>,        // Custom URL path for published content
    
    // Metadata
    pub section: Option<String>,          // For opengraph section tracking
    pub description: Option<String>,      // For social media links
    pub cover: Option<String>,            // Reference image, should be collected
    pub image: Option<String>,            // Href, not to be collected, just used
    pub dates: Option<FrontDates>,        // Creation and modification dates
    pub links: Option<Vec<String>>,       // External links
    pub title: Option<String>,            // Custom title
    
    // Display options
    pub display: DisplayParams,           // Layout and title display settings
    
    // Other custom fields
    pub other: serde_json::Value,         // Any additional custom fields
}
```

### Obsidian References
Markdown content includes Obsidian-specific syntax:
- References: `[[page-name]]` or `[[page-name|display text]]`
- Embeds: `![[page-name]]` or `![[page-name#section]]`

## User/Tenant Model

In Marble, users and tenants are equivalent:
- User represents an individual with authentication credentials
- Each user has their own isolated data space (equivalent to an Obsidian vault)
- No sharing or multi-user access to a single vault is supported

## Authentication

The core API includes a minimal interface for user authentication:

```rust
// Example authentication interfaces
pub trait UserAuthentication {
    // Verify user credentials
    fn authenticate(&self, username: &str, password: &str) -> Result<bool, AuthError>;
}

// Default implementation from configuration
pub struct ConfigUserAuth {
    users: HashMap<String, String>, // username -> password hash
}
```

User management is intentionally minimal:
- Users are created manually through configuration
- No public API for user creation/management
- User records are stored in the database
- Future expansion possible if needed

## Data Structures

Core data structures:
- User/authentication models
- Obsidian markdown representation
- Obsidian canvas representation
- Media object metadata
- Path management utilities

## Dependencies

Potential dependencies:
- `opendal` for storage abstraction
- `thiserror` for error handling
- `serde` for serialization
- `argon2` or similar for password hashing
- `config` for configuration management

## Integration Points

- Authentication interfaces used by WebDAV server
- Content type definitions used by both sides
- Path utilities for mapping between raw and processed storage

## Future Work

- Define exact format specifications for Obsidian compatibility
- Establish path transformation rules between raw and processed
- Create error type hierarchy
