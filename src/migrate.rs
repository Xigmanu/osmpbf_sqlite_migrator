use indicatif::MultiProgress;
use osmpbfreader::OsmPbfReader;
use rusqlite::Connection;
use std::{fs::File, path::Path, sync::mpsc, thread};

use crate::{
    db::{DbMsg, create_db, write_to_db},
    osm::collect_members,
    pb::make_pb,
    println_staged,
};

pub fn migrate(in_path: &str, out_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open(&Path::new(in_path))?;
    let mut pbf = OsmPbfReader::new(f);

    let (nodes, ways, relations, count) = collect_members(&mut pbf)?;

    println_staged!(
        3,
        format!(
            "Collected members: nodes={}, ways={}, relations={}",
            nodes.len(),
            ways.len(),
            relations.len()
        )
    );

    let conn = Connection::open(out_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    create_db(&conn)?;

    let m = MultiProgress::new();

    let pb0 = m.add(make_pb(count)?);
    let pb1 = m.add(make_pb(nodes.len() as u64)?);
    let pb2 = m.add(make_pb(ways.len() as u64)?);

    let (tx, rx) = mpsc::channel::<DbMsg>();
    let db_path = out_path.to_owned();

    let tx_nodes = tx.clone();
    let tx_ways = tx.clone();

    println_staged!(4, "Writing...");
    let writer = thread::spawn(move || write_to_db(&db_path, rx, &pb0));
    let nodes_worker = thread::spawn(move || {
        pb1.set_message("Preparing nodes");
        for n in nodes.values() {
            tx_nodes
                .send(DbMsg::Node(n.id.0, n.lat(), n.lon()))
                .unwrap();
            pb1.inc(1);
        }
    });
    let ways_worker = thread::spawn(move || {
        pb2.set_message("Preparing ways");
        for w in ways.values() {
            tx_ways.send(DbMsg::Way(w.id.0, w.is_closed())).unwrap();
            pb2.inc(1);
        }
    });

    nodes_worker.join().unwrap();
    ways_worker.join().unwrap();

    tx.send(DbMsg::End).unwrap();
    writer.join().unwrap()?;

    m.clear().unwrap();

    Ok(())
}
