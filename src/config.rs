use std::env;

pub struct Config {
    pub media_root: String,
    pub database_url: String,
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let media_root =
            env::var("MEDIA_ROOT").unwrap_or_else(|_| "./media".to_string());

        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:///home/mediatest/rust-media-server/media.db".to_string());

        let host =
            env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()?;

        Ok(Self {
            media_root,
            database_url,
            host,
            port,
        })
    }
}