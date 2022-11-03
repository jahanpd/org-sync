use serde::{Deserialize, Serialize};
use bendy;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Messages {
    Pushed { path: String, peerid: Vec<u8> },
    Added { path: String, peerid: Vec<u8> },
    Changed { path: String, patch: String, peerid: Vec<u8> },
    Removed { path: String, peerid: Vec<u8> },
    FileCheck { filepath: Vec<u8>, timestamp: i64 },
    FileUpdate { path: String, current: String, previous: String, timestamp: i64, data: Vec<u8> },
}

// test
pub fn test() {
let test = Messages::Added { path: "org".into(), peerid: vec![0, 1,2,3]};
let bencode = bendy::serde::to_bytes(&test).unwrap();
let comm: Messages = bendy::serde::from_bytes::<Messages>(&bencode).unwrap();
dbg!(comm);
}

pub fn to_bytes(msg: Messages) -> Vec<u8> {
    bendy::serde::to_bytes(&msg).unwrap()
}

pub fn from_bytes(msg: Vec<u8>) -> Option<Messages>{
    bendy::serde::from_bytes::<Messages>(&msg).ok()
}
