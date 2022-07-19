use anyhow::{bail, Ok};

use crate::Result;
use std::{
    cmp::max,
    fs::{self, DirEntry, Metadata},
    path::PathBuf,
    rc::Rc,
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

/// Custom type for predicate that will skip files. Rc for clonability.
type SkipPredicate = Rc<dyn Fn(&DirEntry, &PathBuf, &Metadata) -> bool>;

impl FileNode {
    fn new(directory_path: &str) -> Result<Self> {
        Self::traverse_and_build(
            directory_path,
            vec![
                ignore_by_file_name("target".into()),
                ignore_dir_starting_with_dot(),
            ],
        )
    }

    fn traverse_and_build(
        directory_path: &str,
        skip_predicates: Vec<SkipPredicate>,
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
                children.push(Self::traverse_and_build(
                    path.to_str().unwrap(),
                    skip_predicates.clone(),
                )?);
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
}

/// Returned SkipPredicate will skip directiory that starts with "."
fn ignore_dir_starting_with_dot() -> SkipPredicate {
    Rc::new(
        move |entry: &DirEntry, _: &PathBuf, metadata: &Metadata| -> bool {
            metadata.is_dir() && entry.file_name().to_str().unwrap().starts_with(".")
        },
    )
}

/// Returned SkipPredicate will skip files with given file name.
fn ignore_by_file_name(file_name: String) -> SkipPredicate {
    Rc::new(move |entry: &DirEntry, _: &PathBuf, _: &Metadata| -> bool {
        entry.file_name().to_str().unwrap().eq(&file_name)
    })
}

/// Function takes cursor and goes through whole directory.
fn traverse_directory<F>(path: &str, mut cursor: F) -> Result<()>
where
    F: FnMut(DirEntry, PathBuf, Metadata) -> Result<()>,
{
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::metadata(&path)?;

        cursor(entry, path, metadata)?;
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
        let root = FileNode::new(".").unwrap();
        println!("{:?}", root)
    }
}
