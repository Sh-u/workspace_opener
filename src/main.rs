use crossterm::{
    event::{ self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent },
    execute,
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen },
};
use std::{ error::Error, fs::read_to_string, io };
use tui::{
    backend::{ Backend, CrosstermBackend },
    layout::{ Constraint, Direction, Layout },
    style::{ Style, Modifier, Color },
    text::{ Span, Spans, Text },
    widgets::{ Block, Borders, Paragraph, List, ListItem, ListState },
    Frame,
    Terminal,
};

const PATH: &str =
    "C:/Program Files/WindowsApps/Microsoft.WindowsTerminal_1.15.2874.0_x64__8wekyb3d8bbwe/wt.exe";

const CONFIG: &'static str = "config.txt";

#[derive(Debug, Clone, PartialEq)]
enum State {
    Start,
    ChoosePreset,
    CreatePreset,
    End,
}
struct StatefulList {
    state: ListState,
    items: Vec<(String, State)>,
}

impl StatefulList<> {
    fn with_items(items: Vec<(String, State)>) -> Self {
        StatefulList { state: ListState::default(), items }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };

        self.state.select(Some(i))
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 { 0 } else { i - 1 }
            }
            None => 0,
        };

        self.state.select(Some(i))
    }

    fn choose(&mut self, current_state: State) {
        match self.state.selected() {
            Some(i) => {
                let slc = self.items.get(i).unwrap();

                if let Some(created_items) = slc.1.create_items() {
                    self.items = created_items;
                }
            }
            None => (),
        }
    }
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

    let items = match app.state {
        State::Start => {
            app.items.items
                .iter()
                .map(|item| {
                    let mut lines = vec![Spans::from(item.0.as_str())];
                    ListItem::new(lines).style(Style::default().fg(Color::LightBlue))
                })
                .collect::<Vec<ListItem>>()
        }
        _ => vec![ListItem::new(Spans::default())],
    };

    let main_block = Block::default().title("Main").borders(Borders::ALL);
    let items = List::new(items)
        .block(main_block)
        .highlight_style(Style::default().bg(Color::LightYellow))
        .highlight_symbol("> ");

    f.render_stateful_widget(items, chunks[0], &mut app.items.state);

    let input_block = Block::default().title("Input").borders(Borders::ALL);
    f.render_widget(input_block, chunks[1]);
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
            match key.code {
                KeyCode::Char('q') => {
                    break;
                }
                KeyCode::Down => app.items.next(),
                KeyCode::Up => app.items.previous(),
                KeyCode::Enter => {
                    app.items.choose(current_state);
                }
                _ => {}
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
    input: String,
    input_mode: InputMode,
    messages: Vec<String>,
}

impl State {
    fn create_items(&self) -> Option<Vec<(String, State)>> {
        match self {
            State::Start =>
                Some(
                    vec![
                        ("Choose Preset.".to_string(), State::ChoosePreset),
                        ("Create Preset.".to_string(), State::CreatePreset)
                    ]
                ),
            _ => None,
        }
    }
}

impl App {
    fn default() -> App {
        App {
            state: State::Start,
            items: StatefulList::with_items(State::Start.create_items().unwrap()),
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
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