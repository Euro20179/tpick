pub fn hsl2rgb(mut h: f32, s: f32, l: f32) -> (f32, f32, f32) {
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

    return (h, s, l);
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
    b = if b != 0 {b * 40 + 50 } else { 0 };
    r = if r != 0 {r * 40 + 50 } else { 0 };
    g = if g != 0 {g * 40 + 50 } else { 0 };

    return (r, g, b);
}
