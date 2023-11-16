#[macro_use]
mod math;
mod color_conversions;
mod ui;
mod color_representation;

use color_representation::*;

use std::fmt::Display;
use std::io::Read;

use base64::engine::general_purpose;
use base64::prelude::*;
use termios::Termios;

use color_conversions::*;


fn cls() {
    print!("\x1b[2J\x1b[0H");
}

unsafe fn query_winsize(fd: i32, ws_struct: &mut libc::winsize) {
    libc::ioctl(fd, libc::TIOCGWINSZ, ws_struct);
}

fn render_ansi256(selected_item: u8, _square_count: u32) {
    for low_nr in 0..16 {
        print!("\x1b[38;5;{}m{:3} ", low_nr, low_nr);
    }
    println!();
    for x in 0..6 {
        for y in 0..6 {
            for z in 0..6 {
                let clr = (x + 16) + (6 * y) + (36 * z);
                print!("\x1b[38;5;{}m{:3} ", clr, clr);
            }
        }
        println!();
    }
    for grey_nr in 232..256 {
        print!("\x1b[38;5;{}m{:3} ", grey_nr, grey_nr);
    }
    println!();
    println!("\x1b[0m");
    println!("\x1b[2K{}", selected_item);
}

fn ansi256_renderer(
    _curr_color: &ColorRepresentation,
    selected_item: u8,
    square_count: u32,
    _step: f32,
) {
    print!("\x1b[0H");
    render_ansi256(selected_item, square_count);
}

fn render_rgb(curr_color: &ColorRepresentation, square_count: u32, step: f32, rgb_idx: usize) {
    //the way this renders will have all sliders colors update live based on the value of the other
    //channels in the color

    //keep track of the colors in a list
    let mut colors = [curr_color.r, curr_color.g, curr_color.b];
    //this is the index of the color that will be modified
    let modifier_idx = rgb_idx;
    //set it to 0 for the start of the slider
    colors[modifier_idx] = 0.0;
    //find the label
    let label = ['R', 'G', 'B'][rgb_idx];
    print!("{}", label);
    //create the starting color based on the list of colors
    let mut color =
        ColorRepresentation::from_color(&format!("rgb({},{},{})", colors[0], colors[1], colors[2]));
    for i in 0..square_count {
        //print a square with the correct color
        print!("\x1b[38;2;{}m█", color.toansi(false));
        //modifies this slider's color to be i% of 255
        colors[modifier_idx] = (i as f32 / square_count as f32) * 255.0;
        color.modify_rgb((colors[0], colors[1], colors[2]));
    }
    println!("\x1b[0m");
    render_carrot_on_current_line(
        ([curr_color.r, curr_color.g, curr_color.b][modifier_idx] / 255.0 * 360.0 / step).floor()
            as usize
            + 1,
    );
}

fn rgb_renderer(curr_color: &ColorRepresentation, selected_item: u8, square_count: u32, step: f32) {
    for i in 0..=2 {
        print!("\x1b[{};0H", i * 2 + 1);
        if selected_item == i {
            print!("\x1b[32m");
        }
        render_rgb(curr_color, square_count, step, i as usize);
    }
}

fn render_hsl(curr_color: &ColorRepresentation, square_count: u32, step: f32, hsl_idx: usize) {
    //works similarly to render_rgb
    let (h, s, l) = curr_color.hsl();
    let mut colors = [h, s, l];
    let modifier_idx = hsl_idx;
    colors[modifier_idx] = 0.0;
    let label = ['H', 'S', 'L'][hsl_idx];
    let modifier_multiplier = [360.0, 1.0, 1.0][hsl_idx];
    print!("{}", label);
    let mut color =
        ColorRepresentation::from_color(&format!("hsl({},{},{})", colors[0], colors[1], colors[2]));
    for i in 0..square_count {
        print!("\x1b[38;2;{}m█", color.toansi(false));
        colors[modifier_idx] = (i as f32 / square_count as f32) * modifier_multiplier;
        color.modify_hsl((colors[0], colors[1], colors[2]));
    }
    println!("\x1b[0m");
    render_carrot_on_current_line(
        ([h, s, l][modifier_idx] / modifier_multiplier * 360.0 / step).floor() as usize + 1,
    );
}

fn hsl_renderer(curr_color: &ColorRepresentation, selected_item: u8, square_count: u32, step: f32) {
    for i in 0..=2 {
        print!("\x1b[{};0H", i * 2 + 1);
        if selected_item == i {
            print!("\x1b[32m");
        }
        render_hsl(curr_color, square_count, step, i as usize);
    }
}

fn render_a(square_count: u32) {
    print!("A");
    let mut sat_color_rep = ColorRepresentation::from_color("#000000");
    for i in 0..square_count {
        print!("\x1b[38;2;{}m█", sat_color_rep.toansi(false));
        sat_color_rep.modify_hsl((0.0, 0.0, (i as f32 / square_count as f32)))
    }
    println!("\x1b[0m");
}

fn render_carrot_on_current_line(col: usize) {
    println!("\x1b[2K\x1b[{}C^", col);
}

fn render_sliders(
    curr_color: &ColorRepresentation,
    alpha: u8,
    renderer: fn(&ColorRepresentation, u8, u32, f32),
    square_count: u32,
    step: f32,
    selected_item: u8,
    enable_alpha: bool,
) {
    renderer(curr_color, selected_item, square_count, step);

    if enable_alpha {
        print!("\x1b[7;0H");
        if selected_item == 3 {
            print!("\x1b[32m");
        }
        render_alpha_display(alpha, square_count, step);
    }
}

fn render_alpha_display(alpha: u8, square_count: u32, step: f32) {
    render_a(square_count);
    println!(
        "\x1b[2K {}^",
        " ".repeat(((alpha as f32 / 255.0 * 360.0) / step).floor() as usize)
    );
}

fn render_display(program_state: &ProgramState, square_count: u32, step: f32) {
    render_sliders(
        &program_state.curr_color,
        program_state.curr_color.a,
        match program_state.selection_type {
            SelectionType::HSL => hsl_renderer,
            SelectionType::RGB => rgb_renderer,
            SelectionType::ANSI256 => ansi256_renderer,
        },
        square_count,
        step,
        program_state.selected_item,
        program_state.enable_alpha,
    );
    for _ in 0..3 {
        println!(
            "\x1b[38;2;{}m████████\x1b[0m",
            program_state.curr_color.toansi(false)
        );
    }
    print!("\x1b[J");
    program_state
        .output_type
        .render_output(&program_state.curr_color, program_state.enable_alpha);
}

struct ProgramState {
    selection_type: SelectionType,
    selected_item: u8,
    enable_alpha: bool,
    output_type: OutputType,
    curr_color: ColorRepresentation,
}

//TODO: remove u8 requirement, keep track of that with ProgramState.selected_item
#[derive(Copy, Clone)]
enum SelectionType {
    HSL,
    RGB,
    ANSI256,
}

impl SelectionType {
    fn label_from_selected_item(&self, selected_item: u8) -> char {
        match self {
            SelectionType::HSL => ['H', 'S', 'L', 'A'][selected_item as usize],
            SelectionType::ANSI256 => 'e',
            Self::RGB => ['R', 'G', 'B', 'A'][selected_item as usize],
        }
    }

    fn modify_color_based_on_selected_item(
        &self,
        curr_color: &mut ColorRepresentation,
        selected_item: u8,
        mut new_value: f32,
    ) {
        match self {
            SelectionType::HSL => {
                let (h, s, l) = curr_color.hsl();
                if selected_item == 1 || selected_item == 2 {
                    new_value /= 100.0;
                }
                let mut modifiables = [h, s, l, curr_color.a as f32];
                modifiables[selected_item as usize] = new_value;
                curr_color.add_hsla([
                    modifiables[0] - h,
                    modifiables[1] - s,
                    modifiables[2] - l,
                    modifiables[3] - curr_color.a as f32,
                ]);
            }
            SelectionType::RGB => {
                let (r, g, b) = (curr_color.r, curr_color.g, curr_color.b);
                let mut modifiables = [r, g, b, curr_color.a as f32];
                modifiables[selected_item as usize] = new_value;
                curr_color.add_rgba([
                    modifiables[0] - r,
                    modifiables[1] - g,
                    modifiables[2] - b,
                    modifiables[3] - curr_color.a as f32,
                ]);
            }
            _ => todo!(),
        }
    }
}

#[derive(Clone)]
enum OutputType {
    HSL,
    RGB,
    HEX,
    ANSI,
    CUSTOM(String),
    ALL,
}

impl Display for OutputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use OutputType::*;
        write!(
            f,
            "{}",
            match self {
                HSL => "HSL",
                RGB => "RGB",
                HEX => "HEX",
                ANSI => "ANSI",
                CUSTOM(n) => n,
                ALL => "ALL",
            }
        )
    }
}

impl OutputType {
    fn render_output(&self, curr_color: &ColorRepresentation, enable_alpha: bool) {
        println!(
            "\x1b[2K{}",
            curr_color.get_formatted_output_clr(self, enable_alpha)
        )
    }
}

fn read_ansi_color(reader: &mut std::io::Stdin, clr_num: u8) -> String {
    println!("\x1b]4;{};?\x07", clr_num);
    let mut clr_buf = String::new();
    let mut b = [0; 1];
    loop {
        reader.read_exact(&mut b).unwrap();
        if b[0] == 7 {
            break;
        }
        clr_buf += &String::from(b[0] as char);
    }
    //parses out garbage, gives us rr/gg/bb
    let data = &clr_buf
        .as_str()
        .split(";")
        .nth(2)
        .unwrap()
        .split(":")
        .nth(1)
        .unwrap();
    let mut hexes = data.split("/");
    let r = &hexes.next().unwrap()[0..2];
    let g = &hexes.next().unwrap()[0..2];
    let b = &hexes.next().unwrap()[0..2];
    return format!("#{}{}{}", r, g, b);
}

fn get_ansi_30_and_90(reader: &mut std::io::Stdin) -> Vec<String> {
    let mut data = Vec::with_capacity(16);
    for i in 0..16 {
        data.push(read_ansi_color(reader, i));
    }
    return data;
}

fn paste_to_clipboard(data: &str) {
    let b64 = general_purpose::STANDARD.encode(data);
    print!("\x1b]52;c;{}\x07", b64);
}

fn read_clipboard(reader: &mut std::io::Stdin) -> String {
    println!("\x1b]52;c;?\x07");

    let mut clip_buf = String::new();
    let mut b = [0; 1];
    loop {
        reader.read_exact(&mut b).unwrap();
        if b[0] == 7 {
            break;
        }
        clip_buf += &String::from(b[0] as char);
    }

    let clip_data = clip_buf.split(";").nth(2).unwrap();

    return String::from_utf8(general_purpose::STANDARD.decode(clip_data).unwrap()).unwrap();
}

//returns oldtermios, newtermios
fn setup_term() -> (termios::Termios, termios::Termios) {
    let mut tios = Termios::from_fd(0).unwrap();
    let mut tios_initial = Termios::from_fd(0).unwrap();
    let _ = termios::tcgetattr(0, &mut tios);
    let _ = termios::tcgetattr(0, &mut tios_initial);

    tios.c_lflag &= !(termios::ICANON | termios::ECHO);
    termios::tcsetattr(0, termios::TCSANOW, &tios).unwrap();

    return (tios_initial, tios);
}

fn main() {
    let (tios_initial, _tios) = setup_term();

    let mut reader = std::io::stdin();
    let mut buf = [0; 32];

    let mut wsz = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    unsafe {
        query_winsize(0, &mut wsz);
    }

    //this variable keeps track of the step for the step increase for the HSL/RGB rendering
    let step = (360.0 / wsz.ws_col as f32).ceil();

    let square_count = (361.0 / step).ceil() as u32;

    let mut program_state = ProgramState {
        selected_item: 0,
        selection_type: SelectionType::HSL,
        output_type: OutputType::HSL,
        enable_alpha: false,
        curr_color: ColorRepresentation::from_color("rgb(0, 255, 255)"),
    };

    cls();

    let low_rgb = get_ansi_30_and_90(&mut reader);

    loop {
        let (h, s, l) = program_state.curr_color.hsl();

        render_display(&program_state, square_count, step);

        let bytes_read = reader.read(&mut buf).unwrap();

        let data = String::from_utf8(buf[0..bytes_read].to_vec()).unwrap();

        if data == "q" {
            break;
        }

        for i in 0..=9 {
            if data == i.to_string() {
                let mult = i as f32 / 10.0;
                match program_state.selection_type {
                    SelectionType::HSL => match program_state.selected_item % 4 {
                        0 => program_state.curr_color.modify_hsl((359.0 * mult, s, l)),
                        1 => program_state.curr_color.modify_hsl((h, mult, l)),
                        2 => program_state.curr_color.modify_hsl((h, s, mult)),
                        3 => program_state.curr_color.modify_a((255.0 * mult) as i32),
                        _ => todo!("this should never happen"),
                    },
                    SelectionType::RGB => match program_state.selected_item % 4 {
                        0 => program_state.curr_color.r = 255.0 * mult,
                        1 => program_state.curr_color.g = 255.0 * mult,
                        2 => program_state.curr_color.b = 255.0 * mult,
                        3 => program_state.curr_color.modify_a((255.0 * mult) as i32),
                        _ => todo!("this should never happen"),
                    },
                    SelectionType::ANSI256 => {
                        program_state.selected_item = (255.0 * (i as f32 / 10.0)) as u8;
                        let (r, g, b) = ansi2562rgb(program_state.selected_item, &low_rgb);
                        program_state
                            .curr_color
                            .modify_rgb((r as f32, g as f32, b as f32));
                    }
                }
            }
        }

        if data == "$" {
            let mult = 1.0;
            match program_state.selection_type {
                SelectionType::HSL => match program_state.selected_item % 4 {
                    0 => program_state.curr_color.modify_hsl((359.0 * mult, s, l)),
                    1 => program_state.curr_color.modify_hsl((h, mult, l)),
                    2 => program_state.curr_color.modify_hsl((h, s, mult)),
                    3 => program_state.curr_color.modify_a((255.0 * mult) as i32),
                    _ => todo!("this should never happen"),
                },
                SelectionType::RGB => match program_state.selected_item % 4 {
                    0 => program_state.curr_color.r = 255.0,
                    1 => program_state.curr_color.g = 255.0,
                    2 => program_state.curr_color.b = 255.0,
                    3 => program_state.curr_color.a = 255,
                    _ => todo!("this should never happen"),
                },
                SelectionType::ANSI256 => {
                    program_state.selected_item = 255;
                    let (r, g, b) = ansi2562rgb(program_state.selected_item, &low_rgb);
                    program_state
                        .curr_color
                        .modify_rgb((r as f32, g as f32, b as f32));
                }
            }
        }

        if data == "l" || data == "h" {
            let amnt_mult = if data == "l" { 1.0 } else { -1.0 };

            match program_state.selection_type {
                SelectionType::HSL => {
                    let mut toadd = [0.0; 4];
                    if program_state.selected_item == 0 || program_state.selected_item == 3 {
                        toadd[0] = 1.0 * amnt_mult;
                    } else {
                        toadd[program_state.selected_item as usize] = 0.01 * amnt_mult;
                    }
                    program_state.curr_color.add_hsla(toadd);
                }
                SelectionType::RGB => {
                    let mut toadd = [0.0; 4];
                    toadd[(program_state.selected_item % 4) as usize] = 1.0 * amnt_mult;
                    program_state.curr_color.add_rgba(toadd);
                }
                SelectionType::ANSI256 => {
                    if program_state.selected_item == 255 && amnt_mult == 1.0 {
                        program_state.selected_item = 0;
                    } else if program_state.selected_item == 0 && amnt_mult == -1.0 {
                        program_state.selected_item = 255;
                    } else {
                        if amnt_mult < 0.0 {
                            program_state.selected_item -= 1;
                        } else {
                            program_state.selected_item += 1;
                        }
                        let (r, g, b) = ansi2562rgb(program_state.selected_item, &low_rgb);
                        program_state
                            .curr_color
                            .modify_rgb((r as f32, g as f32, b as f32));
                    }
                }
            }
        } else if data == "k" {
            program_state.selected_item = if program_state.selected_item == 0 {
                2 + program_state.enable_alpha as u8
            } else {
                program_state.selected_item - 1
            };
        } else if data == "j" {
            program_state.selected_item = if program_state.selected_item == 2 {
                if program_state.enable_alpha {
                    3
                } else {
                    0
                }
            } else {
                program_state.selected_item + 1
            }
        } else if data == "i" {
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
        } else if data == "I" {
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
                program_state
                    .selection_type
                    .modify_color_based_on_selected_item(
                        &mut program_state.curr_color,
                        program_state.selected_item,
                        n,
                    );
            } else {
                print!("\x1b[s\x1b[30;1H\x1b[31m{}\x1b[0m\x1b[u", "Invalid number");
            }
        } else if data == "o" {
            program_state.output_type = match program_state.output_type {
                OutputType::HSL => OutputType::RGB,
                OutputType::RGB => OutputType::HEX,
                OutputType::HEX => OutputType::ANSI,
                OutputType::ANSI => OutputType::HSL,
                OutputType::CUSTOM(..) => OutputType::HSL,
                OutputType::ALL => OutputType::HSL,
            }
        } else if data == "O" {
            let how_to_select = ui::input(
                "Type m for menu f for a custom format, or a to display all outputs: ",
                &mut reader,
                30,
                1,
            );
            if how_to_select == "f" {
                let fmt = ui::input("Format: ", &mut reader, 30, 1);
                program_state.output_type = OutputType::CUSTOM(fmt);
            } else if how_to_select == "a" {
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
        } else if data == "n" {
            let clr = ui::input("New color: ", &mut reader, 30, 1);
            program_state.curr_color = ColorRepresentation::from_color(&clr);
        } else if data == "y" {
            paste_to_clipboard(
                &program_state.curr_color.get_formatted_output_clr(
                    &program_state.output_type,
                    program_state.enable_alpha,
                ),
            )
        } else if data == "Y" {
            paste_to_clipboard(
                &program_state
                    .curr_color
                    .get_output_clr(&program_state.output_type, program_state.enable_alpha),
            );
        } else if data == "p" {
            let data = read_clipboard(&mut reader);
            program_state.curr_color = ColorRepresentation::from_color(&data);
        } else if data == "a" {
            cls();
            program_state.enable_alpha = !program_state.enable_alpha;
        }
    }
    termios::tcsetattr(0, termios::TCSANOW, &tios_initial).unwrap();
}
