use axum::Router;
use axum::routing::get;
use tower_http::cors::CorsLayer;

use super::handlers;
use super::state::AppState;

/// Arma el `Router` de Axum: rutas GET + CORS abierto + estado.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/systems", get(handlers::list_systems))
        .route("/lines", get(handlers::list_lines))
        .route("/lines/{id}", get(handlers::get_line))
        .route("/stations", get(handlers::list_stations))
        // `/stations/nearby` ANTES de `/stations/{id}` para que "nearby"
        // no se interprete como un id.
        .route("/stations/nearby", get(handlers::nearby_stations))
        .route("/stations/{id}", get(handlers::get_station))
        // API pública de portafolio: cualquier origen puede llamarla.
        .layer(CorsLayer::permissive())
        .with_state(state)
}
