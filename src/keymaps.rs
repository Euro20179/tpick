use std::collections::HashMap;

use crate::cls;
use crate::hashmap;
use crate::ui;
use crate::ColorRepresentation;
use crate::ProgramState;
use crate::{paste_to_clipboard, read_clipboard};
use crate::{OutputType, SelectionType};

pub enum Action {
    Break,
}

fn read_keymap_config() -> HashMap<String, String> {
    let mut config_folder = std::env!("XDG_CONFIG_HOME").to_owned();
    if config_folder == "" {
        config_folder = String::from(std::env!("HOME")) + &String::from("/.config");
    }
    let tpick_config_path = config_folder + &String::from("/tpick");
    let keymap_path = tpick_config_path + &String::from("/keymaps.json");
    let keymap_config_file_path = std::path::Path::new(&keymap_path);
    let mut default = hashmap! {
        "quit".to_owned() => "q".to_owned(),
        "quit-and-copy".to_owned() => "\x0A".to_owned(),
        "set-max-value".to_owned() => "$".to_owned(),
        "increase-value".to_owned() => "l".to_owned(),
        "decrease-value".to_owned() => "h".to_owned(),
        "decrease-value-10".to_owned() => "H".to_owned(),
        "increase-value-10".to_owned() => "L".to_owned(),
        "up".to_owned() => "k".to_owned(),
        "down".to_owned() => "j".to_owned(),
        "cycle-selection-type".to_owned() => "i".to_owned(),
        "set-value".to_owned() => "I".to_owned()
    };
    if keymap_config_file_path.exists() {
        let data = std::fs::read_to_string(keymap_config_file_path).unwrap();
        let map: HashMap<String, String> =
            serde_json::from_str(&data).expect("Invalid keymap config file");
        let keys = map.keys();
        for key in keys{
            default.insert(key.to_string(), map.get(key).unwrap().clone());
        }
    }
    return default;
}

macro_rules! hash_get {
    ($map:expr, $get:literal || $backup:literal) => {
        $map.get($get).unwrap_or(&$backup.to_owned()).to_owned()
    };
    ($map:expr, $get:literal) => {
        $map.get($get).unwrap().to_owned()
    };
}

pub fn init_keymaps(
) -> std::collections::HashMap<String, fn(&mut ProgramState, &str) -> Option<Action>> {
    let mut key_maps =
        std::collections::HashMap::<String, fn(&mut ProgramState, &str) -> Option<Action>>::new();

    let config_keymaps = read_keymap_config();

    key_maps.insert(hash_get!(config_keymaps, "quit"), |_program_state, _key| {
        Some(Action::Break)
    });

    key_maps.insert(
        hash_get!(config_keymaps, "quit-and-copy"),
        |program_state, _key| {
            paste_to_clipboard(
                &program_state
                    .output_type
                    .render_output(&program_state.curr_color, program_state.enable_alpha),
            );
            Some(Action::Break)
        },
    );

    key_maps.insert(
        hash_get!(config_keymaps, "set-max-value"),
        |program_state, _key| {
            let max_values = program_state.selection_type.max_values();
            let sel_type = program_state.selection_type;
            let new_value = max_values[program_state.selected_item as usize % max_values.len()];
            sel_type.modify_color_based_on_selected_item(program_state, new_value);
            None
        },
    );

    for i in 0..=9 {
        key_maps.insert(i.to_string(), |program_state, key| {
            let mult = key.parse::<f32>().unwrap() / 10.0;
            let max_values = program_state.selection_type.max_values();
            let max_value = max_values[program_state.selected_item as usize % max_values.len()];
            let sel_type = program_state.selection_type;
            sel_type.modify_color_based_on_selected_item(program_state, max_value * mult);
            None
        });
    }

    key_maps.insert(
        hash_get!(config_keymaps, "increase-value"),
        |program_state, _key| {
            let increments = program_state.selection_type.increments();
            let inc = increments[program_state.selected_item as usize % increments.len()];
            let colors = program_state.selection_type.colors(&program_state);
            let color_count = colors.len();
            let sel_type = program_state.selection_type;
            let new_value = colors[program_state.selected_item as usize % color_count] + inc;
            sel_type.modify_color_based_on_selected_item(program_state, new_value);
            None
        },
    );

    key_maps.insert(
        hash_get!(config_keymaps, "decrease-value"),
        |program_state, _key| {
            let increments = program_state.selection_type.increments();
            let inc = increments[program_state.selected_item as usize % increments.len()];
            let colors = program_state.selection_type.colors(&program_state);
            let color_count = colors.len();
            let sel_type = program_state.selection_type;
            let new_value = colors[program_state.selected_item as usize % color_count] + inc * -1.0;
            sel_type.modify_color_based_on_selected_item(program_state, new_value);
            None
        },
    );

    key_maps.insert(
        hash_get!(config_keymaps, "increase-value-10"),
        |program_state, _key| {
            let increments = program_state.selection_type.increments();
            let inc = increments[program_state.selected_item as usize % increments.len()];
            let colors = program_state.selection_type.colors(&program_state);
            let color_count = colors.len();
            let sel_type = program_state.selection_type;
            let new_value = colors[program_state.selected_item as usize % color_count] + inc * 10.0;
            sel_type.modify_color_based_on_selected_item(program_state, new_value);
            None
        },
    );

    key_maps.insert(
        hash_get!(config_keymaps, "decrease-value-10"),
        |program_state, _key| {
            let increments = program_state.selection_type.increments();
            let inc = increments[program_state.selected_item as usize % increments.len()];
            let colors = program_state.selection_type.colors(&program_state);
            let color_count = colors.len();
            let sel_type = program_state.selection_type;
            let new_value =
                colors[program_state.selected_item as usize % color_count] + inc * -10.0;
            sel_type.modify_color_based_on_selected_item(program_state, new_value);
            None
        },
    );

    key_maps.insert(hash_get!(config_keymaps, "up"), |program_state, _key| {
        let items = program_state.selection_type.max_values();
        program_state.selected_item = if program_state.selected_item == 0 {
            (items.len() - 2) as u8 + program_state.enable_alpha as u8
        } else {
            program_state.selected_item - 1
        };
        None
    });

    key_maps.insert(hash_get!(config_keymaps, "down"), |program_state, _key| {
        let items = program_state.selection_type.max_values();
        program_state.selected_item = if program_state.selected_item as usize == items.len() - ((2 - program_state.enable_alpha as usize) as usize) {
            0
        } else {
            program_state.selected_item + 1
        };
        None
    });

    key_maps.insert(
        hash_get!(config_keymaps, "cycle-selection-type"),
        |program_state, _key| {
            program_state.selection_type = match program_state.selection_type {
                SelectionType::HSL => SelectionType::RGB,
                SelectionType::RGB => {
                    cls();
                    SelectionType::CYMK
                }
                SelectionType::CYMK => {
                    cls();
                    SelectionType::ANSI256
                }
                SelectionType::ANSI256 => {
                    cls();
                    program_state.selected_item = 0;
                    SelectionType::HSL
                }
            };
            None
        },
    );

    key_maps.insert(
        hash_get!(config_keymaps, "set-value"),
        |program_state, _key| {
            let mut reader = std::io::stdin();
            let n = ui::input(
                &format!(
                    "Set value {}: ",
                    program_state
                        .selection_type
                        .label_from_selected_item(program_state.selected_item)
                ),
                &mut reader,
                30,
                1,
            );
            let number = n.parse();
            if let Ok(n) = number {
                let sel_type = program_state.selection_type;
                sel_type.modify_color_based_on_selected_item(program_state, n);
            } else {
                print!("\x1b[s\x1b[30;1H\x1b[31m{}\x1b[0m\x1b[u", "Invalid number");
            };
            None
        },
    );

    key_maps.insert("o".to_owned(), |program_state, _key| {
        program_state.output_type = match program_state.output_type {
            OutputType::HSL => OutputType::RGB,
            OutputType::RGB => OutputType::HEX,
            OutputType::HEX => OutputType::CYMK,
            OutputType::CYMK => OutputType::ANSI,
            OutputType::ANSI => OutputType::HSL,
            OutputType::CUSTOM(..) => OutputType::HSL,
            OutputType::ALL => OutputType::HSL,
        };
        None
    });

    key_maps.insert("O".to_owned(), |program_state, _key| {
        let mut reader = std::io::stdin();
        let how_to_select = ui::selection_menu(
            vec!["select output", "custom format", "all outputs"],
            &mut reader,
            30,
            1,
        );
        if how_to_select == "custom format" {
            let fmt = ui::input("Format: ", &mut reader, 30, 1);
            program_state.output_type = OutputType::CUSTOM(fmt);
        } else if how_to_select == "all outputs" {
            program_state.output_type = OutputType::ALL
        } else {
            let o_type = ui::selection_menu(
                vec![
                    OutputType::HSL,
                    OutputType::RGB,
                    OutputType::HEX,
                    OutputType::ANSI,
                ],
                &mut reader,
                20,
                1,
            );
            program_state.output_type = o_type
        }
        None
    });

    key_maps.insert("n".to_owned(), |program_state, _key| {
        let mut reader = std::io::stdin();
        let clr = ui::input("New color: ", &mut reader, 30, 1);
        program_state.curr_color = ColorRepresentation::from_color(&clr, &program_state.clr_std);
        None
    });

    key_maps.insert("y".to_owned(), |program_state, _key| {
        paste_to_clipboard(
            &program_state
                .curr_color
                .get_formatted_output_clr(&program_state.output_type, program_state.enable_alpha),
        );
        None
    });

    key_maps.insert("Y".to_owned(), |program_state, _key| {
        paste_to_clipboard(
            &program_state
                .curr_color
                .get_output_clr(&program_state.output_type, program_state.enable_alpha),
        );
        None
    });

    key_maps.insert("p".to_owned(), |program_state, _key| {
        let mut reader = std::io::stdin();
        let data = read_clipboard(&mut reader);
        program_state.curr_color = ColorRepresentation::from_color(&data, &program_state.clr_std);
        None
    });

    key_maps.insert("a".to_owned(), |program_state, _key| {
        match program_state.selection_type {
            SelectionType::ANSI256 => {}
            _ => {
                cls();
                program_state.enable_alpha = !program_state.enable_alpha;
            }
        }
        None
    });

    return key_maps;
}
