#[macro_use]
mod math;
mod color_conversions;
mod color_representation;
mod keymaps;
mod ui;

use color_representation::*;
use keymaps::Action;

use std::fmt::Display;
use std::io::Read;
use std::os::fd::AsRawFd;

use base64::engine::general_purpose;
use base64::prelude::*;
use clap::ColorChoice;
use clap::Parser;
use termios::Termios;

use color_conversions::*;

fn cls() {
    eprint!("\x1b[2J\x1b[0H");
}

unsafe fn query_winsize(fd: i32, ws_struct: &mut libc::winsize) {
    libc::ioctl(fd, libc::TIOCGWINSZ, ws_struct);
}

fn render_ansi256(selected_item: u8, _square_count: u32) {
    for low_nr in 0..16 {
        eprint!("\x1b[38;5;{}m{:3} ", low_nr, low_nr);
    }
    eprintln!();
    for x in 0..6 {
        for y in 0..6 {
            for z in 0..6 {
                let clr = (x + 16) + (6 * y) + (36 * z);
                eprint!("\x1b[38;5;{}m{:3} ", clr, clr);
            }
        }
        eprintln!();
    }
    for grey_nr in 232..256 {
        eprint!("\x1b[38;5;{}m{:3} ", grey_nr, grey_nr);
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
    let mut color =
        ColorRepresentation::from_color(&format!("rgb({},{},{})", colors[0], colors[1], colors[2]));
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
    let mut color =
        ColorRepresentation::from_color(&format!("hsl({},{},{})", colors[0], colors[1], colors[2]));
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

fn render_a(square_count: u32) {
    eprint!("A");
    let mut sat_color_rep = ColorRepresentation::from_color("#000000");
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
    enable_alpha: bool,
) {
    renderer(curr_color, selected_item, square_count, step);

    if enable_alpha {
        eprint!("\x1b[7;0H");
        if selected_item == 3 {
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
        },
        square_count,
        step,
        program_state.selected_item,
        program_state.enable_alpha,
    );
    for _ in 0..3 {
        eprintln!(
            "\x1b[38;2;{}m████████\x1b[0m",
            program_state.curr_color.toansi(false)
        );
    }
    eprint!("\x1b[J");
    eprint!(
        "{}",
        program_state
            .output_type
            .render_output(&program_state.curr_color, program_state.enable_alpha)
    );
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

    fn max_values(&self) -> Vec<f32> {
        match self {
            SelectionType::HSL => vec![359.0, 100.0, 100.0, 255.0],
            SelectionType::RGB => vec![255.0, 255.0, 255.0, 255.0],
            SelectionType::ANSI256 => vec![255.0],
        }
    }

    fn increments(&self) -> Vec<f32> {
        match self {
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
    fn render_output(&self, curr_color: &ColorRepresentation, enable_alpha: bool) -> String {
        format!(
            "\x1b[2K{}",
            curr_color.get_formatted_output_clr(self, enable_alpha)
        )
    }
}

fn read_ansi_color(reader: &mut std::io::Stdin, clr_num: u8) -> String {
    eprintln!("\x1b]4;{};?\x07", clr_num);
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

///clr can be 10 or 11
fn query_color(clr: u8, reader: &mut std::io::Stdin) -> String {
    eprint!("\x1b]{};?\x07", clr);
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

fn get_input(reader: &mut std::io::Stdin) -> String {
    let mut buf = [0; 32];

    let bytes_read = reader.read(&mut buf).unwrap();

    String::from_utf8(buf[0..bytes_read].to_vec()).unwrap()
}

#[derive(Parser, Debug)]
#[command(color = ColorChoice::Auto, long_about = "A color picker")]
struct Args {
    color: Option<String>,
    #[arg(short, long)]
    print_on_exit: bool,
    #[arg(short, long, help = "Enables use of --bg-clr and --fg-clr")]
    use_custom_colors: bool,
    #[arg(short, long)]
    bg_clr: Option<String>,
    #[arg(short, long)]
    fg_clr: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut starting_clr = args.color.unwrap_or("#ff0000".to_string());

    let requested_bg_color =
        ColorRepresentation::from_color(&args.bg_clr.unwrap_or("#000000".to_string())).tohex(false);
    let requested_fg_color =
        ColorRepresentation::from_color(&args.fg_clr.unwrap_or("#ffffff".to_string())).tohex(false);
    let use_custom_colors = args.use_custom_colors;

    let mut reader = std::io::stdin();

    if starting_clr == "-".to_string() {
        starting_clr = String::new();
        let _ = reader.read_line(&mut starting_clr);
        starting_clr = starting_clr.trim().to_string();
    }

    let tty = std::fs::File::open("/dev/tty").unwrap();
    let tty_fd = tty.as_raw_fd();
    unsafe { libc::dup2(tty_fd, 0) };

    let (tios_initial, _tios) = setup_term();

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
    let step = (360.0
        / (wsz.ws_col - 1/*the minus 1 is because we need to leave space for the label*/) as f32)
        .ceil();

    let square_count = (361.0 / step).ceil() as u32;

    let mut program_state = ProgramState {
        selected_item: 0,
        selection_type: SelectionType::HSL,
        output_type: OutputType::HSL,
        enable_alpha: false,
        curr_color: ColorRepresentation::from_color(starting_clr.as_str()),
    };

    let key_mappings = keymaps::init_keymaps();

    cls();

    let bg_color = query_color(11, &mut reader);
    let fg_color = query_color(10, &mut reader);
    if use_custom_colors{
        eprint!("\x1b]11;#{}\x07", requested_bg_color);
        eprint!("\x1b]10;#{}\x07", requested_fg_color);
    }

    loop {
        render_display(&program_state, square_count, step);
        eprint!("\x1b[?25l");

        let data = get_input(&mut reader);

        if let Some(f) = key_mappings.get(&data) {
            if let Some(action) = f(&mut program_state, &data) {
                match action {
                    Action::Break => break,
                }
            }
        }
    }
    termios::tcsetattr(0, termios::TCSANOW, &tios_initial).unwrap();
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
