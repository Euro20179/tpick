use clap::ColorChoice;
use clap::Parser;

use crate::ColorNameStandard;
use crate::ProgramState;
use crate::SelectionType;
use crate::contrast_action;

#[derive(Debug, PartialEq, Clone, clap::ValueEnum)]
pub enum RequestedOutputType {
    HSL,
    RGB,
    HEX,
    CUSTOM,
}

#[derive(Parser, Debug)]
#[command(color = ColorChoice::Auto, long_about = "A color picker")]
pub struct Args {
    pub color: Option<String>,
    #[arg(short, long)]
    pub print_on_exit: bool,
    #[arg(short, long)]
    pub list_colors: bool,
    #[arg(
        short,
        long,
        help = "Color naming standard",
        long_help = "Color naming standard\nx11: Colors used in the X11 display server\nw3c: Colors standardized for the web\nThis is used to resolve conflicting names such as 'green'\nsee \x1b[34m\x1b]8;;https://en.wikipedia.org/wiki/X11_color_names#Clashes_between_web_and_X11_colors_in_the_CSS_color_scheme\x1b\\this wikipedia article\x1b]8;;\x07\x1b[0m for more information"
    )]
    pub clr_standard: Option<ColorNameStandard>,
    #[arg(short = 'C', long, help = "Enables use of --bg-clr and --fg-clr")]
    pub custom_colors: bool,
    #[arg(short, long)]
    pub bg_clr: Option<String>,
    #[arg(short, long)]
    pub fg_clr: Option<String>,
    #[arg(long = "cmp", help = "These colors will show up to compare against (seperate each color with a space)")]
    pub compare: Option<String>,
    #[arg(short, long, help = "colors to mix with and display")]
    pub mix_colors: Option<String>,
    #[arg(
        short,
        long,
        help = "The starting input type",
        long_help = "The starting input type"
    )]
    pub input_type: Option<SelectionType>,
    #[arg(
        short = 'O',
        long,
        help = "The output cycle to use (defined in your config)"
    )]
    pub output_cycle: Option<String>,
    #[arg(short, long, help = "The output format type")]
    pub output_type: Option<RequestedOutputType>,
    #[arg(
        short = 'F',
        long = "of",
        help = "Custom format for the CUSTOM format type"
    )]
    pub output_fmt: Option<String>,
    #[command(subcommand)]
    pub action: Option<Actions>,
}

#[derive(clap::Subcommand, Debug)]
#[command(about = "Convert a color from one format to another")]
pub enum Actions {
    #[command(aliases = ["to"])]
    Convert(ConvertArgs),
    #[command()]
    Mix(MixArgs),
    #[command(about = "Inverts the given color")]
    Invert(InvertArgs),
    #[command(about = "Check contrast against other colors")]
    Contrast(ContrastArgs)
}

#[derive(Parser, Debug)]
#[command()]
pub struct ContrastArgs{
    #[arg(help = "The colors to contrast against")]
    pub colors: Vec<String>
}

#[derive(Parser, Debug)]
#[command()]
pub struct InvertArgs {
    #[arg(short, long, help = "preview the color in a color square")]
    pub preview: bool,
}

#[derive(Parser, Debug)]
#[command(about = "Mix 2 colors")]
pub struct MixArgs {
    pub color: String,
    pub with: Vec<String>,
    #[arg(short, long, help = "preview the color in a color square")]
    pub preview: bool,
}

#[derive(Parser, Debug)]
#[command(about = "Convert one color format to another")]
pub struct ConvertArgs {
    #[arg(short, help = "Enable alpha")]
    pub alpha: bool,
    #[arg(help = "Format type")]
    pub to: RequestedOutputType,
    #[arg(help = "Custom format for the CUSTOM format type")]
    pub fmt: Option<String>,
}
