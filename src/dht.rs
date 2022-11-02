// DHT to RDB logic
use serde::{Deserialize, Serialize};
use bendy;
use bendy::encoding::{ToBencode, Error};
use libp2p::kad::{Kademlia, record::Key, Quorum, Record, QueryId};
use libp2p::kad::record::store::MemoryStore;
use sha256;
use chrono::prelude::*;

use crate::types::FilePath;

pub fn add_to_dht(kademlia: &mut Kademlia<MemoryStore>, file: String, entry: DhtEntry) {
    let key = Key::new(&file);
    let value = entry.to_bytes();
    let mut record = Record {
        key,
        value,
        publisher: None,
        expires: None,
    };
    kademlia.put_record(record, Quorum::One)
        .expect("Failed to PUT record");
}

pub fn delete_from_version(kademlia: &mut Kademlia<MemoryStore>, index: String) {
    let base_index = "versions:col:idx";
    let cols = vec!["current_version", "timestamp", "previous_version", "patch", "locations"];
    for c in cols.into_iter() {
        let key = Key::new(&base_index.replace("col", c).replace("idx", index.as_ref()));
        kademlia.remove_record(&key)
    }
}

pub fn check_col_from_version(kademlia: &mut Kademlia<MemoryStore>, index: String, col: String) -> QueryId {
    let base_index = "versions:col:idx";
    let key = Key::new(&base_index.replace("col", &col).replace("idx", index.as_ref()));
    kademlia.get_record(key, Quorum::One)
}

pub fn path_to_hash(path: FilePath) -> Option<String> {
    // let input = std::fs::File::open(path.to_path()).unwrap();
    // let reader = std::io::BufReader::new(input);
    let bytes = std::fs::read(path.to_path()).ok()?;
    let bpath = path.to_bytes();
    let comb = [bpath, bytes].concat();
    Some(sha256::digest_bytes(&comb))
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq)]
pub struct DhtEntry {
    pub current: String,
    pub previous: Option<String>,
    pub timestamp: i64,
}

impl DhtEntry {
    pub fn to_bytes(&self) -> Vec<u8> {
        bendy::serde::to_bytes(&self).unwrap()
    }
    pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        bendy::serde::from_bytes::<DhtEntry>(&bytes).ok()
    }
}
