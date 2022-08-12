use anyhow::{bail, Context};
use clap::clap_derive::ArgEnum;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use indexmap::IndexMap;
use std::fs;
use std::io::Write;
use std::str::{from_utf8, FromStr};

use crate::repository::RGitRepository;
use crate::Result;

#[derive(Debug, Clone, Copy, ArgEnum, PartialEq)]
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
    pub object_type: Option<GitObjectType>,

    /// object specific fields.
    pub kvlm: Option<IndexMap<String, Vec<String>>>,
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
            kvlm: None,
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
            GitObjectType::Commit => match &self.kvlm {
                Some(kvlm) => kvlm_serialize(kvlm),
                None => "kvlm is not set".to_string(),
            },
            GitObjectType::Tree => todo!(),
            GitObjectType::Tag => todo!(),
            GitObjectType::Blob => self.data.as_ref().expect("git blob has empty data").into(),
        }
    }

    pub fn deserialize(&mut self, data: String) {
        match self.object_type.as_ref().unwrap() {
            GitObjectType::Commit => {
                self.kvlm = Some(kvlm_parse(data, None, None).expect("failed to kvlm parse"))
            }
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

fn kvlm_parse(
    raw: String,
    start: Option<usize>,
    dct: Option<IndexMap<String, Vec<String>>>,
) -> Result<IndexMap<String, Vec<String>>> {
    let start = start.unwrap_or_default();
    let mut dct = dct.unwrap_or_default();

    let spc = raw[start..].find(' ').map(|i| i + start);
    let nl = raw[start..].find("\n").map(|i| i + start);

    // If space appears before newline, we have a keyword.
    //
    // If newline appears first (or there's no space at all, in which
    // case find returns -1), we assume a blank line.  A blank line
    // means the remainder of the data is the message.

    debug!(
        "kvlm_parse - nl: {:?}, start: {}, spc: {:?}, dct: {:?}",
        nl, start, spc, dct,
    );
    if spc.is_none() || (spc.is_some() && nl.is_some() && (nl.unwrap() < spc.unwrap())) {
        // assert!(nl.unwrap() == start);

        dct.insert("".into(), vec![raw[start + 1..].into()]);
        return Ok(dct);
    }

    let spc = spc.unwrap();

    // Recursive case - we read a key-value pair and recurse for the next.
    let key = &raw[start..spc];

    // Find the end of the value. Continuation lines begin with a
    // space, so we loop until we find a "\n" not followed by a space.
    let mut end = start;

    loop {
        match raw[end + 1..].find("\n").map(|i| i + end + 1) {
            Some(v) => end = v,
            None => break,
        }

        if !raw
            .chars()
            .nth(end + 1)
            .unwrap_or_default()
            .eq(&char::from_u32(32).unwrap())
        {
            break;
        }
    }

    // Grab the value. Also, drop the leading space on continuation lines.
    let value = raw[spc + 1..end].replace("\n ", "\n");

    dct.entry(key.to_owned()).or_insert(vec![]).push(value);

    kvlm_parse(raw, Some(end + 1), Some(dct))
}

fn kvlm_serialize(kvlm: &IndexMap<String, Vec<String>>) -> String {
    let mut ret: String = String::from("");

    for (k, v) in kvlm {
        if k == "" {
            continue;
        }
        v.iter().for_each(|val| {
            ret += format!("{} {}\n", &k, val.replace("\n", "\n ")).as_str();
            // ret += &k + " " + &(val.replace("\n", "\n ")) + "\n";
        })
    }

    ret += format!("\n{}", kvlm[""][0]).as_str();
    ret
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::kvlm_parse;

    #[test]
    fn test_kvlm_parse() {
        let content = "tree 29ff16c9c14e2652b22f8b78bb08a5a07930c147
parent 206941306e8a8af65b66eaaaea388a7ae24d49a0
author Thibault Polge <thibault@thb.lt> 1527025023 +0200
committer Thibault Polge <thibault@thb.lt> 1527025044 +0200

Create first draft";

        let values = kvlm_parse(content.to_string(), None, None);
        let wanted = IndexMap::from([
            (
                String::from("tree"),
                vec![String::from("29ff16c9c14e2652b22f8b78bb08a5a07930c147")],
            ),
            (
                String::from("parent"),
                vec![String::from("206941306e8a8af65b66eaaaea388a7ae24d49a0")],
            ),
            (
                String::from("author"),
                vec![
                    String::from("Thibault"),
                    String::from("Polge"),
                    String::from("<thibault@thb.lt>"),
                    String::from("1527025023"),
                    String::from("+0200"),
                ],
            ),
            (
                String::from("committer"),
                vec![
                    String::from("Thibault"),
                    String::from("Polge"),
                    String::from("<thibault@thb.lt>"),
                    String::from("1527025044"),
                    String::from("+0200"),
                ],
            ),
            (String::from(""), vec![String::from("Create first draft")]),
        ]);
        assert_eq!(values.unwrap(), wanted);
    }
}
