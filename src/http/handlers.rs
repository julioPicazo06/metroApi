//! Handlers: solo GET. Traducen HTTP ↔ repositorio.

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use super::state::AppState;
use crate::domain::{Line, NearbyStation, Station, System};
use crate::repository::{DEFAULT_NEARBY_RADIUS_M, StationQuery};

/// Cuerpo JSON de error, siempre con la misma forma.
#[derive(Serialize)]
pub(crate) struct ErrorBody {
    error: String,
}

/// Errores HTTP que devolvemos de forma controlada.
pub(crate) enum ApiError {
    NotFound(String),
    BadRequest(String),
}

// Convierte nuestro error en respuesta HTTP (status + JSON).
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
        };
        (status, Json(ErrorBody { error: message })).into_response()
    }
}

#[derive(Serialize)]
pub(crate) struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
pub(crate) struct SystemsResponse {
    systems: Vec<System>,
}

#[derive(Serialize)]
pub(crate) struct LinesResponse {
    lines: Vec<Line>,
}

#[derive(Serialize)]
pub(crate) struct StationsResponse {
    stations: Vec<Station>,
}

/// Línea más sus estaciones en orden (solo `GET /lines/{id}`).
#[derive(Serialize)]
pub(crate) struct LineWithStations {
    #[serde(flatten)]
    line: Line,
    stations: Vec<Station>,
}

#[derive(Serialize)]
pub(crate) struct NearbyResponse {
    radius_m: u32,
    stations: Vec<NearbyStation>,
}

/// Query de `GET /stations`. Todos los campos son opcionales.
#[derive(Deserialize)]
pub(crate) struct ListStationsParams {
    system: Option<String>,
    line: Option<String>,
    q: Option<String>,
}

/// Query de nearby: strings para poder devolver 400 JSON si no son números.
#[derive(Deserialize)]
pub(crate) struct NearbyParams {
    lat: Option<String>,
    lng: Option<String>,
    radius_m: Option<String>,
}

/// `GET /health` — liveness, no toca el repositorio.
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

/// `GET /systems`
///
/// `State` inyecta el `AppState` compartido (el extractor de Axum).
pub async fn list_systems(State(state): State<AppState>) -> Json<SystemsResponse> {
    // `let` guarda el resultado del repo antes de envolverlo en JSON.
    let systems = state.repo.list_systems();
    Json(SystemsResponse { systems })
}

/// `GET /lines`
pub async fn list_lines(State(state): State<AppState>) -> Json<LinesResponse> {
    let lines = state.repo.list_lines();
    Json(LinesResponse { lines })
}

/// `GET /lines/{id}` — línea + estaciones ordenadas. 404 si no existe.
///
/// `Path` extrae `{id}` de la URL.
pub async fn get_line(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<LineWithStations>, ApiError> {
    let line = state
        .repo
        .get_line(&id)
        .ok_or_else(|| ApiError::NotFound(format!("line not found: {id}")))?;
    let stations = state.repo.list_stations(StationQuery {
        line: Some(id),
        ..StationQuery::default()
    });
    Ok(Json(LineWithStations { line, stations }))
}

/// `GET /stations?system=&line=&q=`
///
/// `Query` parsea la query string a un struct.
pub async fn list_stations(
    State(state): State<AppState>,
    Query(params): Query<ListStationsParams>,
) -> Json<StationsResponse> {
    let stations = state.repo.list_stations(StationQuery {
        system: empty_to_none(params.system),
        line: empty_to_none(params.line),
        q: empty_to_none(params.q),
    });
    Json(StationsResponse { stations })
}

/// `GET /stations/{id}` — 404 si no existe.
pub async fn get_station(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Station>, ApiError> {
    let station = state
        .repo
        .get_station(&id)
        .ok_or_else(|| ApiError::NotFound(format!("station not found: {id}")))?;
    Ok(Json(station))
}

/// `GET /stations/nearby?lat=&lng=&radius_m=`
///
/// Radio por defecto: 1000 m. 400 si lat/lng no son números válidos.
pub async fn nearby_stations(
    State(state): State<AppState>,
    Query(params): Query<NearbyParams>,
) -> Result<Json<NearbyResponse>, ApiError> {
    let lat = parse_coord(params.lat.as_deref(), "lat", -90.0, 90.0)?;
    let lng = parse_coord(params.lng.as_deref(), "lng", -180.0, 180.0)?;
    let radius_m = match params.radius_m.as_deref() {
        None | Some("") => DEFAULT_NEARBY_RADIUS_M,
        Some(raw) => parse_radius(raw)?,
    };

    let stations = state.repo.nearby(lat, lng, radius_m);
    Ok(Json(NearbyResponse {
        radius_m: radius_m.round() as u32,
        stations,
    }))
}

fn empty_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn parse_coord(raw: Option<&str>, field: &str, min: f64, max: f64) -> Result<f64, ApiError> {
    let Some(text) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return Err(ApiError::BadRequest(format!("{field} is required")));
    };
    let value: f64 = text
        .parse()
        .map_err(|_| ApiError::BadRequest(format!("{field} must be a number")))?;
    if !value.is_finite() || value < min || value > max {
        return Err(ApiError::BadRequest(format!(
            "{field} must be between {min} and {max}"
        )));
    }
    Ok(value)
}

fn parse_radius(raw: &str) -> Result<f64, ApiError> {
    let value: f64 = raw
        .trim()
        .parse()
        .map_err(|_| ApiError::BadRequest("radius_m must be a number".to_string()))?;
    if !value.is_finite() || value < 0.0 {
        return Err(ApiError::BadRequest(
            "radius_m must be a number greater than or equal to 0".to_string(),
        ));
    }
    Ok(value)
}
