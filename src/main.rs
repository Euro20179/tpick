#[macro_use]
mod math;
mod cli;
mod color_conversions;
mod color_representation;
mod keymaps;
mod ui;

use clap::Parser;
use cli::*;
use color_representation::*;
use keymaps::Action;
use termios::Termios;

use std::collections::HashMap;
use std::fmt::Display;
use std::io::Read;
use std::os::fd::AsRawFd;
use std::process::exit;

use base64::engine::general_purpose;
use base64::prelude::*;

use color_conversions::*;

fn cls() {
    eprint!("\x1b[2J\x1b[0H");
}

fn render_ansi256(selected_item: u8, _square_count: u32) {
    eprint!(" ");
    for low_nr in 0..16 {
        eprint!("\x1b[38;5;{}m{:<3} ", low_nr, low_nr);
    }
    eprintln!();
    for x in 0..6 {
        eprint!(" ");
        for y in 0..6 {
            for z in 0..6 {
                let clr = (x + 16) + (6 * y) + (36 * z);
                eprint!("\x1b[38;5;{}m{:<3} ", clr, clr);
            }
        }
        eprintln!();
    }
    eprint!(" ");
    for grey_nr in 232..256 {
        eprint!("\x1b[38;5;{}m{:<3} ", grey_nr, grey_nr);
    }
    eprintln!();
    eprintln!("\x1b[0m");
    eprintln!("\x1b[2K{}", selected_item);
}

fn ansi256_renderer(
    _curr_color: &ColorRepresentation,
    selected_item: u8,
    square_count: u32,
    _step: f32,
) {
    eprint!("\x1b[0H");
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
    eprint!("{}", label);
    //create the starting color based on the list of colors
    let mut color = ColorRepresentation::from_color(
        &format!("rgb({},{},{})", colors[0], colors[1], colors[2]),
        &ColorNameStandard::W3C,
    );
    for i in 0..square_count {
        //print a square with the correct color
        eprint!("\x1b[38;2;{}m█", color.toansi(false));
        //modifies this slider's color to be i% of 255
        colors[modifier_idx] = (i as f32 / square_count as f32) * 255.0;
        color.modify_rgb((colors[0], colors[1], colors[2]));
    }
    eprintln!("\x1b[0m");
    render_carrot_on_current_line(
        ([curr_color.r, curr_color.g, curr_color.b][modifier_idx] / 255.0 * 360.0 / step).floor()
            as usize
            + 1,
    );
}

fn rgb_renderer(curr_color: &ColorRepresentation, selected_item: u8, square_count: u32, step: f32) {
    for i in 0..=2 {
        eprint!("\x1b[{};0H", i * 2 + 1);
        if selected_item == i {
            eprint!("\x1b[32m");
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
    let modifier_multiplier = [360.0, 100.0, 100.0][hsl_idx];
    eprint!("{}", label);
    let mut color = ColorRepresentation::from_color(
        &format!("hsl({},{},{})", colors[0], colors[1], colors[2]),
        &ColorNameStandard::W3C,
    );
    for i in 0..square_count {
        eprint!("\x1b[38;2;{}m█", color.toansi(false));
        colors[modifier_idx] = (i as f32 / square_count as f32) * modifier_multiplier;
        color.modify_hsl((colors[0], colors[1], colors[2]));
    }
    eprintln!("\x1b[0m");
    render_carrot_on_current_line(
        ([h, s, l][modifier_idx] / modifier_multiplier * 360.0 / step).floor() as usize + 1,
    );
}

fn hsl_renderer(curr_color: &ColorRepresentation, selected_item: u8, square_count: u32, step: f32) {
    for i in 0..=2 {
        eprint!("\x1b[{};0H", i * 2 + 1);
        if selected_item == i {
            eprint!("\x1b[32m");
        }
        render_hsl(curr_color, square_count, step, i as usize);
    }
}

fn render_cymk(curr_color: &ColorRepresentation, square_count: u32, step: f32, cymk_idx: usize) {
    //works similarly to render_rgb
    let (c, y, m, k) = curr_color.cymk();
    let mut colors = [c, y, m, k];
    let modifier_idx = cymk_idx;
    colors[modifier_idx] = 0.0;
    let label = ['C', 'Y', 'M', 'K'][cymk_idx];
    let modifier_multiplier = [100.0, 100.0, 100.0, 100.0][cymk_idx];
    eprint!("{}", label);
    let mut color = ColorRepresentation::from_color(
        &format!(
            "cymk({},{},{}, {})",
            colors[0], colors[1], colors[2], colors[3]
        ),
        &ColorNameStandard::W3C,
    );
    for i in 0..square_count {
        eprint!("\x1b[38;2;{}m█", color.toansi(false));
        colors[modifier_idx] = (i as f32 / square_count as f32) * modifier_multiplier;
        color.modify_cymk((colors[0], colors[1], colors[2], colors[3]));
    }
    eprintln!("\x1b[0m");
    render_carrot_on_current_line(
        ([c, y, m, k][modifier_idx] / modifier_multiplier * 360.0 / step).floor() as usize + 1,
    );
}

fn cymk_renderer(
    curr_color: &ColorRepresentation,
    selected_item: u8,
    square_count: u32,
    step: f32,
) {
    for i in 0..=3 {
        eprint!("\x1b[{};0H", i * 2 + 1);
        if selected_item == i {
            eprint!("\x1b[32m");
        }
        render_cymk(curr_color, square_count, step, i as usize);
    }
}

fn render_a(square_count: u32) {
    eprint!("A");
    let mut sat_color_rep = ColorRepresentation::from_color("#000000", &ColorNameStandard::W3C);
    for i in 0..square_count {
        eprint!("\x1b[38;2;{}m█", sat_color_rep.toansi(false));
        sat_color_rep.modify_hsl((0.0, 0.0, (i as f32 / square_count as f32)))
    }
    eprintln!("\x1b[0m");
}

fn render_carrot_on_current_line(col: usize) {
    eprintln!("\x1b[2K\x1b[{}C^", col);
}

fn render_sliders(
    curr_color: &ColorRepresentation,
    alpha: u8,
    renderer: fn(&ColorRepresentation, u8, u32, f32),
    square_count: u32,
    step: f32,
    selected_item: u8,
    selected_type: &SelectionType,
    enable_alpha: bool,
) {
    renderer(curr_color, selected_item, square_count, step);

    if enable_alpha {
        if selected_item as usize == selected_type.increments().len() - 1 {
            eprint!("\x1b[32m");
        }
        render_alpha_display(alpha, square_count, step);
    }
}

fn render_alpha_display(alpha: u8, square_count: u32, step: f32) {
    render_a(square_count);
    eprintln!(
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
            SelectionType::CYMK => cymk_renderer,
        },
        square_count,
        step,
        program_state.selected_item,
        &program_state.selection_type,
        program_state.enable_alpha,
    );
    eprint!(
        "\x1b[38;2;{}m████████\n████████\n████████\x1b[0m",
        program_state.curr_color.toansi(false)
    );
    eprint!(
        "\x1b[K {}",
        program_state
            .output_type
            .render_output(&program_state.curr_color, program_state.enable_alpha)
    );
}

#[derive(serde::Deserialize, Clone)]
struct ConfigOutput {
    order: Vec<String>,
}

#[derive(serde::Deserialize, Clone)]
struct Config {
    keybinds: Option<HashMap<String, String>>,
    outputs: Option<Vec<HashMap<String, ConfigOutput>>>,
}

struct ProgramState {
    selection_type: SelectionType,
    selected_item: u8,
    enable_alpha: bool,
    output_type: OutputType,
    curr_color: ColorRepresentation,
    clr_std: ColorNameStandard,
    output_idx: usize,
    output_order: Vec<OutputType>,
    config: Config,
}

impl ProgramState {
    fn new(
        selection_type: SelectionType,
        output_type: OutputType,
        starting_clr: &str,
        clr_std: ColorNameStandard,
        output_order: Vec<OutputType>,
        cfg: Config,
    ) -> ProgramState {
        ProgramState {
            selected_item: 0,
            selection_type,
            output_type,
            enable_alpha: false,
            clr_std,
            curr_color: ColorRepresentation::from_color(starting_clr, &clr_std),
            output_idx: 0,
            output_order,
            config: cfg,
        }
    }

    fn next_output(&mut self) {
        self.output_idx = (self.output_idx + 1) % self.output_order.len();
        match self.output_type {
            OutputType::ALL => self.output_idx = 0,
            _ => {}
        }
        let v = &self.output_order[self.output_idx];
        self.output_type = v.clone();
    }
}

#[derive(Copy, Clone, PartialEq, Debug, clap::ValueEnum)]
enum SelectionType {
    HSL,
    RGB,
    ANSI256,
    CYMK,
}

impl SelectionType {
    fn label_from_selected_item(&self, selected_item: u8) -> char {
        match self {
            SelectionType::HSL => ['H', 'S', 'L', 'A'][selected_item as usize],
            SelectionType::ANSI256 => 'e',
            Self::RGB => ['R', 'G', 'B', 'A'][selected_item as usize],
            Self::CYMK => ['C', 'Y', 'M', 'K', 'A'][selected_item as usize],
        }
    }

    fn max_values(&self) -> Vec<f32> {
        match self {
            SelectionType::HSL => vec![359.0, 100.0, 100.0, 255.0],
            SelectionType::RGB => vec![255.0, 255.0, 255.0, 255.0],
            SelectionType::ANSI256 => vec![255.0],
            SelectionType::CYMK => vec![100.0, 100.0, 100.0, 100.0, 255.0],
        }
    }

    fn increments(&self) -> Vec<f32> {
        match self {
            SelectionType::CYMK => vec![1.0, 1.0, 1.0, 1.0, 1.0],
            Self::HSL | Self::RGB => vec![1.0, 1.0, 1.0, 1.0],
            Self::ANSI256 => vec![1.0],
        }
    }

    fn colors(&self, program_state: &ProgramState) -> Vec<f32> {
        match self {
            Self::RGB => {
                let (r, g, b) = program_state.curr_color.rgb();
                vec![r, g, b, program_state.curr_color.a as f32]
            }
            Self::ANSI256 => vec![program_state.selected_item as f32],
            Self::HSL => {
                let (h, s, l) = program_state.curr_color.hsl();
                vec![h, s, l, program_state.curr_color.a as f32]
            }
            Self::CYMK => {
                let (c, y, m, k) = program_state.curr_color.cymk();
                vec![c, y, m, k, program_state.curr_color.a as f32]
            }
        }
    }

    fn modify_color_based_on_selected_item(
        &self,
        program_state: &mut ProgramState,
        new_value: f32,
    ) {
        let selected_item = program_state.selected_item;
        match self {
            SelectionType::HSL => {
                let (h, s, l) = program_state.curr_color.hsl();
                let mut modifiables = [h, s, l, program_state.curr_color.a as f32];
                modifiables[selected_item as usize] = new_value;
                program_state.curr_color.add_hsla([
                    modifiables[0] - h,
                    modifiables[1] - s,
                    modifiables[2] - l,
                    modifiables[3] - program_state.curr_color.a as f32,
                ]);
            }
            SelectionType::RGB => {
                let (r, g, b) = (
                    program_state.curr_color.r,
                    program_state.curr_color.g,
                    program_state.curr_color.b,
                );
                let mut modifiables = [r, g, b, program_state.curr_color.a as f32];
                modifiables[selected_item as usize] = new_value;
                program_state.curr_color.add_rgba([
                    modifiables[0] - r,
                    modifiables[1] - g,
                    modifiables[2] - b,
                    modifiables[3] - program_state.curr_color.a as f32,
                ]);
            }
            SelectionType::CYMK => {
                let (c, y, m, k) = program_state.curr_color.cymk();
                let mut modifiables = [c, y, m, k, program_state.curr_color.a as f32];
                modifiables[selected_item as usize] = new_value;
                program_state.curr_color.add_cymka([
                    modifiables[0] - c,
                    modifiables[1] - y,
                    modifiables[2] - m,
                    modifiables[3] - k,
                    modifiables[4] - program_state.curr_color.a as f32,
                ]);
            }
            Self::ANSI256 => {
                let mut reader = std::io::stdin();
                let low_rgb = get_ansi_30_and_90(&mut reader);
                let (r, g, b) = ansi2562rgb(new_value as u8, &low_rgb);
                program_state
                    .curr_color
                    .modify_rgb((r as f32, g as f32, b as f32));
                program_state.selected_item = new_value as u8;
            }
        }
    }
}

#[derive(Clone, Debug)]
enum OutputType {
    HSL,
    RGB,
    HEX,
    ANSI,
    CYMK,
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
                CYMK => "CYMK",
                CUSTOM(n) => n,
                ALL => "ALL",
            }
        )
    }
}

impl OutputType {
    fn render_output(&self, curr_color: &ColorRepresentation, enable_alpha: bool) -> String {
        format!(
            "{}",
            curr_color.get_formatted_output_clr(self, enable_alpha)
        )
    }

    fn from_str(data: &str) -> Self {
        match data.to_lowercase().as_str() {
            "hsl" => Self::HSL,
            "rgb" => Self::RGB,
            "hex" => Self::HEX,
            "cymk" => Self::CYMK,
            "ansi" => Self::ANSI,
            "all" => Self::ALL,
            _ => Self::CUSTOM(data.to_string()),
        }
    }

    fn get_order_by_name(config: &Config, name: &str) -> Option<Vec<Self>> {
        if name == "default" {
            return Some(Self::default_order(config));
        }
        if let Some(output_cfg) = &config.outputs {
            let outputs = &output_cfg[0];
            if let Some(default) = outputs.get(name) {
                return Some(
                    default
                        .order
                        .iter()
                        .map(|item| OutputType::from_str(item))
                        .collect(),
                );
            }
        }
        None
    }

    fn default_order(config: &Config) -> Vec<Self> {
        if let Some(output_cfg) = &config.outputs {
            let outputs = &output_cfg[0];
            if let Some(default) = outputs.get("default") {
                return default
                    .order
                    .iter()
                    .map(|item| OutputType::from_str(item))
                    .collect();
            }
        }
        vec![
            OutputType::HSL,
            OutputType::RGB,
            OutputType::HEX,
            OutputType::CYMK,
            OutputType::ANSI,
        ]
    }
}

fn read_osc_response(reader: &mut std::io::Stdin, end_byte: u8) -> String {
    let mut result_str = String::new();
    let mut b = [0; 1];
    loop {
        reader.read_exact(&mut b).unwrap();
        if b[0] == end_byte {
            break;
        }
        if b[0] == b'\\' && result_str.ends_with("\x1b") {
            result_str = result_str.strip_suffix("\x1b").unwrap().to_string();
            break;
        }
        result_str += &String::from(b[0] as char);
    }
    return result_str;
}

fn query_window_area(reader: &mut std::io::Stdin) -> [i32; 2] {
    eprint!("\x1b[18t");
    let area = read_osc_response(reader, b't');
    let mut split = area.split(";");
    let rows = split.nth(1).unwrap().parse().unwrap_or(80);
    let cols = split.next().unwrap().parse().unwrap_or(24);
    return [rows, cols];
}

fn read_ansi_color(reader: &mut std::io::Stdin, clr_num: u8) -> [u8; 3] {
    eprint!("\x1b]4;{};?\x07", clr_num);
    let clr_buf = read_osc_response(reader, 7);
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
    [
        u8::from_str_radix(r, 16).unwrap_or(0),
        u8::from_str_radix(g, 16).unwrap_or(0),
        u8::from_str_radix(b, 16).unwrap_or(0),
    ]
}

fn get_ansi_30_and_90(reader: &mut std::io::Stdin) -> Vec<String> {
    let mut data = Vec::with_capacity(16);
    for i in 0..16 {
        let clr = read_ansi_color(reader, i);
        data.push(format!("#{}{}{}", clr[0], clr[1], clr[2]));
    }
    return data;
}

///clr can be 10 or 11
fn query_color(clr: u8, reader: &mut std::io::Stdin) -> String {
    eprint!("\x1b]{};?\x07", clr);
    let clr_buf = read_osc_response(reader, 7);
    //parses out garbage, gives us rr/gg/bb
    let data = &clr_buf
        .as_str()
        .split(";")
        .nth(1)
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

fn paste_to_clipboard(data: &str) {
    let b64 = general_purpose::STANDARD.encode(data);
    eprint!("\x1b]52;c;{}\x07", b64);
}

fn read_clipboard(reader: &mut std::io::Stdin) -> String {
    eprintln!("\x1b]52;c;?\x07");

    let clip_buf = read_osc_response(reader, 7);

    let clip_data = clip_buf.split(";").nth(2).unwrap();

    return String::from_utf8(general_purpose::STANDARD.decode(clip_data).unwrap()).unwrap();
}

//returns oldtermios, newtermios
fn setup_term() -> (termios::Termios, termios::Termios) {
    let tty = std::fs::File::open("/dev/tty").unwrap();
    let tty_fd = tty.as_raw_fd();
    unsafe { libc::dup2(tty_fd, 0) };
    let mut tios = Termios::from_fd(0).unwrap();
    let mut tios_initial = Termios::from_fd(0).unwrap();
    let _ = termios::tcgetattr(0, &mut tios);
    let _ = termios::tcgetattr(0, &mut tios_initial);

    tios.c_lflag &= !(termios::ICANON | termios::ECHO);
    termios::tcsetattr(0, termios::TCSANOW, &tios).unwrap();

    return (tios_initial, tios);
}
fn close_term(initial_ios: &termios::Termios) {
    termios::tcsetattr(0, termios::TCSANOW, &initial_ios).unwrap();
}

fn get_input(reader: &mut std::io::Stdin) -> String {
    let mut buf = [0; 32];

    let bytes_read = reader.read(&mut buf).unwrap();

    String::from_utf8(buf[0..bytes_read].to_vec()).unwrap()
}

fn convert(conversion: ConvertArgs, curr_color: &ColorRepresentation) {
    println!(
        "{}",
        match conversion.to {
            RequestedOutputType::HSL => OutputType::HSL.render_output(curr_color, conversion.alpha),
            RequestedOutputType::RGB => OutputType::RGB.render_output(curr_color, conversion.alpha),
            RequestedOutputType::HEX => OutputType::HEX.render_output(curr_color, conversion.alpha),
            _ => OutputType::CUSTOM(conversion.fmt.unwrap_or("%xD".to_string()))
                .render_output(curr_color, conversion.alpha),
        }
    );
}

fn main() {
    let args = Args::parse();

    let mut starting_clr = args.color.unwrap_or("#ff0000".to_string());

    let requested_bg_color = ColorRepresentation::from_color(
        &args.bg_clr.unwrap_or("#000000".to_string()),
        &ColorNameStandard::W3C,
    )
    .tohex(false);
    let requested_fg_color = ColorRepresentation::from_color(
        &args.fg_clr.unwrap_or("#ffffff".to_string()),
        &ColorNameStandard::W3C,
    )
    .tohex(false);
    let use_custom_colors = args.custom_colors;

    let requested_input_type = args.input_type.unwrap_or(SelectionType::HSL);

    let mut reader = std::io::stdin();

    if starting_clr == "-".to_string() {
        starting_clr = String::new();
        let _ = reader.read_line(&mut starting_clr);
        starting_clr = starting_clr.trim().to_string();
    }
    let clr_std = args.clr_standard.unwrap_or(ColorNameStandard::W3C);

    let output_type = match args.output_type.clone().unwrap_or(RequestedOutputType::HSL) {
        RequestedOutputType::HSL => OutputType::HSL,
        RequestedOutputType::RGB => OutputType::RGB,
        RequestedOutputType::HEX => OutputType::HEX,
        RequestedOutputType::CUSTOM => OutputType::CUSTOM(args.output_fmt.unwrap_or("%D".to_string()).to_owned()),
    };
    let used_custom_output_type = if let Some(..) = args.output_type { true } else { false };
    let (tios_initial, _tios) = setup_term();

    if args.list_colors {
        for (k, v) in clr_std.list_colors() {
            println!(
                "{}: {}",
                k,
                output_type.render_output(
                    &ColorRepresentation::from_color(
                        &format!("{};{};{}", v[0], v[1], v[2]),
                        &clr_std
                    ),
                    true
                )
            )
        }
        close_term(&tios_initial);
        return;
    }
    let mut config_folder = std::env!("XDG_CONFIG_HOME").to_owned();
    if config_folder == "" {
        config_folder = String::from(std::env!("HOME")) + &String::from("/.config");
    }
    let tpick_config_path = config_folder + &String::from("/tpick");
    let config_path = tpick_config_path + &String::from("/config.toml");

    let data = std::fs::read_to_string(config_path).unwrap_or("".to_string());
    let cfg = toml::from_str(&data).unwrap();

    let cycle_to_use = args.output_cycle.unwrap_or("default".to_owned());
    let cycle = OutputType::get_order_by_name(&cfg, &cycle_to_use);

    if let None = cycle {
        eprintln!("Invalid cycle: {}", cycle_to_use);
        exit(1);
    }

    let output_cycle = cycle.unwrap();

    let mut program_state = ProgramState::new(
        requested_input_type,
        if used_custom_output_type { output_type } else { output_cycle[0].clone() },
        &starting_clr,
        clr_std,
        output_cycle,
        cfg.to_owned(),
    );

    if let Some(ConvertSub::Convert(conversion)) = args.convert {
        convert(conversion, &program_state.curr_color);
        close_term(&tios_initial);
        return;
    }

    let key_mappings = keymaps::init_keymaps(&program_state.config);

    eprint!("\x1b[?1049h");

    let bg_color = query_color(11, &mut reader);
    let fg_color = query_color(10, &mut reader);
    if use_custom_colors {
        eprint!("\x1b]11;#{}\x07", requested_bg_color);
        eprint!("\x1b]10;#{}\x07", requested_fg_color);
    }
    eprint!("\x1b[?25l");

    cls();

    loop {
        let [_rows, cols] = query_window_area(&mut reader);

        //this variable keeps track of the step for the step increase for the HSL/RGB rendering
        let step = (360.0
            / (cols - 1/*the minus 1 is because we need to leave space for the label*/) as f32)
            .ceil();

        let square_count = (360.0 / step).ceil() as u32;
        render_display(&program_state, square_count, step);
        //after it finishes rendering, there should be nothing below it
        eprint!("\x1b[J");

        let data = get_input(&mut reader);

        if let Some(f) = key_mappings.get(&data) {
            if let Some(action) = f(&mut program_state, &data) {
                match action {
                    Action::Break => break,
                }
            }
        }
    }
    close_term(&tios_initial);
    eprint!("\x1b[?1049l");
    eprint!("\x1b[?25h");
    eprint!("\x1b]11;{}\x07", bg_color);
    eprint!("\x1b]10;{}\x07", fg_color);
    if args.print_on_exit {
        cls();
        println!(
            "{}",
            program_state
                .output_type
                .render_output(&program_state.curr_color, program_state.enable_alpha)
        );
    }
}
