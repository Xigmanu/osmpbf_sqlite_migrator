use std::{collections::HashMap, fs::File};

use osmpbfreader::{Node, OsmPbfReader, Relation, Way};

use crate::{pb::make_pb, println_staged};

pub struct OsmData {
    pub count: u64,
    pub node_map: HashMap<i64, Node>,
    pub way_map: HashMap<i64, Way>,
    pub relation_map: HashMap<i64, Relation>,
}

impl OsmData {
    pub fn from_reader(r: &mut OsmPbfReader<File>) -> Result<OsmData, Box<dyn std::error::Error>> {
        let mut n_map: HashMap<i64, Node> = HashMap::new();
        let mut w_map: HashMap<i64, Way> = HashMap::new();
        let mut r_map: HashMap<i64, Relation> = HashMap::new();

        println_staged!(1, "Reading osm pbf file...");

        let count = r.par_iter().count() as u64;
        r.rewind()?;

        println_staged!(2, format!("Processing {} objects...", count));
        let pb = make_pb(count)?;

        let mut counter: (u64, u64, u64) = (0, 0, 0);
        for blob in r.par_iter() {
            let blob = blob?;
            if blob.is_node() {
                let node = blob.node().unwrap().to_owned();
                n_map.insert(node.id.0, node);
                counter.0 += 1;
            }
            if blob.is_way() {
                let way = blob.way().unwrap().to_owned();
                w_map.insert(way.id.0, way);
                counter.1 += 1;
            }
            if blob.is_relation() {
                let relation = blob.relation().unwrap().to_owned();
                r_map.insert(relation.id.0, relation);
                counter.2 += 1;
            }

            pb.set_message(format!(
                "Collecting: nodes={}, ways={}, relations={}",
                counter.0, counter.1, counter.2
            ));
            pb.inc(1);
        }

        Ok(OsmData {
            count,
            node_map: n_map,
            way_map: w_map,
            relation_map: r_map,
        })
    }
}
