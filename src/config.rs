use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "mdrender",
    author = "MDRender Authors",
    version,
    about = "Markdown to HTML converter with light/dark modes, theme support, and web UI"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[command(flatten)]
    pub build_args: Config,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Launch the interactive Web UI dashboard
    Ui(UiArgs),
    /// Archive current input and output folders to a numbered subfolder in archive directory
    Archive(ArchiveArgs),
    /// Build markdown to HTML documentation (default if no command given)
    Build(Config),
}

#[derive(Args, Debug, Clone)]
pub struct UiArgs {
    /// Port to listen on
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,

    /// Host address to bind to
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Automatically open browser on start
    #[arg(long, default_value_t = false)]
    pub open: bool,

    /// Input directory containing markdown files
    #[arg(short, long, default_value = "input")]
    pub input: PathBuf,

    /// Output directory for generated HTML and assets
    #[arg(short, long, default_value = "output")]
    pub output: PathBuf,

    /// Archive root directory
    #[arg(long, default_value = "archive")]
    pub archive_dir: PathBuf,

    /// Site title
    #[arg(short, long, default_value = "MDRender Docs")]
    pub title: String,
}

#[derive(Args, Debug, Clone)]
pub struct ArchiveArgs {
    /// Preferred name for the archive (e.g. GraphTutorials)
    pub name: String,

    /// Archive root directory
    #[arg(long, default_value = "archive")]
    pub archive_dir: PathBuf,

    /// Input directory to archive and clean
    #[arg(short, long, default_value = "input")]
    pub input: PathBuf,

    /// Output directory to archive and clean
    #[arg(short, long, default_value = "output")]
    pub output: PathBuf,

    /// Do not clean input and output folders after copy
    #[arg(long, default_value_t = false)]
    pub no_clean: bool,
}

#[derive(Args, Debug, Clone)]
pub struct Config {
    /// Launch the interactive Web UI dashboard
    #[arg(long, default_value_t = false)]
    pub ui: bool,

    /// Input directory containing markdown files
    #[arg(short, long, default_value = "input")]
    pub input: PathBuf,

    /// Output directory for generated HTML and assets
    #[arg(short, long, default_value = "output")]
    pub output: PathBuf,

    /// Site title displayed in header and browser tab
    #[arg(short, long, default_value = "MDRender Docs")]
    pub title: String,

    /// Clean output directory before building
    #[arg(long, default_value_t = false)]
    pub clean: bool,

    /// Watch input directory for changes and automatically rebuild
    #[arg(short, long, default_value_t = false)]
    pub watch: bool,

    /// Scaffold sample markdown files in the input directory
    #[arg(long, default_value_t = false)]
    pub init: bool,
}
