use std::{collections::binary_heap::Iter, error::Error, io::ErrorKind, string::ParseError};

use super::model::{App, AppConfig, InputMode, Popup, Preset, State, StatefulList};
use log::warn;
use tui::{style::Color, widgets::ListState};

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

    pub fn change_selected_item_name(&mut self, new_item: &str) -> Option<(String, String)> {
        match self.list_state.selected() {
            Some(index) => match self.items.get_mut(index) {
                Some(i) => {
                    let old_name = i.0.to_string();
                    i.0 = Preset::get_prefix(index) + new_item;
                    Some((old_name, i.0.to_string()))
                }
                None => None,
            },
            None => None,
        }
    }
}

impl Preset {
    pub fn new(input: &Vec<String>) -> Self {
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

    pub fn get_prefix(index: usize) -> String {
        match index {
            0 => "Name: ".to_string(),
            1 => "Terminal path: ".to_string(),
            2 => "Windows amount: ".to_string(),
            x => format!("Arg {}: ", x - 2),
        }
    }

    pub fn change_name(&mut self, index: usize, new_name: &str) -> Result<(), String> {
        match index {
            0 => {
                self.name = new_name.to_string();
            }
            1 => {
                self.terminal_path = new_name.to_string();
            }
            2 => {
                if let Ok(v) = new_name.parse::<u8>() {
                    if v == 0 {
                        return Err(String::from("Windows cannot have a value of 0."));
                    }
                    let old_windows = self.windows;

                    if v < self.windows {
                        for _n in 0..old_windows - v {
                            self.args.pop();
                        }
                    } else {
                        for _n in 0..v - old_windows {
                            self.args.push(format!(""))
                        }
                    }

                    self.windows = v;
                } else {
                    return Err(String::from("Windows has to be a number."));
                }
            }
            _ => {
                for n in 1..=self.args.len() {
                    if let Some(arg) = self.args.get_mut(n) {
                        *arg = format!("Arg {}: ", n + 1);
                    } else {
                        return Err(String::from("Error changing args name."));
                    }
                }
            }
        }

        Ok(())
    }
    pub fn into_items(&self) -> Vec<String> {
        let mut items = vec![];
        items.push(Self::get_prefix(0) + self.name.as_str());
        items.push(Self::get_prefix(1) + self.terminal_path.as_str());
        items.push(Self::get_prefix(2) + &self.windows.to_string());
        for arg in self.args.iter().enumerate() {
            warn!("arg index: {}", &arg.0);
            items.push(Self::get_prefix(arg.0 + 3) + arg.1);
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
            items: StatefulList::with_items(State::Start.create_items().unwrap()),
            prompts: Vec::new(),
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            popup: Popup::default(),
            current_preset: None,
        }
    }
    pub fn get_state(&self) -> State {
        self.state.clone()
    }

    pub fn handle_state_change(
        &mut self,
        (item_name, new_state): (&str, State),
        possible_presets: Option<&Vec<Preset>>,
    ) {
        if new_state == self.state {
            return ();
        }

        self.state = new_state;
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
                match possible_presets {
                    Some(presets) if presets.len() > 0 => {
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
            State::EditPreset => {
                self.input_mode = InputMode::Normal;
                self.items.items.clear();
                match possible_presets {
                    Some(presets) if presets.len() > 0 => {
                        self.current_preset = Some(presets.first().unwrap().clone());

                        let new_items = self.current_preset.clone().unwrap().into_items();

                        for item in new_items {
                            self.items.items.push((item, State::ChangePresetField));
                        }
                    }
                    _ => {}
                }
            }
            State::ChangePresetField => {
                self.input_mode = InputMode::Edit;
                let index = item_name.find(":").unwrap();
                let new_input = item_name[index + 1..].trim();
                self.input = new_input.to_string();
            }
            _ => {
                self.items.items.clear();
                self.prompts.clear();

                self.input_mode = InputMode::Normal;
                let new_items = self.state.create_items().unwrap();

                for item in new_items {
                    self.items.items.push(item);
                }
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
