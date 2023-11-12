use super::{
    model::{
        App,
        AppConfig,
        InputMode,
        Item,
        Popup,
        Preset,
        PresetCreationHelper,
        PresetValue,
        PresetInfo,
        State,
        StatefulList,
        ShellType,
        PresetInfoValue,
        Settings,
        WriteType,
    },
    api::CONFIG,
};
use crossterm::event::KeyCode;
use log::error;
use std::{ collections::VecDeque, fmt::Display };
use tui::{ style::Color, widgets::ListState };
use std::{ fs::File, io::{ BufWriter, Write } };
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
            State::Start =>
                Some(
                    vec![
                        Item::new("Choose Preset".to_string(), State::ChoosePreset, None),
                        Item::new("Create Preset".to_string(), State::CreatePreset, None),
                        Item::new("Settings".to_string(), State::Settings, None)
                    ]
                ),
            State::Settings =>
                Some(
                    vec![
                        Item::new(
                            format!("Debug mode: {}", app_config.unwrap().settings.debug_mode),
                            State::ChangeFieldName,
                            None
                        )
                    ]
                ),
            _ => None,
        }
    }

    fn create_prompts(&self) -> Option<Vec<String>> {
        match self {
            State::CreatePreset =>
                Some(
                    vec!["Enter preset name:".to_string(), "Enter tabs amount (1-10):".to_string()]
                ),
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
                if i >= self.items.len() - 1 { self.items.len() - 1 } else { i + 1 }
            }
            None => 0,
        };

        self.list_state.select(Some(i))
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 { 0 } else { i - 1 }
            }
            None => 0,
        };

        self.list_state.select(Some(i))
    }

    pub fn get_selected_item(&self) -> Option<Item> {
        match self.list_state.selected() {
            Some(i) =>
                match self.items.get(i) {
                    Some(i) => Some(i.clone()),
                    None => None,
                }
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
            Some(i) => {
                self.items.remove(i);
            }
            None => {
                return Err(String::from("Cannot delete item: ITEM WITH THIS INDEX WAS NOT FOUND."));
            }
        }
        Ok(())
    }
}

impl Preset {
    pub fn from_input(input: &Vec<String>) -> Self {
        let name = input.get(0).unwrap().to_string();
        let tabs = input.get(1).unwrap().parse::<u8>().expect("Failed to parse tabs arg.");
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

        let preset_info = PresetInfo {
            wt_profile: String::new(),
            init_shell: ShellType::WindowsPowershell,
            target_shell: ShellType::WindowsPowershell,
        };
        Preset {
            name,
            tabs,
            windows,
            args,
            preset_info,
        }
    }

    pub fn new(
        name: String,
        tabs: u8,
        windows: Vec<u8>,
        args: Vec<String>,
        preset_info: PresetInfo
    ) -> Preset {
        Preset { name, tabs, windows, args, preset_info }
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
                    for _n in 0..old_tabs - new_tabs {
                        let Some(deleted_window) = self.windows.pop() else {
                            return Err(
                                String::from("Trying to delete window from an empty collection")
                            );
                        };
                        for _n in 0..deleted_window {
                            self.args.pop();
                        }
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
            PresetValue::PresetInfo(new_preset_info_value) =>
                match new_preset_info_value {
                    PresetInfoValue::WtProfile(new_wt_profile) => {
                        self.preset_info.wt_profile = new_wt_profile;
                    }
                    PresetInfoValue::InitShell(new_init_shell) => {
                        self.preset_info.init_shell = new_init_shell;
                    }
                    PresetInfoValue::TargetShell(new_target_shell) => {
                        self.preset_info.target_shell = new_target_shell;
                    }
                }
        }

        Ok(())
    }
    pub fn into_items(&self) -> Vec<Item> {
        let mut items = vec![];

        let name = Item::new(
            format!("Name: {}", self.name.as_str()),
            State::ChangeFieldName,
            Some(PresetValue::Name(self.name.to_string()))
        );

        let tabs = Item::new(
            format!("Tabs: {}", &self.tabs.to_string()),
            State::ChangeFieldName,
            Some(PresetValue::Tabs(self.tabs))
        );

        items.push(name);
        items.push(tabs);

        for (tab_index, windows_amount) in self.windows.iter().enumerate() {
            let window = Item::new(
                format!("Tab (#{}), windows: {}", tab_index + 1, &windows_amount.to_string()),
                State::ChangeFieldName,
                Some(PresetValue::Windows(tab_index, *windows_amount))
            );
            items.push(window);
        }

        let mut window_index = 0;
        let mut arg_count = 0;
        for (arg_index, arg) in self.args.iter().enumerate() {
            let Some(current_window) = self.windows.get(window_index) else {
                error!(
                    "Cannot create items from preset windows: INDEX OUT OF BOUNDS.\nWindows: {:?} \nIndex: {}",
                    self.windows,
                    window_index
                );
                panic!();
            };

            if arg_count >= *current_window {
                if window_index + 1 < self.windows.len() {
                    window_index += 1;
                    arg_count = 0;
                }
            }
            let arg = Item::new(
                format!("Tab (#{}), window (#{}), Arg: {}", window_index + 1, arg_count + 1, arg),
                State::ChangeFieldName,
                Some(PresetValue::Args(arg_index, arg.to_string()))
            );
            items.push(arg);
            arg_count += 1;
        }

        let wt_profile = Item::new(
            format!("Windows terminal profile name: {}", self.preset_info.wt_profile),
            State::ChangeFieldName,
            Some(
                PresetValue::PresetInfo(
                    PresetInfoValue::WtProfile(self.preset_info.wt_profile.clone())
                )
            )
        );
        items.push(wt_profile);

        let init_shell = Item::new(
            format!(
                "Init shell (powershell/pwsh/cmd): {}",
                self.preset_info.init_shell.as_string()
            ),
            State::ChangeFieldName,
            Some(
                PresetValue::PresetInfo(
                    PresetInfoValue::InitShell(self.preset_info.init_shell.clone())
                )
            )
        );
        items.push(init_shell);

        let target_shell = Item::new(
            format!(
                "Target shell (powershell/pwsh/cmd/bash/zsh/fish): {}",
                self.preset_info.target_shell.as_string()
            ),
            State::ChangeFieldName,
            Some(
                PresetValue::PresetInfo(
                    PresetInfoValue::TargetShell(self.preset_info.target_shell.clone())
                )
            )
        );
        items.push(target_shell);

        items
    }

    pub fn default() -> Preset {
        let preset_info = PresetInfo::default();
        Preset {
            name: String::from("Test Preset"),
            tabs: 3,
            windows: vec![2, 1, 1],
            args: vec![
                String::from("arg w1"),
                String::from("arg w1"),
                String::from("arg w2"),
                String::from("arg w3")
            ],
            preset_info,
        }
    }

    pub fn get_preset_info(&self) -> PresetInfo {
        self.preset_info.clone()
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
            selected_input: Vec::new(),
            cursor_idx: 0,
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            popup: Popup::default(),
            current_preset: None,
            debug_mode: false,
        }
    }
    pub fn get_state(&self) -> State {
        self.state.clone()
    }

    pub fn select_left_input(&mut self, cursor_idx: usize) -> () {
        let selected_input = &mut self.selected_input;
        match selected_input.last() {
            Some(last) if *last == cursor_idx - 1 => {
                selected_input.pop();
            }
            _ => {
                selected_input.insert(0, cursor_idx - 1);
            }
        }
    }

    pub fn select_right_input(&mut self, cursor_idx: usize) -> () {
        let selected_input = &mut self.selected_input;
        match selected_input.iter().find(|pos| **pos == cursor_idx) {
            Some(_) => {
                selected_input.remove(0);
            }
            _ => {
                selected_input.push(cursor_idx);
            }
        }
    }

    pub fn move_cursor_left(&mut self) {
        let current_point = self.cursor_idx;
        if current_point == 0 {
            return;
        }
        self.cursor_idx = current_point - 1;
        self.selected_input.clear();
    }

    pub fn move_cursor_right(&mut self) {
        let current_point = self.cursor_idx;
        if current_point >= self.input.len() {
            return;
        }
        self.cursor_idx = current_point + 1;

        self.selected_input.clear();
    }

    pub fn insert_char(&mut self, ch: char) {
        let cursor_idx = self.cursor_idx;
        let input = &mut self.input;
        let selected_input = &mut self.selected_input;

        if !selected_input.is_empty() {
            let mut chars: Vec<char> = input.chars().collect();
            for &index in selected_input.iter().rev() {
                chars.remove(index);
            }
            chars.push(ch);
            *input = chars.into_iter().collect();

            selected_input.clear();
        } else {
            input.insert(cursor_idx, ch);
        }

        self.cursor_idx = cursor_idx + 1;
    }

    pub fn delete_characters(&mut self) {
        let cursor_idx = self.cursor_idx;
        let selected_input_len = self.selected_input.len();

        if selected_input_len > 0 {
            let last_selected = *self.selected_input.last().unwrap();

            let mut chars: Vec<char> = self.input.chars().collect();
            for &index in self.selected_input.iter().rev() {
                chars.remove(index);
            }
            self.input = chars.into_iter().collect();
            self.selected_input.clear();

            if cursor_idx >= last_selected {
                self.cursor_idx = cursor_idx - selected_input_len;
            }
            return;
        }

        if cursor_idx == 0 {
            return;
        }

        self.input.remove(cursor_idx - 1);
        self.cursor_idx = cursor_idx - 1;
    }

    fn move_cursor_left_word_pos(input: &str, cursor_idx: usize) -> usize {
        if cursor_idx == 0 {
            return cursor_idx;
        }

        let mut chars = input[..cursor_idx].char_indices().rev().peekable();

        while let Some(first) = chars.next() {
            if !first.1.is_whitespace() {
                if let Some(next) = chars.peek() {
                    if next.1.is_whitespace() {
                        return first.0;
                    }
                }
            }
        }

        return 0;
    }

    fn move_cursor_right_word_pos(input: &str, cursor_idx: usize) -> usize {
        if cursor_idx >= input.len() {
            return input.len();
        }

        let mut chars = input[cursor_idx..].char_indices().peekable();

        while let Some(first) = chars.next() {
            if first.1.is_whitespace() {
                if let Some(next) = chars.peek() {
                    if !next.1.is_whitespace() {
                        return next.0 + cursor_idx;
                    }
                }
            }
        }
        let last_char = input
            .chars()
            .into_iter()
            .rev()
            .position(|ch| ch.is_ascii_alphabetic());

        if let Some(pos) = last_char {
            return std::cmp::max(input.len() - pos, 0);
        } else {
            return input.len();
        }
    }

    pub fn handle_control_key_action(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char('c') => {
                let selected_input = &self.selected_input;
                if selected_input.is_empty() {
                    return;
                }
                let first = *selected_input.first().unwrap();
                let last = *selected_input.last().unwrap();

                let input = &self.input;
                let input = &input[first..=last];
                if let Err(err) = cli_clipboard::set_contents(input.to_string()) {
                    error!("Something went wrong while trying to copy: {:?}", err);
                }
            }
            KeyCode::Char('v') => {
                if let Ok(cli_contents) = cli_clipboard::get_contents() {
                    let cursor_idx = self.cursor_idx;
                    let input = &mut self.input;

                    input.insert_str(cursor_idx, &cli_contents);

                    let new_index = cursor_idx + 1 + cli_contents.len();

                    self.cursor_idx = new_index;
                }
            }
            KeyCode::Left => {
                let cursor_idx = self.cursor_idx;
                self.selected_input.clear();
                let new_index = App::move_cursor_left_word_pos(&self.input, cursor_idx);
                self.cursor_idx = new_index;
            }
            KeyCode::Right => {
                let cursor_idx = self.cursor_idx;
                self.selected_input.clear();
                let new_index = App::move_cursor_right_word_pos(&self.input, cursor_idx);
                self.cursor_idx = new_index;
            }
            _ => {}
        }
    }

    pub fn handle_shift_key_action(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Left => {
                let cursor_idx = self.cursor_idx;

                if cursor_idx == 0 {
                    return;
                }

                let new_idx = cursor_idx - 1;
                self.select_left_input(cursor_idx);
                self.cursor_idx = new_idx;
            }
            KeyCode::Right => {
                let cursor_idx = self.cursor_idx;
                let input = self.input.as_str();

                if cursor_idx >= input.len() {
                    return;
                }

                let new_idx = cursor_idx + 1;
                self.select_right_input(cursor_idx);
                self.cursor_idx = new_idx;
            }
            _ => {}
        }
    }

    fn create_preset_value(&mut self, pch: &mut PresetCreationHelper) -> Result<(), ()> {
        let msg_length = self.messages.len();
        if msg_length == 1 {
            let Ok(tabs_amount) = self.input.parse::<u8>() else {
                return Err(());
            };
            if tabs_amount < 1 || tabs_amount > 10 {
                return Err(());
            }

            for n in 1..=tabs_amount {
                self.prompts.push(format!("Enter windows amount (1-4) for tab number {}: ", n));
            }
            pch.max_windows = tabs_amount as usize;
        } else if msg_length > 1 && pch.windows.len() < pch.max_windows {
            let Ok(input) = self.input.parse::<u8>() else {
                return Err(());
            };
            if input < 1 || input > 4 {
                return Err(());
            }
            pch.windows.push_back(input);

            let Some(&windows_back) = pch.windows.back() else {
                return Err(());
            };
            for n in 1..=windows_back {
                self.prompts.push(format!("Enter arg {} for window {}", n, pch.windows.len()));
            }
        }

        Ok(())
    }

    fn format_input(input: &str) -> String {
        let mut input = input.trim();
        if input.ends_with(",") {
            input = &input[..input.len() - 1];
        }

        input.to_string()
    }

    pub fn handle_creating_preset(
        &mut self,
        pch: &mut PresetCreationHelper,
        app_config: &mut AppConfig
    ) {
        if let Err(_) = self.create_preset_value(pch) {
            return;
        }

        let input = Self::format_input(&self.input.drain(..).collect::<String>());

        self.messages.push(input);
        self.cursor_idx = 0;

        if self.messages.len() == self.prompts.len() {
            pch.reset();

            app_config.write_preset_to_file(&self.messages, WriteType::Create, CONFIG).unwrap();

            self.popup.activate_popup("Preset created successfuly :)", Color::Green);

            self.handle_state_change(("", self.previous_state), None);
        }
    }

    pub fn handle_deleting_preset(&mut self, app_config: &mut AppConfig) {
        let Some(item) = self.items.get_selected_item() else {
            return;
        };

        if let Err(err) = app_config.delete_preset_by_name(&item.name) {
            error!("{}", err);
            return;
        }
        if let Err(err) = self.items.delete_selected_item() {
            error!("{}", err);
            return;
        }
        app_config
            .write_preset_to_file(&self.messages, WriteType::Edit, CONFIG)
            .expect("Error when writing to a file of a deleted preset.");

        if self.items.items.is_empty() {
            self.handle_state_change(("", self.previous_state), Some(&app_config));
        }
    }

    pub fn handle_editing_preset(&mut self, app_config: &mut AppConfig) {
        let Some(item) = self.items.get_selected_item() else {
            return;
        };

        if let Some(pr) = &self.current_preset {
            let Some(pr) = app_config.get_mut_preset_by_name(&pr.name) else {
                return;
            };
            let Some(mut pr_value) = item.preset_value else {
                return;
            };

            let input = App::format_input(&self.input);

            if let Err(err) = pr_value.update_value(&input) {
                error!("{}", err);
                return;
            }

            if let Err(err) = pr.change_field_value(pr_value) {
                error!("{}", err);
                return;
            }

            self.current_preset = Some(pr.clone());
        } else {
            let Some(index) = self.items.get_selected_item_index() else {
                return;
            };

            if let Err(err) = app_config.settings.change_name(index, &self.input) {
                error!("{}", err);
                return;
            }
        }

        app_config
            .write_preset_to_file(&self.messages, WriteType::Edit, CONFIG)
            .expect("Error when writing to a file of an edited preset.");

        self.handle_state_change(("", self.previous_state), Some(&app_config));
    }

    pub fn cancel_preset_creation(&mut self, pch: &mut PresetCreationHelper) {
        if self.popup.active {
            self.popup.deactivate_popup();
            return;
        }
        pch.reset();
        self.handle_state_change(("", State::Start), None);
    }

    pub fn go_back(&mut self, app_config: &AppConfig) {
        if self.popup.active {
            self.popup.deactivate_popup();
            return;
        }

        self.handle_state_change(("", self.previous_state), Some(app_config));
    }

    pub fn edit_preset(&mut self, app_config: &mut AppConfig) {
        let Some(item) = self.items.get_selected_item() else {
            return;
        };

        if item.leading_state != State::RunConfig {
            return;
        }

        let Some(preset) = app_config.get_mut_preset_by_name(&item.name) else {
            return;
        };

        self.current_preset = Some(preset.clone());
        self.handle_state_change(("", State::EditPreset), Some(&app_config));
    }
    pub fn choose_item(&mut self, app_config: &AppConfig) {
        if self.popup.active {
            self.popup.deactivate_popup();
        }
        let Some(item) = self.items.get_selected_item() else {
            return;
        };

        self.handle_state_change((item.name.as_str(), item.leading_state), Some(app_config));
    }

    pub fn handle_state_change(
        &mut self,
        (item_name, new_state): (&str, State),
        app_config: Option<&AppConfig>
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
        self.selected_input.clear();
        self.cursor_idx = 0;
        self.messages.clear();

        match new_state {
            State::CreatePreset => {
                self.items.items.clear();
                let new_prompts = self.state.create_prompts().unwrap();

                for prompt in new_prompts {
                    self.prompts.push(prompt);
                }
                self.input_mode = InputMode::Input;
                self.items.list_state.select(Some(0));
            }
            State::ChoosePreset => {
                self.items.items.clear();
                let Some(config) = app_config else {
                    return;
                };

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
                self.items.list_state.select(Some(0));
            }
            State::EditPreset => {
                self.input_mode = InputMode::Normal;
                self.items.items.clear();
                match app_config {
                    Some(config) if !config.presets.is_empty() => {
                        let mut new_items = self.current_preset.clone().unwrap().into_items();

                        self.items.items.append(&mut new_items);
                    }
                    _ => {}
                }
            }
            State::ChangeFieldName => {
                self.input_mode = InputMode::Edit;
                let index = item_name.find(":").unwrap();
                let new_input = item_name[index + 1..].trim();
                self.input = new_input.to_string();
                self.cursor_idx = self.input.len();
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
                if let Some(config) = app_config {
                    self.debug_mode = config.settings.debug_mode;
                }
            }
        }
    }
}

impl AppConfig {
    pub fn get_preset_by_name(&self, name: &str) -> Option<&Preset> {
        if let Some(found) = self.presets.iter().find(|preset| preset.name == name) {
            Some(found)
        } else {
            None
        }
    }
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

    pub fn default() -> AppConfig {
        let settings = Settings::default();

        let app_config = AppConfig {
            presets: vec![],
            settings,
        };

        app_config
    }

    pub fn add_presets(&mut self, presets: Vec<Preset>) {
        for preset in presets {
            self.presets.push(preset);
        }
    }

    pub fn write_preset_to_file(
        &mut self,
        app_messages: &Vec<String>,
        write_type: WriteType,
        config_path: &str
    ) -> Result<(), ()> {
        if let WriteType::Create = write_type {
            let new_preset = Preset::from_input(app_messages);
            self.presets.push(new_preset);
        }

        let config_file = File::create(config_path);

        if config_file.is_err() {
            error!("Error while reading the config file.");
            panic!();
        }

        let mut writer = BufWriter::new(config_file.unwrap());
        serde_json::to_writer(&mut writer, &self).unwrap();
        writer.flush().unwrap();

        Ok(())
    }

    pub fn create_wt_command(&self, selected_name: &str) -> Result<(String, String), String> {
        let Some(preset) = self.get_preset_by_name(selected_name) else {
            error!("Cannot find the matching preset name while trying to run the config.");
            panic!();
        };

        let wt_profile = &preset.preset_info.wt_profile;
        let wt_profile = match wt_profile.is_empty() {
            false => format!(" -p \"{}\"", wt_profile),
            _ => wt_profile.to_string(),
        };

        let init_shell = &preset.preset_info.init_shell;
        let target_shell = &preset.preset_info.target_shell;

        let mut init_shell_name = init_shell.as_string();

        let command_runner = match target_shell {
            ShellType::WindowsPowershell => "powershell -NoExit -Command",
            ShellType::Powershell => "pwsh -NoExit -Command",
            ShellType::Cmd => "cmd /k",
            ShellType::Bash => "wsl ~ -e bash -l -i -c",
            ShellType::Zsh => "wsl ~ -e zsh -l -i -c",
            ShellType::Fish => "wsl ~ -e fish -l -i -c",
        };

        let mut args = preset.args.clone();

        for arg in args.iter_mut() {
            let mut temp: String = arg
                .split(",")
                .map(|s| format!("{}\\;", s))
                .collect();

            match *target_shell {
                ShellType::Zsh | ShellType::Bash | ShellType::Fish => {
                    temp.push_str(&format!("exec {}\\;", target_shell.as_string()));
                }
                _ => {}
            }

            *arg = format!("{} '{}'", command_runner, temp);
        }

        let escape_char = match init_shell_name.as_str() {
            "powershell" => "`",
            _args => "",
        };

        let w_len = preset.windows.len();

        let mut windows: Vec<String> = Vec::with_capacity(w_len);

        let mut arg_idx = vec![0; preset.windows.iter().sum::<u8>() as usize];
        arg_idx
            .iter_mut()
            .enumerate()
            .for_each(|(i, x)| {
                *x = i;
            });
        for window in &preset.windows {
            match window {
                1 =>
                    windows.push(
                        format!("{} {}", wt_profile, args.get(arg_idx.remove(0)).unwrap())
                    ),
                2 =>
                    windows.push(
                        format!(
                            "{} {}{}; sp{} {}",
                            wt_profile,
                            args.get(arg_idx.remove(0)).unwrap(),
                            escape_char,
                            wt_profile,
                            args.get(arg_idx.remove(0)).unwrap()
                        )
                    ),
                3 =>
                    windows.push(
                        format!(
                            "{} {}{}; sp{} -s .66 {}{}; sp{} -s .5 {}",
                            wt_profile,
                            args.get(arg_idx.remove(0)).unwrap(),
                            escape_char,
                            wt_profile,
                            args.get(arg_idx.remove(0)).unwrap(),
                            escape_char,
                            wt_profile,
                            args.get(arg_idx.remove(0)).unwrap()
                        )
                    ),
                4 =>
                    windows.push(
                        format!(
                            "{} {}{}; sp{} {}{}; sp{} {}{}; mf left{}; sp{} {}",
                            wt_profile,
                            args.get(arg_idx.remove(0)).unwrap(),
                            escape_char,
                            wt_profile,
                            args.get(arg_idx.remove(0)).unwrap(),
                            escape_char,
                            wt_profile,
                            args.get(arg_idx.remove(0)).unwrap(),
                            escape_char,
                            escape_char,
                            wt_profile,
                            args.get(arg_idx.remove(0)).unwrap()
                        )
                    ),
                _ => {}
            }
        }

        for s in &mut windows[0..w_len - 1].iter_mut() {
            s.push_str(&format!("`; nt"));
        }

        let windows = windows.into_iter().collect::<String>();

        let arg = format!("wt.exe{}", windows);
        log::warn!("{}", &arg);

        init_shell_name.push_str(".exe");

        Ok((init_shell_name, arg))
    }

    pub fn new(presets: Vec<Preset>, settings: Settings) -> AppConfig {
        AppConfig {
            presets,
            settings,
        }
    }
}

impl Settings {
    pub fn change_name(&mut self, index: usize, new_name: &str) -> Result<(), String> {
        match index {
            0 => {
                match new_name {
                    "true" => {
                        self.debug_mode = true;
                    }
                    "false" => {
                        self.debug_mode = false;
                    }
                    _ => {
                        return Err(
                            String::from(
                                "Cannot change the setting name: EXPECTED 'true' or 'false'."
                            )
                        );
                    }
                }
            }
            _ => {
                return Err(String::from("Cannot change the setting name: INDEX OUT OF BOUNDS."));
            }
        }
        Ok(())
    }

    pub fn default() -> Settings {
        Settings {
            debug_mode: false,
        }
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
                    return Err(
                        String::from(
                            "PresetValue, update_value error parsing tabs: CANNOT PARSE GIVEN STRING."
                        )
                    );
                };
                if new_tabs > 10 {
                    return Err(
                        String::from(
                            "PresetValue, update_value error creating tabs: INVALID AMOUNT OF TABS"
                        )
                    );
                }
                *tabs = new_tabs;
            }
            PresetValue::Windows(_, windows) => {
                let Ok(new_windows) = new_val.parse::<u8>() else {
                    return Err(
                        String::from(
                            "PresetValue update_value error parsing windows: CANNOT PARSE GIVEN STRING."
                        )
                    );
                };
                if new_windows > 4 {
                    return Err(
                        String::from(
                            "PresetValue, update_value error creating windows: INVALID AMOUNT OF WINDOWS"
                        )
                    );
                }
                *windows = new_windows;
            }
            PresetValue::Args(_, arg) => {
                *arg = new_val.to_string();
            }
            PresetValue::PresetInfo(preset_info_value) =>
                match preset_info_value {
                    PresetInfoValue::WtProfile(name) => {
                        *name = new_val.to_string();
                    }
                    PresetInfoValue::InitShell(init_shell) => {
                        *init_shell = match new_val {
                            "powershell" | "cmd" | "pwsh" => ShellType::from_str(new_val)?,
                            _ => {
                                return Err(
                                    String::from(
                                        "PresetValue update_value error parsing PresetInfo: INIT SHELL MUST BE 'powershell' OR 'cmd' OR 'pwsh'."
                                    )
                                );
                            }
                        };
                    }
                    PresetInfoValue::TargetShell(target_shell) => {
                        *target_shell = ShellType::from_str(new_val)?;
                    }
                    _ => {
                        return Err(
                            String::from(
                                "PresetValue update_value error parsing PresetInfo: INDEX OUT OF BOUNDS."
                            )
                        );
                    }
                }
        }

        Ok(())
    }
}

impl ShellType {
    pub fn as_string(&self) -> String {
        match self {
            ShellType::WindowsPowershell => "powershell".to_string(),
            ShellType::Powershell => "pwsh".to_string(),
            ShellType::Cmd => "cmd".to_string(),
            ShellType::Bash => "bash".to_string(),
            ShellType::Zsh => "zsh".to_string(),
            ShellType::Fish => "fish".to_string(),
        }
    }

    pub fn from_str(name: &str) -> Result<Self, String> {
        match name {
            "powershell" => Ok(ShellType::WindowsPowershell),
            "pwsh" => Ok(ShellType::Powershell),
            "cmd" => Ok(ShellType::Cmd),
            "bash" => Ok(ShellType::Bash),
            "zsh" => Ok(ShellType::Zsh),
            "fish" => Ok(ShellType::Fish),
            _ => {
                return Err("Incorrect shell name.".to_string());
            }
        }
    }
}

impl Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellType::WindowsPowershell => write!(f, "powershell"),
            ShellType::Powershell => write!(f, "pwsh"),
            ShellType::Cmd => write!(f, "cmd"),
            ShellType::Bash => write!(f, "bash"),
            ShellType::Zsh => write!(f, "zsh"),
            ShellType::Fish => write!(f, "fish"),
        }
    }
}

impl PresetInfo {
    pub fn default() -> PresetInfo {
        PresetInfo {
            wt_profile: String::new(),
            init_shell: ShellType::WindowsPowershell,
            target_shell: ShellType::WindowsPowershell,
        }
    }

    pub fn new(wt_profile: String, init_shell: ShellType, target_shell: ShellType) -> PresetInfo {
        PresetInfo {
            wt_profile,
            init_shell,
            target_shell,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presetvalue_update_success() {
        let mut pv = PresetValue::Args(0, "Not".to_string());
        let target = PresetValue::Args(0, "Changed".to_string());
        pv.update_value("Changed").unwrap();

        assert_eq!(pv, target);
    }

    #[test]
    #[should_panic]
    fn presetvalue_update_fail_windows() {
        let mut pv = PresetValue::Windows(0, 4);
        pv.update_value("5").unwrap();
    }

    #[test]
    #[should_panic]
    fn presetvalue_update_fail_tabs() {
        let mut pv = PresetValue::Tabs(10);
        pv.update_value("11").unwrap();
    }
}
