use std::sync::Arc;

use crate::repository::TransitRepository;

/// Estado compartido entre todos los handlers.
///
/// `Clone` es barato: solo clona el `Arc` (un contador), no el catálogo.
#[derive(Clone)]
pub struct AppState {
    /// `dyn` = cualquier tipo que implemente el trait (hoy JSON, mañana GTFS).
    pub repo: Arc<dyn TransitRepository>,
}
