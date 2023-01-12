use std::{fs, io::Read};

use serde_json::json;

use crate::lib::{api::write_preset_to_file, model::WriteType};

use super::model::{AppConfig, Preset, Settings};

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
        terminal_path: "".to_string(),
        shell_name: "".to_string(),
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
fn test_write_to_file() {
    let settings = Settings {
        terminal_path: "".to_string(),
        shell_name: "".to_string(),
    };

    let mut app_config = AppConfig {
        presets: vec![],
        settings,
    };

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
    let test_string =  "{\"presets\":[{\"name\":\"Test Preset\",\"tabs\":3,\"windows\":[2,1,1],\"args\":[\"arg w1\",\"arg w1\",\"arg w2\",\"arg w3\"]}],\"settings\":{\"terminal_path\":\"\",\"shell_name\":\"\"}}".to_string();
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
    println!("{:?}", file_contents);
    println!("{:?}", test_string.to_string());

    let assertion = file_contents == test_string.to_string();
    fs::remove_file(config_path).expect("Failed to delete file");
    if !assertion {
        panic!("");
    }
}
