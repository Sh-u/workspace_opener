use super::model::{App, AppConfig, InputMode, Preset, State, WriteType};
use crossterm::event::{self, Event, KeyCode};
use log::*;
use std::io::BufReader;
use std::iter;
use std::process::Stdio;
use std::{
    fs::{self, read_to_string, File},
    io::{BufWriter, Write},
};

use std::thread::sleep;
use std::time::Duration;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
const CONFIG: &'static str = "config.json";

const PATH: &str = "../../../../mnt/c/Users/kacperek/AppData/Local/Microsoft/WindowsApps/wt.exe";
// const PATH: &'static str = "../../../../bin/fish";

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
        State::EditPreset | State::ChangeFieldName => {
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

    if app.debug_mode {
        controls.push(Span::raw(", State:"));
        controls.push(Span::styled(
            format!(" {:?}", app.get_state()),
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
        ));

        controls.push(Span::raw(", InputMode:"));
        controls.push(Span::styled(
            format!(" {:?}", app.input_mode),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::LightYellow),
        ));

        controls.push(Span::raw(format!(", prompts len: {}", app.prompts.len())));
        controls.push(Span::raw(format!(", msg len: {}", app.messages.len())));
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

            f.render_stateful_widget(items, chunks[1], &mut app.items.list_state);
        }
    }
    let controls = Paragraph::new(Text::from(Spans::from(controls)));

    f.render_widget(controls, chunks[0]);
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

fn run_config(selected_name: &str, app_config: &mut AppConfig) {
    let Some(preset) = app_config
        .find_preset_by_name(selected_name)
        .map(|val| &*val) else {
            error!("Cannot find the matching preset name while trying to run the config.");
            panic!();
        };

    let arg = String::new();

    let mut process = std::process::Command::new("powershell.exe")
        .arg("wt.exe split-pane -p \"Command Prompt\"")
        .spawn()
        .expect("Failed to launch the target process.");
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    let cfg_file_string = fs::read_to_string(CONFIG).unwrap();
    let mut app_config: AppConfig = serde_json::from_str(&cfg_file_string).unwrap();
    let mut windows = vec![];
    let mut done = false;
    loop {
        if app.state == State::RunConfig {
            let selected_item = app
                .items
                .get_selected_item()
                .expect("There is no selected item when trying to run the config.");
            run_config(selected_item.0.as_str(), &mut app_config);
            break;
        }

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

                        if msg_length == 1 {
                            let Ok(tabs_amount) = app.input.as_str().trim().parse::<u8>() else {continue;};

                            for n in 1..=tabs_amount {
                                app.prompts
                                    .push(format!("Enter windows amount for tab number {}: ", n));
                            }
                            windows.reserve(tabs_amount as usize);
                        } else if msg_length > 1 {
                            if windows.capacity() != windows.len() {
                                windows.push(app.input.parse::<u8>().unwrap());
                            } else if !done {
                                for (index, &windows_amount) in windows.iter().enumerate() {
                                    for n in 1..=windows_amount {
                                        app.prompts
                                            .push(format!("Enter arg {} for window {}", n, index));
                                    }
                                }

                                done = true;
                            }
                        }

                        app.messages.push(app.input.drain(..).collect());

                        if app.messages.len() == app.prompts.len() {
                            windows.clear();

                            if write_preset_to_file(
                                &mut app_config,
                                &app.messages,
                                WriteType::Create,
                            )
                            .is_ok()
                            {
                                app.popup
                                    .activate_popup("Preset created successfuly :)", Color::Green);

                                app.handle_state_change(("", app.previous_state), None);
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
                                        Some(&app_config),
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
                            app.handle_state_change((i.0.as_str(), i.1), Some(&app_config));
                        }
                    }
                    KeyCode::Esc => {
                        if app.popup.active {
                            app.popup.deactivate_popup();
                            continue;
                        }

                        app.handle_state_change(("", app.previous_state), Some(&app_config));
                    }
                    _ => {}
                },
                InputMode::Edit => match key.code {
                    KeyCode::Enter => {
                        let Some(index) = app.items.get_selected_item_index() else {continue;};

                        if let Some(pr) = &app.current_preset {
                            if let Some(pr) = app_config.find_preset_by_name(&pr.name) {
                                if let Err(err) = pr.change_name(index, &app.input) {
                                    error!("{}", err);
                                    continue;
                                }

                                app.current_preset = Some(pr.clone());
                            }
                        } else {
                            if let Err(err) = app_config.settings.change_name(index, &app.input) {
                                error!("{}", err);
                                continue;
                            }
                        }

                        write_preset_to_file(&mut app_config, &app.messages, WriteType::Edit)
                            .expect("Error when writing to a file of an edited preset.");

                        warn!("previous state: {:?}", app.previous_state);
                        app.handle_state_change(("", app.previous_state), Some(&app_config));
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.handle_state_change(("", app.previous_state), Some(&app_config));
                    }
                    _ => {}
                },
            }
        }
    }
}
