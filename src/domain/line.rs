use serde::{Deserialize, Serialize};

/// Una línea de un sistema (p. ej. Metro Línea 1).
///
/// El orden de las estaciones NO vive aquí: lo guarda el repositorio
/// para armar `GET /lines/{id}`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Line {
    /// Identificador estable, p. ej. `"metro-1"`.
    pub id: String,
    /// Nombre para mostrar, p. ej. `"Línea 1"`.
    pub name: String,
    /// Color oficial aproximado en hex, p. ej. `"#F94F8E"`.
    pub color: String,
    /// Sistema al que pertenece, p. ej. `"metro"`.
    pub system_id: String,
}
