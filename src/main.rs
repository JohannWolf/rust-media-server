mod api;
mod config;
mod database;
mod scanner;
mod metadata;
mod media;

use api::create_router;
use config::Config;
use database::init_database;
use scanner::{scan_directory,watch_directory};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Rust Media Server...");

    // Load application configuration.
    let config = Config::from_env()?;

    println!("Media directory: {}", config.media_root);
    println!("Database: {}", config.database_url);

    // Initialize SQLite and run pending migrations.
    let pool = init_database(&config.database_url).await?;

    println!("Database initialized successfully.");

    // Scan the media directory when the server starts.
    let media_path = Path::new(&config.media_root);

    let media_files = scan_directory(media_path)?;

    println!("Found {} media files:", media_files.len());

    for file in &media_files {
        println!("  {}", file.display());
    }

    // Process every discovered media file.
    for file in &media_files {
        println!("Processing: {}", file.display());

        // Extract media metadata using ffprobe.
        let metadata = match metadata::extract_metadata(file) {
            Ok(metadata) => metadata,

            Err(error) => {
                eprintln!(
                    "Could not extract metadata from '{}': {}",
                    file.display(),
                    error
                );

                // Don't let one broken media file stop the entire scan.
                continue;
            }
        };

        // Get filesystem information about the file.
        let file_size = match std::fs::metadata(file) {
            Ok(metadata) => metadata.len(),

            Err(error) => {
                eprintln!(
                    "Could not read filesystem metadata for '{}': {}",
                    file.display(),
                    error
                );

                continue;
            }
        };

        // Store the media information in SQLite.
        if let Err(error) = media::upsert_media(
            &pool,
            &metadata,
            &file.to_string_lossy(),
            file_size as i64,
        )
        .await
        {
            eprintln!(
                "Could not save '{}' to database: {}",
                file.display(),
                error
            );

            continue;
        }

        println!(
            "  Added: {} ({})",
            metadata.filename,
            metadata.media_type
        );
    }

     // Start the filesystem watcher in a separate Tokio task.
    //
    // `tokio::spawn` allows the watcher to run concurrently with
    // the Axum HTTP server.
    let watcher_path = media_path.to_path_buf();

    tokio::spawn(async move {
        if let Err(error) = watch_directory(&watcher_path).await {
            eprintln!("Filesystem watcher stopped: {}", error);
        }
    });

    // Create the Axum application.
    let app = create_router(pool.clone());

    let address = format!("{}:{}", config.host, config.port);

    let listener = tokio::net::TcpListener::bind(&address).await?;

    println!("Server listening on http://{}", address);

    axum::serve(listener, app).await?;

    Ok(())
}