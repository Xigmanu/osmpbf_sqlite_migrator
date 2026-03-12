use std::{collections::HashMap, fs::File};

use osmpbfreader::{Node, OsmPbfReader, Relation, Way};

use crate::{pb::make_pb, println_staged};

pub fn collect_members(
    reader: &mut OsmPbfReader<File>,
) -> Result<
    (
        HashMap<i64, Node>,
        HashMap<i64, Way>,
        HashMap<i64, Relation>,
        u64,
    ),
    Box<dyn std::error::Error>,
> {
    let mut nodes: HashMap<i64, Node> = HashMap::new();
    let mut ways: HashMap<i64, Way> = HashMap::new();
    let mut relations: HashMap<i64, Relation> = HashMap::new();

    println_staged!(1, "Reading osm pbf file...");

    let count = reader.par_iter().count() as u64;
    reader.rewind()?;

    println_staged!(2, format!("Processing {} objects...", count));
    let pb = make_pb(count)?;

    let mut counter: (u64, u64, u64) = (0, 0, 0);
    for blob in reader.par_iter() {
        let blob = blob?;
        if blob.is_node() {
            let node = blob.node().unwrap().to_owned();
            nodes.insert(node.id.0, node);
            counter.0 += 1;
        }
        if blob.is_way() {
            let way = blob.way().unwrap().to_owned();
            ways.insert(way.id.0, way);
            counter.1 += 1;
        }
        if blob.is_relation() {
            let relation = blob.relation().unwrap().to_owned();
            relations.insert(relation.id.0, relation);
            counter.2 += 1;
        }

        pb.set_message(format!(
            "Collecting: nodes={}, ways={}, relations={}",
            counter.0, counter.1, counter.2
        ));
        pb.inc(1);
    }

    Ok((nodes, ways, relations, count))
}
