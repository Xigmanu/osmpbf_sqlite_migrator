use clap::Parser;

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
    pbf: String,
    #[arg(short, long)]
    out: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let w = migrate(&args.pbf, &args.out).await;
    match w {
        Ok(_) => println!("Finished"),
        Err(e) => eprintln!("Unexpected error occurred: {}", e),
    }
}
