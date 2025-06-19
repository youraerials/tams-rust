# TAMS Rust Implementation

A high-performance Time-addressable Media Store (TAMS) API server implementation in Rust.

## Overview

This project implements the [TAMS API specification v6.0](https://github.com/bbc/tams) from the BBC, providing a REST API for managing time-addressable media content with support for sources, flows, segments, and webhook notifications.

## Quick Start

### 1. Initial Setup

Run the setup script to create the database and prepare the environment:

```bash
./setup.sh
```

This script will:

- Create necessary directories (`data/`, `media_storage/`, `temp_uploads/`)
- Create the SQLite database with proper schema
- Install `sqlx-cli` if not present
- Set up environment variables and SQLx query cache
- Prepare the project for compilation

### 2. Start the Server

Use the startup script for a reliable server start:

```bash
./start_server.sh
```

This script will:

- Ensure all directories exist
- Set the correct `DATABASE_URL` environment variable
- Verify database connectivity and recreate if needed
- Build and start the server with proper environment configuration

### 3. Alternative Manual Start

You can also run the server manually after setup:

```bash
# Basic start
cargo run

# With debug logging
RUST_LOG=debug cargo run

# With custom environment
DATABASE_URL="sqlite:./data/tams.db" cargo run
```

## Testing the API

### Test Endpoint

The server provides a test endpoint at `http://localhost:8080/test` that serves a basic HTML page for API testing. You can:

1. Open your browser to `http://localhost:8080/test`
2. Use it as a starting point for building custom test interfaces
3. Access the API documentation at `http://localhost:8080/`

### Example API Calls

```bash
# Get API information
curl http://localhost:8080/

# Get service capabilities
curl http://localhost:8080/service

# List sources
curl http://localhost:8080/sources

# Create a new source
curl -X POST http://localhost:8080/sources \
  -H "Content-Type: application/json" \
  -d '{"id":"test-source","format":"video","tags":{}}'
```

## Features

### ✅ Implemented

- **Core Data Models**: Complete TAMS data structures (Sources, Flows, Segments, Media Objects)
- **Database Layer**: SQLite-based persistence with connection pooling
- **Media Storage**: Local filesystem storage with presigned URL generation
- **Configuration Management**: TOML-based configuration with comprehensive settings
- **Error Handling**: Comprehensive error types with proper HTTP status mapping
- **Authentication**: JWT Bearer token and Basic Auth support
- **Webhook System**: Async webhook notifications for all TAMS events
- **Time Utilities**: Robust TAMS timestamp parsing and validation
- **HTTP Handlers**: Complete REST API endpoint implementations
- **Logging**: Structured logging with configurable levels and formats

### 🔧 Core Architecture

- **Async/Await**: Built on Tokio for high-performance async I/O
- **Type Safety**: Leverages Rust's type system for compile-time guarantees
- **Memory Efficient**: Zero-copy where possible, minimal allocations
- **Thread Safe**: Arc/Mutex patterns for safe concurrent access
- **Error Propagation**: Result-based error handling throughout

## API Endpoints

The implementation provides all TAMS v6.0 REST endpoints:

### Core Endpoints

- `GET /` - Root endpoint with API information
- `GET /service` - Service capabilities and information
- `GET /test` - Test page for API interaction

### Sources Management

- `GET /sources` - List sources with pagination
- `POST /sources` - Create new source
- `GET /sources/{sourceId}` - Get specific source
- `PUT /sources/{sourceId}` - Update source
- `DELETE /sources/{sourceId}` - Delete source

### Flows Management

- `GET /flows` - List flows with pagination
- `POST /flows` - Create new flow
- `GET /flows/{flowId}` - Get specific flow
- `PUT /flows/{flowId}` - Update flow
- `DELETE /flows/{flowId}` - Delete flow

### Flow Segments

- `GET /flows/{flowId}/segments` - List flow segments
- `POST /flows/{flowId}/segments` - Add segments to flow
- `DELETE /flows/{flowId}/segments` - Delete segments by timerange

### Storage Management

- `GET /flows/{flowId}/storage` - Get presigned upload URLs

### Media Objects

- `GET /objects/{objectId}` - Get media object metadata
- `HEAD /objects/{objectId}` - Check media object existence

### Webhooks

- `GET /service/webhooks` - List registered webhooks
- `POST /service/webhooks` - Register new webhook
- `DELETE /service/webhooks/{url}` - Unregister webhook

### Flow Deletion Requests

- `GET /flow-delete-requests` - List deletion requests
- `POST /flow-delete-requests` - Create deletion request
- `GET /flow-delete-requests/{id}` - Get deletion request status

## Configuration

The server is configured via `config.toml`:

```toml
[server]
host = "127.0.0.1"
port = 8080
workers = 4

[database]
url = "sqlite:./data/tams.db"
max_connections = 10
connection_timeout_seconds = 30

[media_storage]
base_path = "./media_storage"
max_file_size = 104857600  # 100MB
temp_path = "./temp_uploads"

[service]
name = "TAMS Rust Implementation"
description = "Time-addressable Media Store API server in Rust"
version = "6.0"
media_store_type = "http_object_store"
public_url_base = "http://localhost:8080/media"

[auth]
require_auth = false
jwt_secret = "your-256-bit-secret-key-change-this-in-production"
basic_auth_username = "admin"
basic_auth_password = "password"

[cors]
allowed_origins = ["*"]
allowed_methods = ["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS"]
allowed_headers = ["*"]

[logging]
level = "info"
format = "compact"  # "compact", "pretty", or "json"

[pagination]
default_limit = 50
max_limit = 1000

[cleanup]
temp_file_retention_hours = 24
orphaned_object_retention_days = 7
```

## Prerequisites

- **Rust**: 1.70 or higher
- **SQLite**: 3.35 or higher
- **sqlx-cli**: Automatically installed by setup script

## Project Structure

```
tams-rust/
├── src/
│   ├── main.rs           # Application entry point and routing
│   ├── config.rs         # Configuration loading and structures
│   ├── models.rs         # TAMS data models and types
│   ├── database.rs       # Database operations and migrations
│   ├── storage.rs        # Media storage abstraction
│   ├── handlers.rs       # HTTP request handlers
│   ├── auth.rs           # Authentication middleware
│   ├── webhooks.rs       # Webhook notification system
│   ├── time_utils.rs     # Time parsing and validation utilities
│   └── error.rs          # Error types and HTTP mapping
├── config.toml           # Server configuration
├── setup.sh              # Initial setup and database creation
├── start_server.sh       # Server startup script
├── create_db.sql         # Database schema
├── test.html             # Test page served at /test endpoint
├── api-spec.yaml         # OpenAPI specification
└── Cargo.toml            # Rust dependencies and metadata
```

## Setup Script Details

The `setup.sh` script performs the following operations:

1. **Directory Creation**: Creates `data/`, `media_storage/`, and `temp_uploads/` directories
2. **Database Setup**: Creates SQLite database from `create_db.sql` schema
3. **Environment Configuration**: Sets up `.env` file with `DATABASE_URL`
4. **Dependencies**: Installs `sqlx-cli` for database query preparation
5. **Query Cache**: Prepares SQLx offline query cache for compilation

**Usage:**

```bash
chmod +x setup.sh
./setup.sh
```

## Start Server Script Details

The `start_server.sh` script ensures reliable server startup:

1. **Environment Setup**: Sets absolute paths and environment variables
2. **Database Verification**: Checks database existence and connectivity
3. **Auto-Recovery**: Recreates database if corrupted or missing
4. **Server Launch**: Builds and starts the server with proper configuration

**Usage:**

```bash
chmod +x start_server.sh
./start_server.sh
```

**Environment Variables:**

- `DATABASE_URL`: Automatically set to absolute SQLite path
- `RUST_LOG`: Optional logging level (defaults to "info")

## Data Models

### Core Types

- **Source**: Media source with format and metadata
- **Flow**: Media flow with encoding parameters and timerange
- **FlowSegment**: Time-bounded segment within a flow
- **MediaObject**: Actual media file with references
- **TimeRange**: Start/end timestamps in TAMS format (`seconds:nanoseconds`)

### Time Format

TAMS uses a specific timestamp format: `{unix_seconds}:{nanoseconds}`

Examples:

- `1609459200:000000000` (2021-01-01 00:00:00 UTC)
- `1609459200:500000000` (2021-01-01 00:00:00.5 UTC)

## Webhook Events

The system sends webhook notifications for:

- `source.created` - New source created
- `source.updated` - Source modified
- `source.deleted` - Source removed
- `flow.created` - New flow created
- `flow.updated` - Flow modified
- `flow.deleted` - Flow removed
- `segments.added` - Segments added to flow
- `segments.deleted` - Segments removed from flow

## Authentication

### JWT Bearer Tokens

```bash
curl -H "Authorization: Bearer <token>" http://localhost:8080/sources
```

### Basic Auth

```bash
curl -u admin:password http://localhost:8080/sources
```

## Storage

Media files are stored in a local directory structure:

```
media_storage/
├── object-id-1
├── object-id-2
└── ...
```

Upload URLs are generated for secure file transfers and expire after 1 hour by default.

## Development

### Build and Test

```bash
# Check compilation
cargo check

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy

# Build release
cargo build --release
```

### Logging

Configure logging via environment or config:

```bash
# Debug logging
RUST_LOG=debug cargo run

# JSON format logging
RUST_LOG=info cargo run
# (Configure format in config.toml)
```

### Database Management

```bash
# Reset database
rm -f data/tams.db
./setup.sh

# Query database directly
sqlite3 data/tams.db "SELECT * FROM sources;"

# Prepare SQLx queries after schema changes
cargo sqlx prepare
```

## Troubleshooting

### Common Issues

1. **Compilation Errors**: Run `./setup.sh` to prepare SQLx queries
2. **Database Errors**: Delete `data/tams.db` and re-run setup
3. **Permission Issues**: Check file permissions on database and directories
4. **Port Already in Use**: Change port in `config.toml` or stop conflicting services

### Logs and Debugging

```bash
# Verbose logging
RUST_LOG=debug ./start_server.sh

# Check database
sqlite3 data/tams.db ".tables"

# Test API connectivity
curl -v http://localhost:8080/
```

## Performance

- **Async I/O**: All operations are non-blocking
- **Connection Pooling**: Database connections are pooled and reused
- **Streaming**: Large file operations use streaming where possible
- **Memory Efficient**: Minimal allocations and zero-copy operations
- **Concurrent**: Handles multiple requests concurrently

## Current Status

### Completed ✅

- All core modules implemented
- Database schema and operations
- HTTP routing and handlers
- Authentication middleware
- Webhook system
- Media storage abstraction
- Configuration management
- Error handling
- Time utilities
- Logging setup
- Setup and startup scripts

### Next Steps 🔧

1. **Integration Testing**: Add end-to-end API tests
2. **Enhanced Test Page**: Improve test.html with interactive API testing
3. **Media Upload/Download**: Complete file upload/download endpoints
4. **Background Tasks**: Implement deletion request processing
5. **Monitoring**: Add metrics and health checks
6. **Docker**: Add containerization support

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run `cargo fmt` and `cargo clippy`
6. Submit a pull request

## License

Apache 2.0 License

## References

- [TAMS API Specification](https://github.com/bbc/tams)
- [OpenAPI Specification](./api-spec.yaml)
- [BBC Technical Standards](https://www.bbc.co.uk/makerbox/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Axum Documentation](https://docs.rs/axum/)
