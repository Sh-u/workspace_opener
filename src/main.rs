use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::*;
pub mod lib;

use lib::api::run_app;
use serde::{Deserialize, Serialize};
use simplelog::{Config, WriteLogger};
use std::fs::OpenOptions;
use std::{
    collections::VecDeque,
    error::Error,
    fs::{self, read_to_string, File},
    io::{self, BufWriter, Write},
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::line::BOTTOM_LEFT,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
const PATH: &str =
    "C:/Program Files/WindowsApps/Microsoft.WindowsTerminal_1.15.2874.0_x64__8wekyb3d8bbwe/wt.exe";

const CONFIG: &'static str = "config.json";

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
