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
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(path)
        .output()?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);

        return Err(format!(
            "ffprobe failed for '{}': {}",
            path.display(),
            error_message
        )
        .into());
    }

    let json = String::from_utf8(output.stdout)?;

    let filename = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();

    parse_ffprobe_output(&json, filename)
}

fn parse_ffprobe_output(
    json: &str,
    filename: String,
) -> Result<MediaMetadata, Box<dyn std::error::Error>> {
    let probe: FfprobeOutput = serde_json::from_str(json)?;

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
                    if media_type == "unknown" {
                        media_type = "audio".to_string();
                    }

                    audio_codec = stream.codec_name.clone();
                }
                _ => {}
            }
        }
    }

    let duration_seconds = probe
        .format
        .as_ref()
        .and_then(|format| format.duration.as_deref())
        .and_then(|duration| duration.parse::<f64>().ok());

    let container = probe
        .format
        .as_ref()
        .and_then(|format| format.format_name.clone());

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

//Test
#[cfg(test)]
mod tests {
    use super::parse_ffprobe_output;

    #[test]
    fn parses_video_metadata() {
        let json = r#"
        {
            "format": {
                "format_name": "matroska,webm",
                "duration": "125.500000"
            },
            "streams": [
                {
                    "codec_type": "video",
                    "codec_name": "h264",
                    "width": 1920,
                    "height": 1080
                },
                {
                    "codec_type": "audio",
                    "codec_name": "aac"
                }
            ]
        }
        "#;

        let metadata =
            parse_ffprobe_output(json, "movie.mkv".to_string()).unwrap();

        assert_eq!(metadata.filename, "movie.mkv");
        assert_eq!(metadata.media_type, "video");
        assert_eq!(metadata.container.as_deref(), Some("matroska,webm"));
        assert_eq!(metadata.video_codec.as_deref(), Some("h264"));
        assert_eq!(metadata.audio_codec.as_deref(), Some("aac"));
        assert_eq!(metadata.width, Some(1920));
        assert_eq!(metadata.height, Some(1080));
        assert_eq!(metadata.duration_seconds, Some(125.5));
    }

    #[test]
    fn parses_audio_only_metadata() {
        let json = r#"
        {
            "format": {
                "format_name": "mp3",
                "duration": "180.250000"
            },
            "streams": [
                {
                    "codec_type": "audio",
                    "codec_name": "mp3"
                }
            ]
        }
        "#;

        let metadata =
            parse_ffprobe_output(json, "song.mp3".to_string()).unwrap();

        assert_eq!(metadata.filename, "song.mp3");
        assert_eq!(metadata.media_type, "audio");
        assert_eq!(metadata.container.as_deref(), Some("mp3"));
        assert_eq!(metadata.audio_codec.as_deref(), Some("mp3"));
        assert_eq!(metadata.video_codec, None);
        assert_eq!(metadata.width, None);
        assert_eq!(metadata.height, None);
        assert_eq!(metadata.duration_seconds, Some(180.25));
    }

    #[test]
    fn handles_missing_duration() {
        let json = r#"
        {
            "format": {
                "format_name": "matroska"
            },
            "streams": [
                {
                    "codec_type": "video",
                    "codec_name": "h264",
                    "width": 1280,
                    "height": 720
                }
            ]
        }
        "#;

        let metadata =
            parse_ffprobe_output(json, "movie.mkv".to_string()).unwrap();

        assert_eq!(metadata.media_type, "video");
        assert_eq!(metadata.duration_seconds, None);
        assert_eq!(metadata.width, Some(1280));
        assert_eq!(metadata.height, Some(720));
    }

    #[test]
    fn rejects_invalid_json() {
        let result =
            parse_ffprobe_output("this is not valid json", "movie.mkv".to_string());

        assert!(result.is_err());
    }
}