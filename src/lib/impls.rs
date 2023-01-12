use super::model::{
    App, AppConfig, InputMode, Item, Popup, Preset, PresetCreationHelper, PresetValue, Settings,
    State, StatefulList,
};
use log::{error, warn};
use std::collections::VecDeque;
use tui::{style::Color, widgets::ListState};

impl Item {
    pub fn new(name: String, leading_state: State, preset_value: Option<PresetValue>) -> Item {
        Item {
            name,
            leading_state,
            preset_value,
        }
    }
}

impl State {
    fn create_items(&self, app_config: Option<&AppConfig>) -> Option<Vec<Item>> {
        match self {
            State::Start => Some(vec![
                Item::new("Choose Preset.".to_string(), State::ChoosePreset, None),
                Item::new("Create Preset.".to_string(), State::CreatePreset, None),
                Item::new("Settings.".to_string(), State::Settings, None),
            ]),
            State::Settings => Some(vec![
                Item::new(
                    format!(
                        "Terminal path: {}",
                        app_config.unwrap().settings.terminal_path
                    ),
                    State::ChangeFieldName,
                    None,
                ),
                Item::new(
                    format!(
                        "Shell name: {}",
                        app_config.unwrap().settings.shell_name
                    ),
                    State::ChangeFieldName,
                    None,
                ),
            ]),
            _ => None,
        }
    }

    fn create_prompts(&self) -> Option<Vec<String>> {
        match self {
            State::CreatePreset => Some(vec![
                "Enter preset name:".to_string(),
                "Enter tabs amount (number only):".to_string(),
            ]),
            _ => None,
        }
    }
}

impl StatefulList {
    fn with_items(items: Vec<Item>) -> Self {
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

    pub fn get_selected_item(&self) -> Option<Item> {
        match self.list_state.selected() {
            Some(i) => match self.items.get(i) {
                Some(i) => Some(i.clone()),
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

    pub fn delete_selected_item(&mut self) -> Result<(), String> {
        match self.list_state.selected() {
            Some(i) => {self.items.remove(i);},
            None => return Err(String::from("Cannot delete item: ITEM WITH THIS INDEX WAS NOT FOUND.")),
        }
        Ok(())
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

    pub fn change_field_value(&mut self, preset_value: PresetValue) -> Result<(), String> {
        match preset_value {
            PresetValue::Name(name) => {
                self.name = name.to_string();
            }
            PresetValue::Tabs(new_tabs) => {
                if new_tabs == 0 {
                    return Err(String::from("Tabs cannot have a value of 0."));
                }
                let old_tabs = self.tabs;

                if new_tabs < self.tabs {
                    for _n in 0..*self.windows.last().unwrap() {
                        self.args.pop();
                    }

                    for _n in 0..old_tabs - new_tabs {
                        self.windows.pop();
                    }
                } else {
                    for _n in 0..new_tabs - old_tabs {
                        self.windows.push(1);
                        self.args.push("".to_string());
                    }
                }
                self.tabs = new_tabs;
            }
            PresetValue::Windows(index, new_windows) => {
                if new_windows == 0 {
                    return Err(String::from("Windows cannot have a value of 0."));
                }
                let args_before = self.windows[0..index].iter().sum::<u8>();
                let Some(old_windows) = self.windows.get_mut(index as usize) else {
                        return Err(String::from("Cannot find windows with current index."));
                    };

                let mut args_place_index = (args_before + *old_windows) as usize;

          
                if new_windows < *old_windows {
                    for _n in 0..*old_windows - new_windows {
                        self.args.remove(args_place_index - 1);
                        args_place_index -= 1;
                    }
                } else {
                    for _n in 0..new_windows - *old_windows {
                        self.args.insert(args_place_index, "".to_string());
                    }
                }

                *old_windows = new_windows;
            }

            PresetValue::Args(index, new_name) => {
                let Some(current_arg) = self.args.get_mut(index as usize) else {                  
                    return Err(String::from("Cannot find windows with current index."));
                };
                *current_arg = new_name;
            }
        }

        Ok(())
    }
    pub fn into_items(&self) -> Vec<Item> {
        let mut items = vec![];

        let name = Item::new(
            format!("Name: {}", self.name.as_str()),
            State::ChangeFieldName,
            Some(PresetValue::Name(self.name.to_string())),
        );

        let tabs = Item::new(
            format!("Tabs: {}", &self.tabs.to_string()),
            State::ChangeFieldName,
            Some(PresetValue::Tabs(self.tabs)),
        );

        items.push(name);
        items.push(tabs);

        for (tab_index, windows_amount) in self.windows.iter().enumerate() {
            let window = Item::new(
                format!(
                    "Tab (#{}), windows: {}",
                    tab_index + 1,
                    &windows_amount.to_string()
                ),
                State::ChangeFieldName,
                Some(PresetValue::Windows(tab_index, *windows_amount)),
            );
            items.push(window);
        }

        let mut window_index = 0;
        let mut arg_count = 0;
        for (arg_index, arg) in self.args.iter().enumerate() {
           
            let Some(current_window) = self.windows.get(window_index) else {
                error!("Cannot create items from preset windows: INDEX OUT OF BOUNDS.\nWindows: {:?} \nIndex: {}", self.windows, window_index);
                panic!();
            };

            if arg_count >= *current_window {
                if window_index + 1 < self.windows.len() {
                    window_index += 1;
                    arg_count = 0;
                }
            }
            warn!("w inedx: {}, arg count {}", window_index, arg_count);
            let arg = Item::new(
                format!(
                    "Tab (#{}), window (#{}), Arg: {}",
                    window_index + 1,
                    arg_count + 1,
                    arg
                ),
                State::ChangeFieldName,
                Some(PresetValue::Args(arg_index, arg.to_string())),
            );
            items.push(arg);
            arg_count += 1;
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
            return;
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
                let Some(config) = app_config else {return;};

                match &config.presets {
                    presets if !presets.is_empty() => {
                        for preset in presets {
                            let item = Item::new(preset.name.to_string(), State::RunConfig, None);
                            self.items.items.push(item);
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
                match app_config {
                    Some(config) if !config.presets.is_empty() => {
                        let new_items = self.current_preset.clone().unwrap().into_items();
                        for item in new_items {
                            self.items.items.push(item);
                        }
                    }
                    _ => {}
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
    pub fn get_mut_preset_by_name(&mut self, name: &str) -> Option<&mut Preset> {
        if let Some(found) = self.presets.iter_mut().find(|preset| preset.name == name) {
            Some(found)
        } else {
            None
        }
    }

    pub fn delete_preset_by_name(&mut self, name: &str) -> Result<(), String> {
        if let Some(index) = self.presets.iter().position(|pr| pr.name == name) {
            self.presets.remove(index);
        } else {
            return Err(String::from("Cannot delete preset: PRESET WITH GIVEN NAME WAS NOT FOUND."));
        }

        Ok(())
    }
}

impl Settings {
    pub fn change_name(&mut self, index: usize, new_name: &str) -> Result<(), String> {
        match index {
            0 => {
                self.terminal_path = new_name.to_string();
            }
            1 => {
                self.shell_name = new_name.to_string();
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

impl PresetValue {
    pub fn update_value(&mut self, new_val: &str) -> Result<(), String> {
        match self {
            PresetValue::Name(name) => {
                *name = new_val.to_string();
            }

            PresetValue::Tabs(tabs) => {
                let Ok(new_tabs) = new_val.parse::<u8>() else {
                    return Err(String::from("PresetValue, update_value error parsing tabs: CANNOT PARSE GIVEN STRING."));};
                *tabs = new_tabs;
            }
            PresetValue::Windows(_, windows) => {
                let Ok(new_windows) = new_val.parse::<u8>() else {
                    return Err(String::from("PresetValue update_value error parsing windows: CANNOT PARSE GIVEN STRING."));};
                *windows = new_windows;
            }
            PresetValue::Args(_, arg) => {
                *arg = new_val.to_string();
            }
        }

        Ok(())
    }
}



