use std::collections::HashMap;

use crate::ConfigOutput;
use crate::cls;
use crate::hashmap;
use crate::ui;
use crate::ColorRepresentation;
use crate::Config;
use crate::ProgramState;
use crate::{paste_to_clipboard, read_clipboard};
use crate::{OutputType, SelectionType};

pub enum Action {
    Break,
}

fn read_keymap_from_config(config: &Config) -> HashMap<String, String> {
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
        "set-value".to_owned() => "I".to_owned(),
        "change-output".to_owned() => "o".to_owned(),
        "select-output".to_owned() => "O".to_owned(),
        "input-new-color".to_owned() => "n".to_owned(),
        "copy".to_owned() => "y".to_owned(),
        "copy-raw".to_owned() => "Y".to_owned(),
        "paste".to_owned() => "p".to_owned(),
        "toggle-alpha".to_owned() => "a".to_owned(),
        "00%".to_owned() => "0".to_owned(),
        "10%".to_owned() => "1".to_owned(),
        "20%".to_owned() => "2".to_owned(),
        "30%".to_owned() => "3".to_owned(),
        "40%".to_owned() => "4".to_owned(),
        "50%".to_owned() => "5".to_owned(),
        "60%".to_owned() => "6".to_owned(),
        "70%".to_owned() => "7".to_owned(),
        "80%".to_owned() => "8".to_owned(),
        "90%".to_owned() => "9".to_owned(),
    };
    match &config.keybinds {
        None => return default,
        Some(map) => {
            let keys = map.keys();
            for key in keys {
                default.insert(key.to_string(), map.get(key).unwrap().clone());
            }
        }
    }
    return default;
}

macro_rules! hash_get {
    ($map:expr, $get:ident || $backup:literal) => {
        $map.get($get).unwrap_or(&$backup.to_owned()).to_owned()
    };
    ($map:expr, $get:ident) => {
        $map.get($get).unwrap().to_owned()
    };
}

pub fn init_keymaps(
    config: &Config
) -> std::collections::HashMap<String, fn(&mut ProgramState, &str) -> Option<Action>> {
    let mut key_maps =
        std::collections::HashMap::<String, fn(&mut ProgramState, &str) -> Option<Action>>::new();

    let config_keymaps = read_keymap_from_config(config);

    let mut insert = |name: String, cb| {
        let n = &name;
        key_maps.insert(hash_get!(config_keymaps, n), cb);
    };

    insert("quit".to_string(), |_program_state, _key| {
        Some(Action::Break)
    });
    insert("quit-and-copy".to_string(), |program_state, _key| {
        paste_to_clipboard(
            &program_state
                .output_type
                .render_output(&program_state.curr_color, program_state.enable_alpha),
        );
        Some(Action::Break)
    });
    insert("set-max-value".to_string(), |program_state, _key| {
        let max_values = program_state.selection_type.max_values();
        let sel_type = program_state.selection_type;
        let new_value = max_values[program_state.selected_item as usize % max_values.len()];
        sel_type.modify_color_based_on_selected_item(program_state, new_value);
        None
    });

    for i in 0..=9 {
        let t = format!("{}0%", i);
        insert(t, |program_state, key| {
            let mult = key.parse::<f32>().unwrap() / 10.0;
            let max_values = program_state.selection_type.max_values();
            let max_value = max_values[program_state.selected_item as usize % max_values.len()];
            let sel_type = program_state.selection_type;
            sel_type.modify_color_based_on_selected_item(program_state, max_value * mult);
            None
        });
    }

    insert("increase-value".to_string(), |program_state, _key| {
        let increments = program_state.selection_type.increments();
        let inc = increments[program_state.selected_item as usize % increments.len()];
        let colors = program_state.selection_type.colors(&program_state);
        let color_count = colors.len();
        let sel_type = program_state.selection_type;
        let new_value = colors[program_state.selected_item as usize % color_count] + inc;
        sel_type.modify_color_based_on_selected_item(program_state, new_value);
        None
    });

    insert("decrease-value".to_string(), |program_state, _key| {
        let increments = program_state.selection_type.increments();
        let inc = increments[program_state.selected_item as usize % increments.len()];
        let colors = program_state.selection_type.colors(&program_state);
        let color_count = colors.len();
        let sel_type = program_state.selection_type;
        let new_value = colors[program_state.selected_item as usize % color_count] + inc * -1.0;
        sel_type.modify_color_based_on_selected_item(program_state, new_value);
        None
    });

    insert("increase-value-10".to_string(), |program_state, _key| {
        let increments = program_state.selection_type.increments();
        let inc = increments[program_state.selected_item as usize % increments.len()];
        let colors = program_state.selection_type.colors(&program_state);
        let color_count = colors.len();
        let sel_type = program_state.selection_type;
        let new_value = colors[program_state.selected_item as usize % color_count] + inc * 10.0;
        sel_type.modify_color_based_on_selected_item(program_state, new_value);
        None
    });

    insert("decrease-value-10".to_string(), |program_state, _key| {
        let increments = program_state.selection_type.increments();
        let inc = increments[program_state.selected_item as usize % increments.len()];
        let colors = program_state.selection_type.colors(&program_state);
        let color_count = colors.len();
        let sel_type = program_state.selection_type;
        let new_value = colors[program_state.selected_item as usize % color_count] + inc * -10.0;
        sel_type.modify_color_based_on_selected_item(program_state, new_value);
        None
    });

    insert("up".to_string(), |program_state, _key| {
        if let SelectionType::ANSI256 = program_state.selection_type{
            return None;
        }
        let items = program_state.selection_type.max_values();
        program_state.selected_item = if program_state.selected_item == 0 {
            (items.len() - 2) as u8 + program_state.enable_alpha as u8
        } else {
            program_state.selected_item - 1
        };
        None
    });

    insert("down".to_string(), |program_state, _key| {
        if let SelectionType::ANSI256 = program_state.selection_type{
            return None;
        }
        let items = program_state.selection_type.max_values();
        program_state.selected_item = if program_state.selected_item as usize
            == items.len() - ((2 - program_state.enable_alpha as usize) as usize)
        {
            0
        } else {
            program_state.selected_item + 1
        };
        None
    });

    insert("cycle-selection-type".to_string(), |program_state, _key| {
        program_state.selection_type = match program_state.selection_type {
            SelectionType::HSL => SelectionType::RGB,
            SelectionType::RGB => {
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
    });

    insert("set-value".to_string(), |program_state, _key| {
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
    });

    insert("change-output".to_string(), |program_state, _key| {
        cls();
        program_state.next_output();
        None
    });

    insert("select-output".to_string(), |program_state, _key| {
        let mut reader = std::io::stdin();
        let how_to_select = ui::selection_menu(
            vec!["select output", "custom format", "all outputs", "select output cycle"],
            &mut reader,
            30,
            1,
        );
        if how_to_select == "custom format" {
            let fmt = ui::input("Format: ", &mut reader, 30, 1);
            program_state.output_idx = 0; //restart the cycle
            program_state.output_type = OutputType::CUSTOM(fmt);
        } else if how_to_select == "all outputs" {
            program_state.output_type = OutputType::ALL
        } else if how_to_select == "select output cycle" {
            let outputs = &program_state.config.outputs.clone().unwrap_or(vec![hashmap!(
                    "default".to_string() => ConfigOutput {
                        order: vec!["hsl".to_string(), "rgb".to_string(), "hex".to_string(), "ansi".to_owned()]
                    }
            )])[0];
            let items: Vec<&String> = outputs.keys().collect();
            let cycle = ui::selection_menu(items, &mut reader, 30, 1);
            program_state.output_order = OutputType::get_order_by_name(&program_state.config, &cycle).unwrap();
            program_state.output_idx = 0;
            program_state.next_output();
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

    insert("input-new-color".to_string(), |program_state, _key| {
        let mut reader = std::io::stdin();
        let clr = ui::input("New color: ", &mut reader, 30, 1);
        program_state.curr_color = ColorRepresentation::from_color(&clr, &program_state.clr_std);
        None
    });

    insert("copy".to_owned(), |program_state, _key| {
        paste_to_clipboard(
            &program_state
                .curr_color
                .get_formatted_output_clr(&program_state.output_type, program_state.enable_alpha),
        );
        None
    });

    insert("copy-raw".to_owned(), |program_state, _key| {
        paste_to_clipboard(
            &program_state
                .curr_color
                .get_output_clr(&program_state.output_type, program_state.enable_alpha),
        );
        None
    });

    insert("paste".to_owned(), |program_state, _key| {
        let mut reader = std::io::stdin();
        let data = read_clipboard(&mut reader);
        program_state.curr_color = ColorRepresentation::from_color(&data, &program_state.clr_std);
        None
    });

    insert("toggle-alpha".to_owned(), |program_state, _key| {
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
