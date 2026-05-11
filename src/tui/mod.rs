/// Terminal UI module.
///
/// Provides the interactive TUI for reviewing and managing duplicate image
/// clusters. The module is split into three submodules:
/// - `app`: Global application state (`App`, `AppMode`)
/// - `ui`: Pure rendering functions (via `ratatui`)
/// - `events`: Keyboard event loop (via `crossterm`)
pub mod app;
pub mod events;
pub mod ui;
