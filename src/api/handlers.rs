use axum::{
    extract::{Path, State}, http::{HeaderMap, StatusCode, header}, response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use sqlx::SqlitePool;
use crate::config::Config;
use crate::media::{
    get_media_list,
    stream_media,
};

/// Health check endpoint.
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Represents a media item returned by the API.
#[derive(Debug, Serialize)]
pub struct MediaResponse {
    pub id: i64,
    pub filename: String,
}

/// Retrieves the list of available media files.
pub async fn get_media_handler(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<MediaResponse>>, StatusCode> {
    let media = get_media_list(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = media
        .into_iter()
        .map(|item| MediaResponse {
            id: item.id,
            filename: item.filename,
        })
        .collect();

    Ok(Json(response))
}

/// Streams a media file.
pub async fn stream_media_handler(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let range = headers
        .get(axum::http::header::RANGE)
        .and_then(|value| value.to_str().ok());

    stream_media(&pool, id, range).await
}

pub async fn get_playlist(
    axum::extract::State(pool): axum::extract::State<sqlx::SqlitePool>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let media = get_media_list(&pool)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut playlist = String::from("#EXTM3U\n");

    let config = Config::from_env()
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    for item in media {
        playlist.push_str(&format!(
        "#EXTINF:-1,{}\nhttp://{}:{}/api/media/{}/stream\n",
        item.filename,
        config.public_host,
        config.port,
        item.id
    ));
}

    Ok((
        [(header::CONTENT_TYPE, "audio/x-mpegurl; charset=utf-8")],
        playlist,
    ))
}