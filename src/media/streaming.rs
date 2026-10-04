use axum::{
    body::Body,
    http::{
        header,
        HeaderMap,
        HeaderValue,
        StatusCode,
    },
    response::Response,
};

use std::path::Path;

use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncReadExt, SeekFrom},
};

use tokio_util::io::ReaderStream;

use crate::media::get_media_path;
use sqlx::SqlitePool;

/// Streams a media file to an HTTP client.
///
/// This implementation supports:
///
/// - Full file requests
/// - HTTP Range requests
/// - Seeking
///
/// Range requests are important for media players because they allow
/// the client to request only a portion of a large file.
pub async fn stream_media(
    pool: &SqlitePool,
    id: i64,
    range_header: Option<&str>,
) -> Result<Response, StatusCode> {
    // Find the file associated with the media ID.
    let path = get_media_path(pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let path = Path::new(&path);

    // Open the media file asynchronously.
    let mut file = File::open(path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Get the file size.
    let file_size = file
        .metadata()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .len();

    // Empty files cannot be streamed meaningfully.
    if file_size == 0 {
        return Err(StatusCode::NO_CONTENT);
    }

    // Determine the MIME type from the file extension.
    let content_type = content_type_for_file(path);

    // ---------------------------------------------------------
    // Full file request
    // ---------------------------------------------------------
    //
    // If the client did not send a Range header, return the
    // complete file with HTTP 200 OK.
    if range_header.is_none() {
        let stream = ReaderStream::new(file);

        let mut headers = HeaderMap::new();

        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(content_type),
        );

        headers.insert(
            header::CONTENT_LENGTH,
            HeaderValue::from(file_size),
        );

        headers.insert(
            header::ACCEPT_RANGES,
            HeaderValue::from_static("bytes"),
        );

        let mut response = Response::new(Body::from_stream(stream));

        *response.status_mut() = StatusCode::OK;
        *response.headers_mut() = headers;

        return Ok(response);
    }

    // ---------------------------------------------------------
    // Range request
    // ---------------------------------------------------------

    // At this point we know the Range header exists.
    let range_header = range_header.unwrap();

    let (start, end) = parse_range(range_header, file_size)
        .ok_or(StatusCode::RANGE_NOT_SATISFIABLE)?;

    // Calculate how many bytes we need to send.
    //
    // Example:
    //
    // start = 1000
    // end   = 1999
    //
    // length = 1000 bytes
    let content_length = end - start + 1;

    // Move the file cursor to the requested starting position.
    file.seek(SeekFrom::Start(start))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // `take` limits the stream to exactly the requested number
    // of bytes.
    let stream = ReaderStream::new(file.take(content_length));

    let mut headers = HeaderMap::new();

    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(content_type),
    );

    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from(content_length),
    );

    // Example:
    //
    // Content-Range: bytes 1000-1999/5000000
    //
    let content_range = format!(
        "bytes {}-{}/{}",
        start,
        end,
        file_size
    );

    headers.insert(
        header::CONTENT_RANGE,
        HeaderValue::try_from(content_range)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );

    headers.insert(
        header::ACCEPT_RANGES,
        HeaderValue::from_static("bytes"),
    );

    let mut response = Response::new(Body::from_stream(stream));

    *response.status_mut() = StatusCode::PARTIAL_CONTENT;
    *response.headers_mut() = headers;

    Ok(response)
}

/// Parses a single HTTP byte range.
///
/// Supported examples:
///
/// bytes=0-999
/// bytes=1000-
///
/// Returns:
///
/// (start, end)
fn parse_range(
    range_header: &str,
    file_size: u64,
) -> Option<(u64, u64)> {
    // We currently support only the standard "bytes=" format.
    let range = range_header.strip_prefix("bytes=")?;

    // V1 supports a single range.
    //
    // We intentionally don't handle:
    //
    // bytes=0-999,2000-2999
    //
    // yet.
    let range = range.split(',').next()?;

    let (start, end) = range.split_once('-')?;

    let start = start.parse::<u64>().ok()?;

    // `bytes=1000-` means:
    //
    // Start at byte 1000 and continue to the end.
    if end.is_empty() {
        if start >= file_size {
            return None;
        }

        return Some((start, file_size - 1));
    }

    let end = end.parse::<u64>().ok()?;

    // Invalid range.
    if start > end || start >= file_size {
        return None;
    }

    // A client may request an end position beyond the file.
    //
    // Example:
    //
    // bytes=1000-999999999
    //
    // We simply cap it at the last byte.
    let end = end.min(file_size - 1);

    Some((start, end))
}

/// Returns the MIME type used by HTTP for a media file.
fn content_type_for_file(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_lowercase())
        .as_deref()
    {
        Some("mp4") => "video/mp4",
        Some("mkv") => "video/x-matroska",
        Some("avi") => "video/x-msvideo",
        Some("mov") => "video/quicktime",
        Some("webm") => "video/webm",

        Some("mp3") => "audio/mpeg",
        Some("flac") => "audio/flac",
        Some("wav") => "audio/wav",
        Some("m4a") => "audio/mp4",

        _ => "application/octet-stream",
    }
}


//Testing here for now
#[cfg(test)]
mod tests {
    use super::parse_range;

    #[test]
    fn parses_explicit_range() {
        let result = parse_range("bytes=0-999", 10_000);

        assert_eq!(result, Some((0, 999)));
    }

    #[test]
    fn parses_range_to_end_of_file() {
        let result = parse_range("bytes=500-", 10_000);

        assert_eq!(result, Some((500, 9_999)));
    }

    #[test]
    fn limits_end_to_file_size() {
        let result = parse_range("bytes=9000-20000", 10_000);

        assert_eq!(result, Some((9000, 9_999)));
    }

    #[test]
    fn rejects_range_starting_after_file() {
        let result = parse_range("bytes=10000-11000", 10_000);

        assert_eq!(result, None);
    }

    #[test]
    fn rejects_invalid_range() {
        let result = parse_range("bytes=900-500", 10_000);

        assert_eq!(result, None);
    }

    #[test]
    fn rejects_non_byte_range() {
        let result = parse_range("items=0-999", 10_000);

        assert_eq!(result, None);
    }
}