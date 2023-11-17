use std::fmt::Display;
use std::fmt::LowerHex;
use std::str::Split;

use std::io::Write;

use crate::OutputType;
use crate::rgb2hsl;
use crate::hsl2rgb;

#[derive(Clone, Copy)]
pub struct ColorRepresentation {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: u8,
}

impl ColorRepresentation {
    pub fn from_color(clr: &str) -> ColorRepresentation {
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
            let s: f32 = get_next(&mut items);
            let l: f32 = get_next(&mut items);
            a = items.next().unwrap().trim().parse().unwrap();
            (r, g, b) = hsl2rgb(h, s, l);
        } else if clr.starts_with("hsl") {
            let mut items = clr[4..clr.len() - 1].split(",");
            let h: f32 = get_next(&mut items);
            let s: f32 = get_next(&mut items);
            let l: f32 = get_next(&mut items);
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

    pub fn add_rgba(&mut self, rgba: [f32; 4]) {
        self.modify_rgb((self.r + rgba[0], self.g + rgba[1], self.b + rgba[2]));
        self.modify_a((self.a + rgba[3] as u8) as i32);
    }

    pub fn add_hsla(&mut self, hsla: [f32; 4]) {
        let (h, s, l) = self.hsl();
        self.modify_hsl((h + hsla[0], s + hsla[1], l + hsla[2]));
        self.modify_a(self.a as i32  + hsla[3] as i32);
    }

    pub fn hsl(&self) -> (f32, f32, f32) {
        return rgb2hsl(self.r, self.g, self.b);
    }

    pub fn rgb(&self) -> (f32, f32, f32) {
        return (self.r, self.g, self.b);
    }

    pub fn modify_a(&mut self, mut new_value: i32) {
        new_value = clamp_with_bel!(0, new_value, 255);
        self.a = new_value as u8;
    }

    pub fn modify_rgb(&mut self, mut new_value: (f32, f32, f32)) {
        new_value.0 = clamp_with_bel!(0.0, new_value.0, 255.0);
        new_value.1 = clamp_with_bel!(0.0, new_value.1, 255.0);
        new_value.2 = clamp_with_bel!(0.0, new_value.2, 255.0);
        (self.r, self.g, self.b) = new_value;
    }

    pub fn modify_hsl(&mut self, mut new_value: (f32, f32, f32)) {
        new_value.0 = clamp_with_bel!(0.0, new_value.0, 359.0);
        new_value.1 = clamp_with_bel!(0.0, new_value.1, 100.0);
        new_value.2 = clamp_with_bel!(0.0, new_value.2, 100.0);
        (self.r, self.g, self.b) = hsl2rgb(new_value.0, new_value.1, new_value.2);
    }

    pub fn get_output_clr(&self, output_type: &OutputType, enable_alpha: bool) -> String {
        return match output_type {
            OutputType::HSL => self.tohsl(enable_alpha),
            OutputType::ANSI => self.toansi(false),
            OutputType::RGB => self.torgb(enable_alpha),
            OutputType::HEX => self.tohex(enable_alpha),
            OutputType::CUSTOM(fmt) => self.tofmt(fmt),
            OutputType::ALL => {
                format!(
                    "{}\n{}\n{}\n{}",
                    self.tohsl(enable_alpha),
                    self.torgb(enable_alpha),
                    self.tohex(enable_alpha),
                    self.toansi(false)
                )
            }
        };
    }

    pub fn get_formatted_output_clr(&self, output_type: &OutputType, enable_alpha: bool) -> String {
        return match output_type {
            OutputType::CUSTOM(fmt) => self.tofmt(fmt),
            OutputType::ALL => {
                format!(
                    "{}\n{}\n{}\n{}",
                    self.get_formatted_output_clr(&OutputType::HSL, enable_alpha),
                    self.get_formatted_output_clr(&OutputType::RGB, enable_alpha),
                    self.get_formatted_output_clr(&OutputType::HEX, enable_alpha),
                    self.get_formatted_output_clr(&OutputType::ANSI, enable_alpha)
                )
            }
            OutputType::HSL => {
                if enable_alpha {
                    format!("hsla({})", self.tohsl(enable_alpha))
                } else {
                    format!("hsl({})", self.tohsl(false))
                }
            }
            OutputType::HEX => format!("#{}", self.tohex(enable_alpha)),
            OutputType::ANSI => {
                format!("\\x1b[38;2;{}m", self.toansi(false))
            }
            OutputType::RGB => {
                if enable_alpha {
                    format!("rgba({})", self.torgb(enable_alpha))
                } else {
                    format!("rgb({})", self.torgb(false))
                }
            }
        };
    }

    pub fn tofmt(&self, fmt: &str) -> String {
        enum FormatType {
            String,
            Hex
        }
        impl FormatType {
            fn format<T: LowerHex + Display>(&self, val: T) -> String{
                match self {
                    Self::String => format!("{}", val),
                    Self::Hex => format!("{:2x}", val)
                }
            }
        }
        //TODO: add some kind o FormatType::format(&u16) -> String to avoid code repetition as
        //exampled by ch == "R", "G", "B"
        let mut result = String::new();
        //TODO: create custom formatter that allows for string padding and stuff
        //also allow for converting b10 -> b16 so the user can get #RRGGBB output
        let mut is_fmt_char = false;
        let mut fmt_char_type = FormatType::String;
        let (h, s, l) = self.hsl();
        for i in 0..fmt.len() {
            let ch = &fmt[i..i + 1];
            if ch == "%" {
                fmt_char_type = FormatType::String;
                is_fmt_char = true;
                continue;
            }
            if is_fmt_char {
                if ch == "R" {
                    result += &fmt_char_type.format(self.r as u8);
                } else if ch == "G" {
                    result += &fmt_char_type.format(self.g as u8);
                } else if ch == "B" {
                    result += &fmt_char_type.format(self.b as u8);
                } else if ch == "H" {
                    result += &fmt_char_type.format(h as u8);
                } else if ch == "S" {
                    result += &fmt_char_type.format(s as u8);
                } else if ch == "L" {
                    result += &fmt_char_type.format(l as u8);
                }
                else if ch == "x" {
                    fmt_char_type = FormatType::Hex;
                    continue;
                }
            } else {
                result += ch;
            }
            is_fmt_char = false;
        }
        return result;
    }

    pub fn tohsl(&self, enable_alpha: bool) -> String {
        let (h, s, l) = self.hsl();
        if enable_alpha {
            return format!("{:.2}, {:.2}, {:.2}, {:.2}", h, s, l, self.a);
        }
        return format!("{:.2}, {:.2}, {:.2}", h, s, l);
    }

    pub fn torgb(&self, enable_alpha: bool) -> String {
        if enable_alpha {
            return format!(
                "{}, {}, {}, {}",
                self.r as u8, self.g as u8, self.b as u8, self.a
            );
        }
        return format!("{}, {}, {}", self.r as u8, self.g as u8, self.b as u8);
    }

    pub fn tohex(&self, enable_alpha: bool) -> String {
        if enable_alpha {
            return format!(
                "{:02x}{:02x}{:02x}{:02x}",
                self.r as u8, self.g as u8, self.b as u8, self.a
            );
        }
        return format!(
            "{:02x}{:02x}{:02x}",
            self.r as u8, self.g as u8, self.b as u8
        );
    }

    pub fn toansi(&self, _enable_alpha: bool) -> String {
        return format!("{};{};{}", self.r as u8, self.g as u8, self.b as u8);
    }
}

impl Display for ColorRepresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tohex(true))
    }
}
