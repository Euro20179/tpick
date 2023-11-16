use std::{fmt::Display, io::{Write, Read}};

pub fn input(prompt: &str, reader: &mut std::io::Stdin, row: u32, col: u32) -> String {
    print!("\x1b[s");
    print!("\x1b[{};{}H\x1b[2K{}", row, col, prompt);
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
        print!("\x1b[2K\r{}{}", prompt, data);
        let _ = std::io::stdout().flush();
    }
    print!("\x1b[2K");
    print!("\x1b[u");
    return data;
}

pub fn selection_menu<T: Display + Clone>(items: Vec<T>, reader: &mut std::io::Stdin, row: u32, col: u32) -> T {
    print!("\x1b[s");
    let mut curr_selection = 0;
    loop {
        print!("\x1b[{};{}H\x1b[J", row, col);
        for (i, item) in items.iter().enumerate() {
            if i == curr_selection {
                println!("\x1b[32m{}\x1b[0m {}", i, item);
            } else {
                println!("{} {}", i, item);
            }
        }
        let mut b = [0 as u8; 1];
        reader.read(&mut b).unwrap();
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
        } else if b[0] == 10 {
            break;
        }
    }
    //clear the list thing
    print!("\x1b[{};{}H\x1b[J", row, col);
    for _i in 0..items.len() {
        println!("\x1b[2K");
    }
    print!("\x1b[u");
    return items[curr_selection].clone();
}
