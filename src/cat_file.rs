use std::convert::TryInto;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str;
use std::{env, fmt};

use anyhow::Result;
use compress::zlib;

use super::CatFile;

// #[derive(Debug)]
pub enum Object {
    Blob(Vec<u8>),
    // Tree,
    // Commit,
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Object::Blob(b) => {
                let mut slice_len = b.len();
                let mut truncated = false;
                if slice_len > 30 {
                    slice_len = 30;
                    truncated = true;
                }

                let mut dt = f.debug_tuple("Blob");

                if let Ok(s) = String::from_utf8(b[..slice_len].to_vec()) {
                    dt.field(&s);
                } else {
                    dt.field(&&(b[..slice_len]));
                }
                dt.finish()
            }
        }
    }
}

pub fn cat_file(opts: &CatFile) -> Result<Object> {
    println!("{}", opts.object_name);
    let git_dir = nearest_git_dir()?;
    println!("{:?}", nearest_git_dir().unwrap());
    assert_eq!(opts.object_name.len(), 40, "expected 40-char SHA");

    let obj_dir = git_dir.join("objects");
    let obj_path = obj_dir
        .join(&opts.object_name[..2])
        .join(&opts.object_name[2..]);
    if !obj_path.exists() {
        bail!(
            "object {} does not exist at {:?}",
            opts.object_name,
            obj_path
        );
    }

    read_object(&obj_path)
}

fn read_object(path: &Path) -> Result<Object> {
    assert!(path.exists(), "path must exist");

    let stream = File::open(path)?;

    // Snarf it all into buf
    let mut buf = vec![];
    zlib::Decoder::new(stream).read_to_end(&mut buf)?;

    // Iterator for header, then body
    let mut header_then_body = buf.splitn(2, |b| *b == b'\0');
    let header = header_then_body
        .next()
        .ok_or(anyhow!("cannot locate header"))?;

    // Get type and len from header
    let header = str::from_utf8(&header)?;
    let mut header_split = header.split(' ');
    let type_str = header_split
        .next()
        .ok_or(anyhow!("invalid header {:?}", header))?;
    let _len = header_split
        .next()
        .ok_or(anyhow!("invalid header {:?}", header))?
        .parse::<u64>()?;

    let body = header_then_body
        .next()
        .ok_or(anyhow!("no body present in object"))?;

    match type_str {
        // TODO: figure out how I can go from a slice of buf to an owned Vec<u8>
        // without the cloning that occurs in to_vec (and to_owned).
        "blob" => Ok(Object::Blob(body.to_vec())),
        "tree" => {
            println!("{:?}", body);
            parse_tree(body)?;
            Ok(Object::Blob(body.to_vec()))
        }
        _ => Err(anyhow!("unexpected object type {}", header)),
    }
}

#[derive(Debug)]
struct Sha([u8; 20]);

#[derive(Debug)]
struct TreeEntry {
    permissions: u32,
    entry_type: TreeEntryType,
    sha: Sha,
    filename: String,
}

#[derive(Debug)]
enum TreeEntryType {
    Blob,
    Tree,
}

fn parse_tree(mut b: &[u8]) -> Result<Vec<TreeEntry>> {
    let mut result = Vec::new();

    while b.len() > 0 {
        let space_idx = b
            .iter()
            .position(|b| *b == b' ')
            .ok_or(anyhow!("invalid tree object"))?;
        let permissions = str::from_utf8(&b[..space_idx])?;
        let permissions = u32::from_str_radix(permissions, 8)?;
        b = &b[space_idx + 1..];

        let null_idx = b
            .iter()
            .position(|b| *b == b'\0')
            .ok_or(anyhow!("invalid tree object"))?;
        let filename = str::from_utf8(&b[..null_idx])?.to_owned();
        b = &b[null_idx + 1..];

        let sha = Sha(b[..20].try_into()?);
        b = &b[20..];

        let entry_type = if permissions == 0o40000 {
            TreeEntryType::Tree
        } else {
            TreeEntryType::Blob
        };

        result.push(TreeEntry {
            permissions,
            entry_type,
            sha,
            filename,
        })
    }

    Ok(result)
}

fn find_object_in_objects() {}

fn find_object_in_pack() {}

fn nearest_git_dir() -> Result<PathBuf> {
    let mut current = Some(env::current_dir()?);
    while let Some(ref mut current_path) = current {
        let possible_git_dir = current_path.join(".git");
        if possible_git_dir.exists() {
            return Ok(possible_git_dir);
        }
        current = current_path.parent().map(|cp| cp.to_owned());
    }
    bail!("could not find parent .git dir")
}
