mod app;
mod git;
mod keybind;
mod tui;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "rune", about = "TUI Git diff viewer and porcelain")]
struct Cli {
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let path = if cli.path.is_absolute() {
        cli.path
    } else {
        std::env::current_dir()?.join(&cli.path)
    };

    tui::run(&path)
}
