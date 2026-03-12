use std::sync::mpsc::Receiver;

use indicatif::ProgressBar;
use rusqlite::Connection;

const BATCH: usize = 75_000;
const NODES_INSERT_SQL: &str = "INSERT INTO nodes VALUES (?1, ?2, ?3)";
const WAYS_INSERT_SQL: &str = "INSERT INTO ways VALUES (?1, ?2)";

pub enum DbMsg {
    Node(i64, f64, f64),
    Way(i64, bool),
    End,
}

pub fn create_db(conn: &Connection) -> Result<(), rusqlite::Error> {
    _ = conn.execute_batch(
        "CREATE TABLE nodes (id INT PRIMARY KEY, lon REAL, lat REAL);
        CREATE TABLE ways (id INT PRIMARY KEY, is_closed BOOL);
        CREATE TABLE way_nodes (way_id INT REFERENCES ways(id), node_id INT REFERENCES ways(id), PRIMARY KEY(node_id, way_id));"
    )?;
    Ok(())
}

pub fn write_to_db(
    path: &str,
    rx: Receiver<DbMsg>,
    pb: &ProgressBar,
) -> Result<(), rusqlite::Error> {
    let mut conn = Connection::open(path)?;

    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

    pb.set_message("Writing to DB");

    let mut batch_count = 0usize;

    let mut tx = conn.transaction()?;
    let mut stmt_nodes = tx.prepare_cached(NODES_INSERT_SQL)?;
    let mut stmt_ways = tx.prepare_cached(WAYS_INSERT_SQL)?;

    for msg in rx {
        match msg {
            DbMsg::Node(id, lat, lon) => {
                stmt_nodes.execute((id, lat, lon))?;
            }
            DbMsg::Way(id, is_closed) => {
                stmt_ways.execute((id, is_closed))?;
            }
            DbMsg::End => break,
        }

        batch_count += 1;

        if batch_count >= BATCH {
            drop(stmt_nodes);
            drop(stmt_ways);

            tx.commit()?;

            tx = conn.transaction()?;

            stmt_nodes = tx.prepare_cached(NODES_INSERT_SQL)?;
            stmt_ways = tx.prepare_cached(WAYS_INSERT_SQL)?;

            batch_count = 0;
            pb.inc(BATCH as u64);
        }
    }

    drop(stmt_nodes);
    drop(stmt_ways);
    tx.commit()?;

    Ok(())
}
