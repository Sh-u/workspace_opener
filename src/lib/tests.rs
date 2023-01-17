use std::{fs, io::Read};

use serde_json::json;

use crate::lib::{api::write_preset_to_file, model::WriteType};

use super::model::{AppConfig, Item, Preset, PresetValue, Settings, ShellType, State};

#[test]
fn preset_creation_basic() {
    let target = Preset {
        name: String::from("Test Preset"),
        tabs: 3,
        windows: vec![2, 1, 1],
        args: vec![
            String::from("arg w1"),
            String::from("arg w1"),
            String::from("arg w2"),
            String::from("arg w3"),
        ],
    };

    let mut input: Vec<String> = vec![];
    input.push(String::from("Test Preset"));
    input.push(String::from("3"));
    input.push(String::from("2"));
    input.push(String::from("1"));
    input.push(String::from("1"));
    input.push(String::from("arg w1"));
    input.push(String::from("arg w1"));
    input.push(String::from("arg w2"));
    input.push(String::from("arg w3"));

    let from_input = Preset::new(&input);

    assert_eq!(target, from_input);
}
#[test]
fn preset_creation_large() {
    let mut args = vec![
        String::from("arg w1"),
        String::from("arg w1"),
        String::from("arg w2"),
        String::from("arg w3"),
    ];

    for n in 0..20 {
        args.push(format!("arg w4: {}", n))
    }
    let target = Preset {
        name: String::from("Test Preset"),
        tabs: 4,
        windows: vec![2, 1, 1, 20],
        args,
    };

    let mut input: Vec<String> = vec![];
    input.push(String::from("Test Preset"));
    input.push(String::from("4"));
    input.push(String::from("2"));
    input.push(String::from("1"));
    input.push(String::from("1"));
    input.push(String::from("20"));
    input.push(String::from("arg w1"));
    input.push(String::from("arg w1"));
    input.push(String::from("arg w2"));
    input.push(String::from("arg w3"));

    for n in 0..20 {
        input.push(format!("arg w4: {}", n))
    }

    let from_input = Preset::new(&input);

    assert_eq!(target, from_input);
}

#[test]
fn preset_deletion() {
    let preset = Preset {
        name: String::from("Test Preset"),
        tabs: 3,
        windows: vec![2, 1, 1],
        args: vec![
            String::from("arg w1"),
            String::from("arg w1"),
            String::from("arg w2"),
            String::from("arg w3"),
        ],
    };

    let settings = Settings {
        wt_profile: "Ubuntu".to_string(),
        init_shell: ShellType::Powershell,
        target_shell: ShellType::Zsh,
    };

    let mut app_config = AppConfig {
        presets: vec![preset],
        settings: settings.clone(),
    };

    let target = AppConfig {
        presets: vec![],
        settings,
    };

    let items = vec![String::from("Test Preset")];

    if let Err(err) = app_config.delete_preset_by_name(items.get(0).unwrap()) {
        panic!("{}", err);
    };

    assert_eq!(target, app_config);
}
#[test]
fn test_write_to_file_and_create() {
    let mut app_config = create_test_app_config();

    let app_messages = vec![
        String::from("Test Preset"),
        String::from("3"),
        String::from("2"),
        String::from("1"),
        String::from("1"),
        String::from("arg w1"),
        String::from("arg w1"),
        String::from("arg w2"),
        String::from("arg w3"),
    ];

    let config_path = "test.json";
    let test_string =  "{\"presets\":[{\"name\":\"Test Preset\",\"tabs\":3,\"windows\":[2,1,1],\"args\":[\"arg w1\",\"arg w1\",\"arg w2\",\"arg w3\"]}],\"settings\":{\"wt_profile\":\"Ubuntu\",\"init_shell\":\"powershell\",\"target_shell\":\"zsh\"}}".to_string();
    write_preset_to_file(
        &mut app_config,
        &app_messages,
        WriteType::Create,
        config_path,
    )
    .expect("Failed to write file");

    let mut file = fs::File::open(config_path).expect("Failed to open file");
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)
        .expect("Failed to read file");

    let assertion = file_contents == test_string.to_string();
    fs::remove_file(config_path).expect("Failed to delete file");
    if !assertion {
        panic!("");
    }
}

#[test]
fn test_write_to_file_and_delete() {
    let mut app_config = create_test_app_config();

    let app_messages = vec![
        String::from("Test Preset"),
        String::from("3"),
        String::from("2"),
        String::from("1"),
        String::from("1"),
        String::from("arg w1"),
        String::from("arg w1"),
        String::from("arg w2"),
        String::from("arg w3"),
    ];

    let preset = Preset::new(&app_messages);

    app_config.presets.push(preset);

    let config_path = "test2.json";
    app_config
        .delete_preset_by_name("Test Preset")
        .expect("Failed to delete preset");
    let test_string =
        "{\"presets\":[],\"settings\":{\"wt_profile\":\"Ubuntu\",\"init_shell\":\"powershell\",\"target_shell\":\"zsh\"}}".to_string();
    write_preset_to_file(&mut app_config, &app_messages, WriteType::Edit, config_path)
        .expect("Failed to write file");

    let mut file = fs::File::open(config_path).expect("Failed to open file");
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)
        .expect("Failed to read file");

    let assertion = file_contents == test_string.to_string();
    fs::remove_file(config_path).expect("Failed to delete file");
    if !assertion {
        panic!("");
    }
}
#[test]
fn test_into_items() {
    let preset = Preset {
        name: String::from("Test Preset"),
        tabs: 3,
        windows: vec![2, 1, 1],
        args: vec![
            String::from("arg w1"),
            String::from("arg w1"),
            String::from("arg w2"),
            String::from("arg w3"),
        ],
    };

    let items = preset.into_items();
    let mut target = vec![];

    let name = Item::new(
        format!("Name: {}", "Test Preset"),
        State::ChangeFieldName,
        Some(PresetValue::Name("Test Preset".to_string())),
    );

    target.push(name);

    let tabs = Item::new(
        format!("Tabs: {}", "3".to_string()),
        State::ChangeFieldName,
        Some(PresetValue::Tabs(3)),
    );
    target.push(tabs);

    let window = Item::new(
        format!("Tab (#{}), windows: {}", 1, 2),
        State::ChangeFieldName,
        Some(PresetValue::Windows(0, 2)),
    );

    target.push(window);

    let window = Item::new(
        format!("Tab (#{}), windows: {}", 2, 1),
        State::ChangeFieldName,
        Some(PresetValue::Windows(1, 1)),
    );

    target.push(window);

    let window = Item::new(
        format!("Tab (#{}), windows: {}", 3, 1),
        State::ChangeFieldName,
        Some(PresetValue::Windows(2, 1)),
    );

    target.push(window);

    let arg = Item::new(
        format!("Tab (#{}), window (#{}), Arg: {}", 1, 1, "arg w1"),
        State::ChangeFieldName,
        Some(PresetValue::Args(0, "arg w1".to_string())),
    );
    target.push(arg);

    let arg = Item::new(
        format!("Tab (#{}), window (#{}), Arg: {}", 1, 2, "arg w1"),
        State::ChangeFieldName,
        Some(PresetValue::Args(1, "arg w1".to_string())),
    );
    target.push(arg);

    let arg = Item::new(
        format!("Tab (#{}), window (#{}), Arg: {}", 2, 1, "arg w2"),
        State::ChangeFieldName,
        Some(PresetValue::Args(2, "arg w2".to_string())),
    );
    target.push(arg);

    let arg = Item::new(
        format!("Tab (#{}), window (#{}), Arg: {}", 3, 1, "arg w3"),
        State::ChangeFieldName,
        Some(PresetValue::Args(3, "arg w3".to_string())),
    );
    target.push(arg);

    for (index, item) in items.iter().enumerate() {
        assert_eq!(item, target.get(index).unwrap());
    }
}

#[test]
fn test_preset_change_field_value() {
    let mut preset = Preset {
        name: String::from("Test Preset"),
        tabs: 3,
        windows: vec![2, 1, 1],
        args: vec![
            String::from("arg w1"),
            String::from("arg w1"),
            String::from("arg w2"),
            String::from("arg w3"),
        ],
    };

    preset
        .change_field_value(PresetValue::Name("Preset Changed".to_string()))
        .expect("Failed changing preset's name");

    preset
        .change_field_value(PresetValue::Tabs(5))
        .expect("Failed changing preset's tabs");

    let target = Preset {
        name: String::from("Preset Changed"),
        tabs: 5,
        windows: vec![2, 1, 1, 1, 1],
        args: vec![
            String::from("arg w1"),
            String::from("arg w1"),
            String::from("arg w2"),
            String::from("arg w3"),
            String::from(""),
            String::from(""),
        ],
    };
    assert_eq!(preset, target);

    preset
        .change_field_value(PresetValue::Windows(2, 3))
        .expect("Failed changing preset's windows");

    let target = Preset {
        name: String::from("Preset Changed"),
        tabs: 5,
        windows: vec![2, 1, 3, 1, 1],
        args: vec![
            String::from("arg w1"),
            String::from("arg w1"),
            String::from("arg w2"),
            String::from("arg w3"),
            String::from(""),
            String::from(""),
            String::from(""),
            String::from(""),
        ],
    };
    assert_eq!(preset, target);

    preset
        .change_field_value(PresetValue::Args(4, "arg w3".to_string()))
        .expect("Failed changing preset's args");
    preset
        .change_field_value(PresetValue::Args(5, "arg w3".to_string()))
        .expect("Failed changing preset's args");

    let target = Preset {
        name: String::from("Preset Changed"),
        tabs: 5,
        windows: vec![2, 1, 3, 1, 1],
        args: vec![
            String::from("arg w1"),
            String::from("arg w1"),
            String::from("arg w2"),
            String::from("arg w3"),
            String::from("arg w3"),
            String::from("arg w3"),
            String::from(""),
            String::from(""),
        ],
    };

    assert_eq!(preset, target);

    preset
        .change_field_value(PresetValue::Tabs(2))
        .expect("Failed changing preset's tabs");

    let target = Preset {
        name: String::from("Preset Changed"),
        tabs: 2,
        windows: vec![2, 1],
        args: vec![
            String::from("arg w1"),
            String::from("arg w1"),
            String::from("arg w2"),
        ],
    };

    assert_eq!(preset, target);

    preset
        .change_field_value(PresetValue::Windows(0, 1))
        .expect("Failed changing preset's windows");

    let target = Preset {
        name: String::from("Preset Changed"),
        tabs: 2,
        windows: vec![1, 1],
        args: vec![String::from("arg w1"), String::from("arg w2")],
    };

    assert_eq!(preset, target);
}

fn create_test_app_config() -> AppConfig {
    let settings = Settings {
        wt_profile: "Ubuntu".to_string(),
        init_shell: ShellType::Powershell,
        target_shell: ShellType::Zsh,
    };

    let app_config = AppConfig {
        presets: vec![],
        settings,
    };

    app_config
}
