mod app;
mod fs;
mod ui;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rudu", version, about)]
struct Cli {
    #[arg(value_name = "PATH", default_value = ".")]
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let mut app = app::App::new(&cli.path);
    ui::run(&mut app).unwrap();
}
