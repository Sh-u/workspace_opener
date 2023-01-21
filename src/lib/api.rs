use super::model::{App, AppConfig, InputMode, Preset, PresetCreationHelper, State, WriteType};
use crate::lib::model::ShellType;
use crossterm::event::{self, Event, KeyCode};
use log::*;
use std::{
    fs::{self, File},
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
            // controls.push(Span::raw(", "));
            // controls.push(Span::styled(
            //     "EDIT MODE ACTIVATED",
            //     Style::default().add_modifier(Modifier::BOLD).fg(edit_color),
            // ));
        }
        State::ChoosePreset => {
            controls.push(Span::styled(
                ", E",
                Style::default().add_modifier(Modifier::BOLD),
            ));
            controls.push(Span::raw(" to edit"));
            controls.push(Span::styled(
                ", DEL",
                Style::default().add_modifier(Modifier::BOLD),
            ));
            controls.push(Span::raw(" to delete"));
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

        // controls.push(Span::raw(format!(", prompts len: {}", app.prompts.len())));
        // controls.push(Span::raw(format!(", msg len: {}", app.messages.len())));
        // let (item, index) = app.items.get_selected_item().unwrap();
        // controls.push(Span::raw(format!(
        //     ", item: {:?}, index: {:?}",
        //     item,
        //     app.items.get_selected_item_index().unwrap()
        // )));
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
                    let _lines = vec![Spans::from(item.name.as_str())];
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

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    let cfg_file_string = fs::read_to_string(CONFIG).unwrap();
    let mut app_config: AppConfig = serde_json::from_str(&cfg_file_string).unwrap();
    let mut pch = PresetCreationHelper::new();
    loop {
        if app.state == State::RunConfig {
            let selected_item = app
                .items
                .get_selected_item()
                .expect("There is no selected item when trying to run the config.");

            let (program, arg) =
                create_wt_command(selected_item.name.as_str(), &mut app_config).unwrap();

            run_config(program, arg);

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
                        if let Err(_) = handle_creating_preset(&mut pch, app) {
                            continue;
                        };

                        let input = format_input(&app.input.drain(..).collect::<String>());

                        app.messages.push(input);

                        if app.messages.len() == app.prompts.len() {
                            pch.reset();

                            write_preset_to_file(
                                &mut app_config,
                                &app.messages,
                                WriteType::Create,
                                CONFIG,
                            )
                            .unwrap();

                            app.popup
                                .activate_popup("Preset created successfuly :)", Color::Green);

                            app.handle_state_change(("", app.previous_state), None);
                        }
                    }

                    KeyCode::Esc => {
                        if app.popup.active {
                            app.popup.deactivate_popup();
                            continue;
                        }
                        pch.reset();
                        app.handle_state_change(("", State::Start), None);
                    }

                    _ => {}
                },
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('e') => {
                        let Some(item) = app.items.get_selected_item() else {continue;};

                        if item.leading_state != State::RunConfig {
                            continue;
                        };

                        let Some(preset) = app_config.get_mut_preset_by_name(&item.name) else {continue;};

                        app.current_preset = Some(preset.clone());
                        app.handle_state_change(("", State::EditPreset), Some(&app_config));
                    }
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    KeyCode::Enter => {
                        let Some(item) = app.items.get_selected_item() else {continue;};

                        app.handle_state_change(
                            (item.name.as_str(), item.leading_state),
                            Some(&app_config),
                        );
                    }
                    KeyCode::Esc => {
                        if app.popup.active {
                            app.popup.deactivate_popup();
                            continue;
                        }

                        app.handle_state_change(("", app.previous_state), Some(&app_config));
                    }
                    KeyCode::Delete => {
                        let Some(item) = app.items.get_selected_item() else {continue;};
                        if let Err(err) = app_config.delete_preset_by_name(&item.name) {
                            error!("{}", err);
                            continue;
                        }
                        if let Err(err) = app.items.delete_selected_item() {
                            error!("{}", err);
                            continue;
                        }
                        write_preset_to_file(
                            &mut app_config,
                            &app.messages,
                            WriteType::Edit,
                            CONFIG,
                        )
                        .expect("Error when writing to a file of a deleted preset.");
                        if app.items.items.is_empty() {
                            app.handle_state_change(("", app.previous_state), Some(&app_config));
                        }
                    }
                    _ => {}
                },
                InputMode::Edit => match key.code {
                    KeyCode::Enter => {
                        let Some(item) = app.items.get_selected_item() else {continue;};

                        if let Some(pr) = &app.current_preset {
                            let Some(pr) = app_config.get_mut_preset_by_name(&pr.name) else{continue;};
                            let Some(mut pr_value) = item.preset_value else {continue;};

                            let input = format_input(&app.input);

                            if let Err(err) = pr_value.update_value(&input) {
                                error!("{}", err);
                                continue;
                            }

                            if let Err(err) = pr.change_field_value(pr_value) {
                                error!("{}", err);
                                continue;
                            }

                            app.current_preset = Some(pr.clone());
                        } else {
                            let Some(index) = app.items.get_selected_item_index() else {continue;};
                            if let Err(err) = app_config.settings.change_name(index, &app.input) {
                                error!("{}", err);
                                continue;
                            }
                        }

                        write_preset_to_file(
                            &mut app_config,
                            &app.messages,
                            WriteType::Edit,
                            CONFIG,
                        )
                        .expect("Error when writing to a file of an edited preset.");

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

pub fn create_wt_command(
    selected_name: &str,
    app_config: &AppConfig,
) -> Result<(String, String), String> {
    let Some(preset) = app_config.get_preset_by_name(selected_name) else {
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
        ShellType::Bash => "wsl ~ -e bash -c",
        ShellType::Zsh => "wsl ~ -e zsh -c",
    };

    let mut args = preset.args.clone();

    for arg in args.iter_mut() {
        let mut temp: String = arg.split(",").map(|s| format!("{}\\;", s)).collect();
        if *target_shell == ShellType::Zsh || *target_shell == ShellType::Bash {
            temp.push_str(&format!("exec {}\\;", target_shell.as_string()));
        }

        *arg = format!("{} '{}'", command_runner, temp);
        warn!("arg: {}", &arg);
    }

    let escape_char = match init_shell_name.as_str() {
        "powershell" => "`",
        _args => "",
    };

    let w_len = preset.windows.len();

    let mut windows: Vec<String> = Vec::with_capacity(w_len);

    let mut arg_idx = vec![0; preset.windows.iter().sum::<u8>() as usize];
    arg_idx.iter_mut().enumerate().for_each(|(i, x)| *x = i);
    for window in &preset.windows {
        match window {
            1 => windows.push(format!(
                "{} {}",
                wt_profile,
                args.get(arg_idx.remove(0)).unwrap(),
            )),
            2 => windows.push(format!(
                "{} {}{}; sp{} {}",
                wt_profile,
                args.get(arg_idx.remove(0)).unwrap(),
                escape_char,
                wt_profile,
                args.get(arg_idx.remove(0)).unwrap()
            )),
            3 => windows.push(format!(
                "{} {}{}; sp{} -s .66 {} {}; sp{} -s .5 {}",
                wt_profile,
                args.get(arg_idx.remove(0)).unwrap(),
                escape_char,
                wt_profile,
                args.get(arg_idx.remove(0)).unwrap(),
                escape_char,
                wt_profile,
                args.get(arg_idx.remove(0)).unwrap()
            )),
            4 => windows.push(format!(
                "{} {}{}; sp{} {} {}; sp{} {} {}; mf left {}; sp{} {}",
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
                args.get(arg_idx.remove(0)).unwrap(),
            )),
            _ => {}
        }
    }

    for s in &mut windows[0..w_len - 1].iter_mut() {
        s.push_str(&format!("`; nt",));
    }

    let windows = windows.into_iter().collect::<String>();

    let arg = format!("wt.exe{}", windows);
    warn!("{}", &arg);

    init_shell_name.push_str(".exe");

    Ok((init_shell_name, arg))
}

fn run_config(program: String, arg: String) {
    let mut _process = std::process::Command::new(program)
        .arg(arg)
        .spawn()
        .expect("Failed to launch the target process.");
}

pub fn write_preset_to_file(
    app_config: &mut AppConfig,
    app_messages: &Vec<String>,
    write_type: WriteType,
    config_path: &str,
) -> Result<(), ()> {
    if let WriteType::Create = write_type {
        let new_preset = Preset::new(app_messages);
        app_config.presets.push(new_preset);
    }

    let config_file = File::create(config_path);

    if config_file.is_err() {
        error!("Error while reading the config file.");
        panic!();
    }

    let mut writer = BufWriter::new(config_file.unwrap());
    serde_json::to_writer(&mut writer, &app_config).unwrap();
    writer.flush().unwrap();

    Ok(())
}

fn format_input(input: &str) -> String {
    let mut input = input.trim();
    if input.ends_with(",") {
        input = &input[..input.len() - 1];
    }

    input.to_string()
}

fn handle_creating_preset(pch: &mut PresetCreationHelper, app: &mut App) -> Result<(), ()> {
    let msg_length = app.messages.len();
    if msg_length == 1 {
        let Ok(tabs_amount) = app.input.parse::<u8>() else {return Err(())};
        if tabs_amount < 1 || tabs_amount > 10 {
            return Err(());
        };

        for n in 1..=tabs_amount {
            app.prompts
                .push(format!("Enter windows amount (1-4) for tab number {}: ", n));
        }
        pch.max_windows = tabs_amount as usize;
    } else if msg_length > 1 && pch.windows.len() < pch.max_windows {
        let Ok(input) = app.input.parse::<u8>() else {return Err(());};
        if input < 1 || input > 4 {
            return Err(());
        };
        pch.windows.push_back(input);

        let Some(&windows_back) = pch.windows.back() else {return Err(());};
        for n in 1..=windows_back {
            app.prompts
                .push(format!("Enter arg {} for window {}", n, pch.windows.len()));
        }
    }

    Ok(())
}

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
