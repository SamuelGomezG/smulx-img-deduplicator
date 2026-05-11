mod cli;
mod tui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use tracing_subscriber::EnvFilter;

struct TuiGuard;

impl TuiGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        if let Err(e) = execute!(io::stdout(), EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(e.into());
        }
        Ok(TuiGuard)
    }
}

impl Drop for TuiGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cli.log_level))
        .with_writer(std::io::stderr)
        .init();

    eprintln!("Scanning directories...");
    let files = smulx_img_deduplicator::scanner::discover_images(&cli.paths);
    eprintln!("   {} images found.", files.len());

    if files.is_empty() {
        eprintln!("No images found. Check the paths.");
        return Ok(());
    }

    eprintln!("Computing hashes (this may take a while)...");
    let records = smulx_img_deduplicator::hasher::hash_all(&files);
    eprintln!(
        "   {} images processed ({} failed).",
        records.len(),
        files.len() - records.len()
    );

    eprintln!("Building clusters (threshold={})...", cli.threshold);
    let clusters = smulx_img_deduplicator::cluster::build_clusters(&records, cli.threshold);
    eprintln!("   {} similarity groups found.", clusters.len());

    if clusters.is_empty() {
        eprintln!("No similar images found with the current threshold.");
        return Ok(());
    }

    if let Some(json_path) = &cli.export_json {
        eprintln!("Exporting JSON to {:?}...", json_path);
        let file = std::fs::File::create(json_path)?;
        serde_json::to_writer_pretty(file, &clusters)?;
    }

    let _guard = TuiGuard::enter()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = tui::app::App::new(clusters, cli.use_trash);

    let result = run_tui(&mut terminal, &mut app);

    let _ = terminal.show_cursor();

    result
}

fn run_tui<B>(terminal: &mut Terminal<B>, app: &mut tui::app::App) -> Result<()>
where
    B: ratatui::backend::Backend,
    <B as ratatui::backend::Backend>::Error: Send + Sync + 'static,
{
    loop {
        terminal.draw(|f| tui::ui::render(f, app))?;

        tui::events::handle_events(app)?;

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
