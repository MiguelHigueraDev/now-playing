use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};

use crate::auth::require_bearer_token;
use crate::handlers::{get_now_playing, health, update_now_playing};
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let public = Router::new()
        .route("/health", get(health))
        .route("/api/now-playing", get(get_now_playing));

    let protected = Router::new()
        .route("/api/now-playing", post(update_now_playing))
        .layer(from_fn_with_state(state.clone(), require_bearer_token));

    Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state)
}
