use axum::{
    http::{header, Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::state::AppState;

/// Validates `Authorization: Bearer <token>` for protected routes.
pub async fn require_bearer_token(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let Some(header_value) = auth_header else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let expected = format!("Bearer {}", state.auth_token);
    if header_value != expected {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}
