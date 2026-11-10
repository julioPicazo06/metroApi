//! Capa de persistencia.
//!
//! Los handlers solo hablan con `TransitRepository`. Mañana podemos
//! implementar el mismo trait con GTFS o SQLite y no tocar HTTP.

use crate::domain::{Line, NearbyStation, Station, System};

pub mod json;
pub use json::{JsonRepository, LoadError};

/// Radio por defecto de `GET /stations/nearby` (metros).
pub const DEFAULT_NEARBY_RADIUS_M: f64 = 1000.0;

/// Filtros opcionales de `GET /stations`.
///
/// Un `struct` de query (en vez de 3 parámetros sueltos) se puede
/// extender sin cambiar la firma del trait.
#[derive(Clone, Debug, Default)]
pub struct StationQuery {
    pub system: Option<String>,
    pub line: Option<String>,
    /// Búsqueda por nombre (sin distinguir mayúsculas ni acentos).
    pub q: Option<String>,
}

/// Contrato de lectura. Cero Axum: solo tipos de dominio.
///
/// `Send + Sync` permite meter el repo en `Arc` y compartirlo entre
/// requests de Tokio.
pub trait TransitRepository: Send + Sync {
    fn list_systems(&self) -> Vec<System>;
    fn list_lines(&self) -> Vec<Line>;
    fn get_line(&self, id: &str) -> Option<Line>;
    fn list_stations(&self, query: StationQuery) -> Vec<Station>;
    fn get_station(&self, id: &str) -> Option<Station>;
    fn nearby(&self, lat: f64, lng: f64, radius_m: f64) -> Vec<NearbyStation>;
}
