pub mod scanner;
pub mod watcher;

// Re-export the function so main.rs can use it
pub use scanner::scan_directory;
pub use watcher::watch_directory;
