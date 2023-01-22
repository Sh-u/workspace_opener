use std::{any::Any, collections::VecDeque};

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
#[derive(Debug, Clone, PartialEq)]
pub enum PresetValue {
    Name(String),
    Tabs(u8),
    Windows(usize, u8),
    Args(usize, String),
    PresetInfo(PresetInfoValue),
}
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub enum PresetInfoValue {
    WtProfile(String),
    InitShell(ShellType),
    TargetShell(ShellType),
}
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub enum ShellType {
    #[serde(rename = "powershell")]
    WindowsPowershell,
    #[serde(rename = "pwsh")]
    Powershell,
    #[serde(rename = "cmd")]
    Cmd,
    #[serde(rename = "bash")]
    Bash,
    #[serde(rename = "zsh")]
    Zsh,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub(super) name: String,
    pub(super) leading_state: State,
    pub(super) preset_value: Option<PresetValue>,
}
pub struct Popup {
    pub(super) active: bool,
    pub(super) message: String,
    pub(super) color: Color,
}

pub struct StatefulList {
    pub(super) list_state: ListState,
    pub(super) items: Vec<Item>,
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AppConfig {
    pub(super) presets: Vec<Preset>,
    pub(super) settings: Settings,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PresetInfo {
    pub(super) wt_profile: String,
    pub(super) init_shell: ShellType,
    pub(super) target_shell: ShellType,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Settings {
    pub(super) debug_mode: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Preset {
    pub(super) name: String,
    pub(super) tabs: u8,
    pub(super) windows: Vec<u8>,
    pub(super) args: Vec<String>,
    pub(super) preset_info: PresetInfo,
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
