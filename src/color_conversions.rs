use crate::read_ansi_color;

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
    b = if b != 0 { b * 40 + 50 } else { 0 };
    r = if r != 0 { r * 40 + 50 } else { 0 };
    g = if g != 0 { g * 40 + 50 } else { 0 };

    return (r, g, b);
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq)]
pub enum ColorNameStandard {
    X11,
    W3C,
    XTerm,
    MyTerm,
}

impl ColorNameStandard {
    fn get_color(&self, clr: &str) -> String{
        match clr {
            "alice blue" => "#f0f8ff",
            "antique white" => "#faebd7",
            "aqua" => "#00ffff",
            "aquamarine" => "#7fffd4",
            "azure" => "#f0ffff",
            "beige" => "#f5f5dc",
            "bisque" => "#ffe4c4",
            "black" | "0" | "30" => {
                let black = self.black();
                return format!("#{:02x}{:02x}{:02x}", black[0], black[1], black[2]);
            },
            "bright black" | "90" => "#4d4d4d", //xterm default for 90
            "blanched almond" => "#ffebcd",
            "blue" | "4" | "34" => {
                let blue = self.blue();
                return format!("#{:02x}{:02x}{:02x}", blue[0], blue[1], blue[2]);
            },
            "bright blue" | "94" => "#0000ff", //xterm default for 94
            "blue violet" => "#8a2be2",
            "brown" => "#a52a2a",
            "burlywood" => "#deb887",
            "cadet blue" => "#5f9ea0",
            "chartreuse" => "#7fff00",
            "chocolate" => "#d2691e",
            "coral" => "#ff7f50",
            "cornflower blue" => "#6495ed",
            "cornsilk" => "#fff8dc",
            "crimson" => "#dc143c",
            "cyan" | "6" | "36" => {
                let cyan = self.cyan();
                return format!("#{:02x}{:02x}{:02x}", cyan[0], cyan[1], cyan[2]);
            },
            "bright cyan" | "96" => "#00ffff", //xterm default for 96
            "dark blue" => "#00008b",
            "dark cyan" => "#008b8b",
            "dark goldenrod" => "#b8860b",
            "dark gray" => "#a9a9a9",
            "dark green" => "#006400",
            "dark khaki" => "#bdb76b",
            "dark magenta" => "#8b008b",
            "dark olive green" => "#556b2f",
            "dark orange" => "#ff8c00",
            "dark orchid" => "#9932cc",
            "dark red" => "#8b0000",
            "dark salmon" => "#e9967a",
            "dark sea green" => "#8fbc8f",
            "dark slate blue" => "#483d8b",
            "dark slate gray" => "#2f4f4f",
            "dark turquoise" => "#00ced1",
            "dark violet" => "#9400d3",
            "deep pink" => "#ff1493",
            "deep sky blue" => "#00bfff",
            "dim gray" => "#696969",
            "dodger blue" => "#1e90ff",
            "firebrick" => "#b22222",
            "floral white" => "#fffaf0",
            "forest green" => "#228b22",
            "fuchsia" => "#ff00ff",
            "gainsboro*" => "#dcdcdc",
            "ghost white" => "#f8f8ff",
            "gold" => "#ffd700",
            "goldenrod" => "#daa520",
            "gray" => self.gray(),
            "web gray" => "#808080",
            "green" | "2" | "32" => {
                let green = self.green();
                return format!("#{:02x}{:02x}{:02x}", green[0], green[1], green[2]);
            },
            "bright green" | "92" => "#00ff00", //xterm default for 92
            "web green" => "#008000",
            "green yellow" => "#adff2f",
            "honeydew" => "#f0fff0",
            "hot pink" => "#ff69b4",
            "indian red" => "#cd5c5c",
            "indigo" => "#4b0082",
            "ivory" => "#fffff0",
            "khaki" => "#f0e68c",
            "lavender" => "#e6e6fa",
            "lavender blush" => "#fff0f5",
            "lawn green" => "#7cfc00",
            "lemon chiffon" => "#fffacd",
            "light blue" => "#add8e6",
            "light coral" => "#f08080",
            "light cyan" => "#e0ffff",
            "light goldenrod" => "#fafad2",
            "light gray" => "#d3d3d3",
            "light green" => "#90ee90",
            "light pink" => "#ffb6c1",
            "light salmon" => "#ffa07a",
            "light sea green" => "#20b2aa",
            "light sky blue" => "#87cefa",
            "light slate gray" => "#778899",
            "light steel blue" => "#b0c4de",
            "light yellow" => "#ffffe0",
            "lime" => "#00ff00",
            "lime green" => "#32cd32",
            "linen" => "#faf0e6",
            "magenta" | "5" | "35" => {
                let magenta = self.magenta();
                return format!("#{:02x}{:02x}{:02x}", magenta[0], magenta[1], magenta[2]);
            },
            "bright magenta" | "95" => "#ff00ff", //xterm default for 95
            "maroon" => self.maroon(),
            "web maroon" => "#800000",
            "medium aquamarine" => "#66cdaa",
            "medium blue" => "#0000cd",
            "medium orchid" => "#ba55d3",
            "medium purple" => "#9370db",
            "medium sea green" => "#3cb371",
            "medium slate blue" => "#7b68ee",
            "medium spring green" => "#00fa9a",
            "medium turquoise" => "#48d1cc",
            "medium violet red" => "#c71585",
            "midnight blue" => "#191970",
            "mint cream" => "#f5fffa",
            "misty rose" => "#ffe4e1",
            "moccasin" => "#ffe4b5",
            "navajo white" => "#ffdead",
            "navy blue" => "#000080",
            "old lace" => "#fdf5e6",
            "olive" => "#808000",
            "olive drab" => "#6b8e23",
            "orange" => "#ffa500",
            "orange red" => "#ff4500",
            "orchid" => "#da70d6",
            "pale goldenrod" => "#eee8aa",
            "pale green" => "#98fb98",
            "pale turquoise" => "#afeeee",
            "pale violet red" => "#db7093",
            "papaya whip" => "#ffefd5",
            "peach puff" => "#ffdab9",
            "peru" => "#cd853f",
            "pink" => "#ffc0cb",
            "plum" => "#dda0dd",
            "powder blue" => "#b0e0e6",
            "purple" => self.purple(),
            "web purple" => "#800080",
            "rebecca purple" => "#663399",
            "red" | "1" | "31" => {
                let red = self.red();
                return format!("#{:02x}{:02x}{:02x}", red[0], red[1], red[2]);
            },
            "bright red" | "91" => "#ff0000", //xterm default for 91
            "rosy brown" => "#bc8f8f",
            "royal blue" => "#4169e1",
            "saddle brown" => "#8b4513",
            "salmon" => "#fa8072",
            "sandy brown" => "#f4a460",
            "sea green" => "#2e8b57",
            "seashell" => "#fff5ee",
            "sienna" => "#a0522d",
            "silver" => "#c0c0c0",
            "sky blue" => "#87ceeb",
            "slate blue" => "#6a5acd",
            "slate gray" => "#708090",
            "snow" => "#fffafa",
            "spring green" => "#00ff7f",
            "steel blue" => "#4682b4",
            "tan" => "#d2b48c",
            "teal" => "#008080",
            "thistle" => "#d8bfd8",
            "tomato" => "#ff6347",
            "turquoise" => "#40e0d0",
            "violet" => "#ee82ee",
            "wheat" => "#f5deb3",
            "white" | "7" | "37" => {
                let white = self.white();
                return format!("#{:02x}{:02x}{:02x}", white[0], white[1], white[2]);
            },
            "bright white" | "97" => "#ffffff", //xterm default for 97
            "white smoke" => "#f5f5f5",
            "yellow" | "3" | "33" => {
                let yellow = self.yellow();
                return format!("#{:02x}{:02x}{:02x}", yellow[0], yellow[1], yellow[2]);
            },
            "bright yellow" | "93" => "#ffff00",
            "yellow green" => "#9acd32",
            _ => "#000000",
        }.to_owned()
    }
}

//TODO:
//make it so that each color name standard only implements its own colors, this is more expandable

impl ColorNameStandard {
    fn gray(&self) -> &str {
        match self {
            Self::X11 => "#bebebe",
            _ => "#808080",
        }
    }
    fn black(&self) -> [u8; 3]{
        match self {
            Self::MyTerm => {
                let mut reader = std::io::stdin();
                read_ansi_color(&mut reader, 0)
            },
            _ => [0, 0, 0]
        }
    }
    fn red(&self) -> [u8; 3]{
        match self {
            Self::XTerm => [0xcd, 0, 0],
            Self::MyTerm => {
                let mut reader = std::io::stdin();
                read_ansi_color(&mut reader, 1)
            }
            _ => [0xff, 0, 0]
        }
    }
    fn green(&self) -> [u8; 3] {
        match self {
            Self::X11 => [0x00, 0xff, 0x00],
            Self::XTerm => [0, 0xcd, 0],
            Self::MyTerm => {
                let mut reader = std::io::stdin();
                read_ansi_color(&mut reader, 2)
            }
            _ => [0, 0x80, 0],
        }
    }
    fn yellow(&self) -> [u8; 3] {
        match self {
            Self::XTerm => [0xcd, 0xcd, 0x00],
            Self::MyTerm => {
                let mut reader = std::io::stdin();
                read_ansi_color(&mut reader, 3)
            }
            _ => [0xff, 0xff, 0x00]
        }
    }
    fn blue(&self) -> [u8; 3]{
        match self {
            Self::XTerm => [0x00, 0x00, 0xcd],
            Self::MyTerm => {
                let mut reader = std::io::stdin();
                read_ansi_color(&mut reader, 4)
            }
            _ => [0x00, 0x00, 0xff]
        }
    }
    fn magenta(&self) -> [u8; 3]{
        match self {
            Self::XTerm => [0xcd, 0x00, 0xcd],
            Self::MyTerm => {
                let mut reader = std::io::stdin();
                read_ansi_color(&mut reader, 5)
            }
            _ => [0xff, 0x00, 0xff]
        }
    }
    fn cyan(&self) -> [u8; 3]{
        match self {
            Self::XTerm => [0x00, 0xcd, 0xcd],
            Self::MyTerm => {
                let mut reader = std::io::stdin();
                read_ansi_color(&mut reader, 6)
            }
            _ => [0x00, 0xff, 0xff]
        }
    }
    fn white(&self) -> [u8; 3]{
        match self {
            Self::XTerm => [0xe5, 0xe5, 0xe5],
            Self::MyTerm => {
                let mut reader = std::io::stdin();
                read_ansi_color(&mut reader, 7)
            }
            _ => [0xff, 0xff, 0xff]
        }
    }
    fn maroon(&self) -> &str {
        match self {
            Self::X11 => "#b03060",
            _ => "#800000",
        }
    }
    fn purple(&self) -> &str {
        match self {
            Self::X11 => "#a020f0",
            _ => "#800080",
        }
    }
}

pub fn name_to_hex<'a>(name: &str, color_name_standard: &'a ColorNameStandard) -> String{
    return color_name_standard.get_color(name);
}
