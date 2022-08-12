use crate::{
    object::{GitObject, GitObjectType},
    repository::{repo_find, RGitRepository},
    Result,
};
use anyhow::{Context, Ok};
use clap::{Parser, Subcommand};
use std::{collections::HashSet, fs};

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

    CatFile {
        /// Specify the type.
        #[clap(arg_enum, value_name = "TYPE")]
        object_type: GitObjectType,

        /// The object to display
        object: String,
    },

    /// Compute object ID and optionally creates a blob from a file
    HashObject {
        /// Specify the type.
        #[clap(arg_enum, short, value_name = "TYPE")]
        tpe: GitObjectType,

        /// Actually write the object into the database
        #[clap(short)]
        write: bool,

        /// Read object from <file>.
        file: String,
    },

    /// Display history of a given commit.
    Log {
        /// Commit to start at.
        #[clap(default_value = "HEAD")]
        commit: String,
    },
}

impl Commands {
    fn run(&self) {
        match self {
            Commands::Init { path } => repo_create(path.clone().unwrap_or(".".into())).unwrap(),
            Commands::CatFile {
                object_type,
                object,
            } => cmd_cat_file(object_type, object).expect("cmd cat file failed"),
            Commands::HashObject { tpe, write, file } => {
                cmd_hash_object(tpe, *write, file).expect("cmd hash object failed")
            }
            Commands::Log { commit } => cmd_log(commit).expect("cmd log failed"),
        }
    }
}

fn cmd_cat_file(object_type: &GitObjectType, object: &str) -> Result<()> {
    let repo = repo_find::<&str>(None, None)?.context("repo not found")?;

    repo.cat_file(object, Some(object_type.to_string()))?;

    Ok(())
}

fn cmd_hash_object(object_type: &GitObjectType, write: bool, file: &str) -> Result<()> {
    let repo = RGitRepository::init(".", false)?;

    let data = fs::read_to_string(file)?;

    let data = GitObject::new(&repo, Some(data), Some(*object_type))?.object_write(Some(write))?;
    println!("{}", data);

    Ok(())
}

fn cmd_log(commit: &str) -> Result<()> {
    let repo = repo_find::<&str>(None, None)?.context("repo not found")?;

    println!("digraph wyaglog{{");
    log_graphviz(
        &repo,
        &repo.object_find(commit, None, None),
        &mut HashSet::new(),
    )?;
    println!("}}");
    Ok(())
}

fn log_graphviz(repo: &RGitRepository, sha: &str, seen: &mut HashSet<String>) -> Result<()> {
    if seen.contains(sha) {
        return Ok(());
    }
    seen.insert(sha.to_string());

    let commit = repo.object_read(sha)?;
    assert_eq!(
        commit.object_type.context("object type is None")?,
        GitObjectType::Commit
    );

    let kvlm = commit.kvlm.context("kvlm is empty")?;
    if !kvlm.contains_key("parent") {
        // initial commit.
        return Ok(());
    }

    let parents = &kvlm["parent"];

    for p in parents {
        println!("c_{} -> c_{}", sha, p);
        log_graphviz(repo, p, seen)?
    }

    Ok(())
}
