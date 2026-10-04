pub mod scanner_logic;
pub mod watcher;

// Re-export the function so main.rs can use it
pub use scanner_logic::scan_directory;
pub use watcher::watch_directory;
