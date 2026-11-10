//! Repositorio respaldado por JSON: lee `data/*.json` al arrancar
//! y después sirve todo desde memoria (inmutable).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::{StationQuery, TransitRepository};
use crate::domain::{Line, NearbyStation, Station, System};

/// Forma de cada archivo en `data/` (metro.json, metrobus.json).
///
/// `station_ids` en cada línea define el orden de las estaciones;
/// las estaciones en sí viven en `stations` para no duplicarlas.
#[derive(Debug, Deserialize)]
struct SystemFile {
    system: System,
    lines: Vec<LineRecord>,
    stations: Vec<Station>,
}

/// Línea tal como viene en el JSON, con el orden de estaciones.
#[derive(Debug, Deserialize)]
struct LineRecord {
    id: String,
    name: String,
    color: String,
    system_id: String,
    station_ids: Vec<String>,
}

/// Error al cargar o validar los JSON al arranque.
#[derive(Debug)]
pub enum LoadError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    Invalid(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io { path, source } => {
                write!(f, "cannot read {}: {source}", path.display())
            }
            LoadError::Parse { path, source } => {
                write!(f, "cannot parse {}: {source}", path.display())
            }
            LoadError::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadError::Io { source, .. } => Some(source),
            LoadError::Parse { source, .. } => Some(source),
            LoadError::Invalid(_) => None,
        }
    }
}

/// Catálogo en memoria. Después de `load_from_dir` no se muta.
pub struct JsonRepository {
    systems: Vec<System>,
    lines: Vec<Line>,
    stations: Vec<Station>,
    lines_by_id: HashMap<String, Line>,
    stations_by_id: HashMap<String, Station>,
    /// Orden de estaciones por línea (ids), copiado de `station_ids`.
    line_station_ids: HashMap<String, Vec<String>>,
}

impl JsonRepository {
    /// Lee los JSON de un directorio y arma índices en memoria.
    pub fn load_from_dir(dir: &Path) -> Result<Self, LoadError> {
        // Orden fijo: así GET /systems siempre lista Metro y luego Metrobús.
        let files = ["metro.json", "metrobus.json"];

        let mut systems = Vec::new();
        let mut lines = Vec::new();
        let mut stations = Vec::new();
        let mut line_station_ids = HashMap::new();

        for file_name in files {
            let path = dir.join(file_name);
            let raw = fs::read_to_string(&path).map_err(|source| LoadError::Io {
                path: path.clone(),
                source,
            })?;
            let parsed: SystemFile =
                serde_json::from_str(&raw).map_err(|source| LoadError::Parse {
                    path: path.clone(),
                    source,
                })?;

            for record in parsed.lines {
                if record.system_id != parsed.system.id {
                    return Err(LoadError::Invalid(format!(
                        "line {} has system_id {} but file system is {}",
                        record.id, record.system_id, parsed.system.id
                    )));
                }
                line_station_ids.insert(record.id.clone(), record.station_ids);
                lines.push(Line {
                    id: record.id,
                    name: record.name,
                    color: record.color,
                    system_id: record.system_id,
                });
            }

            stations.extend(parsed.stations);
            systems.push(parsed.system);
        }

        let lines_by_id = lines
            .iter()
            .map(|line| (line.id.clone(), line.clone()))
            .collect::<HashMap<_, _>>();

        let mut stations_by_id = HashMap::new();
        for station in &stations {
            if stations_by_id
                .insert(station.id.clone(), station.clone())
                .is_some()
            {
                return Err(LoadError::Invalid(format!(
                    "duplicate station id: {}",
                    station.id
                )));
            }
        }

        // Cada station_id de una línea debe existir en el catálogo.
        for (line_id, ids) in &line_station_ids {
            for station_id in ids {
                if !stations_by_id.contains_key(station_id) {
                    return Err(LoadError::Invalid(format!(
                        "line {line_id} references unknown station {station_id}"
                    )));
                }
            }
        }

        Ok(Self {
            systems,
            lines,
            stations,
            lines_by_id,
            stations_by_id,
            line_station_ids,
        })
    }
}

impl TransitRepository for JsonRepository {
    fn list_systems(&self) -> Vec<System> {
        self.systems.clone()
    }

    fn list_lines(&self) -> Vec<Line> {
        self.lines.clone()
    }

    fn get_line(&self, id: &str) -> Option<Line> {
        self.lines_by_id.get(id).cloned()
    }

    fn list_stations(&self, query: StationQuery) -> Vec<Station> {
        // Si filtran por línea y esa línea tiene orden, respetamos el orden.
        let mut result = if let Some(line_id) = &query.line {
            if let Some(ids) = self.line_station_ids.get(line_id) {
                ids.iter()
                    .filter_map(|id| self.stations_by_id.get(id).cloned())
                    .collect()
            } else {
                self.stations
                    .iter()
                    .filter(|station| station.line_ids.iter().any(|id| id == line_id))
                    .cloned()
                    .collect()
            }
        } else {
            self.stations.clone()
        };

        if let Some(system) = &query.system {
            result.retain(|station| station.system_ids.iter().any(|id| id == system));
        }

        if let Some(q) = &query.q {
            let needle = normalize(q);
            if !needle.is_empty() {
                result.retain(|station| {
                    normalize(&station.name).contains(&needle)
                        || normalize(&station.id).contains(&needle)
                });
            }
        }

        result
    }

    fn get_station(&self, id: &str) -> Option<Station> {
        self.stations_by_id.get(id).cloned()
    }

    fn nearby(&self, lat: f64, lng: f64, radius_m: f64) -> Vec<NearbyStation> {
        let mut found = Vec::new();
        for station in &self.stations {
            let distance = haversine_m(lat, lng, station.lat, station.lng);
            if distance <= radius_m {
                found.push(NearbyStation {
                    station: station.clone(),
                    distance_m: distance.round() as u32,
                });
            }
        }
        found.sort_by_key(|item| item.distance_m);
        found
    }
}

/// Quita acentos y pasa a minúsculas para buscar `"cuauhtemoc"` ≈ `"Cuauhtémoc"`.
fn normalize(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' | 'Á' | 'À' | 'Ä' | 'Â' => 'a',
            'é' | 'è' | 'ë' | 'ê' | 'É' | 'È' | 'Ë' | 'Ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' | 'Í' | 'Ì' | 'Ï' | 'Î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' | 'Ó' | 'Ò' | 'Ö' | 'Ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' | 'Ú' | 'Ù' | 'Ü' | 'Û' => 'u',
            'ñ' | 'Ñ' => 'n',
            _ => c,
        })
        .collect::<String>()
        .to_lowercase()
}

/// Distancia en metros entre dos puntos WGS84 (fórmula de haversine).
fn haversine_m(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    const EARTH_RADIUS_M: f64 = 6_371_000.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlng = (lng2 - lng1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlng / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_M * a.sqrt().asin()
}
