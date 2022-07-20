use clap::{Parser, Subcommand};

use crate::repository::repo_create;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn run(&self) {
        self.command.run();
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// initializes .git
    Init {
        /// Optional path for .git repository.
        path: Option<String>,
    },
}

impl Commands {
    fn run(&self) {
        match self {
            Commands::Init { path } => repo_create(path.clone().unwrap_or(".".into())).unwrap(),
        }
    }
}
