#[macro_use]
extern crate anyhow;

pub(crate) mod buf_utils;
mod cat_file;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CatFile {
    object_name: String,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "our stupid content tracker")]
enum Git {
    // #[structopt(flatten)]
    CatFile(CatFile),
    // CatFile {
    //     object_name: String,
    // },
}

fn main() {
    println!("Hello, world!");
    let g: Git = Git::from_args();
    println!("{:?}", g);
    let result = match g {
        Git::CatFile(cf) => cat_file::cat_file(&cf),
    };
    match result {
        Ok(obj) => println!("{:?}", obj),
        Err(e) => panic!("{:?}", e),
    }
}
