use super::model::{ App, AppConfig, InputMode, PresetCreationHelper, State };
use crossterm::event::{ self, Event, KeyCode };
use std::fs::{ self };
use tui::{
    backend::Backend,
    layout::{ Constraint, Direction, Layout, Rect },
    style::{ Color, Modifier, Style },
    text::{ Span, Spans, Text },
    widgets::{ Block, Borders, List, ListItem, Paragraph },
    Frame,
    Terminal,
};

const CONTROL_MODIFIER: crossterm::event::KeyModifiers = crossterm::event::KeyModifiers::CONTROL;
const SHIFT_MODIFIER: crossterm::event::KeyModifiers = crossterm::event::KeyModifiers::SHIFT;
pub const CONFIG: &'static str = "config.json";

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    let cfg_file_string = fs::read_to_string(CONFIG).unwrap();
    let mut app_config: AppConfig = serde_json::from_str(&cfg_file_string).unwrap();
    let mut pch = PresetCreationHelper::new();

    loop {
        if app.state == State::RunConfig {
            let selected_item = app.items
                .get_selected_item()
                .expect("There is no selected item when trying to run the config.");

            let (program, arg) = app_config.create_wt_command(selected_item.name.as_str()).unwrap();

            run_config(program, arg);

            break;
        }

        terminal.draw(|f| ui(f, app)).unwrap();

        let event = event::read().unwrap();

        if let Event::Key(key) = event {
            match app.input_mode {
                InputMode::Input => {
                    if key.modifiers == CONTROL_MODIFIER {
                        app.handle_control_key_action(key.code);
                        continue;
                    } else if key.modifiers == SHIFT_MODIFIER {
                        app.handle_shift_key_action(key.code);
                        continue;
                    }

                    match key.code {
                        KeyCode::Char(ch) => {
                            app.insert_char(ch);
                        }
                        KeyCode::Backspace => {
                            app.delete_characters();
                        }
                        KeyCode::Left => {
                            app.move_cursor_left();
                        }
                        KeyCode::Right => {
                            app.move_cursor_right();
                        }

                        KeyCode::Enter => {
                            app.handle_creating_preset(&mut pch, &mut app_config);
                        }

                        KeyCode::Esc => {
                            app.cancel_preset_creation(&mut pch);
                        }

                        _ => {}
                    }
                }
                InputMode::Normal =>
                    match key.code {
                        KeyCode::Char('q') => {
                            break;
                        }
                        KeyCode::Char('e') => {
                            app.edit_preset(&mut app_config);
                        }
                        KeyCode::Down => app.items.next(),
                        KeyCode::Up => app.items.previous(),
                        KeyCode::Enter => {
                            app.choose_item(&mut app_config);
                        }
                        KeyCode::Esc => {
                            app.go_back(&app_config);
                        }
                        KeyCode::Delete => {
                            app.handle_deleting_preset(&mut app_config);
                        }
                        _ => {}
                    }
                InputMode::Edit => {
                    if key.modifiers == CONTROL_MODIFIER {
                        app.handle_control_key_action(key.code);
                        continue;
                    } else if key.modifiers == SHIFT_MODIFIER {
                        app.handle_shift_key_action(key.code);
                        continue;
                    }

                    match key.code {
                        KeyCode::Char(ch) => {
                            app.insert_char(ch);
                        }
                        KeyCode::Backspace => {
                            app.delete_characters();
                        }
                        KeyCode::Left => {
                            app.move_cursor_left();
                        }
                        KeyCode::Right => {
                            app.move_cursor_right();
                        }
                        KeyCode::Enter => {
                            app.handle_editing_preset(&mut app_config);
                        }
                        KeyCode::Esc => {
                            app.handle_state_change(("", app.previous_state), Some(&app_config));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(f.size());

    let edit_color = Color::Rgb(51, 153, 255);

    let mut controls = vec![
        Span::raw("Press "),
        Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to go back"),
        Span::styled(", q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to exit")
    ];

    let main_block = Block::default().borders(Borders::ALL);
    let mut main_block_style = Style::default().bg(Color::Black);
    let highlight_style = Style::default().bg(edit_color);

    let input_block = Block::default().title("Input").borders(Borders::ALL);

    match app.state {
        State::EditPreset | State::ChangeFieldName => {
            main_block_style = main_block_style.fg(edit_color);
        }
        State::ChoosePreset => {
            controls.push(Span::styled(", E", Style::default().add_modifier(Modifier::BOLD)));
            controls.push(Span::raw(" to edit"));
            controls.push(Span::styled(", DEL", Style::default().add_modifier(Modifier::BOLD)));
            controls.push(Span::raw(" to delete"));
        }
        _ => {}
    }

    if app.popup.active {
        let popup_block = Block::default().borders(Borders::ALL);
        let area = centered_rect(60, 20, size);
        let popup_message = Paragraph::new(Span::from(app.popup.message.to_string())).style(
            Style::default().fg(app.popup.color)
        );
        f.render_widget(popup_message.block(popup_block), area);
    }

    if app.debug_mode {
        controls.push(Span::raw(", State:"));
        controls.push(
            Span::styled(
                format!(" {:?}", app.get_state()),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Red)
            )
        );

        controls.push(Span::raw(", InputMode:"));
        controls.push(
            Span::styled(
                format!(" {:?}", app.input_mode),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::LightYellow)
            )
        );

        controls.push(Span::raw(", Cursor:"));
        controls.push(
            Span::styled(
                format!(" {:?}", app.cursor_idx),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::LightGreen)
            )
        );

        controls.push(Span::raw(", Selected::"));
        controls.push(
            Span::styled(
                format!(" {:?}", app.selected_input),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Magenta)
            )
        );
    }

    match app.input_mode {
        InputMode::Input => {
            let mut prompts: Vec<ListItem> = Vec::new();

            for (index, prompt) in app.prompts
                .iter()
                .take(app.messages.len() + 1)
                .enumerate() {
                prompts.push(ListItem::new(Span::from(prompt.as_str())));
                if let Some(msg) = app.messages.get(index) {
                    prompts.push(
                        ListItem::new(Span::from(msg.as_str())).style(
                            Style::default().fg(edit_color).add_modifier(Modifier::ITALIC)
                        )
                    );
                }
            }

            let prompts = List::new(prompts).block(main_block.clone());

            let input_block = input_block.style(Style::default().fg(edit_color));

            let spans = create_spans(&app.input, &app.selected_input, edit_color);

            let user_input = Paragraph::new(Text::from(Spans::from(spans)));

            f.render_widget(prompts, chunks[1]);
            f.render_widget(user_input.block(input_block), chunks[2]);

            f.set_cursor(chunks[2].x + (app.cursor_idx as u16) + 1, chunks[2].y + 1);
        }
        _ => {
            let items = app.items.items
                .iter()
                .map(|item| {
                    let _lines = vec![Spans::from(item.name.as_str())];
                    ListItem::new(_lines).style(Style::default().fg(Color::White))
                })
                .collect::<Vec<ListItem>>();

            if let InputMode::Edit = app.input_mode {
                let input_block = input_block.style(Style::default().fg(edit_color));
                let spans = create_spans(&app.input, &app.selected_input, edit_color);
                let user_input = Paragraph::new(Text::from(Spans::from(spans)));

                f.render_widget(user_input.block(input_block), chunks[2]);
                f.set_cursor(chunks[2].x + (app.cursor_idx as u16) + 1, chunks[2].y + 1);
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

fn create_spans<'a>(input: &'a str, indices: &'a [usize], edit_color: Color) -> Vec<Span<'a>> {
    let mut spans: Vec<Span<'a>> = vec![];

    if indices.len() > 0 {
        let first_select_idx = *indices.first().unwrap();
        let last_selected_idx = *indices.last().unwrap();

        if indices.len() == input.len() {
            spans.push(Span::styled(input, Style::default().bg(Color::Cyan)));
        } else if first_select_idx > 0 {
            let substr_before = &input[0..first_select_idx];
            spans.push(Span::styled(substr_before, Style::default().fg(edit_color)));

            let substr_highlighted = &input[first_select_idx..=last_selected_idx];
            spans.push(Span::styled(substr_highlighted, Style::default().bg(Color::Cyan)));

            if last_selected_idx != input.len() - 1 {
                let substr_after = &input[last_selected_idx + 1..];
                spans.push(Span::styled(substr_after, Style::default().fg(edit_color)));
            }
        } else {
            let substr_highlighted = &input[..=last_selected_idx];
            let substr_after = &input[last_selected_idx + 1..];
            spans.push(Span::styled(substr_highlighted, Style::default().bg(Color::Cyan)));
            spans.push(Span::styled(substr_after, Style::default().fg(edit_color)));
        }
    } else {
        spans.push(Span::styled(input, Style::default().fg(edit_color)));
    }

    spans
}
fn run_config(program: String, arg: String) {
    let mut _process = std::process::Command
        ::new(program)
        .arg(arg)
        .spawn()
        .expect("Failed to launch the target process.");
}

fn centered_rect(percent_x: u16, percent_y: u16, rect: Rect) -> Rect {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ].as_ref()
        )
        .split(rect);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ].as_ref()
        )
        .split(layout[1])[1]
}
