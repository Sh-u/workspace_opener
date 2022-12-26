use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::*;

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

#[derive(Debug, Copy, Clone, PartialEq)]
enum State {
    Start,
    ChoosePreset,
    CreatePreset,
}
struct Popup {
    active: bool,
    message: String,
}

impl Popup {
    fn default() -> Self {
        Popup {
            active: false,
            message: String::new(),
        }
    }

    fn activate_popup(&mut self, message: &str) {
        self.active = true;
        self.message = message.to_string();
    }

    fn deactivate_popup(&mut self) {
        self.active = false;
    }
}

struct StatefulList {
    list_state: ListState,
    items: Vec<(String, State)>,
}
#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    presets: Vec<Preset>,
}
#[derive(Serialize, Deserialize, Debug)]
struct Preset {
    name: String,
    terminal_path: String,
    windows: u8,
    args: Vec<String>,
}

impl Preset {
    fn new(input: &Vec<String>) -> Self {
        Preset {
            name: input.get(0).unwrap().to_string(),
            terminal_path: input.get(1).unwrap().to_string(),
            windows: input.get(2).unwrap().parse::<u8>().unwrap(),
            args: input
                .iter()
                .skip(3)
                .map(|arg| arg.to_string())
                .collect::<Vec<String>>(),
        }
    }
}

impl StatefulList {
    fn with_items(items: Vec<(String, State)>) -> Self {
        let index = match items.len() {
            0 => None,
            _ => Some(0),
        };

        let mut list = StatefulList {
            list_state: ListState::default(),
            items,
        };

        list.list_state.select(index);
        list
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    self.items.len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.list_state.select(Some(i))
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.list_state.select(Some(i))
    }

    fn get_selected_item(&mut self) -> Option<(String, State)> {
        match self.list_state.selected() {
            Some(i) => match self.items.get(i) {
                Some(i) => Some(i.to_owned()),
                None => None,
            },
            None => None,
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, rect: Rect) -> Rect {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(rect);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(layout[1])[1]
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(10)].as_ref())
        .split(f.size());

    let main_block = Block::default().title("Main").borders(Borders::ALL);

    let input_block = Block::default().title("Input").borders(Borders::ALL);

    if app.popup.active {
        let popup_block = Block::default().title("Popup").borders(Borders::ALL);
        let area = centered_rect(60, 20, size);
        let popup_message = Paragraph::new(Span::from(app.popup.message.to_string()));
        f.render_widget(popup_message.block(popup_block), area);
    }

    if app.items.items.is_empty() {
        let mut prompts: Vec<ListItem> = Vec::new();

        for (index, prompt) in app.prompts.iter().take(app.messages.len() + 1).enumerate() {
            prompts.push(ListItem::new(Span::from(prompt.as_str())));
            if let Some(msg) = app.messages.get(index) {
                prompts.push(
                    ListItem::new(Span::from(msg.as_str())).style(
                        Style::default()
                            .fg(Color::LightRed)
                            .add_modifier(Modifier::ITALIC),
                    ),
                )
            }
        }

        let prompts = List::new(prompts).block(main_block.clone());

        let input_block = input_block.style(Style::default().fg(Color::LightRed));

        let user_input = Paragraph::new(Text::from(app.input.as_str()));

        f.render_widget(prompts, chunks[0]);
        f.render_widget(user_input.block(input_block), chunks[1]);
        f.set_cursor(chunks[1].x + app.input.len() as u16 + 1, chunks[1].y + 1);
    } else {
        let items = app
            .items
            .items
            .iter()
            .map(|item| {
                let _lines = vec![Spans::from(item.0.as_str())];
                ListItem::new(_lines).style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
            })
            .collect::<Vec<ListItem>>();

        let items = List::new(items)
            .block(main_block)
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("> ");

        f.render_stateful_widget(items, chunks[0], &mut app.items.list_state);
        f.render_widget(input_block, chunks[1]);
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    let cfg_file_string = fs::read_to_string(CONFIG).unwrap();
    let mut app_config: AppConfig = serde_json::from_str(&cfg_file_string).unwrap();

    loop {
        terminal.draw(|f| ui(f, app)).unwrap();

        let event = event::read().unwrap();

        if let Event::Key(key) = event {
            match app.input_mode {
                InputMode::Editing => match key.code {
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Enter => {
                        if app.input.is_empty() {
                            continue;
                        }

                        let _prompts_len = app.prompts.len();
                        let msg_length = app.messages.len();
                        info!("msg {}", msg_length);
                        match msg_length {
                            x if x == _prompts_len && x != 2 && x != 3 => {
                                continue;
                            }
                            2 => {
                                let windows_number = app.input.as_str().trim().parse::<u8>();

                                if let Ok(num) = windows_number {
                                    for n in 1..=num {
                                        app.prompts.push(format!(
                                            "Enter args for window number {} (split by spaces): ",
                                            n
                                        ));
                                    }
                                } else {
                                    continue;
                                }
                            }
                            _ => {}
                        }

                        app.messages.push(app.input.drain(..).collect());

                        if app.messages.len() == app.prompts.len() {
                            let new_preset = Preset::new(&app.messages);
                            app_config.presets.push(new_preset);

                            app.messages.clear();
                            app.prompts.clear();

                            let config_file = File::create(CONFIG);

                            if config_file.is_err() {
                                error!("Error while reading the config file.");
                                panic!();
                            }

                            let mut writer = BufWriter::new(config_file.unwrap());
                            serde_json::to_writer(&mut writer, &app_config).unwrap();
                            writer.flush().unwrap();

                            app.popup.activate_popup(
                                "Preset created successfuly :) Press ESC to close popup.",
                            );
                            app.handle_state_change(State::Start);
                        }
                    }

                    KeyCode::Esc => {
                        if app.popup.active {
                            app.popup.deactivate_popup()
                        }
                        app.handle_state_change(State::Start);
                    }

                    _ => {}
                },
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    KeyCode::Enter => {
                        let selected_item = app.items.get_selected_item();
                        if let Some(i) = selected_item {
                            app.handle_state_change(i.1);
                        }
                    }
                    KeyCode::Esc => {
                        if app.popup.active {
                            app.popup.deactivate_popup()
                        }
                        if app.get_state() != State::Start {
                            app.handle_state_change(State::Start);
                        }
                    }
                    _ => {}
                },
            }
        }
    }
}
#[derive(Debug)]
enum InputMode {
    Normal,
    Editing,
}
struct App {
    state: State,
    items: StatefulList,
    prompts: Vec<String>,
    input: String,
    input_mode: InputMode,
    messages: Vec<String>,
    popup: Popup,
}

impl State {
    fn create_items(&self) -> Option<Vec<(String, State)>> {
        match self {
            State::Start => Some(vec![
                ("Choose Preset.".to_string(), State::ChoosePreset),
                ("Create Preset.".to_string(), State::CreatePreset),
            ]),
            _ => None,
        }
    }

    fn create_prompts(&self) -> Option<Vec<String>> {
        match self {
            State::CreatePreset => Some(vec![
                "Enter your new preset name:".to_string(),
                "Enter a valid path to your terminal:".to_string(),
                "Enter windows amount (number only): ".to_string(),
            ]),
            _ => None,
        }
    }
}

impl App {
    fn default() -> App {
        App {
            state: State::Start,
            items: StatefulList::with_items(State::Start.create_items().unwrap()),
            prompts: Vec::new(),
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            popup: Popup::default(),
        }
    }
    fn get_state(&self) -> State {
        self.state.clone()
    }

    fn handle_state_change(&mut self, new_state: State) {
        self.state = new_state;
        self.items.items.clear();
        self.prompts.clear();
        match new_state {
            State::CreatePreset => {
                let new_prompts = self.state.create_prompts().unwrap();

                for prompt in new_prompts {
                    self.prompts.push(prompt);
                }
                self.input_mode = InputMode::Editing;
            }
            _ => {
                self.input_mode = InputMode::Normal;
                let new_items = self.state.create_items().unwrap();

                for item in new_items {
                    self.items.items.push(item);
                }
            }
        };
    }
}

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
