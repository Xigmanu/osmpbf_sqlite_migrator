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

//TODO Use args
fn main() {
    println!("### Beginning migration ###");
    match migrate("assets/puerto-rico-260219.osm.pbf", "out/result.db") {
        Ok(()) => println!("### FINISHED ###"),
        Err(e) => eprintln!("Unexpected error occurred: {e}"),
    };
}
