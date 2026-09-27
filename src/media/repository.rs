use sqlx::SqlitePool;

use crate::metadata::MediaMetadata;

/// Inserts a media file into the database.
///
/// If the file already exists, its metadata is updated instead.
pub async fn upsert_media(
    pool: &SqlitePool,
    metadata: &MediaMetadata,
    path: &str,
    size_bytes: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO media (
            path,
            filename,
            media_type,
            container,
            size_bytes,
            duration_seconds,
            video_codec,
            audio_codec,
            width,
            height
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)

        ON CONFLICT(path) DO UPDATE SET
            filename = excluded.filename,
            media_type = excluded.media_type,
            container = excluded.container,
            size_bytes = excluded.size_bytes,
            duration_seconds = excluded.duration_seconds,
            video_codec = excluded.video_codec,
            audio_codec = excluded.audio_codec,
            width = excluded.width,
            height = excluded.height,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(path)
    .bind(&metadata.filename)
    .bind(&metadata.media_type)
    .bind(&metadata.container)
    .bind(size_bytes)
    .bind(metadata.duration_seconds)
    .bind(&metadata.video_codec)
    .bind(&metadata.audio_codec)
    .bind(metadata.width.map(|value| value as i64))
    .bind(metadata.height.map(|value| value as i64))
    .execute(pool)
    .await?;

    Ok(())
}