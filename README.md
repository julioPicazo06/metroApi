# CDMX Transit API

API **pública de portafolio**, de **solo lectura**, sobre el transporte de la Ciudad de México (Metro y Metrobús).

## Esto no es una API oficial

**No es una API del Gobierno de la Ciudad de México, ni del STC Metro, ni de Metrobús.**  
Es un proyecto personal de portafolio. Los datos de v1 están hardcodeados en JSON a partir de información pública conocida de la red (nombres de estaciones, orden de línea y coordenadas aproximadas extraídas de fuentes abiertas como Wikipedia). Las estaciones casi no cambian, pero **no hay garantía de exactitud ni de actualización oficial**.

Para datos oficiales, consulta [datos.cdmx.gob.mx](https://datos.cdmx.gob.mx) (incluido GTFS).

## Qué hay en v1

- Rust 2024, Axum + Tokio + Serde.
- Persistencia: archivos JSON en `data/` cargados al arrancar (después, inmutables en memoria).
- Arquitectura en capas + patrón Repositorio (`TransitRepository`): los handlers no saben si los datos vienen de JSON.
- Solo Metro **Línea 1 completa** (20 estaciones, Observatorio → Pantitlán), con correspondencias conocidas.
- Metrobús aparece como sistema, sin líneas todavía.
- CORS abierto.
- Bind local: `127.0.0.1:3000` (variable `BIND_ADDR`; al desplegar usa `0.0.0.0:3000`).

No hay POST/PATCH/DELETE. No hay Docker ni deploy en v1.

## Cómo correrla

```bash
cargo run
```

La API queda en `http://127.0.0.1:3000`.

Si el puerto 3000 está ocupado, o al desplegar:

```bash
BIND_ADDR=127.0.0.1:3001 cargo run   # otro puerto local
BIND_ADDR=0.0.0.0:3000 cargo run     # accesible desde fuera de la máquina
```

Colección Postman: `postman/cdmx-transit-api.postman_collection.json`  
Variable: `{{baseUrl}}` = `http://127.0.0.1:3000`

## Endpoints (solo GET)

| Método | Ruta | Notas |
|--------|------|--------|
| GET | `/health` | Liveness |
| GET | `/systems` | Metro y Metrobús |
| GET | `/lines` | Líneas cargadas (v1: solo `metro-1`) |
| GET | `/lines/{id}` | Línea + estaciones **en orden** |
| GET | `/stations` | Query opcional: `system`, `line`, `q` |
| GET | `/stations/{id}` | 404 si no existe |
| GET | `/stations/nearby` | Query: `lat`, `lng`, `radius_m` (default 1000) |

Códigos: **200**, **404** (recurso inexistente), **400** (`lat`/`lng`/`radius_m` inválidos).

`q` busca por nombre (o id) **sin distinguir mayúsculas ni acentos**: `cuauhtemoc` encuentra `Cuauhtémoc`.

## Ejemplos curl

```bash
curl http://127.0.0.1:3000/health

curl http://127.0.0.1:3000/systems

curl http://127.0.0.1:3000/lines

curl http://127.0.0.1:3000/lines/metro-1

curl http://127.0.0.1:3000/stations

curl "http://127.0.0.1:3000/stations?system=metro&line=metro-1"

curl "http://127.0.0.1:3000/stations?q=pino"

curl "http://127.0.0.1:3000/stations?q=cuauhtemoc"

curl http://127.0.0.1:3000/stations/metro-insurgentes

curl "http://127.0.0.1:3000/stations/nearby?lat=19.423292&lng=-99.163177"

curl "http://127.0.0.1:3000/stations/nearby?lat=19.423292&lng=-99.163177&radius_m=800"
```

## Identificadores

Estables, en inglés, pensados para no romper la API al agregar líneas:

- Sistemas: `metro`, `metrobus`
- Líneas: `metro-1`, `metro-2`, … `metro-a`, `metro-b`, `metrobus-1`, …
- Estaciones: `metro-observatorio`, `metro-pino-suarez`, …

`line_ids` y `transfers` de una estación de L1 **ya mencionan** otras líneas (L2, L3, L7, L9, …) aunque esas líneas aún no estén cargadas como catálogo. `GET /lines/metro-7` responde 404 hasta que exista el archivo de esa línea.

## Estructura del código

```
src/
  main.rs              composition root
  domain/              System, Line, Station, Transfer
  repository/          trait TransitRepository + JsonRepository
  http/                AppState, handlers, routes
data/
  metro.json
  metrobus.json
```

## Roadmap

- Resto de líneas del Metro (2–12, A, B).
- Líneas de Metrobús.
- Importar GTFS de [datos.cdmx.gob.mx](https://datos.cdmx.gob.mx) detrás del mismo `TransitRepository`.
- SQLite (o similar) cuando el JSON se quede corto.
- Despliegue público (bind `0.0.0.0`) cuando se pida.

## Licencia

MIT. Los nombres de estaciones y la red de transporte son del STC / Metrobús / Gobierno de la CDMX; este repo no reclama esa autoría.
# metroApi
