use sled;
use std::path::{PathBuf};
use crate::dht::VersionRow;
use crate::netcommand::{Command};
use futures::channel::{mpsc};

#[derive()]
pub struct Database {
    pub base: sled::Db,
    pub versions: sled::Tree,
    pub files: sled::Tree,
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
    let versions = base.open_tree("versions").unwrap();
    let files = base.open_tree("files").unwrap();

    Database {
        base: base,
        versions: versions,
        files: files
    }
}

impl Database {
    pub fn reset(mut self) -> Database {
        self.base.drop_tree("versions").unwrap();
        self.base.drop_tree("files").unwrap();
        Database {
            versions: self.base.open_tree("versions").unwrap(),
            files: self.base.open_tree("files").unwrap(),
            base: self.base
        }
    }

    pub fn add_to_version(&mut self, index: String, row: VersionRow) {
        let base_index = "versions.col.idx";
        let cols = vec!["current_version", "timestamp", "previous_version", "patch", "locations"];
        for c in cols.into_iter() {
        }
    }

    pub fn delete_from_version(&mut self, index: String) {
        let base_index = "versions.col.idx";
        let cols = vec!["current_version", "timestamp", "previous_version", "patch", "locations"];
        for c in cols.into_iter() {
        }

    }

    pub fn get_col_from_version(&mut self, send: &dyn FnMut(Command), index: String, col: String) {
        let base_index = "versions.col.idx";
        // TODO send command to get value to mimic a kademlia request

    }

}

pub enum DbRetrieve {
    StartupCheck { dht: Option<Vec<u8>>, local: Option<Vec<u8>> },
}
