use indicatif::MultiProgress;
use osmpbfreader::OsmPbfReader;
use rusqlite::Connection;
use std::{collections::HashSet, fs::File, path::Path};

use tokio::{
    sync::mpsc::{self, Sender},
    task,
};

use crate::{
    db::{DbMsg, write_to_db},
    osm::OsmData,
    pb::make_pb,
    println_staged,
};

pub async fn migrate(in_path: &str, out_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", in_path);
    let f = File::open(&Path::new(in_path))?;
    let mut pbf = OsmPbfReader::new(f);

    let data = OsmData::from_reader(&mut pbf)?;

    let m = MultiProgress::new();
    let (tx, mut rx) = mpsc::channel::<DbMsg>(100_000);
    let pb0 = m.add(make_pb(data.count)?);
    let db_path = out_path.to_owned();

    let writer = task::spawn_blocking(move || {
        let mut conn = Connection::open(db_path).unwrap();
        if let Err(e) = write_to_db(&mut conn, &mut rx, &pb0) {
            eprintln!("ERROR: {}", e);
        }
    });

    migrate_initial(&data, &m, tx.clone()).await?;
    migrate_way_nodes(&data, &m, tx.clone()).await?;
    drop(tx);

    writer.await?;
    m.clear()?;

    Ok(())
}

async fn migrate_initial(
    data: &OsmData,
    m: &MultiProgress,
    tx: Sender<DbMsg>,
) -> Result<(), Box<dyn std::error::Error>> {
    println_staged!(4, "Writing...");

    let pb_nodes = m.add(make_pb(data.node_map.len() as u64).unwrap());
    let tx_nodes = tx.clone();
    let node_map = data.node_map.clone();
    let node_task = task::spawn_blocking(move || {
        pb_nodes.set_message("Preparing nodes");
        for n in node_map.values() {
            tx_nodes
                .blocking_send(DbMsg::Node(n.id.0, n.lat(), n.lon()))
                .unwrap();

            pb_nodes.inc(1);
        }
    });

    let tx_ways = tx.clone();
    let pb_ways = m.add(make_pb(data.way_map.len() as u64).unwrap());
    let way_map = data.way_map.clone();
    let way_task = task::spawn_blocking(move || {
        pb_ways.set_message("Preparing ways");
        for w in way_map.values() {
            tx_ways
                .blocking_send(DbMsg::Way(w.id.0, w.is_closed()))
                .unwrap();
            pb_ways.inc(1);
        }
    });

    node_task.await?;
    way_task.await?;

    Ok(())
}

async fn migrate_way_nodes(
    data: &OsmData,
    m: &MultiProgress,
    tx: Sender<DbMsg>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pb1 = m.add(make_pb(data.node_map.len() as u64)?);
    let data = data.way_map.clone();
    let tx_way_nodes = tx.clone();
    task::spawn_blocking(move || {
        pb1.set_message("Preparing nodes");
        for w in data.values() {
            let mut unique: HashSet<&osmpbfreader::NodeId> = HashSet::new();
            for n_id in w.nodes.iter() {
                if !unique.contains(&n_id) {
                    tx_way_nodes
                        .blocking_send(DbMsg::WayNodes(w.id.0, n_id.0))
                        .unwrap();
                }
                unique.insert(n_id);
            }
            pb1.inc(1);
        }
    })
    .await?;

    Ok(())
}
