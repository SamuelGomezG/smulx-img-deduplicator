use crate::tui::app::{App, AppMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(frame.area());

    let main_area = chunks[0];
    let status_area = chunks[1];

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
        .split(main_area);

    render_cluster_list(frame, app, columns[0]);
    render_file_list(frame, app, columns[1]);
    render_status_bar(frame, app, status_area);

    match &app.mode {
        AppMode::ConfirmDelete => render_confirm_modal(frame, app),
        AppMode::ErrorMessage(msg) => render_error_modal(frame, msg),
        _ => {}
    }
}

fn render_cluster_list(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = matches!(app.mode, AppMode::ClusterList);
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .clusters
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let marked = c.files.iter().filter(|f| f.marked_for_deletion).count();
            let label = if marked > 0 {
                format!(
                    "Grupo {} ({} archivos, {} marcados)",
                    i + 1,
                    c.files.len(),
                    marked
                )
            } else {
                format!("Grupo {} ({} archivos)", i + 1, c.files.len())
            };
            ListItem::new(label)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected_cluster));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Grupos ")
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_file_list(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = matches!(app.mode, AppMode::FileList | AppMode::ConfirmDelete);
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .current_cluster()
        .map(|cluster| {
            cluster
                .files
                .iter()
                .map(|f| {
                    let checkbox = if f.marked_for_deletion { "[x]" } else { "[ ]" };
                    let size_mb = f.size_bytes as f64 / 1_048_576.0;
                    let name = f
                        .path
                        .file_name()
                        .and_then(|n: &std::ffi::OsStr| n.to_str())
                        .unwrap_or("?");
                    let line = format!(
                        "{} {:<40} {:>6.1} MB  d={}",
                        checkbox, name, size_mb, f.distance
                    );
                    let style = if f.marked_for_deletion {
                        Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::CROSSED_OUT)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Line::from(Span::styled(line, style)))
                })
                .collect()
        })
        .unwrap_or_default();

    let mut state = ListState::default();
    state.select(Some(app.selected_file));

    let title = app
        .current_cluster()
        .map(|c| format!(" Archivos - hash {:016x} ", c.root_hash))
        .unwrap_or_else(|| " Archivos ".to_string());

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let total_clusters = app.clusters.len();
    let total_marked: usize = app
        .clusters
        .iter()
        .flat_map(|c| &c.files)
        .filter(|f| f.marked_for_deletion)
        .count();

    let text = format!(
        " {} grupos | {} archivos marcados | {}",
        total_clusters, total_marked, app.status_message
    );

    let paragraph =
        Paragraph::new(text).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(paragraph, area);
}

fn render_confirm_modal(frame: &mut Frame, app: &App) {
    let count = app
        .current_cluster()
        .map(|c| c.files.iter().filter(|f| f.marked_for_deletion).count())
        .unwrap_or(0);
    let action = if app.use_trash {
        "mover a papelera"
    } else {
        "ELIMINAR PERMANENTEMENTE"
    };
    let msg = format!(
        "Confirmas {} {} archivo(s)?\n\n  [Y] Si    [N] / Esc = Cancelar",
        action, count
    );
    render_centered_modal(frame, " Confirmar ", &msg, Color::Yellow);
}

fn render_error_modal(frame: &mut Frame, msg: &str) {
    render_centered_modal(frame, " Error ", msg, Color::Red);
}

fn render_centered_modal(frame: &mut Frame, title: &str, msg: &str, color: Color) {
    let area = centered_rect(60, 30, frame.area());
    frame.render_widget(Clear, area);
    let block = Paragraph::new(msg).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color)),
    );
    frame.render_widget(block, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
