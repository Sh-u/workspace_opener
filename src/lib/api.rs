use super::model::{App, AppConfig, InputMode, Preset, State, WriteType};
use crossterm::event::{self, Event, KeyCode};
use log::*;
use std::iter;
use std::{
    fs::{self, read_to_string, File},
    io::{BufWriter, Write},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
const CONFIG: &'static str = "config.json";

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
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let edit_color = Color::Rgb(51, 153, 255);

    let mut controls = vec![
        Span::raw("Press "),
        Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to go back"),
        Span::styled(", q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to exit"),
    ];

    let main_block = Block::default().borders(Borders::ALL);
    let mut main_block_style = Style::default().bg(Color::Black);
    let highlight_style = Style::default().bg(edit_color);

    let input_block = Block::default().title("Input").borders(Borders::ALL);

    match app.state {
        State::EditPreset | State::ChangePresetField => {
            main_block_style = main_block_style.fg(edit_color);
            controls.push(Span::raw(", "));
            controls.push(Span::styled(
                "EDIT MODE ACTIVATED",
                Style::default().add_modifier(Modifier::BOLD).fg(edit_color),
            ));
        }
        State::ChoosePreset => {
            controls.push(Span::styled(
                ", E",
                Style::default().add_modifier(Modifier::BOLD),
            ));
            controls.push(Span::raw(" to edit preset"));
        }
        _ => {}
    }

    if app.popup.active {
        let popup_block = Block::default().borders(Borders::ALL);
        let area = centered_rect(60, 20, size);
        let popup_message = Paragraph::new(Span::from(app.popup.message.to_string()))
            .style(Style::default().fg(app.popup.color));
        f.render_widget(popup_message.block(popup_block), area);
    }

    match app.input_mode {
        InputMode::Input => {
            let mut prompts: Vec<ListItem> = Vec::new();

            for (index, prompt) in app.prompts.iter().take(app.messages.len() + 1).enumerate() {
                prompts.push(ListItem::new(Span::from(prompt.as_str())));
                if let Some(msg) = app.messages.get(index) {
                    prompts.push(
                        ListItem::new(Span::from(msg.as_str())).style(
                            Style::default()
                                .fg(edit_color)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    )
                }
            }

            let prompts = List::new(prompts).block(main_block.clone());

            let input_block = input_block.style(Style::default().fg(edit_color));

            let user_input = Paragraph::new(Text::from(app.input.as_str()));

            f.render_widget(prompts, chunks[1]);
            f.render_widget(user_input.block(input_block), chunks[2]);
            f.set_cursor(chunks[2].x + app.input.len() as u16 + 1, chunks[2].y + 1);
        }
        _ => {
            let items = app
                .items
                .items
                .iter()
                .map(|item| {
                    let _lines = vec![Spans::from(item.0.as_str())];
                    ListItem::new(_lines).style(Style::default().fg(Color::White))
                })
                .collect::<Vec<ListItem>>();

            if let InputMode::Edit = app.input_mode {
                let input_block = input_block.style(Style::default().fg(edit_color));

                let user_input = Paragraph::new(Text::from(app.input.as_str()));

                f.render_widget(user_input.block(input_block), chunks[2]);
                f.set_cursor(chunks[2].x + app.input.len() as u16 + 1, chunks[2].y + 1);
            } else {
                f.render_widget(input_block, chunks[2]);
            }

            let items = List::new(items)
                .block(main_block)
                .style(main_block_style)
                .highlight_style(highlight_style)
                .highlight_symbol("> ");

            controls.push(Span::raw("."));
            let controls = Paragraph::new(Text::from(Spans::from(controls)));

            f.render_widget(controls, chunks[0]);
            f.render_stateful_widget(items, chunks[1], &mut app.items.list_state);
        }
    }
}

fn write_preset_to_file(
    app_config: &mut AppConfig,
    app_messages: &Vec<String>,
    write_type: WriteType,
) -> Result<(), ()> {
    if let WriteType::Create = write_type {
        let new_preset = Preset::new(app_messages);
        app_config.presets.push(new_preset);
    }

    let config_file = File::create(CONFIG);

    if config_file.is_err() {
        error!("Error while reading the config file.");
        panic!();
    }

    let mut writer = BufWriter::new(config_file.unwrap());
    serde_json::to_writer(&mut writer, &app_config).unwrap();
    writer.flush().unwrap();

    Ok(())
}
pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    let cfg_file_string = fs::read_to_string(CONFIG).unwrap();
    let mut app_config: AppConfig = serde_json::from_str(&cfg_file_string).unwrap();

    loop {
        terminal.draw(|f| ui(f, app)).unwrap();

        let event = event::read().unwrap();

        if let Event::Key(key) = event {
            match app.input_mode {
                InputMode::Input => match key.code {
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
                            if write_preset_to_file(
                                &mut app_config,
                                &app.messages,
                                WriteType::Create,
                            )
                            .is_ok()
                            {
                                app.popup
                                    .activate_popup("Preset created successfuly :)", Color::Green);

                                app.handle_state_change(("", State::Start), None);
                            }
                        }
                    }

                    KeyCode::Esc => {
                        if app.popup.active {
                            app.popup.deactivate_popup();
                            continue;
                        }
                        app.handle_state_change(("", State::Start), None);
                    }

                    _ => {}
                },
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('e') => {
                        let selected_item = app.items.get_selected_item();
                        match selected_item {
                            Some(item) if item.1 == State::RunConfig => {
                                let preset = app_config.find_preset_by_name(&item.0);

                                if let Some(preset) = preset {
                                    app.current_preset = Some(preset.clone());
                                    app.handle_state_change(
                                        ("", State::EditPreset),
                                        Some(&app_config.presets),
                                    );
                                }
                            }
                            _ => {
                                continue;
                            }
                        }
                    }
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    KeyCode::Enter => {
                        let selected_item = app.items.get_selected_item();

                        if let Some(i) = selected_item {
                            app.handle_state_change((i.0.as_str(), i.1), Some(&app_config.presets));
                        }
                    }
                    KeyCode::Esc => {
                        if app.popup.active {
                            app.popup.deactivate_popup();
                            continue;
                        }

                        app.handle_state_change(("", State::Start), None);
                    }
                    _ => {}
                },
                InputMode::Edit => match key.code {
                    KeyCode::Enter => {
                        if let Some(pr) = &app.current_preset {
                            if let Some(pr) = app_config.find_preset_by_name(&pr.name) {
                                let index = app.items.get_selected_item_index().unwrap();

                                if let Err(err) = pr.change_name(index, &app.input) {
                                    error!("{}", err);
                                    continue;
                                }

                                app.current_preset = Some(pr.clone());
                            }
                        }
                        info!("current : {:?}", &app.current_preset);
                        write_preset_to_file(&mut app_config, &app.messages, WriteType::Edit)
                            .expect("Error when writing to a file of an edited preset.");

                        app.handle_state_change(("", State::EditPreset), Some(&app_config.presets));
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.handle_state_change(("", State::EditPreset), Some(&app_config.presets));
                    }
                    _ => {}
                },
            }
        }
    }
}
