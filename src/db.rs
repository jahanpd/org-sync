use sled;
use std::path::{PathBuf};
use crate::netcommand::{Command};
use futures::channel::{mpsc};

#[derive()]
pub struct Database {
    pub base: sled::Db,
}

/// Generate new db
pub fn new() -> Database {
    let home = std::env::var("HOME").unwrap();
    let pathstr = "~/.config/org-sync/";
    let mut dbpath = PathBuf::from(pathstr.replace("~", &home).as_str());
    if !dbpath.is_dir(){
        std::fs::create_dir_all(dbpath.as_path()).expect("Unable to make db path");
        println!("Path to db created {:?}", dbpath.as_path())
    }
    dbpath.push("db");
    let base = sled::open(dbpath.as_path()).unwrap();

    Database {
        base: base,
    }
}

impl Database {
    pub fn insert(&mut self, key: Vec<u8>, val: Vec<u8>){
         _ = self.base.insert(key, val);
    }
    pub fn get(&mut self, key: Vec<u8>) -> Option<sled::IVec>{
        match self.base.get(key) {
            Ok(result) => result,
            _ => None
        }
    }
}
