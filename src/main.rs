use base64::engine::general_purpose;
use base64::prelude::*;
use std::io::{BufRead, Read};
use std::os::fd::AsFd;
use std::str::Split;

use termios::Termios;

macro_rules! min {
    ($i1:expr, $i2:expr) => {
        if $i1 < $i2 {
            $i1
        } else {
            $i2
        }
    };
}

macro_rules! max {
    ($i1:expr, $i2:expr) => {
        if $i1 > $i2 {
            $i1
        } else {
            $i2
        }
    };
}

fn hsl2rgb(mut h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let (r, g, b);
    h %= 360.0;
    let lp = if l < 0.5 { l } else { 1.0 - l };

    let max = l + lp * s;
    let min = l - lp * s;

    let f = |x| x / 60.0 * (max - min) + min;

    if h < 60.0 {
        r = max;
        g = f(h);
        b = min;
    }
    else if h < 120.0 {
        r = f(120.0 - h);
        g = max;
        b = min;
    }
    else if h < 180.0 {
        r = min;
        g = max;
        b = f(h - 120.0);
    }
    else if h < 240.0 {
        r = min;
        g = f(240.0 - h);
        b = max;
    }
    else if h < 300.0 {
        r = f(h - 240.0);
        g = min;
        b = max;
    }
    else {
        r = max;
        g = min;
        b = f(360.0 - h);
    }
    return ((r * 255.0), (g * 255.0), (b * 255.0));
}

fn rgb2hsl(mut r: f32, mut g: f32, mut b: f32) -> (f32, f32, f32){
    r /= 255.0;
    g /= 255.0;
    b /= 255.0;
    let min = min!(min!(r, g), b);
    let max = max!(max!(r, g), b);

    let mut h ;
    let s ;
    let l ;

    let delta = max - min;

    l = (max + min) / 2.0;

    if delta == 0.0 {
        h = 0.0;
    }
    else if max == r {
        h = ((g - b) / delta) % 6.0;
    }
    else if max == g {
        h = (b - r) / delta + 2.0;
    }
    else {
        h = (r - g) / delta + 4.0;
    }

    h = (h * 60.0).round();

    if h < 0.0 {
        h += 360.0;
    }

    s = if delta == 0.0 { 0.0 } else { delta / (1.0 - (2.0 * l - 1.0).abs())};

    return (h, s, l);
}

macro_rules! clamp {
    ($min:expr, $value:expr, $max:expr) => {
        max!(min!($max, $value), $min)
    };
}

struct ColorRepresentation {
    r: f32,
    g: f32,
    b: f32,
    a: u8,
}

impl ColorRepresentation {
    fn from_color(clr: &str) -> ColorRepresentation {
        let mut r: f32 = 0.0;
        let mut g: f32  = 0.0;
        let mut b: f32  = 0.0;
        let mut a = 255;

        let get_next = |split: &mut Split<'_, &str>| split.next().unwrap().trim().parse().unwrap();
        let get_rgb = |items: &mut Split<'_, &str>| {
            return (get_next(items), get_next(items), get_next(items));
        };

        if clr.starts_with("\\x1b") {
            //\x1b[38;2;
            let mut items = clr[10..clr.len() - 1].split(";");
            (r, g, b) = get_rgb(&mut items);
        }
        else if clr.contains(";"){
            let mut items = clr.split(";");
            (r, g, b) = get_rgb(&mut items);
        }
        else if clr.starts_with("rgba") {
            let mut items = clr[5..clr.len() - 1].split(",");
            (r, g, b) = get_rgb(&mut items);
            a = items.next().unwrap().trim().parse().unwrap();
        }
        else if clr.starts_with("rgb") {
            let mut items = clr[4..clr.len() - 1].split(",");
            (r, g, b) = get_rgb(&mut items);
        }
        else if clr.starts_with("hsla") {
            let mut items = clr[5..clr.len() - 1].split(",");
            let h: f32 = get_next(&mut items);
            let s: f32 = get_next(&mut items) / 100.0;
            let l: f32 = get_next(&mut items) / 100.0;
            a = items.next().unwrap().trim().parse().unwrap();
            (r, g, b) = hsl2rgb(h, s, l);
        }
        else if clr.starts_with("hsl") {
            let mut items = clr[4..clr.len() - 1].split(",");
            let h: f32 = get_next(&mut items);
            let s: f32 = get_next(&mut items) / 100.0;
            let l: f32 = get_next(&mut items) / 100.0;
            (r, g, b) = hsl2rgb(h, s, l);
        }
        //#RGB or #RGBA or #RRGGBB or #RRGGBBAA
        else if clr.starts_with("#") && (clr.len() == 4 || clr.len() == 5 || clr.len() == 7 || clr.len() == 9) {
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
            let color_dat = &clr[1..];
        }
        ColorRepresentation {
            r,
            g,
            b,
            a
        }
    }

    fn hsl(&self) -> (f32, f32, f32) {
        return rgb2hsl(self.r as f32, self.g as f32, self.b as f32);
    }

    fn hsla(&self) -> (f32, f32, f32, u8) {
        let data = rgb2hsl(self.r as f32, self.g as f32, self.b as f32);
        return (data.0, data.1, data.2, self.a);
    }

    fn modify_a(&mut self, mut new_value: i32) {
        if new_value < 0 {
            new_value = 0;
        }
        if new_value > 255 {
            new_value = 255;
        }
        self.a = new_value as u8;
    }

    fn modify_rgb(&mut self, mut new_value: (f32, f32, f32)){
        new_value.0 = clamp!(0.0, new_value.0, 255.0);
        new_value.1 = clamp!(0.0, new_value.1, 255.0);
        new_value.2 = clamp!(0.0, new_value.2, 255.0);
        (self.r, self.g, self.b) = new_value;
    }

    fn modify_hsl(&mut self, mut new_value: (f32, f32, f32)){
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
            },
            OutputType::ANSI => {
                self.toansi()
            },
            OutputType::RGB => {
                if enable_alpha {
                    self.torgba()
                } else {
                    self.torgb()
                }
            },
            OutputType::HEX => {
                if enable_alpha {
                    self.tohexa()
                } else {
                    self.tohex()
                }
            }
        }
    }

    fn get_formatted_output_clr(&self, output_type: &OutputType, enable_alpha: bool) -> String {
        return match output_type {
            OutputType::HSL => {
                if enable_alpha {
                    format!("hsla({})", self.tohsla())
                } else {
                    format!("hsl({})", self.tohsl())
                }
            },
            OutputType::HEX => {
                if enable_alpha {
                    format!("#{}", self.tohexa())
                } else {
                    format!("#{}", self.tohex())
                }
            },
            OutputType::ANSI => {
                format!("\\x1b[38;2;{}m", self.toansi())
            },
            OutputType::RGB => {
                if enable_alpha {
                    format!("rgba({})", self.torgba())
                } else {
                    format!("rgb({})", self.torgb())
                }
            }
        }
    }

    fn tohsl(&self) -> String {
        let (h, s, l) = self.hsl();
        return format!("{}, {}, {}", h, s * 100.0, l * 100.0);
    }

    fn tohsla(&self) -> String {
        let (h, s, l, a) = self.hsla();
        return format!("{}, {}, {}, {}", h, s * 100.0, l * 100.0, a);
    }

    fn torgb(&self) -> String {
        return format!("{}, {}, {}", self.r, self.g, self.b);
    }

    fn torgba(&self) -> String {
        return format!("{}, {}, {}, {}", self.r, self.g, self.b, self.a);
    }

    fn tohex(&self) -> String {
        return format!("{:02x}{:02x}{:02x}", self.r as u8, self.g as u8, self.b as u8);
    }

    fn tohexa(&self) -> String {
        return format!("{:02x}{:02x}{:02x}{:02x}", self.r as u8, self.g as u8, self.b as u8, self.a);
    }

    fn toansi(&self) -> String {
        return format!("{};{};{}", self.r.round() as u8, self.g.round() as u8, self.b.round() as u8);
    }
}

fn cls() {
    print!("\x1b[2J\x1b[0H");
}

unsafe fn query_winsize(fd: i32, ws_struct: &mut libc::winsize){
    libc::ioctl(fd, libc::TIOCGWINSZ, ws_struct);
}

fn render_h(h: f32, s: f32, l: f32, square_count: u32){
    print!("\x1b[0H");
    print!("H");
    let mut sat_color_rep = ColorRepresentation::from_color(&format!("hsl({}, {}, {})", 0.0, s, l));
    for i in 0..square_count {
        print!("\x1b[38;2;{}m█", sat_color_rep.toansi());
        sat_color_rep.modify_hsl(((i as f32 / square_count as f32) * 360.0, s, l))
    }
    // for sq in hsquares {
    //     print!("\x1b[38;2;{}m█", sq.toansi());
    //     print!("\x1b[0m");
    // }
    println!("\x1b[0m");
}

fn render_s(h: f32, s: f32, l: f32, square_count: u32){
    print!("\x1b[3;0H");
    print!("S");
    let mut sat_color_rep = ColorRepresentation::from_color(&format!("hsl({}, {}, {})", h, 0.0, l));
    for i in 0..square_count{
        print!("\x1b[38;2;{}m█", sat_color_rep.toansi());
        sat_color_rep.modify_hsl((h, (i as f32 / square_count as f32), l))
    }
    println!("\x1b[0m");
}

fn render_l(h: f32, s: f32, l: f32, square_count: u32){
    print!("\x1b[5;0H");
    print!("L");
    let mut sat_color_rep = ColorRepresentation::from_color(&format!("hsl({}, {}, {})", h, s, 0.0));
    for i in 0..square_count{
        print!("\x1b[38;2;{}m█", sat_color_rep.toansi());
        sat_color_rep.modify_hsl((h, s, (i as f32 / square_count as f32)))
    }
    println!("\x1b[0m");
}

fn render_a(square_count: u32) {
    print!("\x1b[7;0H");
    print!("A");
    let mut sat_color_rep = ColorRepresentation::from_color("#000000");
    for i in 0..square_count{
        print!("\x1b[38;2;{}m█", sat_color_rep.toansi());
        sat_color_rep.modify_hsl((0.0, 0.0, (i as f32 / square_count as f32)))
    }
    println!("\x1b[0m");
}

fn render_hsl_display(curr_color: &ColorRepresentation, square_count: u32, step: f32, selected_item: &SelectedItemHSL, enable_alpha: bool){
    let (h, s, l) = curr_color.hsl();
    if let SelectedItemHSL::H = selected_item{
        print!("\x1b[32m");
    }
    render_h(h, s, l, square_count);
    println!("\x1b[2K {}^", " ".repeat((curr_color.hsl().0 / step).floor() as usize));
    if let SelectedItemHSL::S = selected_item {
        print!("\x1b[32m");
    }
    render_s(h, s, l, square_count);
    //everything is measured as a percentage of 360 to keep the relative positioning of everything
    //the same
    println!("\x1b[2K {}^", " ".repeat((curr_color.hsl().1 * 360.0 / step).floor() as usize));
    if let SelectedItemHSL::L = selected_item {
        print!("\x1b[32m");
    }
    render_l(h, s, l, square_count);
    println!("\x1b[2K {}^", " ".repeat((curr_color.hsl().2 * 360.0 / step).floor() as usize));
    if enable_alpha {
        if let SelectedItemHSL::A = selected_item{
            print!("\x1b[32m");
        }
        render_alpha_display(curr_color, square_count, step);
    }
}

fn render_alpha_display(curr_color: &ColorRepresentation, square_count: u32, step: f32){
    render_a(square_count);
    println!("\x1b[2K {}^", " ".repeat(((curr_color.a as f32 / 255.0 * 360.0) / step).floor() as usize));
}

fn render_display(curr_color: &ColorRepresentation, square_count: u32, step: f32, input_type: &SelectionType, output_type: &OutputType, enable_alpha: bool){
    match input_type {
        SelectionType::HSL(item) => render_hsl_display(curr_color, square_count, step, item, enable_alpha),
        _ => todo!()
    }
    println!("\x1b[38;2;{}m████████\x1b[0m", curr_color.toansi());
    println!("\x1b[38;2;{}m████████\x1b[0m", curr_color.toansi());
    println!("\x1b[38;2;{}m████████\x1b[0m", curr_color.toansi());
    print!("\x1b[2K");
    output_type.render_output(curr_color, enable_alpha);

}

#[derive(Copy, Clone)]
enum SelectedItemHSL {
    H,
    S,
    L,
    A
}

#[derive(Copy, Clone)]
enum SelectedItemRGB {
    H,
    S,
    L,
    A
}

#[derive(Copy, Clone)]
enum SelectionType{
    HSL(SelectedItemHSL),
    RGB(SelectedItemRGB)
}

enum OutputType {
    HSL,
    RGB,
    HEX,
    ANSI
}

impl OutputType {
    fn render_output(&self, curr_color: &ColorRepresentation, enable_alpha: bool) {
        println!("{}", curr_color.get_formatted_output_clr(self, enable_alpha))
    }
}

fn read_clipboard(reader: &mut std::io::Stdin) -> String{
    println!("\x1b]52;c;?\x07");

    let mut clip_buf = String::new();
    let mut b = [0; 1];
    loop{
        reader.read_exact(&mut b).unwrap();
        if b[0] == 7{
            break;
        }
        clip_buf += &String::from(b[0] as char);
    }

    let clip_data = clip_buf.split(";").nth(2).unwrap();

    return String::from_utf8(general_purpose::STANDARD.decode(clip_data).unwrap()).unwrap();

}

//returns oldtermios, newtermios
fn setup_term() -> (termios::Termios, termios::Termios){
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

    let mut wsz = libc::winsize{
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0
    };

    unsafe { query_winsize(0, &mut wsz); }

    //this variable keeps track of the step for the angle increase for the H rendering
    let step = (360.0 / wsz.ws_col as f32).ceil();

    let square_count = (361.0 / step).ceil() as u32;

    let mut curr_color = ColorRepresentation::from_color("rgb(0, 255, 255)");

    let mut input_type = SelectionType::HSL(SelectedItemHSL::H);
    let mut output_type = OutputType::HSL;

    let mut enable_alpha = false;

    cls();

    loop {

        let (h, s, l) = curr_color.hsl();

        render_display(&curr_color, square_count, step, &input_type, &output_type, enable_alpha);


        let bytes_read = reader.read(&mut buf).unwrap();

        let data = String::from_utf8(buf[0..bytes_read].to_vec()).unwrap();

        if data == "q" {
            break;
        }

        let amnt_mult = if data == "l" {
            1.0
        } else { -1.0 };

        for i in 0..=9 {
            if data == i.to_string() {
                let mult = i as f32 / 10.0;
                match input_type {
                    SelectionType::HSL(selected_item) => {
                        match selected_item {
                            SelectedItemHSL::H => curr_color.modify_hsl((359.0 * mult, s, l)),
                            SelectedItemHSL::S => curr_color.modify_hsl((h, mult, l)),
                            SelectedItemHSL::L => curr_color.modify_hsl((h, s, mult)),
                            SelectedItemHSL::A => curr_color.modify_a((255.0 * mult) as i32)
                        }
                    },
                    _ => todo!()
                }
            }
        }

        if data == "$" {
            let mult = 1.0;
            match input_type {
                SelectionType::HSL(selected_item) => {
                    match selected_item {
                        SelectedItemHSL::H => curr_color.modify_hsl((359.0 * mult, s, l)),
                        SelectedItemHSL::S => curr_color.modify_hsl((h, mult, l)),
                        SelectedItemHSL::L => curr_color.modify_hsl((h, s, mult)),
                        SelectedItemHSL::A => curr_color.modify_a((255.0 * mult) as i32)
                    }
                },
                _ => todo!()
            }
        }

        if data == "l" || data == "h" {
            match input_type {
                SelectionType::HSL(selected_item) => match selected_item {
                    SelectedItemHSL::H => {
                        let mod_amount = 1.0 * amnt_mult;
                        curr_color.modify_hsl((h + mod_amount, s, l))
                    }
                    SelectedItemHSL::S => {
                        let mod_amount = 0.01 * amnt_mult;
                        curr_color.modify_hsl((h, s + mod_amount, l))
                    }
                    SelectedItemHSL::L => {
                        let mod_amount = 0.01 * amnt_mult;
                        curr_color.modify_hsl((h, s, l + mod_amount))
                    }
                    SelectedItemHSL::A => {
                        let mod_amount = 1.0 * amnt_mult;
                        curr_color.modify_a((curr_color.a as f32 + mod_amount) as i32)
                    }
                },
                _ => todo!()
            }
        } else if data == "k" {
            input_type = match input_type {
                SelectionType::HSL(selected_item) => SelectionType::HSL(match selected_item {
                    SelectedItemHSL::H => if enable_alpha { SelectedItemHSL::A } else { SelectedItemHSL::L },
                    SelectedItemHSL::S => SelectedItemHSL::H,
                    SelectedItemHSL::L => SelectedItemHSL::S,
                    SelectedItemHSL::A => SelectedItemHSL::L,
                }),
                _ => todo!()
            }
        } else if data == "j" {
            input_type = SelectionType::HSL(match input_type {
                SelectionType::HSL(selected_item) => match selected_item{
                    SelectedItemHSL::H => SelectedItemHSL::S,
                    SelectedItemHSL::S => SelectedItemHSL::L,
                    SelectedItemHSL::L => if enable_alpha { SelectedItemHSL::A } else { SelectedItemHSL::H },
                    SelectedItemHSL::A => SelectedItemHSL::H
                },
                _ => todo!()
            })
        }
        else if data == "o" {
            output_type = match output_type {
                OutputType::HSL => OutputType::RGB,
                OutputType::RGB => OutputType::HEX,
                OutputType::HEX => OutputType::ANSI,
                OutputType::ANSI => OutputType::HSL,
            }
        }
        else if data == "y" {
            let b64 = general_purpose::STANDARD.encode(curr_color.get_formatted_output_clr(&output_type, enable_alpha));
            print!("\x1b]52;c;{}\x07", b64);
        }
        else if data == "Y" {
            let b64 = general_purpose::STANDARD.encode(curr_color.get_output_clr(&output_type, enable_alpha));
            print!("\x1b]52;c;{}\x07", b64);
        }

        else if data == "p" {
            let data = read_clipboard(&mut reader);
            curr_color = ColorRepresentation::from_color(&data);
        }
        else if data == "a" {
            enable_alpha = !enable_alpha;
        }
    }
    termios::tcsetattr(0, termios::TCSANOW, &tios_initial).unwrap();
}
