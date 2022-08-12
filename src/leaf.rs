use crate::{
    object::{GitObject, GitObjectType},
    Result,
};
use anyhow::{bail, Context, Ok};

#[derive(Debug, Default)]
struct GitTreeLeaf {
    mode: String,
    path: String,
    sha: String,
}

impl GitTreeLeaf {
    fn new(mode: String, path: String, sha: String) -> Self {
        Self { mode, path, sha }
    }

    fn tree_parse_one(raw: &str, start: Option<usize>) -> Result<(usize, Self)> {
        let start = start.unwrap_or(0);
        // find the space terminator of the mode.
        let x = raw.find(" ").context("space not found")?;
        assert!((x - start) == 5 || (x - start) == 6);

        // read the mode.
        let mode = &raw[start..x];

        // find the NULL terminator of the path;
        let y = mode[x..].find(char::from(0)).context("0x00 not found")?;
        // and read the path.
        let path = &raw[x + 1..y];

        // read the SHA and convert to an hex string
        let sha = format!(
            "{:x}",
            isize::from_be_bytes(raw[y + 1..y + 21].as_bytes().try_into()?)
        );

        Ok((y + 21, Self::new(mode.into(), path.into(), sha)))
    }
}

fn tree_parse(raw: &str) -> Result<Vec<GitTreeLeaf>> {
    let mut pos: usize = 0;
    let max = raw.len();
    let mut ret = vec![];

    while pos < max {
        let (v, data) = GitTreeLeaf::tree_parse_one(raw, Some(pos))?;
        pos = v;
        ret.push(data);
    }
    Ok(ret)
}

fn tree_serializer(obj: GitObject) {
    assert!(obj.object_type.unwrap() == GitObjectType::Tree);

    let ret = 
}
