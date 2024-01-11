use std::collections::HashMap;

use crate::{hashmap, read_ansi_color, color_representation::ColorRepresentation};

pub type ColorInt = u32;
///Number from 0-1
type Percentage = f32;

fn luminance(color: [f32; 3]) -> f32 {
    const RED: f32 = 0.2126;
    const GREEN: f32 = 0.7152;
    const BLUE: f32 = 0.0722;
    const GAMMA: f32 = 2.4;
    let adjusted: Vec<_> = color.iter().map(|v| {
        let small = v / 255.0;
        return if small <= 0.03928 {
            small / 12.92
        } else {
            ((small + 0.055) / 1.055).powf(GAMMA)
        }
    }).collect();
    return adjusted[0] * RED + adjusted[1] * GREEN + adjusted[2] * BLUE;
}

pub fn contrast(col1: [f32; 3], col2: [f32; 3]) -> f32 {
    let lum1 = luminance(col1);
    let lum2 = luminance(col2);
    let brightest = max!(lum1, lum2);
    let darkest = min!(lum1, lum2);
    return (brightest + 0.05) / (darkest + 0.05);
}

pub fn hsl2rgb(mut h: f32, mut s: f32, mut l: f32) -> (f32, f32, f32) {
    s /= 100.0;
    l /= 100.0;
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
    } else if h < 120.0 {
        r = f(120.0 - h);
        g = max;
        b = min;
    } else if h < 180.0 {
        r = min;
        g = max;
        b = f(h - 120.0);
    } else if h < 240.0 {
        r = min;
        g = f(240.0 - h);
        b = max;
    } else if h < 300.0 {
        r = f(h - 240.0);
        g = min;
        b = max;
    } else {
        r = max;
        g = min;
        b = f(360.0 - h);
    }
    return ((r * 255.0), (g * 255.0), (b * 255.0));
}

pub fn rgb2hsl(mut r: f32, mut g: f32, mut b: f32) -> (f32, f32, f32) {
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
        h = 0.0;
    } else if max == r {
        h = ((g - b) / delta) % 6.0;
    } else if max == g {
        h = (b - r) / delta + 2.0;
    } else {
        h = (r - g) / delta + 4.0;
    }

    h = h * 60.0;

    if h < 0.0 {
        h += 360.0;
    }

    s = if delta == 0.0 {
        0.0
    } else {
        delta / (1.0 - (2.0 * l - 1.0).abs())
    };

    return (h, s * 100.0, l * 100.0);
}

//REMOVE the # before giving to this function
pub fn hex62rgb(hex: &str) -> (u8, u8, u8) {
    let r = i64::from_str_radix(&hex[0..2], 16).unwrap();
    let g = i64::from_str_radix(&hex[2..4], 16).unwrap();
    let b = i64::from_str_radix(&hex[4..6], 16).unwrap();
    return (r as u8, g as u8, b as u8);
}

pub fn ansi2562rgb(ansi: u8, low_rgb: &Vec<String>) -> (u8, u8, u8) {
    if ansi < 16 {
        return hex62rgb(&low_rgb[ansi as usize][1..]);
    }
    if ansi > 231 {
        let s = (ansi - 232) * 10 + 8;
        return (s, s, s);
    }

    let n = ansi - 16;
    let mut b = n % 6;
    let mut g = (n - b) / 6 % 6;
    let mut r = (n - b - g * 6) / 36 % 6;
    b = if b != 0 { b * 40 + 55 } else { 0 };
    r = if r != 0 { r * 40 + 55 } else { 0 };
    g = if g != 0 { g * 40 + 55 } else { 0 };

    return (r, g, b);
}

pub fn rgb2number(r: f32, g: f32, b: f32) -> u32 {
    (r as u32 * ((256u32).pow(2)) + g as u32 * 256) as u32 + b as u32
}

pub fn number2rgb(number: ColorInt) -> (u8, u8, u8) {
    let b = number % 256;
    let g = ((number - b) / 256) % 256;
    let r = ((number - b) / 256u32.pow(2)) - g / 256;
    return (r as u8, g as u8, b as u8);
}

pub fn rgb2ansi256(r: u8, g: u8, b: u8) -> u8 {
    let low_rgb: Vec<String> = (0..16).map(|_| "#000000".to_owned()).collect();
    let mut ansi_table: HashMap<i32, u8> = HashMap::new();
    let mut numbers: Vec<[u8; 3]> = vec![];
    for clr in 16..=231 {
        let (r, g, b) = ansi2562rgb(clr, &low_rgb);
        ansi_table.insert(rgb2number(r as f32, g as f32, b as f32) as i32, clr);
        numbers.push([r, g, b]);
    }
    let components = [r, g, b];
    let mut res = vec![];
    for comp_no in 0..components.len() {
        let mut i = 0;
        while i < numbers.len() - 1 {
            let (s, b) = (numbers[i][comp_no], numbers[i + 1][comp_no]);
            if s <= components[comp_no] && components[comp_no] <= b {
                let avg = ((s as u16 + b as u16) / 2) as u8;
                let closest;
                if components[comp_no] < avg {
                    closest = s;
                } else {
                    closest = b;
                }
                res.push(closest);
                break;
            }
            i += 1;
        }
    }
    return *ansi_table
        .get(&(rgb2number(res[0] as f32, res[1] as f32, res[2] as f32) as i32))
        .unwrap();
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq, Copy)]
pub enum ColorNameStandard {
    X11,
    W3C,
    XTerm,
    MyTerm,
}

impl ColorNameStandard {
    pub fn list_colors(&self) -> HashMap<&str, [u8; 3]> {
        type TermClrConvertFn = fn(&ColorNameStandard) -> [u8; 3];
        let low_color_map: HashMap<[&str; 3], Box<fn(&ColorNameStandard) -> [u8; 3]>> = hashmap! {
            ["black", "0", "30"] => Box::new(ColorNameStandard::black as TermClrConvertFn),
            ["red" , "1" , "31"] => Box::new(ColorNameStandard::red as TermClrConvertFn),
            ["yellow" , "3" , "33"] => Box::new(ColorNameStandard::yellow as TermClrConvertFn),
            ["green" , "2" , "32"] => Box::new(ColorNameStandard::green as TermClrConvertFn),
            ["blue" , "4" , "34"] => Box::new(ColorNameStandard::blue as TermClrConvertFn),
            ["magenta" , "5" , "35"] => Box::new(ColorNameStandard::magenta as TermClrConvertFn),
            ["cyan" , "6" , "36" ] => Box::new(ColorNameStandard::cyan as TermClrConvertFn),
            ["white" , "7" , "37"] => Box::new(ColorNameStandard::white as TermClrConvertFn),
        };
        let bright_low_color_map: HashMap<[&str; 2], Box<TermClrConvertFn>> = hashmap! {
            ["bright black", "90"] => Box::new(ColorNameStandard::bright_black as TermClrConvertFn),
            ["bright red", "91"] => Box::new(ColorNameStandard::bright_red as TermClrConvertFn),
            ["bright green", "92" ] => Box::new(ColorNameStandard::bright_green as TermClrConvertFn),
            ["bright blue", "94"] => Box::new(ColorNameStandard::bright_blue as TermClrConvertFn),
            ["bright cyan", "96"] => Box::new(ColorNameStandard::bright_cyan as TermClrConvertFn),
            ["bright magenta", "95"] => Box::new(ColorNameStandard::bright_magenta as TermClrConvertFn),
            ["bright white", "97"] => Box::new(ColorNameStandard::bright_white as TermClrConvertFn),
            ["bright yellow", "93"] => Box::new(ColorNameStandard::bright_yellow as TermClrConvertFn),
        };
        let mut data: HashMap<&str, [u8; 3]> = hashmap! {
            "alice blue" => [0xf0, 0xf8, 0xff],
            "antique white" => [0xfa, 0xeb, 0xd7],
            "aqua" => [0x00, 0xff, 0xff],
            "aquamarine" => [0x7f, 0xff, 0xd4],
            "azure" => [0xf0, 0xff, 0xff],
            "beige" => [0xf5, 0xf5, 0xdc],
            "bisque" => [0xff, 0xe4, 0xc4],
            "blanched almond" => [0xff, 0xeb, 0xcd],
            "blue violet" => [0x8a, 0x2b, 0xe2],
            "brown" => [0xa5, 0x2a, 0x2a],
            "burlywood" => [0xde, 0xb8, 0x87],
            "cadet blue" => [0x5f, 0x9e, 0xa0],
            "chartreuse" => [0x7f, 0xff, 0x00],
            "chocolate" => [0xd2, 0x69, 0x1e],
            "coral" => [0xff, 0x7f, 0x50],
            "cornflower blue" => [0x64, 0x95, 0xed],
            "cornsilk" => [0xff, 0xf8, 0xdc],
            "crimson" => [0xdc, 0x14, 0x3c],
            "dark blue" => [0x00, 0x00, 0x8b],
            "dark cyan" => [0x00, 0x8b, 0x8b],
            "dark goldenrod" => [0xb8, 0x86, 0x0b],
            "dark gray" => [0xa9, 0xa9, 0xa9],
            "dark green" => [0x00, 0x64, 0x00],
            "dark khaki" => [0xbd, 0xb7, 0x6b],
            "dark magenta" => [0x8b, 0x00, 0x8b],
            "dark olive green" => [0x55, 0x6b, 0x2f],
            "dark orange" => [0xff, 0x8c, 0x00],
            "dark orchid" => [0x99, 0x32, 0xcc],
            "dark red" => [0x8b, 0x00, 0x00],
            "dark salmon" => [0xe9, 0x96, 0x7a],
            "dark sea green" => [0x8f, 0xbc, 0x8f],
            "dark slate blue" => [0x48, 0x3d, 0x8b],
            "dark slate gray" => [0x2f, 0x4f, 0x4f],
            "dark turquoise" => [0x00, 0xce, 0xd1],
            "dark violet" => [0x94, 0x00, 0xd3],
            "deep pink" => [0xff, 0x14, 0x93],
            "deep sky blue" => [0x00, 0xbf, 0xff],
            "dim gray" => [0x69, 0x69, 0x69],
            "dodger blue" => [0x1e, 0x90, 0xff],
            "firebrick" => [0xb2, 0x22, 0x22],
            "floral white" => [0xff, 0xfa, 0xf0],
            "forest green" => [0x22, 0x8b, 0x22],
            "fuchsia" => [0xff, 0x00, 0xff],
            "gainsboro*" => [0xdc, 0xdc, 0xdc],
            "ghost white" => [0xf8, 0xf8, 0xff],
            "gold" => [0xff, 0xd7, 0x00],
            "goldenrod" => [0xda, 0xa5, 0x20],
            "gray" => self.gray(),
            "web gray" => [0x80, 0x80, 0x80],
            "web green" => [0x00, 0x80, 0x00],
            "green yellow" => [0xad, 0xff, 0x2f],
            "honeydew" => [0xf0, 0xff, 0xf0],
            "hot pink" => [0xff, 0x69, 0xb4],
            "indian red" => [0xcd, 0x5c, 0x5c],
            "indigo" => [0x4b, 0x00, 0x82],
            "ivory" => [0xff, 0xff, 0xf0],
            "khaki" => [0xf0, 0xe6, 0x8c],
            "lavender" => [0xe6, 0xe6, 0xfa],
            "lavender blush" => [0xff, 0xf0, 0xf5],
            "lawn green" => [0x7c, 0xfc, 0x00],
            "lemon chiffon" => [0xff, 0xfa, 0xcd],
            "light blue" => [0xad, 0xd8, 0xe6],
            "light coral" => [0xf0, 0x80, 0x80],
            "light cyan" => [0xe0, 0xff, 0xff],
            "light goldenrod" => [0xfa, 0xfa, 0xd2],
            "light gray" => [0xd3, 0xd3, 0xd3],
            "light green" => [0x90, 0xee, 0x90],
            "light pink" => [0xff, 0xb6, 0xc1],
            "light salmon" => [0xff, 0xa0, 0x7a],
            "light sea green" => [0x20, 0xb2, 0xaa],
            "light sky blue" => [0x87, 0xce, 0xfa],
            "light slate gray" => [0x77, 0x88, 0x99],
            "light steel blue" => [0xb0, 0xc4, 0xde],
            "light yellow" => [0xff, 0xff, 0xe0],
            "lime" => [0x00, 0xff, 0x00],
            "lime green" => [0x32, 0xcd, 0x32],
            "linen" => [0xfa, 0xf0, 0xe6],
            "maroon" => self.maroon(),
            "web maroon" => [0x80, 0x00, 0x00],
            "medium aquamarine" => [0x66, 0xcd, 0xaa],
            "medium blue" => [0x00, 0x00, 0xcd],
            "medium orchid" => [0xba, 0x55, 0xd3],
            "medium purple" => [0x93, 0x70, 0xdb],
            "medium sea green" => [0x3c, 0xb3, 0x71],
            "medium slate blue" => [0x7b, 0x68, 0xee],
            "medium spring green" => [0x00, 0xfa, 0x9a],
            "medium turquoise" => [0x48, 0xd1, 0xcc],
            "medium violet red" => [0xc7, 0x15, 0x85],
            "midnight blue" => [0x19, 0x19, 0x70],
            "mint cream" => [0xf5, 0xff, 0xfa],
            "misty rose" => [0xff, 0xe4, 0xe1],
            "moccasin" => [0xff, 0xe4, 0xb5],
            "navajo white" => [0xff, 0xde, 0xad],
            "navy blue" => [0x00, 0x00, 0x80],
            "old lace" => [0xfd, 0xf5, 0xe6],
            "olive" => [0x80, 0x80, 0x00],
            "olive drab" => [0x6b, 0x8e, 0x23],
            "orange" => [0xff, 0xa5, 0x00],
            "orange red" => [0xff, 0x45, 0x00],
            "orchid" => [0xda, 0x70, 0xd6],
            "pale goldenrod" => [0xee, 0xe8, 0xaa],
            "pale green" => [0x98, 0xfb, 0x98],
            "pale turquoise" => [0xaf, 0xee, 0xee],
            "pale violet red" => [0xdb, 0x70, 0x93],
            "papaya whip" => [0xff, 0xef, 0xd5],
            "peach puff" => [0xff, 0xda, 0xb9],
            "peru" => [0xcd, 0x85, 0x3f],
            "pink" => [0xff, 0xc0, 0xcb],
            "plum" => [0xdd, 0xa0, 0xdd],
            "powder blue" => [0xb0, 0xe0, 0xe6],
            "purple" => self.purple(),
            "web purple" => [0x80, 0x00, 0x80],
            "rebecca purple" => [0x66, 0x33, 0x99],
            "rosy brown" => [0xbc, 0x8f, 0x8f],
            "royal blue" => [0x41, 0x69, 0xe1],
            "saddle brown" => [0x8b, 0x45, 0x13],
            "salmon" => [0xfa, 0x80, 0x72],
            "sandy brown" => [0xf4, 0xa4, 0x60],
            "sea green" => [0x2e, 0x8b, 0x57],
            "seashell" => [0xff, 0xf5, 0xee],
            "sienna" => [0xa0, 0x52, 0x2d],
            "silver" => [0xc0, 0xc0, 0xc0],
            "sky blue" => [0x87, 0xce, 0xeb],
            "slate blue" => [0x6a, 0x5a, 0xcd],
            "slate gray" => [0x70, 0x80, 0x90],
            "snow" => [0xff, 0xfa, 0xfa],
            "spring green" => [0x00, 0xff, 0x7f],
            "steel blue" => [0x46, 0x82, 0xb4],
            "tan" => [0xd2, 0xb4, 0x8c],
            "teal" => [0x00, 0x80, 0x80],
            "thistle" => [0xd8, 0xbf, 0xd8],
            "tomato" => [0xff, 0x63, 0x47],
            "turquoise" => [0x40, 0xe0, 0xd0],
            "violet" => [0xee, 0x82, 0xee],
            "wheat" => [0xf5, 0xde, 0xb3],
            "white smoke" => [0xf5, 0xf5, 0xf5],
            "yellow green" => [0x9a, 0xcd, 0x32],
        };
        for clr_list in low_color_map.keys() {
            for clr in clr_list {
                data.insert(clr, (low_color_map[clr_list])(self));
            }
        }
        for clr_list in bright_low_color_map.keys() {
            for clr in clr_list {
                data.insert(clr, (bright_low_color_map[clr_list])(self));
            }
        }
        data
    }
    fn get_color(&self, clr: &str) -> String {
        let clrs = self.list_colors();
        let clr = clrs.get(&clr).unwrap_or(&[0u8, 0u8, 0u8]);
        return format!("#{:02x}{:02x}{:02x}", clr[0], clr[1], clr[2]);
    }
}

//TODO:
//make it so that each color name standard only implements its own colors, this is more expandable

fn get_ansi_clr_num_with_reader(num: u8) -> [u8; 3] {
    let mut reader = std::io::stdin();
    read_ansi_color(&mut reader, num)
}

impl ColorNameStandard {
    fn gray(&self) -> [u8; 3] {
        match self {
            Self::X11 => [0xbe, 0xbe, 0xbe],
            _ => [0x80, 0x80, 0x80],
        }
    }
    fn black(&self) -> [u8; 3] {
        match self {
            Self::MyTerm => get_ansi_clr_num_with_reader(0),
            _ => [0, 0, 0],
        }
    }
    fn bright_black(&self) -> [u8; 3] {
        match self {
            Self::MyTerm => get_ansi_clr_num_with_reader(8),
            Self::XTerm => [0x4d, 0x4d, 0x4d],
            _ => [0xff, 0, 0],
        }
    }
    fn red(&self) -> [u8; 3] {
        match self {
            Self::XTerm => [0xcd, 0, 0],
            Self::MyTerm => get_ansi_clr_num_with_reader(1),
            _ => [0xff, 0, 0],
        }
    }
    fn bright_red(&self) -> [u8; 3] {
        match self {
            Self::MyTerm => get_ansi_clr_num_with_reader(9),
            _ => [0xff, 0, 0],
        }
    }
    fn green(&self) -> [u8; 3] {
        match self {
            Self::X11 => [0x00, 0xff, 0x00],
            Self::XTerm => [0, 0xcd, 0],
            Self::MyTerm => get_ansi_clr_num_with_reader(2),
            _ => [0, 0x80, 0],
        }
    }
    fn bright_green(&self) -> [u8; 3] {
        match self {
            Self::MyTerm => get_ansi_clr_num_with_reader(0xA),
            _ => [0x00, 0xff, 0x00],
        }
    }
    fn yellow(&self) -> [u8; 3] {
        match self {
            Self::XTerm => [0xcd, 0xcd, 0x00],
            Self::MyTerm => get_ansi_clr_num_with_reader(3),
            _ => [0xff, 0xff, 0x00],
        }
    }
    fn blue(&self) -> [u8; 3] {
        match self {
            Self::XTerm => [0x00, 0x00, 0xcd],
            Self::MyTerm => get_ansi_clr_num_with_reader(4),
            _ => [0x00, 0x00, 0xff],
        }
    }
    fn bright_blue(&self) -> [u8; 3] {
        match self {
            Self::MyTerm => get_ansi_clr_num_with_reader(0xC),
            _ => [0x00, 0x00, 0xff],
        }
    }
    fn magenta(&self) -> [u8; 3] {
        match self {
            Self::XTerm => [0xcd, 0x00, 0xcd],
            Self::MyTerm => get_ansi_clr_num_with_reader(5),
            _ => [0xff, 0x00, 0xff],
        }
    }
    fn bright_magenta(&self) -> [u8; 3] {
        match self {
            Self::MyTerm => get_ansi_clr_num_with_reader(0xD),
            _ => [0xff, 0x00, 0xff],
        }
    }
    fn cyan(&self) -> [u8; 3] {
        match self {
            Self::XTerm => [0x00, 0xcd, 0xcd],
            Self::MyTerm => get_ansi_clr_num_with_reader(6),
            _ => [0x00, 0xff, 0xff],
        }
    }
    fn bright_cyan(&self) -> [u8; 3] {
        match self {
            Self::MyTerm => get_ansi_clr_num_with_reader(0xE),
            _ => [0x00, 0xff, 0xff],
        }
    }
    fn white(&self) -> [u8; 3] {
        match self {
            Self::XTerm => [0xe5, 0xe5, 0xe5],
            Self::MyTerm => get_ansi_clr_num_with_reader(7),
            _ => [0xff, 0xff, 0xff],
        }
    }
    fn bright_white(&self) -> [u8; 3] {
        match self {
            Self::MyTerm => get_ansi_clr_num_with_reader(0xf),
            _ => [0xff, 0xff, 0xff],
        }
    }
    fn bright_yellow(&self) -> [u8; 3] {
        match self {
            Self::MyTerm => get_ansi_clr_num_with_reader(0xB),
            _ => [0xff, 0xff, 0x00],
        }
    }
    fn maroon(&self) -> [u8; 3] {
        match self {
            Self::X11 => [0xb0, 0x30, 0x60],
            _ => [0x80, 0x00, 0x00],
        }
    }
    fn purple(&self) -> [u8; 3] {
        match self {
            Self::X11 => [0xa0, 0x20, 0xf0],
            _ => [0x80, 0x00, 0x80],
        }
    }
}

pub enum MixSpace {
    RGB,
    HSL,
}

fn color_mix_rgb(clr1: ColorInt, clr2: ColorInt, percent: Percentage) -> ColorInt {
    let (r1, g1, b1) = number2rgb(clr1);
    let (r2, g2, b2) = number2rgb(clr2);
    let clr1_p = 1.0 - percent;
    return rgb2number(
        r1 as f32 * clr1_p + r2 as f32 * percent,
        g1 as f32 * clr1_p + g2 as f32 * percent,
        b1 as f32 * clr1_p + b2 as f32 * percent,
    );
}

fn color_mix_hsl(clr1: ColorInt, clr2: ColorInt, percent: Percentage) -> ColorInt {
    let (r1, g1, b1) = number2rgb(clr1);
    let (h1, s1, l1) = rgb2hsl(r1 as f32, g1 as f32, b1 as f32);
    let (r2, g2, b2) = number2rgb(clr2);
    let (h2, s2, l2) = rgb2hsl(r2 as f32, g2 as f32, b2 as f32);
    let clr1_p = 1.0 - percent;
    let (r, g, b) = hsl2rgb(h1 * clr1_p + h2 * percent, s1 * clr1_p + s2 * percent, l1 * clr1_p + l2 * percent);
    return rgb2number(r, g, b);
}

pub fn color_mix(
    clr1: ColorInt,
    clr2: ColorInt,
    percent: Percentage,
    mix_space: &MixSpace,
) -> ColorInt {
    match mix_space {
        MixSpace::RGB => color_mix_rgb(clr1, clr2, percent),
        MixSpace::HSL => color_mix_hsl(clr1, clr2, percent)
    }
}

pub fn invert(clr: ColorInt) -> ColorInt {
    let (r, g, b) = number2rgb(clr);
    return rgb2number(255.0 - r as f32, 255.0 - g as f32, 255.0 - b as f32);
}

pub fn name_to_hex<'a>(name: &str, color_name_standard: &'a ColorNameStandard) -> String {
    return color_name_standard.get_color(name);
}
