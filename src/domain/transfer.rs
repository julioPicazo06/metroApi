use serde::{Deserialize, Serialize};

/// Correspondencia hacia otra línea (mismo u otro sistema).
///
/// No incluye la línea “actual”: solo destinos a los que se puede cambiar.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transfer {
    /// Sistema destino, p. ej. `"metro"` o `"metrobus"`.
    pub system_id: String,
    /// Línea destino, p. ej. `"metro-7"`.
    pub line_id: String,
}
