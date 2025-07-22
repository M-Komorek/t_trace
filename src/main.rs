use clap::{Parser, Subcommand};
use std::env;
use std::io;

mod config;
mod handlers;
mod model;
mod utils;

include!(concat!(env!("OUT_DIR"), "/", env!("BASH_HOOK_FILENAME")));

#[derive(Subcommand, Debug)]
enum Commands {
    Install,
    Start {
        #[arg(short, long)]
        command: String,
    },
    End,
    Stats,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> io::Result<()> {
    if env::var_os("_T_TRACE_HOOK").is_some() {
        print!("{}", BASH_HOOK_SCRIPT);
        return Ok(());
    }

    let cli = Cli::parse();
    let app_config = config::AppConfig::new()?;

    match cli.command {
        Commands::Install => handlers::handle_install(),
        Commands::Start { command } => handlers::handle_start(&app_config, command),
        Commands::End => handlers::handle_end(&app_config),
        Commands::Stats => handlers::handle_stats(&app_config),
    }
}
