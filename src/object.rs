use anyhow::{bail, Context};
use clap::clap_derive::ArgEnum;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs;
use std::io::Write;
use std::str::{from_utf8, FromStr};

use crate::repository::RGitRepository;
use crate::Result;

#[derive(Debug, Clone, Copy, ArgEnum)]
pub enum GitObjectType {
    Commit,
    Tree,
    Tag,
    Blob,
}

impl FromStr for GitObjectType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "commit" => Ok(Self::Commit),
            "tree" => Ok(Self::Tree),
            "tag" => Ok(Self::Tag),
            "blob" => Ok(Self::Blob),
            _ => bail!("unsupported object format: {}", s),
        }
    }
}

impl ToString for GitObjectType {
    fn to_string(&self) -> String {
        match self {
            GitObjectType::Blob => "blob",
            GitObjectType::Commit => "commit",
            GitObjectType::Tag => "tag",
            GitObjectType::Tree => "tree",
        }
        .into()
    }
}

impl GitObjectType {
    fn fmt(&self) -> String {
        match self {
            GitObjectType::Commit => "commit",
            GitObjectType::Tree => "tree",
            GitObjectType::Tag => "tag",
            GitObjectType::Blob => "blob",
        }
        .to_string()
    }
}

/// An object starts with a header that specifies its type: blob, commit, tag or tree.
/// This header is followed by an ASCII space (0x20), then the size of the object in bytes as an ASCII number,
/// then null (0x00) (the null byte), then the contents of the object.
pub struct GitObject<'a> {
    repo: &'a RGitRepository,
    data: Option<String>,
    object_type: Option<GitObjectType>,
}

impl<'a> GitObject<'a> {
    pub fn new(
        repo: &'a RGitRepository,
        data: Option<String>,
        object_type: Option<GitObjectType>,
    ) -> Result<Self> {
        let mut go = Self {
            repo,
            data: data.clone(),
            object_type,
        };

        if let Some(data) = data {
            go.deserialize(data);
        }
        Ok(go)
    }

    pub fn object_read(raw: String, repo: &'a RGitRepository) -> Result<Self> {
        // read objet type

        let x = raw.find(" ").context("space not found")?;
        let fmt = &raw[0..x];

        // read and validate object size
        let y = raw[x..].find(char::from(0)).expect("0x00 not found");
        debug!(
            "GitObject::object_read - x: {}, y:{}, raw: {}",
            x,
            y,
            &raw[x + 1..x + y]
        );
        let size: usize = raw[x + 1..x + y].parse()?;

        debug!("GitObject: size: {}, raw.lem: {}", raw.len(), size);
        if size != raw.len() - y - x - 1 {
            bail!("malformed object {}: bad length", size);
        }

        Self::new(
            repo,
            Some(raw[x + y + 1..].to_string()),
            Some(GitObjectType::from_str(fmt)?),
        )
    }

    pub fn serialize(&self) -> String {
        match &self.object_type.as_ref().unwrap() {
            GitObjectType::Commit => todo!(),
            GitObjectType::Tree => todo!(),
            GitObjectType::Tag => todo!(),
            GitObjectType::Blob => self.data.as_ref().expect("git blob has empty data").into(),
        }
    }

    pub fn deserialize(&mut self, data: String) {
        match self.object_type.as_ref().unwrap() {
            GitObjectType::Commit => todo!(),
            GitObjectType::Tree => todo!(),
            GitObjectType::Tag => todo!(),
            GitObjectType::Blob => self.data = Some(data),
        }
    }

    /// Writing an object is reading it in reverse: we compute the hash, insert the header, zlib-compress
    /// everything and write the result in place. This really shouldn’t require much explanation, just
    /// notice that the hash is computed after the header is added
    pub fn object_write(&self, actually_write: Option<bool>) -> Result<String> {
        let actually_write = actually_write.unwrap_or(true);

        let data = self.serialize();
        // add header
        let result = format!(
            "{} {}{}{}",
            self.object_type.as_ref().unwrap().fmt(),
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
