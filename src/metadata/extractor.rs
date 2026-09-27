use serde::Deserialize;
use std::path::Path;
use std::process::Command;

/// Metadata extracted from a media file.
/// It is intentionally independent from the database.
#[derive(Debug)]
pub struct MediaMetadata {
    pub filename: String,
    pub container: Option<String>,
    pub media_type: String,
    pub duration_seconds: Option<f64>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Structure used to deserialize the JSON returned by ffprobe.
///
/// We only define the fields we currently need.
/// ffprobe returns many more fields, but we don't need all of them.
#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    format: Option<FfprobeFormat>,
    streams: Option<Vec<FfprobeStream>>,
}

/// Information about the media container.
#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    format_name: Option<String>,
    duration: Option<String>,
}

/// Information about an individual stream.
///
/// A video file might contain:
/// - one video stream
/// - one audio stream
///
/// An audio file may only contain an audio stream.
#[derive(Debug, Deserialize)]
struct FfprobeStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

/// Extract metadata from a media file using ffprobe.
pub fn extract_metadata(path: &Path) -> Result<MediaMetadata, Box<dyn std::error::Error>> {
    // Run ffprobe as an external process.
    //
    // Example command:
    //
    // ffprobe -v quiet -print_format json -show_format -show_streams file.mp4
    //
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(path)
        .output()?;

    // ffprobe returns a non-zero exit code when it cannot analyze
    // the file.
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);

        return Err(format!(
            "ffprobe failed for '{}': {}",
            path.display(),
            error_message
        )
        .into());
    }

    // Convert ffprobe's stdout from bytes into a String.
    let json = String::from_utf8(output.stdout)?;

    // Deserialize the JSON into our Rust structure.
    let probe: FfprobeOutput = serde_json::from_str(&json)?;

    // Determine the media type and extract codec information.
    let mut media_type = "unknown".to_string();
    let mut video_codec = None;
    let mut audio_codec = None;
    let mut width = None;
    let mut height = None;

    if let Some(streams) = &probe.streams {
        for stream in streams {
            match stream.codec_type.as_deref() {
                Some("video") => {
                    media_type = "video".to_string();

                    video_codec = stream.codec_name.clone();
                    width = stream.width;
                    height = stream.height;
                }

                Some("audio") => {
                    // If there is no video stream, this is an audio file.
                    if media_type == "unknown" {
                        media_type = "audio".to_string();
                    }

                    audio_codec = stream.codec_name.clone();
                }

                _ => {}
            }
        }
    }

    // ffprobe reports duration as a String.
    //
    // Example:
    //
    // "3600.123456"
    //
    // Convert it into f64 for easier use inside application.
    let duration_seconds = probe
        .format
        .as_ref()
        .and_then(|format| format.duration.as_deref())
        .and_then(|duration| duration.parse::<f64>().ok());

    // Extract the container format.
    //
    // ffprobe may return values such as:
    //
    // "matroska,webm"
    // "mov,mp4,m4a,3gp,3g2,mj2"
    //
    let container = probe
        .format
        .as_ref()
        .and_then(|format| format.format_name.clone());

    // Extract just the filename from the complete path.
    let filename = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(MediaMetadata {
        filename,
        container,
        media_type,
        duration_seconds,
        video_codec,
        audio_codec,
        width,
        height,
    })
}