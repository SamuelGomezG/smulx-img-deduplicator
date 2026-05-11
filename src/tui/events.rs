use crate::tui::app::{App, AppMode};
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

pub fn handle_events(app: &mut App) -> anyhow::Result<bool> {
    if !event::poll(Duration::from_millis(50))? {
        return Ok(false);
    }

    if let Event::Key(key) = event::read()? {
        match &app.mode {
            AppMode::ClusterList => handle_cluster_list(app, key.code),
            AppMode::FileList => handle_file_list(app, key.code),
            AppMode::ConfirmDelete => handle_confirm_delete(app, key.code),
            AppMode::ErrorMessage(_) => {
                app.mode = AppMode::FileList;
            }
        }
    }
    Ok(true)
}

fn handle_cluster_list(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_cluster + 1 < app.clusters.len() {
                app.selected_cluster += 1;
                app.selected_file = 0;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_cluster > 0 {
                app.selected_cluster -= 1;
                app.selected_file = 0;
            }
        }
        KeyCode::Tab | KeyCode::Enter => {
            app.mode = AppMode::FileList;
        }
        _ => {}
    }
}

fn handle_file_list(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Esc | KeyCode::Tab => {
            app.mode = AppMode::ClusterList;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(cluster) = app.current_cluster()
                && app.selected_file + 1 < cluster.files.len()
            {
                app.selected_file += 1;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_file > 0 {
                app.selected_file -= 1;
            }
        }
        KeyCode::Char(' ') => app.toggle_mark(),
        KeyCode::Enter | KeyCode::Char('x') => {
            let count = app
                .current_cluster()
                .map(|c| c.files.iter().filter(|f| f.marked_for_deletion).count())
                .unwrap_or(0);
            if count > 0 {
                app.mode = AppMode::ConfirmDelete;
            } else {
                app.status_message = "No hay archivos marcados para borrar.".to_string();
            }
        }
        KeyCode::Char('v') => {
            if let Some(cluster) = app.current_cluster()
                && let Some(file) = cluster.files.get(app.selected_file)
            {
                if let Err(e) = open::that(&file.path) {
                    app.status_message = format!("Error al abrir imagen: {}", e);
                } else {
                    app.status_message = "Imagen abierta en el visor.".to_string();
                }
            }
        }
        _ => {}
    }
}

fn handle_confirm_delete(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('y') | KeyCode::Enter => {
            let (deleted, errors) = app.execute_deletion();
            if errors.is_empty() {
                app.status_message = format!("{} archivo(s) eliminado(s).", deleted);
            } else {
                let msg = format!("{} eliminado(s). Errores: {}", deleted, errors.join("; "));
                app.mode = AppMode::ErrorMessage(msg.clone());
                app.status_message = msg;
                return;
            }
            app.mode = AppMode::FileList;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.mode = AppMode::FileList;
            app.status_message = "Borrado cancelado.".to_string();
        }
        _ => {}
    }
}
