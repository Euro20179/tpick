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

    let mut h;
    let s;
    let l;

    let delta = max - min;

    l = (max + min) / 2.0;

    if delta == 0.0 {
        s = 0.0;
        h = 0.0;
    }
    else {
        if r == max {
            h = (g - b) / delta * 60.0;
        }
        else if g == max {
            h = (b - r) / delta * 60.0 + 120.0;
        }
        else {
            h = (r - g) / delta * 60.0 + 240.0;
        }

        h %= 360.0;

        s = if l <= 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - (max + min))
        }
    }

    return (h, s, l);
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
        let mut a = 1;
        let get_next = |split: &mut Split<'_, &str>| split.next().unwrap().trim().parse().unwrap();
        if clr.contains(";"){
            let mut items = clr.split(";");
            r = get_next(&mut items);
            g = get_next(&mut items);
            b = get_next(&mut items);
        }
        else if clr.starts_with("rgba") {
            let mut items = clr[5..clr.len() - 1].split(",");
            r = get_next(&mut items);
            g = get_next(&mut items);
            b = get_next(&mut items);
            a = items.next().unwrap().trim().parse().unwrap();
        }
        else if clr.starts_with("rgb") {
            let mut items = clr[4..clr.len() - 1].split(",");
            r = get_next(&mut items);
            g = get_next(&mut items);
            b = get_next(&mut items);
        }
        else if clr.starts_with("hsla") {
            let mut items = clr[5..clr.len() - 1].split(",");
            let h: f32 = get_next(&mut items);
            let s: f32 = get_next(&mut items);
            let l: f32 = get_next(&mut items);
            a = items.next().unwrap().trim().parse().unwrap();
            (r, g, b) = hsl2rgb(h, s, l);
        }
        else if clr.starts_with("hsl") {
            let mut items = clr[4..clr.len() - 1].split(",");
            let h: f32 = get_next(&mut items);
            let s: f32 = get_next(&mut items);
            let l: f32 = get_next(&mut items);
            (r, g, b) = hsl2rgb(h, s, l);
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

    fn modify_hsl(&mut self, mut new_value: (f32, f32, f32)){
        if new_value.0 < 0.0 || new_value.0 > 360.0{
            new_value.0 = max!(min!(360.0, new_value.0), 0.0);
        }
        if new_value.1 < 0.0 || new_value.1 > 1.0 {
            new_value.1 = max!(min!(1.0, new_value.0), 0.0);
        }
        if new_value.2 < 0.0 || new_value.2 > 1.0 {
            new_value.2 = max!(min!(1.0, new_value.0), 0.0);
        }
        (self.r, self.g, self.b) = hsl2rgb(new_value.0, new_value.1, new_value.2);
    }

    fn tohsl(&self) -> String {
        let (h, s, l) = self.hsl();
        return format!("{}, {}, {}", h, s, l);
    }

    fn torgb(&self) -> String {
        return format!("{}, {}, {}", self.r, self.g, self.b);
    }

    fn tohex(&self) -> String {
        return format!("{:02x}{:02x}{:02x}", self.r as u8, self.g as u8, self.b as u8);
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

fn render_h(h: f32, s: f32, l: f32, hsquares: &Vec<ColorRepresentation>){
    print!("H");
    let mut sat_color_rep = ColorRepresentation::from_color(&format!("hsl({}, {}, {})", 0.0, s, l));
    for i in 0..hsquares.len() {
        print!("\x1b[38;2;{}m█", sat_color_rep.toansi());
        sat_color_rep.modify_hsl(((i as f32 / hsquares.len() as f32) * 360.0, s, l))
    }
    // for sq in hsquares {
    //     print!("\x1b[38;2;{}m█", sq.toansi());
    //     print!("\x1b[0m");
    // }
    println!("\x1b[0m");
}

fn render_s(h: f32, s: f32, l: f32, hsquares: &Vec<ColorRepresentation>){
    print!("S");
    let mut sat_color_rep = ColorRepresentation::from_color(&format!("hsl({}, {}, {})", h, 0.0, l));
    for i in 0..hsquares.len(){
        print!("\x1b[38;2;{}m█", sat_color_rep.toansi());
        sat_color_rep.modify_hsl((h, (i as f32 / hsquares.len() as f32), l))
    }
    println!("\x1b[0m");
}

fn render_l(h: f32, s: f32, l: f32, hsquares: &Vec<ColorRepresentation>){
    print!("L");
    let mut sat_color_rep = ColorRepresentation::from_color(&format!("hsl({}, {}, {})", h, s, 0.0));
    for i in 0..hsquares.len(){
        print!("\x1b[38;2;{}m█", sat_color_rep.toansi());
        sat_color_rep.modify_hsl((h, s, (i as f32 / hsquares.len() as f32)))
    }
    println!("\x1b[0m");
}

fn render_display(curr_color: &ColorRepresentation, hsquares: &Vec<ColorRepresentation>, step: f32, selected_item: &SelectedItem, input_type: &InputType, output_type: &OutputType){
    let (h, s, l) = curr_color.hsl();
    if let SelectedItem::H = selected_item{
        print!("\x1b[32m");
    }
    render_h(h, s, l, hsquares);
    println!(" {}^", " ".repeat((curr_color.hsl().0 / step).floor() as usize));
    if let SelectedItem::S = selected_item {
        print!("\x1b[32m");
    }
    render_s(h, s, l, hsquares);
    println!(" {}^", " ".repeat((curr_color.hsl().1 * 360.0 / step).floor() as usize));
    if let SelectedItem::L = selected_item {
        print!("\x1b[32m");
    }
    render_l(h, s, l, hsquares);
    println!(" {}^", " ".repeat((curr_color.hsl().2 * 360.0 / step).floor() as usize));
    println!("\x1b[38;2;{}m████████\x1b[0m", curr_color.toansi());
    println!("\x1b[38;2;{}m████████\x1b[0m", curr_color.toansi());
    println!("\x1b[38;2;{}m████████\x1b[0m", curr_color.toansi());
    output_type.render_output(curr_color);

}

enum SelectedItem {
    H,
    S,
    L
}

enum OutputType {
    HSL,
    RGB,
    HEX
}

impl OutputType {
    fn render_output(&self, curr_color: &ColorRepresentation) {
        match self {
            OutputType::HSL =>  println!("hsl{:?}", curr_color.hsl()),
            OutputType::RGB => println!("rgb({}, {}, {})", curr_color.r, curr_color.g, curr_color.b),
            OutputType::HEX => println!("#{}", curr_color.tohex()),
        }
    }
}

enum InputType{
    HSL,
    RGB
}

fn main() {
    let mut tios = Termios::from_fd(0).unwrap();
    let mut tios_initial = Termios::from_fd(0).unwrap();
    let _ = termios::tcgetattr(0, &mut tios);
    let _ = termios::tcgetattr(0, &mut tios_initial);

    tios.c_lflag &= !(termios::ICANON | termios::ECHO);
    termios::tcsetattr(0, termios::TCSANOW, &tios).unwrap();

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

    let mut hsquares = vec![];

    for i in 0..((361.0 / step).ceil() as i32) {
        hsquares.push(ColorRepresentation::from_color(&format!("hsl({}.0, 1.0, 0.5)", i as f32 * step)))
    }

    let mut curr_color = ColorRepresentation::from_color("rgb(0, 255, 255)");

    let mut selected_item = SelectedItem::H;

    let mut input_type = InputType::HSL;
    let mut output_type = OutputType::HSL;

    loop {

        let (h, s, l) = curr_color.hsl();

        cls();

        render_display(&curr_color, &hsquares, step, &selected_item, &input_type, &output_type);

        let bytes_read = reader.read(&mut buf).unwrap();

        let data = String::from_utf8(buf[0..bytes_read].to_vec()).unwrap();

        if data == "q" {
            break;
        }

        let amnt_mult = if data == "l" {
            1.0
        } else { -1.0 };

        for i in 1..=9 {
            if data == i.to_string() {
                let mult = i as f32 / 10.0;
                match selected_item {
                    SelectedItem::H => curr_color.modify_hsl((360.0 * mult, s, l)),
                    SelectedItem::S => curr_color.modify_hsl((h, mult, l)),
                    SelectedItem::L => curr_color.modify_hsl((h, s, mult)),
                }
            }
        }

        if data == "l" || data == "h" {
            match selected_item {
                SelectedItem::H => {
                    let mod_amount = 1.0 * amnt_mult;
                    curr_color.modify_hsl((h + mod_amount, s, l))
                }
                SelectedItem::S => {
                    let mod_amount = 0.01 * amnt_mult;
                    curr_color.modify_hsl((h, s + mod_amount, l))
                }
                SelectedItem::L => {
                    let mod_amount = 0.01 * amnt_mult;
                    curr_color.modify_hsl((h, s, l + mod_amount))
                }
            }
        } else if data == "k" {
            selected_item = match selected_item {
                SelectedItem::H => SelectedItem::L,
                SelectedItem::S => SelectedItem::H,
                SelectedItem::L => SelectedItem::S,
            }
        } else if data == "j" {
            selected_item = match selected_item {
                SelectedItem::H => SelectedItem::S,
                SelectedItem::S => SelectedItem::L,
                SelectedItem::L => SelectedItem::H,
            }
        }
        else if data == "o" {
            output_type = match output_type {
                OutputType::HSL => OutputType::RGB,
                OutputType::RGB => OutputType::HEX,
                OutputType::HEX => OutputType::HSL
            }
        }
        else if data == "y" {
            let b64 = general_purpose::STANDARD.encode(match output_type {
                OutputType::HSL => format!("hsl({})", curr_color.tohsl()),
                OutputType::HEX => format!("#{}", curr_color.tohex()),
                OutputType::RGB => format!("rgb({})", curr_color.torgb()),
            });
            print!("\x1b]52;c;{}\x07", b64);
        }
        else if data == "Y" {
            let b64 = general_purpose::STANDARD.encode(match output_type {
                OutputType::HSL => format!("{}", curr_color.tohsl()),
                OutputType::HEX => format!("{}", curr_color.tohex()),
                OutputType::RGB => format!("{}", curr_color.torgb()),
            });
            print!("\x1b]52;c;{}\x07", b64);
        }
    }
    termios::tcsetattr(0, termios::TCSANOW, &tios_initial).unwrap();
}
