mod api;
mod config;
mod database;

use api::create_router;
use config::Config;
use database::init_database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Rust Media Server...");

    let config = Config::from_env()?;

    println!("Media directory: {}", config.media_root);
    println!("Database: {}", config.database_url);

    let _pool = init_database(&config.database_url).await?;

    println!("Database initialized successfully.");

    let app = create_router();

    let address = format!("{}:{}", config.host, config.port);

    let listener = tokio::net::TcpListener::bind(&address).await?;

    println!("Server listening on http://{}", address);

    axum::serve(listener, app).await?;

    Ok(())
}