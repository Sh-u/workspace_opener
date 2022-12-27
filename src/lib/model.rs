#[derive(Debug, Copy, Clone, PartialEq)]
enum State {
    Start,
    ChoosePreset,
    CreatePreset,
    RunConfig,
}
struct Popup {
    active: bool,
    message: String,
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
