pub mod lib;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lib::{api::run_app, model::App};
use log::*;
use simplelog::{Config, WriteLogger};
use std::{error::Error, fs::File, io};
use tui::{backend::CrosstermBackend, Terminal};

const PATH: &str =
    "C:/Program Files/WindowsApps/Microsoft.WindowsTerminal_1.15.2874.0_x64__8wekyb3d8bbwe/wt.exe";

fn main() -> Result<(), Box<dyn Error>> {
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("output.log").unwrap(),
    );
    enable_raw_mode()?;

    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend).unwrap();

    let mut app = App::default();

    run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
