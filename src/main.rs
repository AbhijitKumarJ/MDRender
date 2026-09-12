mod archive;
mod assets;
mod builder;
mod config;
mod html;
mod markdown;
mod server;
mod watcher;

use clap::Parser;
use config::{Cli, Commands, UiArgs};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Ui(args)) => {
            server::start_server(args)?;
        }
        Some(Commands::Archive(args)) => {
            archive::run_archive(&args)?;
        }
        Some(Commands::Build(config)) => {
            if config.ui {
                launch_ui_from_config(&config)?;
            } else {
                execute_build(&config)?;
            }
        }
        None => {
            if cli.build_args.ui {
                launch_ui_from_config(&cli.build_args)?;
            } else {
                execute_build(&cli.build_args)?;
            }
        }
    }

    Ok(())
}

fn launch_ui_from_config(config: &config::Config) -> anyhow::Result<()> {
    let ui_args = UiArgs {
        port: 3000,
        host: "127.0.0.1".to_string(),
        open: true,
        input: config.input.clone(),
        output: config.output.clone(),
        archive_dir: std::path::PathBuf::from("archive"),
        title: config.title.clone(),
    };
    server::start_server(ui_args)
}

fn execute_build(config: &config::Config) -> anyhow::Result<()> {
    if config.init {
        builder::scaffold_sample_content(&config.input)?;
        println!("Sample markdown files created in `{}`.", config.input.display());
        return Ok(());
    }

    if config.watch {
        watcher::watch_and_serve(config)?;
    } else {
        builder::build(config)?;
    }

    Ok(())
}
