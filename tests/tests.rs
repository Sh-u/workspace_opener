use std::{fs, io::Read};
use workspace_opener::workspace_opener_lib::{
    api::{create_wt_command, write_preset_to_file},
    model::{
        AppConfig, Item, Preset, PresetInfo, PresetInfoValue, PresetValue, Settings, ShellType,
        State, WriteType,
    },
};
extern crate workspace_opener;
#[test]
fn preset_creation_basic() {
    let preset_info = PresetInfo::default();

    let target = Preset::new(
        String::from("Test Preset"),
        3,
        vec![2, 1, 1],
        vec![
            String::from("arg w1"),
            String::from("arg w1"),
            String::from("arg w2"),
            String::from("arg w3"),
        ],
        preset_info,
    );

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

    let from_input = Preset::from_input(&input);

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

    let preset_info = PresetInfo::default();
    let target = Preset::new(
        String::from("Test Preset"),
        4,
        vec![2, 1, 1, 20],
        args,
        preset_info,
    );

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

    let from_input = Preset::from_input(&input);

    assert_eq!(target, from_input);
}

#[test]
fn preset_deletion() {
    let preset_info = PresetInfo::default();

    let preset = Preset::new(
        String::from("Test Preset"),
        3,
        vec![2, 1, 1],
        vec![
            String::from("arg w1"),
            String::from("arg w1"),
            String::from("arg w2"),
            String::from("arg w3"),
        ],
        preset_info,
    );

    let settings = Settings::default();

    let mut app_config = AppConfig::new(vec![preset], settings.clone());

    let target = AppConfig::new(vec![], settings);

    let items = vec![String::from("Test Preset")];

    if let Err(err) = app_config.delete_preset_by_name(items.get(0).unwrap()) {
        panic!("{}", err);
    };

    assert_eq!(target, app_config);
}
#[test]
fn test_write_to_file_and_create() {
    let mut app_config = AppConfig::default();

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
    let test_string =  "{\"presets\":[{\"name\":\"Test Preset\",\"tabs\":3,\"windows\":[2,1,1],\"args\":[\"arg w1\",\"arg w1\",\"arg w2\",\"arg w3\"],\"preset_info\":{\"wt_profile\":\"\",\"init_shell\":\"powershell\",\"target_shell\":\"powershell\"}}],\"settings\":{\"debug_mode\":false}}".to_string();
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
    let mut app_config = AppConfig::default();

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

    let preset = Preset::from_input(&app_messages);

    app_config.add_presets(vec![preset]);

    let config_path = "test2.json";
    app_config
        .delete_preset_by_name("Test Preset")
        .expect("Failed to delete preset");
    let test_string = "{\"presets\":[],\"settings\":{\"debug_mode\":false}}".to_string();
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
    let preset_info = PresetInfo::default();

    let preset = Preset::new(
        String::from("Test Preset"),
        3,
        vec![2, 1, 1],
        vec![
            String::from("arg w1"),
            String::from("arg w1"),
            String::from("arg w2"),
            String::from("arg w3"),
        ],
        preset_info,
    );

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

    let wt_profile = Item::new(
        format!("Windows terminal profile name: {}", ""),
        State::ChangeFieldName,
        Some(PresetValue::PresetInfo(PresetInfoValue::WtProfile(
            "".to_string(),
        ))),
    );
    target.push(wt_profile);

    let init_shell = Item::new(
        format!(
            "Init shell (powershell/pwsh/cmd): {}",
            "powershell".to_string()
        ),
        State::ChangeFieldName,
        Some(PresetValue::PresetInfo(PresetInfoValue::InitShell(
            ShellType::WindowsPowershell,
        ))),
    );
    target.push(init_shell);

    let target_shell = Item::new(
        format!(
            "Target shell (powershell/pwsh/cmd/bash/zsh): {}",
            "powershell".to_string()
        ),
        State::ChangeFieldName,
        Some(PresetValue::PresetInfo(PresetInfoValue::TargetShell(
            ShellType::WindowsPowershell,
        ))),
    );
    target.push(target_shell);

    for (index, item) in items.iter().enumerate() {
        assert_eq!(item, target.get(index).unwrap());
    }
}

#[test]
fn test_preset_change_field_value() {
    let mut preset = Preset::default();

    preset
        .change_field_value(PresetValue::Name("Preset Changed".to_string()))
        .expect("Failed changing preset's name");

    preset
        .change_field_value(PresetValue::Tabs(5))
        .expect("Failed changing preset's tabs");

    let target = Preset::new(
        String::from("Preset Changed"),
        5,
        vec![2, 1, 1, 1, 1],
        vec![
            String::from("arg w1"),
            String::from("arg w1"),
            String::from("arg w2"),
            String::from("arg w3"),
            String::from(""),
            String::from(""),
        ],
        preset.get_preset_info(),
    );

    assert_eq!(preset, target);

    preset
        .change_field_value(PresetValue::Windows(2, 3))
        .expect("Failed changing preset's windows");

    let target = Preset::new(
        String::from("Preset Changed"),
        5,
        vec![2, 1, 3, 1, 1],
        vec![
            String::from("arg w1"),
            String::from("arg w1"),
            String::from("arg w2"),
            String::from("arg w3"),
            String::from(""),
            String::from(""),
            String::from(""),
            String::from(""),
        ],
        preset.get_preset_info(),
    );

    assert_eq!(preset, target);

    preset
        .change_field_value(PresetValue::Args(4, "arg w3".to_string()))
        .expect("Failed changing preset's args");
    preset
        .change_field_value(PresetValue::Args(5, "arg w3".to_string()))
        .expect("Failed changing preset's args");

    let target = Preset::new(
        String::from("Preset Changed"),
        5,
        vec![2, 1, 3, 1, 1],
        vec![
            String::from("arg w1"),
            String::from("arg w1"),
            String::from("arg w2"),
            String::from("arg w3"),
            String::from("arg w3"),
            String::from("arg w3"),
            String::from(""),
            String::from(""),
        ],
        preset.get_preset_info(),
    );

    assert_eq!(preset, target);

    preset
        .change_field_value(PresetValue::Tabs(2))
        .expect("Failed changing preset's tabs");

    let target = Preset::new(
        String::from("Preset Changed"),
        2,
        vec![2, 1],
        vec![
            String::from("arg w1"),
            String::from("arg w1"),
            String::from("arg w2"),
        ],
        preset.get_preset_info(),
    );

    assert_eq!(preset, target);

    preset
        .change_field_value(PresetValue::Windows(0, 1))
        .expect("Failed changing preset's windows");

    let target = Preset::new(
        String::from("Preset Changed"),
        2,
        vec![1, 1],
        vec![String::from("arg w1"), String::from("arg w2")],
        preset.get_preset_info(),
    );

    assert_eq!(preset, target);

    preset
        .change_field_value(PresetValue::PresetInfo(PresetInfoValue::WtProfile(
            "Ubuntu".to_owned(),
        )))
        .expect("Failed changing preset's windows");

    preset
        .change_field_value(PresetValue::PresetInfo(PresetInfoValue::TargetShell(
            ShellType::Zsh,
        )))
        .expect("Failed changing preset's windows");

    let preset_info = PresetInfo::new(
        "Ubuntu".to_string(),
        ShellType::WindowsPowershell,
        ShellType::Zsh,
    );
    let target = Preset::new(
        String::from("Preset Changed"),
        2,
        vec![1, 1],
        vec![String::from("arg w1"), String::from("arg w2")],
        preset_info,
    );

    assert_eq!(preset, target);
}

#[test]
fn wt_command_default() {
    let preset = Preset::default();
    let mut app_config = AppConfig::default();
    app_config.add_presets(vec![preset]);

    let target = "wt.exe powershell -NoExit -Command 'arg w1\\;'`; sp powershell -NoExit -Command 'arg w1\\;'`; nt powershell -NoExit -Command 'arg w2\\;'`; nt powershell -NoExit -Command 'arg w3\\;'";
    assert_eq!(
        create_wt_command("Test Preset", &mut app_config).unwrap().1,
        target
    );
}
#[test]
fn wt_command_cmd() {
    let preset_info = PresetInfo::new(String::new(), ShellType::Cmd, ShellType::Cmd);

    let mut app_config = AppConfig::default();

    let preset = Preset::new(
        "Test Preset".to_string(),
        2,
        vec![1, 2],
        vec!["ls".to_string(), "ls".to_string(), "ls".to_string()],
        preset_info,
    );

    app_config.add_presets(vec![preset]);
    let target = "wt.exe cmd /k 'ls\\;'`; nt cmd /k 'ls\\;'; sp cmd /k 'ls\\;'";
    assert_eq!(
        create_wt_command("Test Preset", &mut app_config).unwrap().1,
        target
    );
}

#[test]
fn wt_command_ubuntu_bash() {
    let preset_info = PresetInfo::new(
        String::from("Ubuntu"),
        ShellType::WindowsPowershell,
        ShellType::Bash,
    );

    let mut app_config = AppConfig::default();

    let preset = Preset::new(
        "Test Preset".to_string(),
        2,
        vec![1, 2],
        vec!["ls".to_string(), "ls".to_string(), "ls".to_string()],
        preset_info,
    );

    app_config.add_presets(vec![preset]);
    let target = "wt.exe -p \"Ubuntu\" wsl ~ -e bash -c 'ls\\;exec bash\\;'`; nt -p \"Ubuntu\" wsl ~ -e bash -c 'ls\\;exec bash\\;'`; sp -p \"Ubuntu\" wsl ~ -e bash -c 'ls\\;exec bash\\;'";
    assert_eq!(
        create_wt_command("Test Preset", &mut app_config).unwrap().1,
        target
    );
}

#[test]
fn wt_command_1tab_1window() {
    let preset_info = PresetInfo::default();
    let mut app_config = AppConfig::default();

    let preset = Preset::new(
        "Test Preset".to_string(),
        1,
        vec![1],
        vec!["ls".to_string()],
        preset_info,
    );

    app_config.add_presets(vec![preset]);

    let target = "wt.exe powershell -NoExit -Command 'ls\\;'";

    assert_eq!(
        create_wt_command("Test Preset", &mut app_config).unwrap().1,
        target
    );
}

#[test]
fn wt_command_3tabs_1windows() {
    let preset_info = PresetInfo::default();
    let mut app_config = AppConfig::default();

    let preset = Preset::new(
        "Test Preset".to_string(),
        3,
        vec![1, 1, 1],
        vec!["ls".to_string(), "pwd".to_string(), "cd ..".to_string()],
        preset_info,
    );

    app_config.add_presets(vec![preset]);

    let target = "wt.exe powershell -NoExit -Command 'ls\\;'`; nt powershell -NoExit -Command 'pwd\\;'`; nt powershell -NoExit -Command 'cd ..\\;'";

    assert_eq!(
        create_wt_command("Test Preset", &mut app_config).unwrap().1,
        target
    );
}

#[test]
fn wt_command_2tabs_2windows() {
    let preset_info = PresetInfo::default();
    let mut app_config = AppConfig::default();

    let preset = Preset::new(
        "Test Preset".to_string(),
        2,
        vec![2, 2],
        vec![
            "ls".to_string(),
            "pwd".to_string(),
            "cd ..".to_string(),
            "ls".to_string(),
        ],
        preset_info,
    );

    app_config.add_presets(vec![preset]);

    let target = "wt.exe powershell -NoExit -Command 'ls\\;'`; sp powershell -NoExit -Command 'pwd\\;'`; nt powershell -NoExit -Command 'cd ..\\;'`; sp powershell -NoExit -Command 'ls\\;'";

    assert_eq!(
        create_wt_command("Test Preset", &mut app_config).unwrap().1,
        target
    );
}

#[test]
fn wt_command_2tabs_3windows_1window() {
    let preset_info = PresetInfo::default();
    let mut app_config = AppConfig::default();

    let preset = Preset::new(
        "Test Preset".to_string(),
        2,
        vec![3, 1],
        vec![
            "ls".to_string(),
            "pwd".to_string(),
            "cd ..".to_string(),
            "ls".to_string(),
        ],
        preset_info,
    );

    app_config.add_presets(vec![preset]);

    let target = "wt.exe powershell -NoExit -Command 'ls\\;'`; sp -s .66 powershell -NoExit -Command 'pwd\\;'`; sp -s .5 powershell -NoExit -Command 'cd ..\\;'`; nt powershell -NoExit -Command 'ls\\;'";

    assert_eq!(
        create_wt_command("Test Preset", &mut app_config).unwrap().1,
        target
    );
}

#[test]
fn wt_command_1tabs_4windows() {
    let preset_info = PresetInfo::default();
    let mut app_config = AppConfig::default();

    let preset = Preset::new(
        "Test Preset".to_string(),
        1,
        vec![4],
        vec![
            "ls".to_string(),
            "pwd".to_string(),
            "cd ..".to_string(),
            "ls".to_string(),
        ],
        preset_info,
    );

    app_config.add_presets(vec![preset]);

    let target = "wt.exe powershell -NoExit -Command 'ls\\;'`; sp powershell -NoExit -Command 'pwd\\;'`; sp powershell -NoExit -Command 'cd ..\\;'`; mf left`; sp powershell -NoExit -Command 'ls\\;'";

    assert_eq!(
        create_wt_command("Test Preset", &mut app_config).unwrap().1,
        target
    );
}
