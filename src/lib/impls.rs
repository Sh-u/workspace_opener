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
}

impl Preset {
    fn new(input: &Vec<String>) -> Self {
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
}

impl Popup {
    fn default() -> Self {
        Popup {
            active: false,
            message: String::new(),
        }
    }

    fn activate_popup(&mut self, message: &str) {
        self.active = true;
        self.message = message.to_string();
    }

    fn deactivate_popup(&mut self) {
        self.active = false;
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
            popup: Popup::default(),
        }
    }
    fn get_state(&self) -> State {
        self.state.clone()
    }

    fn handle_state_change(&mut self, new_state: State, presets: Option<&Vec<Preset>>) {
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
            State::ChoosePreset => {
                if let Some(presets) = presets {
                    for preset in presets {
                        self.items
                            .items
                            .push((preset.name.to_string(), State::RunConfig));
                    }
                }
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
