# Rust Media Server

A lightweight self-hosted media server written in Rust for streaming and managing personal media over a local network.

The project is designed to run on a headless Debian server and provide media playback to devices such as Android phones and Google TV using standard HTTP streaming and VLC.

## Project Status

**Early development**

The core application structure is in place, including:

- Rust/Tokio application runtime
- Axum HTTP server
- SQLite database with SQLx
- Database migrations
- Recursive media filesystem scanner
- Filesystem change watcher
- Media metadata extraction using `ffprobe`
- Media library persistence in SQLite
- Basic health-check endpoint

Media streaming and the full REST API are planned next.

## Goals

The main goals of the project are:

- Run efficiently on a headless Linux server
- Store and manage metadata for a personal media library
- Detect new, modified, and removed media files
- Stream media over a local network
- Support common video and audio formats
- Work without requiring an internet connection for local playback
- Provide a simple HTTP API for media discovery
- Support VLC and other standard media clients
- Prefer direct playback when the client supports the media format

Future versions may add transcoding, remote access, authentication, subtitles, thumbnails, and additional client integrations.

## Architecture

```text
                         HOME NETWORK

              ┌──────────────────────────┐
              │                          │
        ┌─────▼─────┐              ┌─────▼─────┐
        │  Android  │              │ Google TV │
        │   + VLC   │              │   + VLC   │
        └─────┬─────┘              └─────┬─────┘
              │                          │
              └────────── HTTP ──────────┘
                           │
                    ┌──────▼──────┐
                    │   Axum API  │
                    │             │
                    │ Rust/Tokio  │
                    └──────┬──────┘
                           │
             ┌─────────────┼─────────────┐
             │             │             │
       ┌─────▼─────┐ ┌────▼─────┐ ┌─────▼─────┐
       │  Scanner  │ │ Metadata │ │  SQLite   │
       │           │ │ ffprobe  │ │  SQLx     │
       └─────┬─────┘ └──────────┘ └───────────┘
             │
       ┌─────▼─────────┐
       │ Media Storage │
       │ MP4/MKV/MP3…  │
       └───────────────┘
```

## Technology Stack

- **Rust** — application language
- **Tokio** — asynchronous runtime
- **Axum** — HTTP server and API framework
- **SQLx** — database access and migrations
- **SQLite** — media library database
- **Serde / serde_json** — JSON serialization and deserialization
- **notify** — filesystem monitoring
- **ffprobe** — media metadata extraction
- **VLC** — planned primary playback client
- **Debian Linux** — target server operating system

## Current Functionality

### Media Scanner

The scanner recursively searches the configured media directory for supported media files.

Currently supported extensions include:

```text
Video:
- mp4
- mkv
- avi
- mov
- webm

Audio:
- mp3
- flac
- wav
- m4a
```

The scanner is case-insensitive, so files such as:

```text
movie.mkv
movie.MKV
movie.Mkv
```

are treated the same way.

### Filesystem Watcher

The server uses the `notify` crate to monitor the media directory recursively.

Filesystem events are currently logged by the application.

The planned event-processing pipeline is:

```text
Filesystem event
       ↓
notify
       ↓
event processor
       ├── Created → metadata extraction → database
       ├── Modified → metadata update
       └── Removed → database deletion
```

### Metadata Extraction

Media metadata is extracted using `ffprobe`.

The application currently extracts information such as:

- filename
- media type
- container format
- duration
- video codec
- audio codec
- video width
- video height

The metadata is represented in Rust using strongly typed structures before being persisted to SQLite.

### Database

SQLite stores the media library.

Each media file has a unique filesystem path.

The application uses SQLx migrations to create and evolve the database schema.

Migrations are intentionally kept as separate incremental files.

## Configuration

Configuration is provided through environment variables.

Example development configuration:

```text
MEDIA_ROOT=./media
DATABASE_URL=sqlite://media.db
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
```

Example Debian server configuration:

```text
MEDIA_ROOT=/mnt/media
DATABASE_URL=sqlite:///var/lib/rust-media-server/media.db
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
```

The server binds to `0.0.0.0` on the Debian host so devices on the local network can connect to it.

## Running Locally

Make sure Rust and `ffprobe` are installed.

Create a media directory:

```bash
mkdir media
```

Place some media files inside it.

Then run:

```bash
cargo run
```

The application will:

1. Load configuration
2. Initialize SQLite
3. Run pending migrations
4. Scan the media directory
5. Extract metadata using `ffprobe`
6. Store media information in SQLite
7. Start the filesystem watcher
8. Start the HTTP server

## Health Check

The current HTTP API includes:

```text
GET /health
```

A successful request returns:

```text
200 OK
```

Example:

```bash
curl http://127.0.0.1:8080/health
```

## Database Migrations

Database schema changes are managed through SQLx migrations.

Existing migrations should not be modified after they have been applied to a database.

Instead, new schema changes should be introduced through new migration files:

```text
migrations/
├── 202609170000_init.sql
├── 202609270000_add_media_metadata.sql
└── future_migration.sql
```

When the application starts, SQLx automatically applies any migrations that have not yet been executed.

## Planned Features

### Media API

```text
GET /api/media
GET /api/media/:id
```

### Streaming

HTTP media streaming with support for:

- Range requests
- seeking
- VLC playback
- direct playback

### Library Management

- Detect newly added files
- Detect modified files
- Detect deleted files
- Keep SQLite synchronized with the filesystem

### Future Features

Possible future additions include:

- FFmpeg transcoding
- Subtitle support
- Thumbnail generation
- Authentication
- Multiple users
- DLNA/UPnP
- Remote access
- Web interface
- Playback history
- Media collections
- Search and filtering

## Design Principles

The project intentionally separates responsibilities between components.

```text
Scanner
    → discovers files

Metadata extractor
    → understands media files

Repository
    → persists media information

Database
    → manages database connection and migrations

API
    → exposes application functionality over HTTP

Streaming
    → serves media to clients
```

This separation allows individual components to evolve without turning the application into a single large module.

## License

Code released under the MIT License