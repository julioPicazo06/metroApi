// `Serialize` convierte el struct a JSON de respuesta.
// `Deserialize` lo lee desde data/*.json.
// `Clone` permite copiar el valor al armar respuestas (los datos son chicos).
use serde::{Deserialize, Serialize};

/// Un sistema de transporte (Metro, Metrobús, …).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct System {
    /// Identificador estable en inglés, p. ej. `"metro"`.
    pub id: String,
    /// Nombre para mostrar (contenido en español: es el nombre real).
    pub name: String,
}
