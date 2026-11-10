//! Composition root: carga JSON, arma `AppState`, hace bind y sirve.
//!
//! Aquí no hay lógica de negocio: solo se ensamblan las capas.

use std::net::SocketAddr;
use std::sync::Arc;

use metro_api::http::routes;
use metro_api::http::state::AppState;
use metro_api::repository::JsonRepository;

/// Bind local por defecto. Al desplegar: `BIND_ADDR=0.0.0.0:3000`.
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3000";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // `CARGO_MANIFEST_DIR` apunta a la raíz del crate, aunque el cwd sea otro.
    let data_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");

    let repo = JsonRepository::load_from_dir(&data_dir)?;
    let state = AppState {
        repo: Arc::new(repo),
    };
    let app = routes::router(state);

    // `let` toma BIND_ADDR del entorno, o el default local.
    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());
    let addr: SocketAddr = bind.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("CDMX Transit API listening on http://{addr}");
    eprintln!("This is NOT an official Mexico City government API.");
    axum::serve(listener, app).await?;
    Ok(())
}
