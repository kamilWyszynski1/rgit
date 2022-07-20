use anyhow::{bail, Context};
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs;
use std::io::Write;
use std::str::{from_utf8, FromStr};

use crate::repository::RGitRepository;
use crate::Result;

pub enum GitObjectType {
    GitCommit,
    GitTree,
    GitTag,
    GitBlob { blob_data: Option<String> },
}

impl GitObjectType {
    fn fmt(&self) -> String {
        match self {
            GitObjectType::GitCommit => "commit",
            GitObjectType::GitTree => "tree",
            GitObjectType::GitTag => "tag",
            GitObjectType::GitBlob { blob_data: _ } => "blob",
        }
        .to_string()
    }
}

impl FromStr for GitObjectType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "commit" => Ok(Self::GitCommit),
            "tree" => Ok(Self::GitTree),
            "tag" => Ok(Self::GitTag),
            "blob" => Ok(Self::GitBlob { blob_data: None }),
            _ => bail!("unsupported object format: {}", s),
        }
    }
}

/// An object starts with a header that specifies its type: blob, commit, tag or tree.
/// This header is followed by an ASCII space (0x20), then the size of the object in bytes as an ASCII number,
/// then null (0x00) (the null byte), then the contents of the object.
pub struct GitObject<'a> {
    repo: &'a RGitRepository,
    object_type: GitObjectType,
}

impl<'a> GitObject<'a> {
    pub fn new(raw: String, repo: &'a RGitRepository) -> Result<Self> {
        // read objet type
        let x = raw.find(" ").context("space not found")?;
        let fmt = &raw[0..x];

        // read and validate object size
        let y = raw[x..].find(char::from(0)).expect("0x00 not found");
        let size: usize = raw[x..y].parse()?;

        if size != raw.len() - y - 1 {
            bail!("malformed object {}: bad length", size);
        }

        Ok(Self {
            repo,
            object_type: GitObjectType::from_str(fmt)?,
        })
    }

    fn serialize(&self) -> String {
        match &self.object_type {
            GitObjectType::GitCommit => todo!(),
            GitObjectType::GitTree => todo!(),
            GitObjectType::GitTag => todo!(),
            GitObjectType::GitBlob { blob_data } => {
                blob_data.as_ref().expect("git blob has empty data").into()
            }
        }
    }

    fn deserialize(&mut self, data: String) {
        match self.object_type {
            GitObjectType::GitCommit => todo!(),
            GitObjectType::GitTree => todo!(),
            GitObjectType::GitTag => todo!(),
            GitObjectType::GitBlob { blob_data: _ } => {
                self.object_type = GitObjectType::GitBlob {
                    blob_data: Some(data),
                }
            }
        }
    }

    /// Writing an object is reading it in reverse: we compute the hash, insert the header, zlib-compress
    /// everything and write the result in place. This really shouldn’t require much explanation, just
    /// notice that the hash is computed after the header is added
    pub fn write(&self, actually_write: Option<bool>) -> Result<String> {
        let actually_write = actually_write.unwrap_or(true);

        let data = self.serialize();
        // add header
        let result = format!(
            "{} {}{}{}",
            self.object_type.fmt(),
            data.len(),
            char::from(0),
            data
        );

        // compute hash
        let mut hasher = Sha1::new();
        hasher.input_str(&result);
        let sha = hasher.result_str();

        if actually_write {
            let path = self
                .repo
                .repo_file(&vec!["objects", &sha[..2], &sha[2..]], Some(actually_write))
                .context("could not create path for object")?;

            let mut e = ZlibEncoder::new(vec![], Compression::default());
            e.write_all(result.as_bytes())?;
            let compressed = e.finish()?;

            fs::write(path, compressed)?;
        }

        Ok(sha)
    }
}
