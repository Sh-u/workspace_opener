fn centered_rect(percent_x: u16, percent_y: u16, rect: Rect) -> Rect {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(rect);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(layout[1])[1]
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(10)].as_ref())
        .split(f.size());

    let main_block = Block::default().title("Main").borders(Borders::ALL);

    let input_block = Block::default().title("Input").borders(Borders::ALL);

    if app.popup.active {
        let popup_block = Block::default().title("Popup").borders(Borders::ALL);
        let area = centered_rect(60, 20, size);
        let popup_message = Paragraph::new(Span::from(app.popup.message.to_string()));
        f.render_widget(popup_message.block(popup_block), area);
    }

    if app.items.items.is_empty() {
        let mut prompts: Vec<ListItem> = Vec::new();

        for (index, prompt) in app.prompts.iter().take(app.messages.len() + 1).enumerate() {
            prompts.push(ListItem::new(Span::from(prompt.as_str())));
            if let Some(msg) = app.messages.get(index) {
                prompts.push(
                    ListItem::new(Span::from(msg.as_str())).style(
                        Style::default()
                            .fg(Color::LightRed)
                            .add_modifier(Modifier::ITALIC),
                    ),
                )
            }
        }

        let prompts = List::new(prompts).block(main_block.clone());

        let input_block = input_block.style(Style::default().fg(Color::LightRed));

        let user_input = Paragraph::new(Text::from(app.input.as_str()));

        f.render_widget(prompts, chunks[0]);
        f.render_widget(user_input.block(input_block), chunks[1]);
        f.set_cursor(chunks[1].x + app.input.len() as u16 + 1, chunks[1].y + 1);
    } else {
        let items = app
            .items
            .items
            .iter()
            .map(|item| {
                let _lines = vec![Spans::from(item.0.as_str())];
                ListItem::new(_lines).style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
            })
            .collect::<Vec<ListItem>>();

        let items = List::new(items)
            .block(main_block)
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("> ");

        f.render_stateful_widget(items, chunks[0], &mut app.items.list_state);
        f.render_widget(input_block, chunks[1]);
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    let cfg_file_string = fs::read_to_string(CONFIG).unwrap();
    let mut app_config: AppConfig = serde_json::from_str(&cfg_file_string).unwrap();

    loop {
        terminal.draw(|f| ui(f, app)).unwrap();

        let event = event::read().unwrap();

        if let Event::Key(key) = event {
            match app.input_mode {
                InputMode::Editing => match key.code {
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Enter => {
                        if app.input.is_empty() {
                            continue;
                        }

                        let _prompts_len = app.prompts.len();
                        let msg_length = app.messages.len();

                        match msg_length {
                            x if x == _prompts_len && x != 2 && x != 3 => {
                                continue;
                            }
                            2 => {
                                let windows_number = app.input.as_str().trim().parse::<u8>();

                                if let Ok(num) = windows_number {
                                    for n in 1..=num {
                                        app.prompts.push(format!(
                                            "Enter args for window number {} (split by commas): ",
                                            n
                                        ));
                                    }
                                } else {
                                    continue;
                                }
                            }
                            _ => {}
                        }

                        app.messages.push(app.input.drain(..).collect());

                        if app.messages.len() == app.prompts.len() {
                            let new_preset = Preset::new(&app.messages);
                            app_config.presets.push(new_preset);

                            app.messages.clear();
                            app.prompts.clear();

                            let config_file = File::create(CONFIG);

                            if config_file.is_err() {
                                error!("Error while reading the config file.");
                                panic!();
                            }

                            let mut writer = BufWriter::new(config_file.unwrap());
                            serde_json::to_writer(&mut writer, &app_config).unwrap();
                            writer.flush().unwrap();

                            app.popup.activate_popup(
                                "Preset created successfuly :) Press ESC to close popup.",
                            );
                            app.handle_state_change(State::Start, None);
                        }
                    }

                    KeyCode::Esc => {
                        if app.popup.active {
                            app.popup.deactivate_popup()
                        }
                        app.handle_state_change(State::Start, None);
                    }

                    _ => {}
                },
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    KeyCode::Enter => {
                        let selected_item = app.items.get_selected_item();

                        if let Some(i) = selected_item {
                            app.handle_state_change(i.1, Some(&app_config.presets));
                        }
                    }
                    KeyCode::Esc => {
                        if app.popup.active {
                            app.popup.deactivate_popup()
                        }
                        if app.get_state() != State::Start {
                            app.handle_state_change(State::Start, None);
                        }
                    }
                    _ => {}
                },
            }
        }
    }
}