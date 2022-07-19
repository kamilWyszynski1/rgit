use anyhow::{bail, Ok};

use crate::Result;
use std::{
    cmp::max,
    fs::{self, DirEntry, Metadata},
    path::PathBuf,
};

#[derive(Debug, PartialEq)]
/// Indicates if a file is normal file with a content or directory.
enum FileType {
    File,
    Directory,
}

#[derive(Debug, PartialEq)]
struct FileNode {
    name: String,
    file_type: FileType,
    content: Vec<u8>,
    children: Option<Vec<FileNode>>,
}

impl FileNode {
    fn new2(directory_path: &str) -> Result<Self> {
        Self::traverse_and_build(
            directory_path,
            vec![
                ignore_by_file_name("target".into()),
                ignore_dir_starting_with_dot2(),
            ],
        )
    }

    fn traverse_and_build(
        directory_path: &str,
        skip_predicates: Vec<Box<dyn Fn(&DirEntry, &PathBuf, &Metadata) -> bool>>,
    ) -> Result<Self> {
        let mut root = Self {
            name: directory_path.into(),
            file_type: FileType::Directory,
            content: vec![],
            children: None,
        };
        let mut children = vec![];

        traverse_directory(directory_path, |entry, path, metadata| {
            for sk in &skip_predicates {
                if sk(&entry, &path, &metadata) {
                    println!("skipped {:?}", entry);

                    return Ok(());
                }
            }
            println!("{:?}", entry);
            if metadata.is_dir() {
                children.push(Self::new(path.to_str().unwrap())?);
            }
            if metadata.is_file() {
                children.push(Self {
                    name: entry.file_name().to_str().unwrap().into(),
                    file_type: FileType::File,
                    content: vec![],
                    children: None,
                });
            }
            Ok(())
        })?;

        if !children.is_empty() {
            root.children = Some(children);
        }
        Ok(root)
    }

    fn new(directory_path: &str) -> Result<Self> {
        let mut root = Self {
            name: directory_path.into(),
            file_type: FileType::Directory,
            content: vec![],
            children: None,
        };
        let mut children = vec![];
        for entry in fs::read_dir(directory_path)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = fs::metadata(&path)?;

            println!("{:?}", metadata);

            if metadata.is_dir() {
                children.push(Self::new(path.to_str().unwrap())?);
            }
            if metadata.is_file() {
                children.push(Self {
                    name: entry.file_name().to_str().unwrap().into(),
                    file_type: FileType::File,
                    content: vec![],
                    children: None,
                });
            }
        }
        if !children.is_empty() {
            root.children = Some(children);
        }
        Ok(root)
    }
}

fn ignore_dir_starting_with_dot2() -> Box<dyn Fn(&DirEntry, &PathBuf, &Metadata) -> bool> {
    Box::new(
        move |entry: &DirEntry, path: &PathBuf, metadata: &Metadata| -> bool {
            metadata.is_dir() && entry.file_name().to_str().unwrap().starts_with(".")
        },
    )
}

fn ignore_by_file_name(file_name: String) -> Box<dyn Fn(&DirEntry, &PathBuf, &Metadata) -> bool> {
    Box::new(
        move |entry: &DirEntry, path: &PathBuf, metadata: &Metadata| -> bool {
            entry.file_name().to_str().unwrap().eq(&file_name)
        },
    )
}

fn traverse_directory<F>(path: &str, mut cursor: F) -> Result<()>
where
    F: FnMut(DirEntry, PathBuf, Metadata) -> Result<()>,
{
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::metadata(&path)?;

        cursor(entry, path, metadata);
    }
    Ok(())
}

/// The longest common subsequence (LCS) problem is the problem of finding the longest subsequence common to all sequences in a set of sequences.
///
/// https://en.wikipedia.org/wiki/Longest_common_subsequence_problem
///
/// L(ASDFG,BCDEG) = 1 + L(ASDF,BCDE)
/// L(ASDFGI,BCDEG) = MAX(L(ASDFG,BCDEG),L(ASDFGI,BCDE))
fn lcs(s1: &str, s2: &str, dp: &mut Vec<Vec<isize>>) -> usize {
    let (m, n) = (s1.len(), s2.len());

    if m == 0 || n == 0 {
        return 0;
    }

    if dp[m - 1][n - 1] != -1 {
        return dp[m - 1][n - 1] as usize;
    }

    if s1.chars().last().unwrap() == s2.chars().last().unwrap() {
        let value = 1 + lcs(&s1[..m - 1], &s2[..n - 1], dp);
        dp[m - 1][n - 1] = value as isize;
        return value;
    }
    let value = max(lcs(s1, &s2[..n - 1], dp), lcs(&s1[..m - 1], s2, dp));
    dp[m - 1][n - 1] = value as isize;
    return value;
}

#[cfg(test)]
mod tests {
    use super::FileNode;

    #[test]
    fn test_new_file_tree() {
        let root = FileNode::new2(".").unwrap();
        println!("{:?}", root)
    }
}
