use std::{fs::File, time::Instant};

use osmpbfreader::{groups, primitive_block_from_blob};
use rusqlite::Connection;

macro_rules! log_info {
    ($t:expr, $g:expr, $b:expr, $n:expr, $w:expr, $r:expr) => {
        println!(
            "[{:?}ms]\tGroup {} - processed block {}: num_nodes={}, num_ways={}, num_relations={}",
            $t, $g, $b, $n, $w, $r
        )
    };
}

fn create_db(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    _ = conn.execute_batch(
        "CREATE TABLE nodes (id NUMBER PRIMARY KEY, lat REAL, lon REAL);
        CREATE TABLE ways (id NUMBER PRIMARY KEY);
        CREATE TABLE relations (id NUMBER PRIMARY KEY)",
    );
    Ok(())
}

fn do_work(osm_path: &str, out_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open(osm_path)?;
    let mut pbf = osmpbfreader::OsmPbfReader::new(f);
    let conn = Connection::open(out_path)?;

    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    create_db(&conn)?;

    let mut stmt_node = conn.prepare_cached("INSERT OR IGNORE INTO nodes VALUES (?1, ?2, ?3)")?;
    let mut stmt_way = conn.prepare_cached("INSERT OR IGNORE INTO ways VALUES (?1)")?;
    let mut stmt_rel = conn.prepare_cached("INSERT OR IGNORE INTO relations VALUES (?1)")?;
    let now = Instant::now();

    let mut block_idx = 0usize;
    for block in pbf.blobs().map(|b| primitive_block_from_blob(&b.unwrap())) {
        let block = block.unwrap();
        let mut group_idx = 0usize;
        for group in block.primitivegroup.iter() {
            let mut num_nodes = 0usize;
            let mut num_ways = 0usize;
            let mut num_relations = 0usize;

            for node in groups::dense_nodes(&group, &block) {
                stmt_node.execute((node.id.0, node.lat(), node.lon()))?;
                num_nodes += 1;
            }
            for way in groups::ways(&group, &block) {
                stmt_way.execute([way.id.0])?;
                num_ways += 1;
            }

            for rel in groups::relations(&group, &block) {
                stmt_rel.execute([rel.id.0])?;
                num_relations += 1;
            }

            log_info!(
                now.elapsed().as_millis(),
                group_idx,
                block_idx,
                num_nodes,
                num_ways,
                num_relations
            );
            group_idx += 1;
        }
        block_idx += 1;
    }

    Ok(())
}

fn main() {
    match do_work("assets/puerto-rico-260219.osm.pbf", "out/result.db") {
        Ok(()) => println!("DONE"),
        Err(e) => eprintln!("You are a Failure: {e}"),
    };
}
