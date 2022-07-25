#![feature(is_some_with)]
use clap::Parser;

#[macro_use]
extern crate log;

mod cli;
mod file;
mod object;
mod repository;

pub type Result<T> = std::result::Result<T, anyhow::Error>;

fn main() {
    env_logger::init();
    
    let cli = cli::Cli::parse();
    cli.run();
}
