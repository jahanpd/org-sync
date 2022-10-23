use sled;
use std::path::{PathBuf};

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
    pub fn reset(self) -> Database {
        self.base.drop_tree("versions").unwrap();
        self.base.drop_tree("files").unwrap();
        Database {
            versions: self.base.open_tree("versions").unwrap(),
            files: self.base.open_tree("files").unwrap(),
            base: self.base
        }
    }
}
