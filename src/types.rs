use serde::{Deserialize, Serialize};


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
#[derive(Debug)]
pub struct FilePath {
    pub home: String,
    pub full: String
}

impl FilePath {
    pub fn sub_home(&self) -> String {
        self.full.replace(&self.home, "")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub version: String,
    pub listen: String,
    pub paths: Vec<String>
}
