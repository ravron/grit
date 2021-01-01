#[macro_use]
extern crate anyhow;

mod cat_file;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CatFile {
    object_name: String,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "the stupid content tracker")]
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
        Err(e) => panic!("{}", e),
    }
}
