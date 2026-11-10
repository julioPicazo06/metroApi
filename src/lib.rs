//! Biblioteca de la API de portafolio CDMX Transit.
//!
//! `main.rs` solo ensambla piezas; la lógica vive aquí, en capas:
//! - `domain`: tipos del negocio (cero HTTP, cero archivos).
//! - `repository`: cómo se leen los datos (hoy JSON, mañana GTFS/SQLite).
//! - `http`: Axum (estado, handlers, rutas).

// `pub mod` hace visible el módulo a `main.rs` y a tests.
pub mod domain;
pub mod http;
pub mod repository;
