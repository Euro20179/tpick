use std::{fmt::Display, io::{Write, Read}};

pub fn input(prompt: &str, reader: &mut std::io::Stdin, row: u32, col: u32) -> String {
    eprint!("\x1b[s");
    eprint!("\x1b[?25h");
    eprint!("\x1b[{};{}H\x1b[2K{}", row, col, prompt);
    let _ = std::io::stdout().flush();
    let mut data = String::new();
    let mut b = [0; 32];
    loop {
        let bytes_read = reader.read(&mut b).unwrap();
        if b[0] == 10 {
            break;
        }
        if b[0] == 127 {
            let st = data.as_str();
            data = String::from(&st[0..st.len() - 1])
        } else {
            data += &String::from_utf8(b[0..bytes_read].into()).unwrap();
        }
        eprint!("\x1b[2K\r{}{}", prompt, data);
        let _ = std::io::stdout().flush();
    }
    eprint!("\x1b[?25l");
    eprint!("\x1b[2K");
    eprint!("\x1b[u");
    return data;
}

pub fn selection_menu<T: Display + Clone>(items: Vec<T>, reader: &mut std::io::Stdin, row: u32, col: u32) -> T {
    eprint!("\x1b[s");
    let mut curr_selection = 0;
    loop {
        eprint!("\x1b[{};{}H\x1b[J", row, col);
        for (i, item) in items.iter().enumerate() {
            if i == curr_selection {
                eprintln!("\x1b[32m{}\x1b[0m {}", i, item);
            } else {
                eprintln!("{} {}", i, item);
            }
        }
        let mut b = [0 as u8; 1];
        reader.read(&mut b).unwrap();
        if b[0] == 10 {
            break;
        }
        if b[0] - 48 < items.len() as u8 {
            curr_selection = (b[0] - 48) as usize;
            break;
        }
        if b[0] == b'j' {
            curr_selection += 1;
            if curr_selection > items.len() - 1 {
                curr_selection = 0;
            }
        } else if b[0] == b'k' {
            if curr_selection == 0 {
                curr_selection = items.len() - 1;
            } else {
                curr_selection -= 1;
            }
        }    }
    //clear the list thing
    eprint!("\x1b[{};{}H\x1b[J", row, col);
    for _i in 0..items.len() {
        eprintln!("\x1b[2K");
    }
    eprint!("\x1b[u");
    return items[curr_selection].clone();
}
