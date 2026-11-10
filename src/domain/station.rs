use serde::{Deserialize, Serialize};

use super::Transfer;

/// Una estación (o parada) de la red.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Station {
    /// Identificador estable, p. ej. `"metro-insurgentes"`.
    pub id: String,
    /// Nombre para mostrar, p. ej. `"Insurgentes"`.
    pub name: String,
    /// Sistemas en los que existe esta estación.
    pub system_ids: Vec<String>,
    /// Líneas que pasan por aquí (aunque el catálogo aún no tenga esa línea completa).
    pub line_ids: Vec<String>,
    /// Latitud WGS84.
    pub lat: f64,
    /// Longitud WGS84.
    pub lng: f64,
    /// Correspondencias hacia otras líneas.
    pub transfers: Vec<Transfer>,
}

/// Estación más un dato calculado: distancia al punto de consulta.
///
/// `flatten` pone los campos de `Station` al mismo nivel que `distance_m`
/// en el JSON, para no anidar `{ "station": { ... } }`.
#[derive(Clone, Debug, Serialize)]
pub struct NearbyStation {
    #[serde(flatten)]
    pub station: Station,
    /// Distancia en metros, redondeada.
    pub distance_m: u32,
}
