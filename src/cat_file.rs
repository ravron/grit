use super::CatFile;
use anyhow::{Error, Result};
use compress::zlib;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str;

#[derive(Debug)]
pub enum Object {
    Blob(Vec<u8>),
    // Tree,
    // Commit,
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
    let mut buf = vec![];
    zlib::Decoder::new(stream).read_to_end(&mut buf)?;

    let mut header_then_body = buf.splitn(2, |b| *b == b'\0');
    let header = header_then_body
        .next()
        .ok_or(anyhow!("cannot locate header"))?;

    let header = str::from_utf8(&header)?;
    let mut header_split = header.split(' ');
    let type_str = header_split
        .next()
        .ok_or(anyhow!("invalid header {:?}", header))?;
    let len = header_split
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
        _ => Err(anyhow!("unexpected object type {}", header)),
    }
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
