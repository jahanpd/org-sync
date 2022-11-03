use serde::{Deserialize, Serialize};
use std::path::Path;
use bendy;

// version struct
#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    pub timestamp: String,
    pub uid: String,
    pub update: bool // if update is true then current machine needs to check and update their files
}

// TODO method to compare versions and return the most recent with update set to True
// TODO method to set boolean to False

// for convenience struct for file and path manipulation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FilePath {
    pub home: String,
    pub full: String
}

impl FilePath {
    pub fn sub_home(&self) -> String {
        self.full.replace(&self.home, "")
    }
    pub fn to_path(&self) -> &Path {
        Path::new(&self.full)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.full.replace(&self.home, "")
                 .as_bytes().to_vec()
    }
    pub fn struct_to_bytes(&self) -> Vec<u8> {
        bendy::serde::to_bytes(&self).unwrap()
    }
    pub fn struct_from_bytes(bytes: Vec<u8>) -> Option<Self> {
        bendy::serde::from_bytes::<FilePath>(&bytes).ok()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub version: String,
    pub listen: String,
    pub paths: Vec<String>
}
