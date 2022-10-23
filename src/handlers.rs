use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{Kademlia, PeerRecord, record::Key, Quorum};

// function to check version
pub fn trigger_check_version(kademlia: &mut Kademlia<MemoryStore>) {
    let key = Key::new(&"version".to_string());
    kademlia.get_record(key, Quorum::One);
    // the above will trigger a kademilia event
    // handle the check version logic in the get_record event
}

pub fn handle_get_request(records: &Vec<PeerRecord>) {
    println!("handling")
}
