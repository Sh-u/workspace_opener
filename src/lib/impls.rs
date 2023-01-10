use std::{borrow::BorrowMut, collections::VecDeque};

use super::model::{
    App, AppConfig, InputMode, Popup, Preset, PresetCreationHelper, Settings, State, StatefulList,
};
use log::warn;
use tui::{style::Color, widgets::ListState};

impl State {
    fn create_items(&self, app_config: Option<&AppConfig>) -> Option<Vec<(String, State)>> {
        match self {
            State::Start => Some(vec![
                ("Choose Preset.".to_string(), State::ChoosePreset),
                ("Create Preset.".to_string(), State::CreatePreset),
                ("Settings.".to_string(), State::Settings),
            ]),
            State::Settings => Some(vec![
                (
                    format!(
                        "Duplicate tab hotkey: {}",
                        app_config.unwrap().settings.duplicate_tab
                    ),
                    State::ChangeFieldName,
                ),
                (
                    format!(
                        "Duplicate pane hotkey: {}",
                        app_config.unwrap().settings.duplicate_pane
                    ),
                    State::ChangeFieldName,
                ),
            ]),
            _ => None,
        }
    }

    fn create_prompts(&self) -> Option<Vec<String>> {
        match self {
            State::CreatePreset => Some(vec![
                "Enter your new preset name:".to_string(),
                "Enter tabs amount (number only):".to_string(),
            ]),
            _ => None,
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

    pub fn next(&mut self) {
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

    pub fn previous(&mut self) {
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

    pub fn get_selected_item(&mut self) -> Option<(String, State)> {
        match self.list_state.selected() {
            Some(i) => match self.items.get(i) {
                Some(i) => Some(i.to_owned()),
                None => None,
            },
            None => None,
        }
    }

    pub fn get_selected_item_index(&self) -> Option<usize> {
        match self.list_state.selected() {
            Some(i) => Some(i),
            None => None,
        }
    }
}

impl Preset {
    pub fn new(input: &Vec<String>) -> Self {
        let name = input.get(0).unwrap().to_string();
        let tabs = input
            .get(1)
            .unwrap()
            .parse::<u8>()
            .expect("Failed to parse tabs arg.");
        let windows = input
            .iter()
            .skip(2)
            .take(tabs as usize)
            .map(|arg| arg.parse::<u8>().expect("Failed to parse a winows arg."))
            .collect::<Vec<u8>>();
        let args = input
            .iter()
            .skip(2 + windows.len())
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>();
        Preset {
            name,
            tabs,
            windows,
            args,
        }
    }

    pub fn change_name(&mut self, index: usize, new_name: &str) -> Result<(), String> {
        let windows_len = self.windows.len();
        let args_len = self.args.len();
        match index {
            0 => {
                self.name = new_name.to_string();
            }
            1 => {
                if let Ok(new_tabs) = new_name.parse::<u8>() {
                    if new_tabs == 0 {
                        return Err(String::from("Tabs cannot have a value of 0."));
                    }
                    let old_tabs = self.tabs;

                    if new_tabs < self.tabs {
                        for _n in 0..old_tabs - new_tabs {
                            self.windows.pop();
                        }
                    } else {
                        for _n in 0..new_tabs - old_tabs {
                            self.windows.push(1)
                        }
                    }

                    self.tabs = new_tabs;
                } else {
                    return Err(String::from("Tabs has to be a number."));
                }
            }
            x if x == 2 + self.tabs as usize => {
                if let Ok(v) = new_name.parse::<u8>() {
                    if v == 0 {
                        return Err(String::from("Windows cannot have a value of 0."));
                    }

                    let index = x - 3;
                    *self.windows.get_mut(index).unwrap() = v;
                } else {
                    return Err(String::from("Windows has to be a number."));
                }
            }
            _ => {
                if let Some(arg) = self.args.get_mut(index - 3) {
                    *arg = format!("{}", new_name);
                } else {
                    return Err(String::from("Error changing args name."));
                }
            }
        }

        Ok(())
    }
    pub fn into_items(&self) -> Vec<String> {
        let mut items = vec![];

        items.push(format!("Name: {}", self.name.as_str()));
        items.push(format!("Tabs: {}", &self.tabs.to_string()));

        for (tab_index, windows_amount) in self.windows.iter().enumerate() {
            items.push(format!(
                "Windows in tab {}: {}",
                tab_index + 1,
                &windows_amount.to_string()
            ));
        }

        let mut windex_argcount = (0, 0);
        for arg in self.args.iter() {
            if windex_argcount.1 >= *self.windows.get(windex_argcount.0).unwrap() {
                windex_argcount = (windex_argcount.0 + 1, 0)
            }
            items.push(format!(
                "Tab (#{}), window (#{}), Arg: {}",
                windex_argcount.0 + 1,
                windex_argcount.1 + 1,
                arg
            ));
            windex_argcount.1 += 1;
        }

        items
    }
}

impl Popup {
    pub fn default() -> Self {
        Popup {
            active: false,
            message: String::new(),
            color: Color::White,
        }
    }

    pub fn activate_popup(&mut self, message: &str, color: Color) {
        self.active = true;
        self.message = message.to_string();
        self.color = color;
    }

    pub fn deactivate_popup(&mut self) {
        self.active = false;
    }
}

impl App {
    pub fn default() -> App {
        App {
            state: State::Start,
            previous_state: State::Start,
            items: StatefulList::with_items(State::Start.create_items(None).unwrap()),
            prompts: Vec::new(),
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            popup: Popup::default(),
            current_preset: None,
            debug_mode: true,
        }
    }
    pub fn get_state(&self) -> State {
        self.state.clone()
    }

    pub fn handle_state_change(
        &mut self,
        (item_name, new_state): (&str, State),
        app_config: Option<&AppConfig>,
    ) {
        if new_state == self.state {
            return ();
        }

        self.previous_state = match new_state {
            State::EditPreset => State::ChoosePreset,
            State::ChangeFieldName => self.get_state(),
            _ => State::Start,
        };
        self.state = new_state;

        self.input.clear();
        self.messages.clear();

        match new_state {
            State::CreatePreset => {
                self.items.items.clear();
                let new_prompts = self.state.create_prompts().unwrap();

                for prompt in new_prompts {
                    self.prompts.push(prompt);
                }
                self.input_mode = InputMode::Input;
            }
            State::ChoosePreset => {
                self.items.items.clear();
                if let Some(config) = app_config {
                    match &config.presets {
                        presets if !presets.is_empty() => {
                            for preset in presets {
                                self.items
                                    .items
                                    .push((preset.name.to_string(), State::RunConfig));
                            }
                        }
                        _ => {
                            self.handle_state_change(("", State::Start), None);
                            self.popup.activate_popup("No presets created.", Color::Red);
                        }
                    }
                }
            }
            State::EditPreset => {
                self.input_mode = InputMode::Normal;
                self.items.items.clear();
                if let Some(config) = app_config {
                    match &config.presets {
                        presets if !presets.is_empty() => {
                            let new_items = self.current_preset.clone().unwrap().into_items();

                            for item in new_items {
                                self.items.items.push((item, State::ChangeFieldName));
                            }
                        }
                        _ => {}
                    }
                }
            }
            State::ChangeFieldName => {
                self.input_mode = InputMode::Edit;
                let index = item_name.find(":").unwrap();
                let new_input = item_name[index + 1..].trim();
                self.input = new_input.to_string();
            }
            State::RunConfig => {}
            _ => {
                self.items.items.clear();
                self.prompts.clear();
                self.current_preset = None;
                self.input_mode = InputMode::Normal;
                let new_items = self.state.create_items(app_config).unwrap();

                for item in new_items {
                    self.items.items.push(item);
                }
                self.items.list_state.select(Some(0));
            }
        };
    }
}

impl AppConfig {
    pub fn find_preset_by_name(&mut self, name: &str) -> Option<&mut Preset> {
        if let Some(found) = self.presets.iter_mut().find(|preset| preset.name == name) {
            Some(found)
        } else {
            None
        }
    }
}

impl Settings {
    pub fn change_name(&mut self, index: usize, new_name: &str) -> Result<(), String> {
        match index {
            0 => {
                self.duplicate_tab = new_name.to_string();
            }
            1 => {
                self.duplicate_pane = new_name.to_string();
            }
            _ => {
                return Err(String::from(
                    "Trying to change the name while using wrong index.",
                ));
            }
        }
        Ok(())
    }
}

impl PresetCreationHelper {
    pub fn new() -> Self {
        PresetCreationHelper {
            windows: VecDeque::new(),
            max_windows: 0,
        }
    }

    pub fn reset(&mut self) {
        self.windows.clear();
        self.max_windows = 0;
    }
}
