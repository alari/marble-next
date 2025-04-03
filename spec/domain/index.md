# Domain Concepts Index

**Last Updated:** 2025-04-03

This directory contains specifications for domain concepts and architectural decisions in the Marble system.

## Core Concepts

- [Glossary](glossary.md) - Definitions of key terms and concepts
- [Architecture Diagram](architecture_diagram.md) - Visual overview of system architecture

## Architecture

- [Storage Architecture](storage_architecture.md) - Content storage design
- [Database Schema](database_schema.md) - PostgreSQL schema specification
- [Crate Dependencies](crate_dependencies.md) - Relationship between crates

## System Sides

- [Write Side](write_side.md) - Content creation and management process
- [Read Side](read_side.md) - Content publishing process

## Templates

- [template](template.md) - Template for creating new domain concept specifications

## Concept Status Summary

| Concept | Status | Description | Related Crates |
|---------|--------|-------------|----------------|
| Storage Architecture | DRAFT | Hybrid storage with S3 and PostgreSQL | marble_storage |
| Database Schema | WIP | Relational schema for metadata | marble_db |
| Write Side | DRAFT | Content ingest and processing | marble_write_processor |
| Read Side | DRAFT | Content publishing | marble_read_processor |
