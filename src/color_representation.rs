use std::fmt::Display;
use std::fmt::LowerHex;
use std::str::Split;

use crate::color_conversions::cymk2rgb;
use crate::color_conversions::hex62rgb;
use crate::color_conversions::name_to_hex;
use crate::color_conversions::rgb2ansi256;
use crate::color_conversions::rgb2cymk;
use crate::color_conversions::ColorNameStandard;
use crate::hsl2rgb;
use crate::rgb2hsl;
use crate::OutputType;

#[macro_export]
macro_rules! hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => {hashmap!($($key => $value),+)};
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = hashmap!(@count $($key),*);
            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    }
}

#[derive(Clone, Copy)]
pub struct ColorRepresentation {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: u8,
}

impl ColorRepresentation {
    pub fn from_color(clr: &str, clr_name_standard: &ColorNameStandard) -> ColorRepresentation {
        let r: f32;
        let g: f32;
        let b: f32;
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
        } else if clr.starts_with("cymk") {
            let mut items = clr[5..clr.len() - 1].split(",");
            let c: f32 = get_next(&mut items);
            let y: f32 = get_next(&mut items);
            let m: f32 = get_next(&mut items);
            let k: f32 = get_next(&mut items);
            (r, g, b) = cymk2rgb(c, y, m, k);
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
        } else {
            let (r8, g8, b8) = hex62rgb(&name_to_hex(clr, clr_name_standard)[1..]);
            (r, g, b) = (r8 as f32, g8 as f32, b8 as f32);
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
        self.modify_a(self.a as i32 + hsla[3] as i32);
    }

    pub fn add_cymka(&mut self, cymka: [f32; 5]) {
        let (c, y, m, k) = self.cymk();
        self.modify_cymk((c + cymka[0], y + cymka[1], m + cymka[2], k + cymka[3]));
        self.modify_a(self.a as i32 + cymka[4] as i32);
    }

    pub fn hsl(&self) -> (f32, f32, f32) {
        return rgb2hsl(self.r, self.g, self.b);
    }

    pub fn cymk(&self) -> (f32, f32, f32, f32) {
        return rgb2cymk(self.r, self.g, self.b);
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

    pub fn modify_cymk(&mut self, mut new_value: (f32, f32, f32, f32)) {
        new_value.0 = clamp_with_bel!(0.0, new_value.0, 100.0);
        new_value.1 = clamp_with_bel!(0.0, new_value.1, 100.0);
        new_value.2 = clamp_with_bel!(0.0, new_value.2, 100.0);
        new_value.3 = clamp_with_bel!(0.0, new_value.3, 100.0);
        (self.r, self.g, self.b) = cymk2rgb(new_value.0, new_value.1, new_value.2, new_value.3);
    }

    pub fn get_output_clr(&self, output_type: &OutputType, enable_alpha: bool) -> String {
        return match output_type {
            OutputType::HSL => self.tohsl(enable_alpha),
            OutputType::ANSI => self.toansi(false),
            OutputType::RGB => self.torgb(enable_alpha),
            OutputType::HEX => self.tohex(enable_alpha),
            OutputType::CYMK => self.tocymk(enable_alpha),
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
            OutputType::CYMK => {
                if enable_alpha {
                    format!("cymka({})", self.tocymk(enable_alpha))
                } else {
                    format!("cymk({})", self.tocymk(false))
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
            Hex,
        }
        impl FormatType {
            fn format<T: LowerHex + Display>(&self, val: T, width: usize) -> String {
                match self {
                    Self::String => format!("{}", val),
                    Self::Hex => format!("{:0width$x}", val),
                }
            }
        }
        let (h, s, l) = self.hsl();
        let (c, y, m, k) = self.cymk();
        let ch_to_value = hashmap! {
            "R" => self.r, "G" => self.g, "B" => self.b,
            "H" => h, "S" => s, "L" => l,
            "C" => c, "Y" => y, "M" => m, "K" => k,
            "A" => self.a as f32,
            "D" => (self.r as u32 * ((256u32).pow(2)) + self.g as u32 * 256) as f32 + self.b,
            "E" => rgb2ansi256(self.r as u8, self.g as u8, self.b as u8) as f32
        };
        let mut result = String::new();
        let mut is_fmt_char = false;
        let mut fmt_char_type = FormatType::String;
        let mut width = String::from("2");
        for i in 0..fmt.len() {
            let ch = &fmt[i..i + 1];
            if ch == "%" {
                fmt_char_type = FormatType::String;
                is_fmt_char = true;
                continue;
            }
            if is_fmt_char {
                if let Some(v) = ch_to_value.get(ch) {
                    result += &fmt_char_type.format(*v as u32, width.parse().unwrap());
                } else if let Ok(v) = ch.parse::<u8>() {
                    width += &v.to_string();
                    continue;
                } else if ch == "x" {
                    fmt_char_type = FormatType::Hex;
                    continue;
                } else if ch == "n" {
                    result += "\n"
                } else if ch == "t" {
                    result += "\t"
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

    pub fn tocymk(&self, enable_alpha: bool) -> String {
        let (c, y, m, k) = self.cymk();
        if enable_alpha {
            return format!("{:.2}, {:.2}, {:.2}, {:.2}, {:.2}", c, y, m, k, self.a);
        }
        else {
            return format!("{:.2}, {:.2}, {:.2}, {:.2}", c, y, m, k)
        }
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
