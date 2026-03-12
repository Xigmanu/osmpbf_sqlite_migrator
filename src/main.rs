use clap::{Parser};

use crate::migrate::migrate;

mod db;
mod migrate;
mod osm;
mod pb;

#[macro_export]
macro_rules! println_staged {
    ($s:expr, $m:expr) => {
        println!("[{}/4] {}", $s, $m)
    };
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    osmpath: String,
    #[arg(short, long)]
    outpath: String
}

//TODO Use args
fn main() {
    let args = Args::parse();

    match migrate(&args.osmpath, &args.outpath) {
        Ok(()) => println!("Finished"),
        Err(e) => eprintln!("Unexpected error occurred: {e}"),
    };
}
