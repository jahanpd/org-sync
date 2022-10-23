use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use futures::channel::{mpsc};
use futures::{prelude::*, select};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher, Config};
use notify::event::{CreateKind, EventKind, ModifyKind, DataChange, RemoveKind};
use std::path::PathBuf;

use crate::netcommand::*;

#[derive(Clone)]
pub struct WatcherSender {
    pub sender: mpsc::Sender<Command>
}



impl WatcherSender {
    pub async fn watch(mut self, paths: Vec<PathBuf>) -> notify::Result<()> {
        let (tx, rx) = std::sync::mpsc::channel();

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        for path in paths.into_iter() {
            watcher.watch(path.as_path().as_ref(), RecursiveMode::Recursive)?;
        }

        loop {
            for res in &rx {
                match res {
                    Ok(Event{kind: EventKind::Create(CreateKind::File), paths: pathlist, ..}) => {
                        println!("added: {:?}", pathlist);
                        self.create_file_hook(pathlist[0].clone());
                    },
                    Ok(Event{kind: EventKind::Remove(RemoveKind::File), paths: pathlist, ..}) => println!("removed: {:?}", pathlist),
                    Ok(Event{ kind: EventKind::Modify(ModifyKind::Data(DataChange::Content)), paths: pathlist, ..}) => println!("changed: {:?}", pathlist),
                    Ok(event) => {},
                    Err(e) => println!("watch error: {:?}", e),
                }
            }
        }
    }

    // TODO add hook for updating local state/DHT and sending message for:
    // 1. change of file
    // 2. adding or deleting files

    fn create_file_hook(&mut self, path: PathBuf) {
        self.sender.try_send(Command::EditFileAdd { path: path })
        .expect("Command receiver not to be dropped.")
    }

    fn delete_file_hook(path: PathBuf, mut sender: mpsc::Sender<Command>) {

    }

    fn change_file_hook(path: PathBuf, mut sender: mpsc::Sender<Command>) {

    }
}
