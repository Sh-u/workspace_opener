use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use tui::{style::Color, widgets::ListState};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum State {
    Start,
    Settings,
    ChoosePreset,
    CreatePreset,
    EditPreset,
    ChangeFieldName,
    RunConfig,
}
#[derive(Debug)]
pub enum InputMode {
    Normal,
    Input,
    Edit,
}
pub enum WriteType {
    Create,
    Edit,
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
    pub(super) settings: Settings,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    // pub(super) terminal_path: String,
    pub(super) duplicate_tab: String,
    pub(super) duplicate_pane: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Preset {
    pub(super) name: String,
    pub(super) tabs: u8,
    pub(super) windows: Vec<u8>,
    pub(super) args: Vec<String>,
}

pub struct App {
    pub(super) state: State,
    pub(super) previous_state: State,
    pub(super) items: StatefulList,
    pub(super) prompts: Vec<String>,
    pub(super) input: String,
    pub(super) input_mode: InputMode,
    pub(super) messages: Vec<String>,
    pub(super) popup: Popup,
    pub(super) current_preset: Option<Preset>,
    pub(super) debug_mode: bool,
}

pub struct PresetCreationHelper {
    pub(super) windows: VecDeque<u8>,
    pub(super) max_windows: usize,
}
