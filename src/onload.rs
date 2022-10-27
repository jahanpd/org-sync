use clap::Parser;
use walkdir::{WalkDir};
use crate::types::{Config, FilePath};
use std::path::{PathBuf};

// function to handle arg iunput and load config to paths
pub fn onload(args: Args) -> (Config, Vec<PathBuf>, Vec<FilePath>) {
    // check for config file and import
    // config file is a json object of type {"version": path, "listen": address, paths: ["path"]}
    let config: Config;
    let home = std::env::var("HOME").unwrap();
    if let Some(config_path) =  &args.config {
        println!("Config path is {:?}", config_path);
        // parse config_path to string
        let path: std::path::PathBuf = config_path.parse().expect("User to provide valid path.");
        let config_string = std::fs::read_to_string(&path).unwrap();
        config = serde_json::from_str(&config_string)
            .unwrap_or(
                Config{
                    version: "~/.version".to_string(),
                    listen: "127.0.0.1/tcp/6234".to_string(),
                    paths: vec!["~/org-sync-test/".to_string()]
                });
    } else {
        config = Config{
            version: "~/.version".to_string(),
            listen: "127.0.0.1/tcp/6234".to_string(),
            // paths: vec!["~/org/".to_string(), "~/org-roam/".to_string()]
            paths: vec!["~/org-sync-test/".to_string()]
        };
    }
    let mut dirs: Vec<_> = vec![];
    for path in config.clone().paths.into_iter() {
        let p = PathBuf::from(path.replace("~", &home).as_str());
        if p.is_dir() {dirs.push(p)}
    }
    let mut paths: Vec<_> = vec![];
    for path in config.clone().paths.into_iter() {
        for entry in WalkDir::new(path.replace("~", &home)).into_iter().filter_map(|e| e.ok()) {
            if !entry.path().is_dir() {
                paths.push(
                    FilePath {
                        home: home.clone(),
                        full: entry.into_path().into_os_string().into_string().unwrap()
                    }
                );
            }
            // println!("{}", entry.path().display());
        }
    };
    // println!("Files list: {:?}", &paths.into_iter().map(|x| x.sub_home()).collect::<Vec<String>>());
    (config, dirs, paths)
}


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Config Location
    #[arg(short, long)]
    config: Option<String>,

    /// Force the state of orgfiles on this machine to be the current state.
    /// Will rewrite local db and then push to DHT
    #[clap(subcommand)]
    pub choice: CliArgs,
}

#[derive(Debug, Parser)]
pub enum CliArgs {
    Query {
        #[clap(subcommand)]
        push: PushPath,
    },
    Serve {
    },
}

#[derive(Debug, Parser)]
pub enum PushPath {
    Push {
        #[clap(long) ]
        path: Option<String>,
    }
}
