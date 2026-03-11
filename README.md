# Tarasque

<img width="2719" height="1796" alt="CleanShot 2026-03-11 at 20 54 33@2x" src="https://github.com/user-attachments/assets/f5fbcc79-6f56-40e0-aba0-f2098accaa8e" />

A desktop mock API server for designing, testing, and debugging HTTP endpoints. Built with Tauri v2, Rust, and Svelte 5.

## Features

- Create and manage multiple API collections
- Define endpoints with multiple response scenarios
- Dynamic response templates with variables (`{{uuid}}`, `{{request.body.field}}`, etc.)
- Scenario match rules (route by headers, query params, or body content)
- Sequence mode to cycle responses on each request
- Environment variables per collection
- Proxy forwarding for unmatched routes
- HTTPS alias with self-signed certificates
- Request logging with detail view
- OpenAPI/Swagger import
- Export to JSON or Postman format
- cURL import
- Test suite runner with assertions
- Hot-reload responses without restarting the server
- Monaco editor for response body editing

## Tech Stack

- **Frontend**: Svelte 5, Tailwind CSS, Monaco Editor
- **Backend**: Rust, Actix-web, Tokio
- **Framework**: Tauri v2
- **Package Manager**: Bun

## Prerequisites

- [Rust](https://rustup.rs/)
- [Bun](https://bun.sh/)
- [Tauri CLI prerequisites](https://v2.tauri.app/start/prerequisites/)

## Development

```bash
# Install dependencies
bun install

# Run in development mode
bun run tauri:dev
```

## Production Build

```bash
bun run tauri:build
```

## Project Structure

```
src/                    # Svelte frontend
  lib/
    components/
      editor/           # Endpoint editor, scenario editor, Monaco
      server/           # Server panel, logs, stats
      sidebar/          # Collection list, running APIs, logs sidebar
    stores/             # Svelte stores (collections, server, logs, settings)
    types.ts            # TypeScript types
src-tauri/              # Rust backend
  src/
    server/             # Actix-web mock server (router, state, rules, templates)
    commands.rs         # Tauri commands
    models.rs           # Data models
    storage.rs          # File persistence
```
