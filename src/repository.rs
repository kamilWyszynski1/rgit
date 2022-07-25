use anyhow::{bail, Context, Ok};
use configparser::ini::Ini;
use flate2::read::ZlibDecoder;

use crate::{object::GitObject, Result};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

pub struct RGitRepository {
    work_tree: PathBuf,
    git_dir: PathBuf,
    conf: HashMap<String, HashMap<String, Option<String>>>,
}

impl RGitRepository {
    /// Creates new .rgit configuration
    pub fn init<P: AsRef<Path>>(path: P, force: bool) -> Result<Self> {
        let path = path.as_ref();
        let git_dir = path.join(".git");

        if !(force || git_dir.is_dir()) {
            bail!("not a git repository {:?}", path.to_str().unwrap());
        }

        let mut rgit_repo = Self {
            git_dir,
            work_tree: path.to_path_buf(),
            conf: HashMap::default(),
        };

        // ead configuration file in .git/config
        match rgit_repo.repo_file(&vec!["config"], None) {
            Some(cf) => {
                if cf.exists() {
                    rgit_repo.conf = Ini::new().load(cf).unwrap();
                }
            }
            None => {
                if !force {
                    bail!("configuration file is missing");
                }
            }
        };

        if !force {
            let vers = rgit_repo
                .conf
                .get("core")
                .unwrap()
                .get("repositoryformatversion")
                .unwrap();

            if vers.is_some_and(|v| v != "0") {
                bail!("Unsupported repositoryformatversion {:?}", vers);
            }
        }

        Ok(rgit_repo)
    }

    /// Computes path under repo's gitdir.
    fn repo_path(&self, path: &[&str]) -> PathBuf {
        let mut path_buf = self.git_dir.to_path_buf();
        path_buf.extend(path);
        return path_buf;
    }

    /// Same as repo_path, but create dirname(*path) if absent.
    /// For example, repo_file(r, \"refs\", \"remotes\", \"origin\", \"HEAD\") will create .git/refs/remotes/origin.
    pub fn repo_file(&self, path: &[&str], mkdir: Option<bool>) -> Option<PathBuf> {
        if self
            .repo_dir(&path[..path.len() - 1], mkdir)
            .is_ok_and(|opt| {
                debug!("repo_file - is ok, is_some: {}", opt.is_some());
                opt.is_some()
            })
        {
            debug!("repo_file - path: {:?}", path);
            return Some(self.repo_path(path));
        }
        None
    }

    /// Same as repo_path, but mkdir *path if absent if mkdir.
    fn repo_dir(&self, path: &[&str], mkdir: Option<bool>) -> Result<Option<PathBuf>> {
        let path = self.repo_path(path);

        debug!(
            "repo_dir - path: {:?}, exists: {}, is_dir: {}",
            path,
            path.exists(),
            path.is_dir()
        );
        if path.exists() {
            if path.is_dir() {
                return Ok(Some(path));
            } else {
                bail!("not a directory {:?}", path.to_str().unwrap())
            }
        }

        if mkdir.unwrap_or_default() {
            std::fs::create_dir_all(&path)?;
            return Ok(Some(path));
        }
        Ok(None)
    }

    /// Read object object_id from Git repository repo.
    /// Return a GitObject whose exact type depends on the object.
    ///
    /// To read an object, we need to know its hash. We then compute its path from this hash (with the formula explained above:
    /// first two characters, then a directory delimiter /, then the remaining part) and look it up inside of the
    /// “objects” directory in the gitdir. That is, the path to e673d1b7eaa0aa01b5bc2442d570a765bdaae751 is
    /// .git/objects/e6/73d1b7eaa0aa01b5bc2442d570a765bdaae751.
    fn object_read(&self, sha: String) -> Result<GitObject> {
        match self.repo_file(&vec!["objects", &sha[0..2], &sha[2..]], None) {
            Some(path) => {
                debug!("object_read - path: {:?}", path);
                let mut z = ZlibDecoder::new(File::open(path).context("could not open a file")?);
                let mut s = String::new();
                z.read_to_string(&mut s)
                    .context("could not read to string")?;

                GitObject::object_read(s, self)
            }
            None => bail!("object not found"),
        }
    }

    fn object_find(&self, name: String, fmt: Option<String>, follow: Option<bool>) -> String {
        name
    }

    pub fn cat_file(&self, obj: String, fmt: Option<String>) -> Result<()> {
        let object = self.object_read(self.object_find(obj, fmt, None))?;
        debug!("cat_file - object found");
        println!("{}", object.serialize());
        Ok(())
    }
}

pub fn repo_create<P: AsRef<Path>>(path: P) -> Result<()> {
    let repo = RGitRepository::init(&path, true)?;

    let path = path.as_ref();

    if repo.work_tree.exists() {
        if !repo.work_tree.is_dir() {
            bail!("{:?} is not a directory", path)
        }
        let is_empty = repo.work_tree.read_dir()?.next().is_none();
        if !is_empty {
            bail!("{:?} is not empty", path)
        }
    }
    fs::create_dir_all(repo.work_tree.clone())?;

    repo.repo_dir(&vec!["branches"], Some(true))?;
    repo.repo_dir(&vec!["objects"], Some(true))?;
    repo.repo_dir(&vec!["refs", "tags"], Some(true))?;
    repo.repo_dir(&vec!["refs", "heads"], Some(true))?;

    // .git/description
    fs::write(
        repo.repo_file(&vec!["description"], None).unwrap(),
        "Unnamed repository; edit this file 'description' to name the repository.\n",
    )?;

    // .git/HEAD
    fs::write(
        repo.repo_file(&vec!["HEAD"], None).unwrap(),
        "ref: refs/heads/master\n",
    )?;

    // .git/config
    repo_default_config().write(repo.repo_file(&vec!["config"], None).unwrap())?;

    Ok(())
}

/// Sets default config for ini file.
fn repo_default_config() -> Ini {
    let mut conf = Ini::new();

    conf.set("core", "repositoryformatversion", Some("0".into()));
    conf.set("core", "filemode", Some("false".into()));
    conf.set("core", "bare", Some("false".into()));

    return conf;
}

/// Searches for .git directory.
pub fn repo_find<P: AsRef<Path>>(
    path: Option<P>,
    required: Option<bool>,
) -> Result<Option<RGitRepository>> {
    // default values.

    let path = path.as_ref().map_or(Path::new("."), AsRef::as_ref);
    let required = required.unwrap_or(true);

    let path = Path::new(path);

    if path.join(".git").is_dir() {
        return Ok(Some(RGitRepository::init(path, false)?));
    }

    let parent = fs::canonicalize(path.join(".."))?;

    if parent == path {
        // Bottom case
        // os.path.join("/", "..") == "/":
        // If parent==path, then path is root.
        if required {
            bail!("No git directory")
        } else {
            return Ok(None);
        }
    }
    return repo_find(Some(parent), Some(required));
}
