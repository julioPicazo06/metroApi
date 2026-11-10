//! Capa HTTP: traduce peticiones GET al repositorio.
//!
//! Aquí sí hay Axum. No hay `std::fs`: los datos ya vienen del repo.

mod handlers;
pub mod routes;
pub mod state;
