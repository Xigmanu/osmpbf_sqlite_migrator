use indicatif::ProgressBar;
use rusqlite::Connection;
use tokio::sync::mpsc::Receiver;

const NODES_INSERT_SQL: &str = "INSERT INTO nodes VALUES (?1, ?2, ?3)";
const WAYS_INSERT_SQL: &str = "INSERT INTO ways VALUES (?1, ?2)";
const WAY_NODES_INSERT_SQL: &str = "INSERT INTO way_nodes VALUES (?1, ?2)";

pub enum DbMsg {
    Node(i64, f64, f64),
    Way(i64, bool),
    WayNodes(i64, i64),
}

pub fn create_db(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE nodes (id INT PRIMARY KEY, lon REAL, lat REAL);
        CREATE TABLE ways (id INT PRIMARY KEY, is_closed BOOL);
        CREATE TABLE way_nodes (way_id INT REFERENCES ways(id), node_id INT REFERENCES ways(id), PRIMARY KEY(node_id, way_id));"
    )
}

pub fn write_to_db(
    conn: &mut Connection,
    rx: &mut Receiver<DbMsg>,
    pb: &ProgressBar,
) -> Result<(), rusqlite::Error> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    create_db(&conn)?;

    pb.set_message("Writing to DB");

    let tx = conn.transaction()?;
    let mut stmt_nodes = tx.prepare_cached(NODES_INSERT_SQL)?;
    let mut stmt_ways = tx.prepare_cached(WAYS_INSERT_SQL)?;
    let mut stmt_way_nodes = tx.prepare_cached(WAY_NODES_INSERT_SQL)?;

    while let Some(msg) = rx.blocking_recv() {
        match msg {
            DbMsg::Node(id, lat, lon) => {
                stmt_nodes.execute((id, lat, lon))?;
            }
            DbMsg::Way(id, is_closed) => {
                stmt_ways.execute((id, is_closed))?;
            }
            DbMsg::WayNodes(way_id, node_id) => {
                stmt_way_nodes.execute((way_id, node_id))?;
            }
        }
        pb.inc(1);
    }

    drop(stmt_nodes);
    drop(stmt_ways);
    drop(stmt_way_nodes);
    tx.commit()?;

    Ok(())
}
