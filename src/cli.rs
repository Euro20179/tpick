use clap::ColorChoice;
use clap::Parser;

use crate::ColorNameStandard;
use crate::SelectionType;

#[derive(Debug, PartialEq, Clone, clap::ValueEnum)]
pub enum RequestedOutputType {
    HSL,
    RGB,
    HEX,
    CYMK,
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
    pub action: Option<ConvertSub>,
}

#[derive(clap::Subcommand, Debug)]
#[command(about = "Convert a color from one format to another")]
pub enum ConvertSub {
    #[command(aliases = ["to"])]
    Convert(ConvertArgs),
    #[command()]
    Mix(MixArgs)
}

#[derive(Parser, Debug)]
#[command(about = "Mix 2 colors")]
pub struct MixArgs{
    pub color: String,
    pub with: String,
    pub percentage: u8,
    #[arg(short, long, help = "preview the color")]
    pub preview: bool
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
