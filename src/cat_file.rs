use std::convert::TryInto;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str;
use std::{env, fmt};

use anyhow::{Context, Result};
use compress::zlib;
use hex::FromHex;

use crate::buf_utils::BufUtils;

use super::CatFile;

#[derive(Debug, Eq, PartialEq)]
pub enum Object {
    Blob(Vec<u8>),
    Tree(Vec<TreeEntry>),
    // Commit,
}

// TODO - Implement for tree and uncomment
// impl Debug for Object {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         match self {
//             Object::Blob(b) => {
//                 let mut slice_len = b.len();
//                 let mut truncated = false;
//                 if slice_len > 30 {
//                     slice_len = 30;
//                     truncated = true;
//                 }
//
//                 let mut dt = f.debug_tuple("Blob");
//
//                 if let Ok(s) = String::from_utf8(b[..slice_len].to_vec()) {
//                     dt.field(&s);
//                 } else {
//                     dt.field(&&(b[..slice_len]));
//                 }
//                 dt.finish()
//             }
//         }
//     }
// }

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
    let mut buf = buf.as_slice();

    let object_type = buf.get_str_until(b' ')?;
    let _object_size = buf.get_str_until(b'\0')?.parse::<u64>()?;

    match object_type {
        // TODO: figure out how I can go from a slice of buf to an owned Vec<u8>
        // without the cloning that occurs in to_vec (and to_owned).
        "blob" => Ok(Object::Blob(buf.to_vec())),
        "tree" => {
            println!("{:?}", buf);
            Ok(Object::Tree(parse_tree(buf)?))
        }
        _ => Err(anyhow!("unexpected object type {}", object_type)),
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Sha([u8; 20]);

impl Sha {
    fn from_hex(hex_str: impl AsRef<[u8]>) -> Result<Self> {
        Ok(Sha(<[u8; 20]>::from_hex(hex_str)?))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TreeEntryType {
    Blob,
    Tree,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TreeEntry {
    permissions: u32,
    entry_type: TreeEntryType,
    sha: Sha,
    filename: String,
}

fn parse_tree_entry(b: &mut &[u8]) -> Result<TreeEntry> {
    let permissions = b.get_str_until(b' ')?;
    let permissions = u32::from_str_radix(permissions, 8)?;

    let filename = b.get_str_until(b'\0')?.to_owned();

    // Would be cool to remove the try_into using const generics once that's a thing.
    let sha_bytes = b.get_n_exact(20)?;
    let sha = Sha(sha_bytes.try_into()?);

    let entry_type = if permissions == 0o40000 {
        TreeEntryType::Tree
    } else {
        TreeEntryType::Blob
    };

    Ok(TreeEntry {
        permissions,
        entry_type,
        sha,
        filename,
    })
}

fn parse_tree(mut b: &[u8]) -> Result<Vec<TreeEntry>> {
    let mut result = Vec::new();

    while b.len() > 0 {
        result.push(parse_tree_entry(&mut b).with_context(|| "invalid tree object")?);
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

/// NOTE: As is, these rely on being run in this repo with some initial commits as loose objects
/// (not packed objects).
#[cfg(test)]
mod test {
    use anyhow::Result;
    use indoc::indoc;

    use crate::cat_file::{cat_file, Object, Sha, TreeEntry, TreeEntryType};
    use crate::CatFile;

    #[test]
    fn blob() -> Result<()> {
        assert_eq!(
            cat_file(&CatFile { object_name: "22d351634acf3113b730bffd3638e14f62ef2af3".to_owned() })?,
            Object::Blob(indoc! {"
                # Generated by Cargo
                # will have compiled files and executables
                /target/

                # Remove Cargo.lock from gitignore if creating an executable, leave it for libraries
                # More information here https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html
                Cargo.lock

                # These are backup files generated by rustfmt
                **/*.rs.bk


                # Added by cargo

                /target
            " }.as_bytes().to_vec())
        );
        Ok(())
    }

    #[test]
    fn tree() -> Result<()> {
        assert_eq!(
            cat_file(&CatFile {
                object_name: "286597eb289fb690c2ec453b63b683b4cb1ce9ba".to_owned()
            })?,
            Object::Tree(vec![
                TreeEntry {
                    permissions: 0o100644,
                    entry_type: TreeEntryType::Blob,
                    sha: Sha::from_hex("22d351634acf3113b730bffd3638e14f62ef2af3")?,
                    filename: ".gitignore".to_owned(),
                },
                TreeEntry {
                    permissions: 0o100644,
                    entry_type: TreeEntryType::Blob,
                    sha: Sha::from_hex("123edcd7ebb3ea8086ee41077b22acc81c8db742")?,
                    filename: "Cargo.toml".to_owned(),
                },
                TreeEntry {
                    permissions: 0o040000,
                    entry_type: TreeEntryType::Tree,
                    sha: Sha::from_hex("305157a396c6858705a9cb625bab219053264ee4")?,
                    filename: "src".to_owned(),
                },
            ]),
        );
        Ok(())
    }
}
