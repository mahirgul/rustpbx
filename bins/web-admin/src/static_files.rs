use axum::{
    http::{header, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "ui/"]
pub struct Asset;

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.starts_with("static/") {
        path = path.trim_start_matches("static/").to_string();
    }

    if path.is_empty() || path == "index.html" {
        path = "index.html".to_string();
    }

    match Asset::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            Response::builder()
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(mime.as_ref()).unwrap(),
                )
                .body(axum::body::Body::from(content.data))
                .unwrap()
        }
        None => {
            // SPA Fallback: If path has no file extension (e.g. /extensions), serve index.html
            if !path.contains('.') {
                if let Some(index_content) = Asset::get("index.html") {
                    return Response::builder()
                        .header(header::CONTENT_TYPE, HeaderValue::from_static("text/html"))
                        .body(axum::body::Body::from(index_content.data))
                        .unwrap();
                }
            }
            (StatusCode::NOT_FOUND, "404 Not Found").into_response()
        }
    }
}
