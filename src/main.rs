#![feature(is_some_with)]
use clap::Parser;

mod cli;
mod file;
mod object;
mod repository;

pub type Result<T> = std::result::Result<T, anyhow::Error>;

fn main() {
    let cli = cli::Cli::parse();
    cli.run();
}
