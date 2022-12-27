use serde::{Deserialize, Serialize};
use tui::{style::Color, widgets::ListState};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum State {
    Start,
    ChoosePreset,
    CreatePreset,
    EditPreset,
    ChangePresetField,
    RunConfig,
}
pub struct Popup {
    pub(super) active: bool,
    pub(super) message: String,
    pub(super) color: Color,
}

pub struct StatefulList {
    pub(super) list_state: ListState,
    pub(super) items: Vec<(String, State)>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub(super) presets: Vec<Preset>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Preset {
    pub(super) name: String,
    pub(super) terminal_path: String,
    pub(super) windows: u8,
    pub(super) args: Vec<String>,
}
#[derive(Debug)]
pub enum InputMode {
    Normal,
    Input,
    Edit,
}
pub struct App {
    pub(super) state: State,
    pub(super) items: StatefulList,
    pub(super) prompts: Vec<String>,
    pub(super) input: String,
    pub(super) input_mode: InputMode,
    pub(super) messages: Vec<String>,
    pub(super) popup: Popup,
}
