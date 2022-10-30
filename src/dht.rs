// DHT to RDB logic
use serde::{Deserialize, Serialize};
use bendy;
use bendy::encoding::{ToBencode, Error};
use libp2p::kad::{Kademlia, record::Key, Quorum, Record};
use libp2p::kad::record::store::MemoryStore;
use sha256;

use crate::types::FilePath;

pub fn add_to_version(kademlia: &mut Kademlia<MemoryStore>, index: String, row: VersionRow) {
    let base_index = "versions.col.idx";
    let cols = vec!["current_version", "timestamp", "previous_version", "patch", "locations"];
    for c in cols.into_iter() {
        let key = Key::new(&base_index.replace("col", c).replace("idx", index.as_ref()));
        let value = row.get_bytes(c).unwrap_or(vec![]);
        let mut record = Record {
            key,
            value,
            publisher: None,
            expires: None,
        };
        kademlia.put_record(record, Quorum::One)
            .expect("Failed to PUT record");
    }
}

pub fn delete_from_version(kademlia: &mut Kademlia<MemoryStore>, index: String) {
    let base_index = "versions.col.idx";
    let cols = vec!["current_version", "timestamp", "previous_version", "patch", "locations"];
    for c in cols.into_iter() {
        let key = Key::new(&base_index.replace("col", c).replace("idx", index.as_ref()));
        kademlia.remove_record(&key)
    }
}

pub fn get_col_from_version(kademlia: &mut Kademlia<MemoryStore>, index: String, col: String) {
    let base_index = "versions.col.idx";
    let key = Key::new(&base_index.replace("col", &col).replace("idx", index.as_ref()));
    kademlia.get_record(key, Quorum::One);
}

pub fn path_to_hash(path: FilePath) -> String {
    // let input = std::fs::File::open(path.to_path()).unwrap();
    // let reader = std::io::BufReader::new(input);
    let bytes = std::fs::read(path.to_path()).unwrap();
    let bpath = path.to_bytes();
    let comb = [bpath, bytes].concat();
    sha256::digest_bytes(&comb)
}

pub struct VersionRow {
    current_version: String,
    timestamp: String,
    previous_version: String,
    patch: String,
    locations: Vec<String>
}

impl VersionRow {
    pub fn get_bytes(&self, field_string: &str) -> Result<Vec<u8>, String> {
        match field_string {
            "current_version" => Ok(self.current_version.as_bytes().to_vec()),
            "timestamp" => Ok(self.timestamp.as_bytes().to_vec()),
            "previous_version" => Ok(self.previous_version.as_bytes().to_vec()),
            "patch" => Ok(self.patch.as_bytes().to_vec()),
            "locations" => Ok(self.locations.to_bencode().unwrap()), // TODO convert to bytes
            _ => Err(format!("invalid field name to get '{}'", field_string))
        }
    }
}
