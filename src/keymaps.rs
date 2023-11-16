use crate::cls;
use crate::ui;
use crate::ColorRepresentation;
use crate::ProgramState;
use crate::{paste_to_clipboard, read_clipboard};
use crate::{OutputType, SelectionType};

pub enum Action {
    Break,
}

pub fn init_keymaps(
) -> std::collections::HashMap<String, fn(&mut ProgramState, &str) -> Option<Action>> {
    let mut key_maps = std::collections::HashMap::<String, fn(&mut ProgramState, &str) -> Option<Action>>::new();

    key_maps.insert("q".to_owned(), |_program_state, _key| Some(Action::Break));

    key_maps.insert("$".to_owned(), |program_state, _key| {
        let max_values = program_state.selection_type.max_values();
        let sel_type = program_state.selection_type;
        let new_value = max_values[program_state.selected_item as usize % max_values.len()];
        sel_type.modify_color_based_on_selected_item(program_state, new_value);
        None
    });

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

    for key in ["l", "h"] {
        key_maps.insert(key.to_owned(), |program_state, key| {
            let amnt_mult = if key == "l" { 1.0 } else { -1.0 };
            let increments = program_state.selection_type.increments();
            let inc = increments[program_state.selected_item as usize % increments.len()];
            let colors = program_state.selection_type.colors(&program_state);
            let color_count = colors.len();
            let sel_type = program_state.selection_type;
            let new_value =
                colors[program_state.selected_item as usize % color_count] + inc * amnt_mult;
            sel_type.modify_color_based_on_selected_item(program_state, new_value);
            None
        });
    }

    for key in ["L", "H"] {
        key_maps.insert(key.to_owned(), |program_state, key| {
            let amnt_mult = if key == "L" { 10.0 } else { -10.0 };
            let increments = program_state.selection_type.increments();
            let inc = increments[program_state.selected_item as usize % increments.len()];
            let colors = program_state.selection_type.colors(&program_state);
            let color_count = colors.len();
            let sel_type = program_state.selection_type;
            let new_value =
                colors[program_state.selected_item as usize % color_count] + inc * amnt_mult;
            sel_type.modify_color_based_on_selected_item(program_state, new_value);
            None
        });
    }

    key_maps.insert("k".to_owned(), |program_state, _key| {
        program_state.selected_item = if program_state.selected_item == 0 {
            2 + program_state.enable_alpha as u8
        } else {
            program_state.selected_item - 1
        };
        None
    });

    key_maps.insert("j".to_owned(), |program_state, _key| {
        program_state.selected_item = if program_state.selected_item == 2 {
            3 * (program_state.enable_alpha as u8)
        } else {
            program_state.selected_item + 1
        };
        None
    });

    key_maps.insert("i".to_owned(), |program_state, _key| {
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

    key_maps.insert("I".to_owned(), |program_state, _key| {
        let mut reader = std::io::stdin();
        let n = ui::input(
            &format!(
                "Type {}: ",
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

    key_maps.insert("o".to_owned(), |program_state, _key| {
        program_state.output_type = match program_state.output_type {
            OutputType::HSL => OutputType::RGB,
            OutputType::RGB => OutputType::HEX,
            OutputType::HEX => OutputType::ANSI,
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
        program_state.curr_color = ColorRepresentation::from_color(&clr);
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
        program_state.curr_color = ColorRepresentation::from_color(&data);
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
