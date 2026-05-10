<div align="center">

# smulx-img-deduplicator

**Encuentra, revisa y elimina imágenes duplicadas o visualmente similares desde la terminal.**

[![build](https://github.com/tu-usuario/smulx-img-deduplicator/actions/workflows/ci.yml/badge.svg)](https://github.com/tu-usuario/smulx-img-deduplicator/actions)
[![license: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![rust: 2024 edition](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)

</div>

```
┌─ Grupos ──────────────────┐ ┌─ Archivos — hash a3f0c1d2e4b56789 ─────────────────────────┐
│ ▶ Grupo 1  (3 archivos)   │ │ ▶ [ ] vacaciones_playa.jpg           2.4 MB   d=0           │
│   Grupo 2  (2 archivos)   │ │   [x] vacaciones_playa_copia.jpg     2.4 MB   d=0           │
│   Grupo 3  (4 archivos)   │ │   [ ] vacaciones_playa_edit.jpg      1.9 MB   d=3           │
│   Grupo 4  (2 archivos)   │ │                                                              │
└───────────────────────────┘ └──────────────────────────────────────────────────────────────┘
  4 grupos | 1 archivo marcado | Tab: foco  Espacio: marcar  Enter: eliminar  q: salir
```

---

## ¿Qué es?

`smulx-img-deduplicator` es una herramienta de CLI/TUI escrita enteramente en Rust que analiza directorios de imágenes, agrupa las visualmente similares mediante **hashing perceptual (dHash)**, y ofrece una interfaz de terminal interactiva para revisar cada grupo y decidir qué eliminar.

A diferencia de comparadores byte a byte, detecta como duplicados imágenes que han sido **redimensionadas, recomprimidas, recortadas, o a las que se les ha ajustado el brillo o contraste**.

---

## Características

- **Alto rendimiento** — procesa miles de imágenes en segundos usando todos los núcleos disponibles (Rayon).
- **Similitud visual** — hashing perceptual con agrupamiento transitivo vía Union-Find: si A ≈ B y B ≈ C, los tres van al mismo grupo aunque A y C no sean directamente similares.
- **Seguro por defecto** — mueve los archivos a la papelera del sistema operativo en lugar de borrarlos permanentemente. Nunca permite eliminar la última copia de un grupo.
- **Interactivo** — TUI de dos paneles para navegar grupos, marcar archivos y confirmar eliminaciones sin salir de la terminal.
- **Sin dependencias externas** — un único binario estático, sin runtime, sin Python, sin C++.

---

## Formatos soportados

`JPEG` · `PNG` · `WebP` · `GIF` · `TIFF` · `BMP`

> Los formatos RAW (CR2, NEF, ARW) no están soportados en esta versión.

---

## Instalación

**Requisito:** [Rust stable](https://rustup.rs/) (edición 2024).

### Desde el código fuente

```bash
git clone https://github.com/tu-usuario/smulx-img-deduplicator
cd smulx-img-deduplicator
make install
```

Instala el binario en `~/.local/bin/smulx-dedup`. Para cambiar el destino:

```bash
make install INSTALL_DIR=/usr/local/bin
```

### Con Cargo

```bash
cargo install --path .
```

---

## Uso

```
smulx-dedup <DIRECTORIO>... [OPCIONES]
```

### Ejemplos

```bash
# Analizar ~/Pictures con el umbral recomendado (5)
smulx-dedup ~/Pictures

# Analizar múltiples directorios a la vez
smulx-dedup ~/Pictures ~/Downloads/fotos /mnt/backup

# Solo duplicados exactos
smulx-dedup ~/Pictures --threshold 0

# Detección más agresiva: recortes, filtros, marcas de agua
smulx-dedup ~/Pictures --threshold 10

# Borrado permanente en lugar de papelera
smulx-dedup ~/Pictures --use-trash false

# Exportar grupos a JSON antes de abrir la TUI
smulx-dedup ~/Pictures --export-json grupos.json
```

### Referencia de opciones

| Opción | Por defecto | Descripción |
|---|---|---|
| `--threshold <N>` | `5` | Umbral de similitud en distancia Hamming. `0` = solo exactos. Rango recomendado: 3–8. |
| `--use-trash` | `true` | Envía los archivos eliminados a la papelera del sistema. |
| `--export-json <RUTA>` | — | Exporta la lista de grupos a JSON antes de abrir la TUI. |
| `--log-level <NIVEL>` | `warn` | Verbosidad del log: `error` `warn` `info` `debug` `trace`. Escribe en stderr. |

### Atajos de teclado

| Tecla | Acción |
|---|---|
| `↑` `↓` · `k` `j` | Navegar por la lista con foco |
| `Tab` | Alternar foco entre el panel de grupos y el de archivos |
| `Espacio` | Marcar / desmarcar archivo para eliminación |
| `Enter` · `x` | Eliminar archivos marcados en el grupo actual (solicita confirmación) |
| `v` | Abrir el archivo seleccionado con el visor por defecto del sistema |
| `q` · `Esc` | Salir sin borrar nada |

---

## Sobre el umbral de similitud

El umbral controla cuán diferentes pueden ser dos imágenes para seguir considerándose similares. Se mide en **distancia Hamming**: número de bits distintos entre dos hashes perceptuales de 64 bits.

| Umbral | Detecta |
|---|---|
| `0` | Duplicados exactos únicamente (contenido idéntico bit a bit) |
| `3`–`5` | Redimensionado, recompresión, ajuste de brillo o contraste leve |
| `6`–`10` | Recortes, filtros, marcas de agua, conversiones de formato con pérdida |

Si no sabes por dónde empezar, **`--threshold 5`** es el punto de partida recomendado.

---

## Desarrollo

### Requisitos

- Rust stable (edición 2024) — instalar con [rustup](https://rustup.rs/)
- `make`

### Primeros pasos

```bash
git clone https://github.com/tu-usuario/smulx-img-deduplicator
cd smulx-img-deduplicator

# Pipeline completo: formato + lint + tests
make check

# Ejecutar contra una galería local
make dev GALLERY_PATH=~/Pictures

# Solo los tests
make test
```

### Targets del Makefile

| Target | Descripción |
|---|---|
| `make check` | `fmt` + `lint` + `test` en una sola invocación. Ejecutar antes de cada commit. |
| `make fmt` | Aplica `rustfmt` al código fuente. |
| `make lint` | Ejecuta `clippy` con warnings como errores. |
| `make test` | Ejecuta tests unitarios e integration tests. |
| `make build` | Compila el binario optimizado (`--release`). |
| `make install` | Compila e instala en `~/.local/bin/` (configurable con `INSTALL_DIR`). |
| `make dev` | Ejecuta el proyecto con argumentos de desarrollo (configurable con `GALLERY_PATH`). |
| `make clean` | Elimina artefactos de compilación. |

### Estructura del proyecto

```
src/
├── main.rs          # Punto de entrada y orquestación de fases
├── cli.rs           # Argumentos de línea de comandos (clap)
├── scanner.rs       # Descubrimiento de archivos (jwalk)
├── hasher.rs        # Hashing perceptual paralelo (rayon + img_hash)
├── bktree.rs        # BK-Tree para búsqueda por similitud
├── cluster.rs       # Agrupamiento por componentes conectadas (Union-Find)
├── error.rs         # Tipos de error (thiserror)
└── tui/
    ├── app.rs       # Estado de la aplicación
    ├── ui.rs        # Renderizado (ratatui)
    └── events.rs    # Event loop (crossterm)
tests/
├── integration_scanner.rs    # Descubrimiento de archivos sobre dirs temporales
└── integration_pipeline.rs   # Pipeline completo con imágenes sintéticas
```

### Tests

El proyecto sigue **TDD estricto**: los tests se escriben antes que el código de producción. Cada módulo de lógica de negocio tiene su bloque `#[cfg(test)]` interno; los tests que cruzan módulos o tocan el sistema de archivos viven en `tests/`.

```bash
# Todos los tests
cargo test --all

# Solo un archivo de integration tests
cargo test --test integration_pipeline

# Con output de logs
SMULX_LOG=debug cargo test --all -- --nocapture
```

---

## Contribuir

Las contribuciones son bienvenidas. Por favor:

1. Abre un issue describiendo el bug o la propuesta antes de enviar un PR.
2. Asegúrate de que `make check` pasa sin errores antes de hacer commit.
3. Sigue el ciclo TDD: tests primero, implementación después.

---

## Licencia

`smulx-img-deduplicator` está disponible bajo la licencia [MIT](./LICENSE).
