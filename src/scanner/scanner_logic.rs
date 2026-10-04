use std::fs;
use std::path::{Path, PathBuf};

// File extensions that the media server currently understands.
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp4",
    "mkv",
    "avi",
    "mov",
    "webm",
    "mp3",
    "flac",
    "wav",
    "m4a",
];

/// Recursively scans a directory and returns all supported media files.
///
/// This is intentionally synchronous for now. The initial scan happens
/// when the application starts, to be improved.
pub fn scan_directory(root: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut media_files = Vec::new();

    scan_directory_recursive(root, &mut media_files)?;

    Ok(media_files)
}

/// Recursive helper used by `scan_directory`.
///
/// `media_files` is passed as a mutable reference so every recursive call
/// can add files to the same Vec instead of creating a new Vec each time.
fn scan_directory_recursive(
    directory: &Path,
    media_files: &mut Vec<PathBuf>,
) -> Result<(), std::io::Error> {
    // Read the entries contained in the current directory.
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        // If this entry is another directory, scan it recursively.
        if path.is_dir() {
            scan_directory_recursive(&path, media_files)?;
            continue;
        }

        // Ignore anything that isn't a regular file.
        if !path.is_file() {
            continue;
        }

        // Check the file extension.
        if is_supported_media_file(&path) {
            media_files.push(path);
        }
    }

    Ok(())
}

/// Returns true when the file has one of our supported media extensions.
fn is_supported_media_file(path: &Path) -> bool {
    // `extension()` returns an Option because a file doesn't necessarily
    // have an extension.
    let Some(extension) = path.extension() else {
        return false;
    };

    // Convert the extension to lowercase.
    let extension = extension.to_string_lossy().to_lowercase();

    SUPPORTED_EXTENSIONS.contains(&extension.as_str())
}