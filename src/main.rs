#[macro_use]
mod math;
mod color_conversions;

use std::io::Read;
use std::str::Split;

use base64::engine::general_purpose;
use base64::prelude::*;
use termios::Termios;

use color_conversions::*;

#[derive(Clone, Copy)]
struct ColorRepresentation {
    r: f32,
    g: f32,
    b: f32,
    a: u8,
}

impl ColorRepresentation {
    fn from_color(clr: &str) -> ColorRepresentation {
        let mut r: f32 = 0.0;
        let mut g: f32 = 0.0;
        let mut b: f32 = 0.0;
        let mut a = 255;

        let get_next = |split: &mut Split<'_, &str>| split.next().unwrap().trim().parse().unwrap();
        let get_rgb = |items: &mut Split<'_, &str>| {
            return (get_next(items), get_next(items), get_next(items));
        };

        if clr.starts_with("\\x1b") {
            //\x1b[38;2;
            let mut items = clr[10..clr.len() - 1].split(";");
            (r, g, b) = get_rgb(&mut items);
        } else if clr.contains(";") {
            let mut items = clr.split(";");
            (r, g, b) = get_rgb(&mut items);
        } else if clr.starts_with("rgba") {
            let mut items = clr[5..clr.len() - 1].split(",");
            (r, g, b) = get_rgb(&mut items);
            a = items.next().unwrap().trim().parse().unwrap();
        } else if clr.starts_with("rgb") {
            let mut items = clr[4..clr.len() - 1].split(",");
            (r, g, b) = get_rgb(&mut items);
        } else if clr.starts_with("hsla") {
            let mut items = clr[5..clr.len() - 1].split(",");
            let h: f32 = get_next(&mut items);
            let s: f32 = get_next(&mut items) / 100.0;
            let l: f32 = get_next(&mut items) / 100.0;
            a = items.next().unwrap().trim().parse().unwrap();
            (r, g, b) = hsl2rgb(h, s, l);
        } else if clr.starts_with("hsl") {
            let mut items = clr[4..clr.len() - 1].split(",");
            let h: f32 = get_next(&mut items);
            let s: f32 = get_next(&mut items) / 100.0;
            let l: f32 = get_next(&mut items) / 100.0;
            (r, g, b) = hsl2rgb(h, s, l);
        }
        //#RGB or #RGBA or #RRGGBB or #RRGGBBAA
        else if clr.starts_with("#")
            && (clr.len() == 4 || clr.len() == 5 || clr.len() == 7 || clr.len() == 9)
        {
            match clr.len() {
                4 => {
                    r = (i64::from_str_radix(&clr[1..2], 16).unwrap() as f32).powi(2);
                    g = (i64::from_str_radix(&clr[2..3], 16).unwrap() as f32).powi(2);
                    b = (i64::from_str_radix(&clr[3..4], 16).unwrap() as f32).powi(2);
                }
                5 => {
                    r = (i64::from_str_radix(&clr[1..2], 16).unwrap() as f32).powi(2);
                    g = (i64::from_str_radix(&clr[2..3], 16).unwrap() as f32).powi(2);
                    b = (i64::from_str_radix(&clr[3..4], 16).unwrap() as f32).powi(2);
                    a = (i64::from_str_radix(&clr[4..5], 16).unwrap()).pow(2) as u8;
                }
                7 => {
                    r = i64::from_str_radix(&clr[1..3], 16).unwrap() as f32;
                    g = i64::from_str_radix(&clr[3..5], 16).unwrap() as f32;
                    b = i64::from_str_radix(&clr[5..7], 16).unwrap() as f32;
                }
                9 => {
                    r = i64::from_str_radix(&clr[1..3], 16).unwrap() as f32;
                    g = i64::from_str_radix(&clr[3..5], 16).unwrap() as f32;
                    b = i64::from_str_radix(&clr[5..7], 16).unwrap() as f32;
                    a = i64::from_str_radix(&clr[7..9], 16).unwrap() as u8;
                }
                _ => {
                    (r, g, b) = (0.0, 0.0, 0.0);
                }
            }
        }
        ColorRepresentation { r, g, b, a }
    }

    fn add_rgba(&mut self, rgba: [f32; 4]) {
        self.modify_rgb((self.r + rgba[0], self.g + rgba[1], self.b + rgba[2]));
        self.modify_a((self.a + rgba[3] as u8) as i32);
    }

    fn add_hsla(&mut self, mut hsla: [f32; 4]) {
        let (h, s, l) = self.hsl();
        self.modify_hsl((h + hsla[0], s + hsla[1], l + hsla[2]));
        self.modify_a((self.a + hsla[3] as u8) as i32);
    }

    fn hsl(&self) -> (f32, f32, f32) {
        return rgb2hsl(self.r, self.g, self.b);
    }

    fn rgb(&self) -> (f32, f32, f32) {
        return (self.r, self.g, self.b);
    }

    fn hsla(&self) -> (f32, f32, f32, u8) {
        let data = rgb2hsl(self.r as f32, self.g as f32, self.b as f32);
        return (data.0, data.1, data.2, self.a);
    }

    fn modify_a(&mut self, mut new_value: i32) {
        new_value = clamp!(0, new_value, 255);
        self.a = new_value as u8;
    }

    fn modify_rgb(&mut self, mut new_value: (f32, f32, f32)) {
        new_value.0 = clamp!(0.0, new_value.0, 255.0);
        new_value.1 = clamp!(0.0, new_value.1, 255.0);
        new_value.2 = clamp!(0.0, new_value.2, 255.0);
        (self.r, self.g, self.b) = new_value;
    }

    fn modify_hsl(&mut self, mut new_value: (f32, f32, f32)) {
        new_value.0 = clamp!(0.0, new_value.0, 359.0);
        new_value.1 = clamp!(0.0, new_value.1, 1.0);
        new_value.2 = clamp!(0.0, new_value.2, 1.0);
        (self.r, self.g, self.b) = hsl2rgb(new_value.0, new_value.1, new_value.2);
    }

    fn get_output_clr(&self, output_type: &OutputType, enable_alpha: bool) -> String {
        return match output_type {
            OutputType::HSL => {
                if enable_alpha {
                    self.tohsla()
                } else {
                    self.tohsl()
                }
            }
            OutputType::ANSI => self.toansi(),
            OutputType::RGB => {
                if enable_alpha {
                    self.torgba()
                } else {
                    self.torgb()
                }
            }
            OutputType::HEX => {
                if enable_alpha {
                    self.tohexa()
                } else {
                    self.tohex()
                }
            }
        };
    }

    fn get_formatted_output_clr(&self, output_type: &OutputType, enable_alpha: bool) -> String {
        return match output_type {
            OutputType::HSL => {
                if enable_alpha {
                    format!("hsla({})", self.tohsla())
                } else {
                    format!("hsl({})", self.tohsl())
                }
            }
            OutputType::HEX => {
                if enable_alpha {
                    format!("#{}", self.tohexa())
                } else {
                    format!("#{}", self.tohex())
                }
            }
            OutputType::ANSI => {
                format!("\\x1b[38;2;{}m", self.toansi())
            }
            OutputType::RGB => {
                if enable_alpha {
                    format!("rgba({})", self.torgba())
                } else {
                    format!("rgb({})", self.torgb())
                }
            }
        };
    }

    fn tohsl(&self) -> String {
        let (h, s, l) = self.hsl();
        return format!("{}, {}, {}", h, (s * 100.0), (l * 100.0));
    }

    fn tohsla(&self) -> String {
        let (h, s, l, a) = self.hsla();
        return format!("{}, {}, {}, {}", h, (s * 100.0), (l * 100.0), a);
    }

    fn torgb(&self) -> String {
        return format!("{}, {}, {}", self.r as u8, self.g as u8, self.b as u8);
    }

    fn torgba(&self) -> String {
        return format!(
            "{}, {}, {}, {}",
            self.r as u8, self.g as u8, self.b as u8, self.a
        );
    }

    fn tohex(&self) -> String {
        return format!(
            "{:02x}{:02x}{:02x}",
            self.r as u8, self.g as u8, self.b as u8
        );
    }

    fn tohexa(&self) -> String {
        return format!(
            "{:02x}{:02x}{:02x}{:02x}",
            self.r as u8, self.g as u8, self.b as u8, self.a
        );
    }

    fn toansi(&self) -> String {
        return format!("{};{};{}", self.r as u8, self.g as u8, self.b as u8);
    }
}

fn cls() {
    print!("\x1b[2J\x1b[0H");
}

unsafe fn query_winsize(fd: i32, ws_struct: &mut libc::winsize) {
    libc::ioctl(fd, libc::TIOCGWINSZ, ws_struct);
}

fn render_rgb(curr_color: &ColorRepresentation, square_count: u32, step: f32, rgb_idx: usize){
    let mut colors = [curr_color.r, curr_color.g, curr_color.b];
    let modifier_idx = rgb_idx;
    colors[modifier_idx] = 0.0;
    let label = ['R', 'G', 'B'][rgb_idx];
    print!("{}", label);
    let mut color = ColorRepresentation::from_color(&format!("rgb({},{},{})", colors[0], colors[1], colors[2]));
    for i in 0..square_count {
        print!("\x1b[38;2;{}m█", color.toansi());
        colors[modifier_idx] = ( i as f32 / square_count as f32 ) * 255.0;
        color.modify_rgb((colors[0], colors[1], colors[2]));
    }
    println!("\x1b[0m");
    render_carrot_on_current_line(([curr_color.r, curr_color.g, curr_color.b][modifier_idx] / 255.0 * 360.0 / step).floor() as usize + 1);
}

fn render_r(curr_color: &ColorRepresentation, square_count: u32, step: f32) {
    render_rgb(curr_color, square_count, step, 0);
}

fn render_g(curr_color: &ColorRepresentation, square_count: u32, step: f32) {
    render_rgb(curr_color, square_count, step, 1);
}

fn render_b(curr_color: &ColorRepresentation, square_count: u32, step: f32) {
    render_rgb(curr_color, square_count, step, 2);
}

fn render_hsl(curr_color: &ColorRepresentation, square_count: u32, step: f32, hsl_idx: usize){
    let (h, s, l) = curr_color.hsl();
    let mut colors = [h, s, l];
    let modifier_idx = hsl_idx;
    colors[modifier_idx] = 0.0;
    let label = ['H', 'S', 'L'][hsl_idx];
    let modifier_multiplier = [360.0, 1.0, 1.0][hsl_idx];
    print!("{}", label);
    let mut color = ColorRepresentation::from_color(&format!("hsl({},{},{})", colors[0], colors[1], colors[2]));
    for i in 0..square_count {
        print!("\x1b[38;2;{}m█", color.toansi());
        colors[modifier_idx] = ( i as f32 / square_count as f32 ) * modifier_multiplier;
        color.modify_hsl((colors[0], colors[1], colors[2]));
    }
    println!("\x1b[0m");
    render_carrot_on_current_line(([h, s, l][modifier_idx] / modifier_multiplier * 360.0 / step).floor() as usize + 1);
}

fn render_h(curr_color: &ColorRepresentation, square_count: u32, step: f32) {
    render_hsl(curr_color, square_count, step, 0);
}

fn render_s(curr_color: &ColorRepresentation, square_count: u32, step: f32) {
    render_hsl(curr_color, square_count, step, 1);
}

fn render_l(curr_color: &ColorRepresentation, square_count: u32, step: f32) {
    render_hsl(curr_color, square_count, step, 2);
}

fn render_a(square_count: u32) {
    print!("A");
    let mut sat_color_rep = ColorRepresentation::from_color("#000000");
    for i in 0..square_count {
        print!("\x1b[38;2;{}m█", sat_color_rep.toansi());
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
    colors: [fn(&ColorRepresentation, u32, f32); 3],
    square_count: u32,
    step: f32,
    selected_item: u8,
    enable_alpha: bool,
) {
    for i in 0..=2 {
        print!("\x1b[{};0H", i * 2 + 1);
        if selected_item == i {
            print!("\x1b[32m");
        }
        colors[i as usize](curr_color, square_count, step);
    }

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
    match program_state.selection_type {
        SelectionType::HSL => render_sliders(
            &program_state.curr_color,
            program_state.curr_color.a,
            [render_h, render_s, render_l],
            square_count,
            step,
            program_state.selected_item,
            program_state.enable_alpha,
        ),
        SelectionType::RGB => render_sliders(
            &program_state.curr_color,
            program_state.curr_color.a,
            [render_r, render_g, render_b],
            square_count,
            step,
            program_state.selected_item,
            program_state.enable_alpha,
        ),
    }
    println!(
        "\x1b[38;2;{}m████████\x1b[0m",
        program_state.curr_color.toansi()
    );
    println!(
        "\x1b[38;2;{}m████████\x1b[0m",
        program_state.curr_color.toansi()
    );
    println!(
        "\x1b[38;2;{}m████████\x1b[0m",
        program_state.curr_color.toansi()
    );
    print!("\x1b[2K");
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
}

enum OutputType {
    HSL,
    RGB,
    HEX,
    ANSI,
}

impl OutputType {
    fn render_output(&self, curr_color: &ColorRepresentation, enable_alpha: bool) {
        println!(
            "{}",
            curr_color.get_formatted_output_clr(self, enable_alpha)
        )
    }
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

    //this variable keeps track of the step for the angle increase for the H rendering
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

    loop {
        let (h, s, l) = program_state.curr_color.hsl();

        render_display(&program_state, square_count, step);

        let bytes_read = reader.read(&mut buf).unwrap();

        let data = String::from_utf8(buf[0..bytes_read].to_vec()).unwrap();

        if data == "q" {
            break;
        }

        let amnt_mult = if data == "l" { 1.0 } else { -1.0 };

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
            }
        }

        if data == "l" || data == "h" {
            match program_state.selection_type {
                SelectionType::HSL => {
                    let mut toadd = [0.0; 4];
                    if program_state.selected_item == 0 || program_state.selected_item == 3 {
                        toadd[0] = 1.0 * amnt_mult;
                    }
                    else {
                        toadd[program_state.selected_item as usize] = 0.01 * amnt_mult;
                    }
                    program_state.curr_color.add_hsla(toadd);
                },
                SelectionType::RGB => {
                    let mut toadd = [0.0; 4];
                    toadd[(program_state.selected_item % 4) as usize] = 1.0 * amnt_mult;
                    program_state.curr_color.add_rgba(toadd);
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
                SelectionType::RGB => SelectionType::HSL,
            };
        } else if data == "o" {
            program_state.output_type = match program_state.output_type {
                OutputType::HSL => OutputType::RGB,
                OutputType::RGB => OutputType::HEX,
                OutputType::HEX => OutputType::ANSI,
                OutputType::ANSI => OutputType::HSL,
            }
        } else if data == "y" {
            let b64 = general_purpose::STANDARD.encode(
                program_state.curr_color.get_formatted_output_clr(
                    &program_state.output_type,
                    program_state.enable_alpha,
                ),
            );
            print!("\x1b]52;c;{}\x07", b64);
        } else if data == "Y" {
            let b64 = general_purpose::STANDARD.encode(
                program_state
                    .curr_color
                    .get_output_clr(&program_state.output_type, program_state.enable_alpha),
            );
            print!("\x1b]52;c;{}\x07", b64);
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
