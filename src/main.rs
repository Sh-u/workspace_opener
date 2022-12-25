use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::*;
use simplelog::{Config, WriteLogger};
use std::{
    error::Error,
    fs::{read_to_string, File},
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols::line::BOTTOM_LEFT,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
const PATH: &str =
    "C:/Program Files/WindowsApps/Microsoft.WindowsTerminal_1.15.2874.0_x64__8wekyb3d8bbwe/wt.exe";

const CONFIG: &'static str = "config.txt";

#[derive(Debug, Copy, Clone, PartialEq)]
enum State {
    Start,
    ChoosePreset,
    CreatePreset,
}
struct StatefulList {
    list_state: ListState,
    items: Vec<(String, State)>,
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

    // fn choose(&mut self) -> Option<State> {
    // match self.list_state.selected() {
    //     Some(i) => {
    //         let selected_items = match self.items.get(i) {
    //             Some(i) => i.to_owned(),
    //             None => {
    //                 return None;
    //             }
    //         };

    //         if let Some(created_items) = selected_items.1.create_items() {
    //             self.items = created_items;
    //             Some(selected_items.1.clone())
    //         } else {
    //             None
    //         }
    //     }
    //     None => None,
    // }
    // }
}

enum StartChoices {
    ChoosePreset,
    CreateNewPreset,
}

impl StartChoices {
    fn as_string(&self) -> String {
        match self {
            StartChoices::ChoosePreset => String::from("Choose preset."),
            StartChoices::CreateNewPreset => String::from("Create new preset."),
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(10)].as_ref())
        .split(f.size());

    let main_block = Block::default().title("Main").borders(Borders::ALL);

    let input_block = Block::default().title("Input").borders(Borders::ALL);

    if app.items.items.is_empty() {
        let mut prompts = app
            .prompts
            .iter()
            .map(|prompt| ListItem::new(Span::from(prompt.as_str())))
            .collect::<Vec<ListItem>>();

        for msg in app.messages.iter() {
            prompts.push(
                ListItem::new(Span::from(msg.as_str())).style(
                    Style::default()
                        .fg(Color::LightRed)
                        .add_modifier(Modifier::ITALIC),
                ),
            )
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
    let presets = read_to_string(CONFIG)
        .expect("Error reading presets config.")
        .lines()
        .map(|line| line.trim().to_string())
        .collect::<Vec<String>>();

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
                        app.messages.push(app.input.drain(..).collect());
                    }

                    KeyCode::Esc => {
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
            State::CreatePreset => Some(vec!["Enter your new preset name.".to_string()]),
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
        File::create("output.txt").unwrap(),
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

// loop {
//     execute!(stdout(), terminal::Clear(ClearType::All)).unwrap();

// }

// Clean up the terminal.
// execute!(
//     stdout(),
//     cursor::Show,
//     terminal::LeaveAlternateScreen,
//     terminal::Clear(ClearType::All)
// ).unwrap();

// panic!();

// let mut path = String::new();
// let mut input = String::new();

// let mut state = State::Start;

// println!("{}", std::env::current_dir().unwrap().display());
// let contents = read_to_string(CONFIG).expect("Error reading file");

// println!("{}", contents.is_empty());
// let lines_num = contents.lines().count();

// println!("Do you want to use an existing preset? Y/N");

// loop {
//     match stdin().read_line(&mut input) {
//         Ok(_) => {
//             println!("input: {}. state: {:?}", input, &state);

//             match state {
//                 State::Start => {
//                     match input.to_lowercase().trim() {
//                         "y" if contents.is_empty() => {
//                             println!(
//                                 "There are no presets created yet. Please enter a valid path of your startup program to create one."
//                             );
//                             state = State::CreatePreset;
//                             input.clear();
//                             continue;
//                         }
//                         "y" => {
//                             println!("Choose your preset by typing the number next to it.");
//                             println!("File contents: \n{}", contents);

//                             state = State::ChoosePreset;
//                             continue;
//                         }
//                         "n" => {
//                             println!("Please enter a valid path of your startup program.");
//                             state = State::CreatePreset;
//                             continue;
//                         }
//                         _ => {
//                             println!("You can only type y/n.");
//                             break;
//                         }
//                     }
//                 }
//                 State::ChoosePreset => {}
//                 State::CreatePreset => {
//                     println!("Path addded.");
//                     path = input.trim().to_string();
//                     break;
//                 }
//             }
//         }
//         Err(err) => println!("An error occured: {}", err),
//     }
// }
// path.push_str("\n");
// match write(CONFIG, &path) {
//     Ok(_) => (),
//     Err(error) => {
//         panic!("Couldn't write to file: {}", error);
//     }
// }

// let command = Command::new(&path.trim()).spawn().expect("failed to execute process");
