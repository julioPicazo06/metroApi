//! Capa de dominio: el vocabulario del transporte.
//!
//! Aquí no hay Axum ni `std::fs`. Si mañana cambiamos JSON por SQLite,
//! estos tipos siguen igual.

// Reexportamos para poder escribir `domain::Station` en vez de
// `domain::station::Station`.
mod line;
mod station;
mod system;
mod transfer;

pub use line::Line;
pub use station::{NearbyStation, Station};
pub use system::System;
pub use transfer::Transfer;
