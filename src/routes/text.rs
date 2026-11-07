use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

use crate::server::{AppState, Payload};

const TEXT_TEMPLATE: &str = include_str!("../templates/text.html");

/// Basic HTML entity escaping to protect against XSS when rendering arbitrary text.
#[must_use]
pub fn escape_html(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            other => escaped.push(other),
        }
    }
    escaped
}

/// Handler for serving the text snippet viewing page.
pub async fn page_handler(State(state): State<AppState>) -> Response {
    let text = match &state.payload {
        Payload::Text { content } => content,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    let escaped_content = escape_html(text);
    let html = TEXT_TEMPLATE.replace("{{CONTENT}}", &escaped_content);

    let notify = state.shutdown_tx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        if let Some(tx) = notify {
            let _ = tx.send(()).await;
        }
    });

    Html(html).into_response()
}
