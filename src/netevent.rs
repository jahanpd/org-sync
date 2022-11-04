use crate::netbehaviour::{OrgBehaviour, OrgBehaviourEvent};
use crate::netcommand::{Command, CliCommand};
use crate::db::*;
use crate::netmessages as nm;
use crate::types::{FilePath};
use crate::dht::*;
use walkdir::{WalkDir};
use std::path::{PathBuf};
use chrono::prelude::*;
use futures::channel::{mpsc};
use futures::{prelude::*, select};
use libp2p::{Swarm, PeerId};
use libp2p::swarm::{SwarmEvent};
use libp2p::gossipsub::error::GossipsubHandlerError;
use libp2p::mdns::{MdnsEvent};
use libp2p::gossipsub::{GossipsubEvent, IdentTopic as Topic};
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{
    record::Key, AddProviderOk, Kademlia, KademliaEvent, PeerRecord, PutRecordOk, QueryResult,
    Quorum, Record,
};
use std::collections::HashMap;
use async_std::io;
use std::error::Error;
use std::slice::Windows;
use libp2p::core::either::EitherError;
use void;

// This is a behemoth of a data structure
// Important lifecycles to understand
// 1. detect add/change/delete file (watch reciever) -> send message to peers -> update DHT
// 2. get message of a/c/d -> note in hashmap watch_pending -> response request file
// 3. to check dht/db entry in sync -> get local and store in db_pending hash with retrieve enum -> call kademlia fn -> check in swarm event
pub struct NetworkEvent {
    pub swarm: Swarm<OrgBehaviour>,
    pub watchreceiver: mpsc::Receiver<Command>,
    pub clireceiver: mpsc::Receiver<CliCommand>,
    pub commandreceiver: mpsc::Receiver<Command>,
    pub commandsender: mpsc::Sender<Command>,
    pub db: Database,
    pub topic: Topic,
    pub transfer_pending: HashMap<String, String>,
    pub key_2_filepath: HashMap<Vec<u8>, FilePath>,
    pub dirs: Vec<PathBuf>,
}

impl NetworkEvent {
    pub fn new (
        swarm: Swarm<OrgBehaviour>,
        watchreceiver: mpsc::Receiver<Command>,
        clireceiver: mpsc::Receiver<CliCommand>,
        commandreceiver: mpsc::Receiver<Command>,
        commandsender: mpsc::Sender<Command>,
        db: Database,
        topic: Topic,
        transfer_pending: HashMap<String, String>,
        key_2_filepath: HashMap<Vec<u8>, FilePath>
    ) -> Self {
        Self {
            swarm,
            watchreceiver,
            clireceiver,
            commandreceiver,
            commandsender,
            db,
            topic,
            transfer_pending,
            key_2_filepath,
            dirs: vec![]
        }
    }

    pub async fn run(mut self)
    {
        loop {
            select! {
                event = self.swarm.select_next_some() => {
                    self.handle_swarm(event).await;
                },
                watch = self.watchreceiver.select_next_some() => {
                    self.handle_watch(watch).await;
                },
                cli = self.clireceiver.select_next_some() => {
                    self.handle_cli(cli).await;
                }
                command = self.commandreceiver.select_next_some() => {
                    self.handle_command(command).await;
                }
            }
        }
    }

    fn check_local_db(&mut self) {
        // checks local files against local db and loads into db_loaded hashmap
        let files = self.get_files_from_dirs(self.dirs.clone());
        for file in files {
            let hash = path_to_hash(file.clone());
            // sha256 hash is the version UID
            // _ = self.db.insert(key, val);
       }

    }

    fn check_dht_vs_local(&mut self) {
        // checks DHT and loads into db_loaded hashmap
        let files = self.get_files_from_dirs(self.dirs.clone());
        // get peers most recent timestamp from
    }

    fn update_filepaths(&mut self) {
        let files = self.get_files_from_dirs(self.dirs.clone());
        for file in files {
            self.key_2_filepath.insert(file.to_bytes(), file);
        }
    }

    pub fn startup_check(&mut self) {

    }

    pub fn add_peer_check(&mut self) {
        self.update_filepaths();
        let files = self.get_files_from_dirs(self.dirs.clone());
        // do get request for each file, will sync local and dht db
        println!("Number of peers {:?}", self.swarm.connected_peers().collect::<Vec<&PeerId>>().len());
        for file in files {
            // this message runs GET on filepath in message
            self.swarm.behaviour_mut().kademlia.get_record(
                Key::new(&file.to_bytes()), Quorum::One);
            // send startup filecheck message
            if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(
                self.topic.clone(),
                nm::to_bytes(nm::Messages::FileCheck {
                    filepath: file.struct_to_bytes(),
                    timestamp: chrono::Utc::now().timestamp()
                })
            ) {
                println!("Publish error: {:?}", e);
            };
        }
        // Ensure base folders in config are available in home
    }

    /// FILE TRANSFER REQUEST RESPONSE
    fn request_file(&mut self, peer: &PeerId, key:&Vec<u8>, entry: DhtEntry) {
        let fp = self.key_2_filepath.get(key).unwrap();
        println!("requesting {:?} from {:?}", fp.sub_home() , peer)
    }

    // KADEMLIA DHT request helpers
    /// GET
    fn get_record(
        &mut self,
        results: Result<libp2p::kad::GetRecordOk, libp2p::kad::GetRecordError>
    ) {
        //
        match results {
            Ok(libp2p::kad::GetRecordOk {records, ..}) => {
                let recvec: Vec<PeerRecord> = records.clone().into_iter().collect();
                println!("Records collected: {:?}", recvec.len());
                let all_same: bool = recvec.windows(2).all(|w| w[0] == w[1]);
                let max_record = recvec.iter().max_by_key(
                    |w| DhtEntry::from_bytes(w.record.value.clone()).unwrap().timestamp
                );
                if !all_same {} // TODO send message to audit key providers
                match max_record {
                    Some(PeerRecord{record, peer}) => {
                    let dht_entry = DhtEntry::from_bytes(record.value.clone()).unwrap();
                    let dht_hash = dht_entry.current.clone();
                    let mut dht_time = dht_entry.timestamp;
                    let key = record.key.to_vec();
                    let local_retrieve = self.db.get(key.clone());
                    let local_fp = FilePath::new_from_key(key.to_vec());
                    let current_hash = match path_to_hash(local_fp.clone()) {
                        Some(hash) => hash,
                        None => {
                            "no_file".into()
                        }
                    };
                    match local_retrieve {
                        Some(entry) => {
                            let local_entry = DhtEntry::from_bytes(entry.to_vec()).unwrap();
                            let mut local_hash = local_entry.current;
                            let mut local_time = local_entry.timestamp;
                            // this check is a failsafe to ensure local db is up to date
                            // this shouldn't happen, but good to handle it in case
                            if (&current_hash != &local_hash) &
                                (&current_hash == &dht_hash) &
                                (&current_hash != "no_file"){
                                // update local database
                                local_time = chrono::Utc::now().timestamp();
                                dht_time = chrono::Utc::now().timestamp();
                                let new_entry = DhtEntry {
                                    current: current_hash.clone(),
                                    previous: Some(local_hash.clone()),
                                    timestamp: local_time
                                };
                                local_hash = current_hash.clone();
                                self.db.insert(key.clone(), new_entry.clone().to_bytes());
                                add_to_dht(
                                    &mut self.swarm.behaviour_mut().kademlia,
                                    local_fp.sub_home(),
                                    new_entry);
                                println!("Updated local and dht as local db not up to date");
                            }
                            if &current_hash == &dht_hash {
                                // update timestamp with larger timestamp
                                if &local_time != &dht_time {
                                let max_time = std::cmp::max(local_time, dht_time);
                                // push entries to dht and local with updates time
                                let new_entry = DhtEntry {
                                    current: current_hash.clone(),
                                    previous: dht_entry.previous,
                                    timestamp: max_time
                                };
                                self.db.insert(key.clone(), new_entry.clone().to_bytes());
                                add_to_dht(
                                    &mut self.swarm.behaviour_mut().kademlia,
                                    local_fp.sub_home(),
                                    new_entry);
                                println!("Updated local and dht with differing timestamps and equal hashes");
                                }
                            } else {
                                // current hash is different to dht
                                // compare timestamps and if dht > local
                                if dht_time > local_time {
                                    // trigger file transfer
                                    // update entry in local is handled on response success
                                    self.request_file(&peer.unwrap(), &key, dht_entry.clone());
                                    println!("Reqeusting file as hashes different and DHT more recent");

                                } else if local_time > dht_time {
                                    // push local time to dht and send message for file tf
                                    let new_entry = DhtEntry {
                                        current: current_hash.clone(),
                                        previous: Some(dht_entry.clone().current),
                                        timestamp: local_time
                                    };
                                    add_to_dht(
                                        &mut self.swarm.behaviour_mut().kademlia,
                                        local_fp.sub_home(),
                                        new_entry);
                                        println!("Updated local and dht with differing timestamps and equal hashes");

                                } else {
                                    // TODO decide how to handle equal timestamps but differing
                                    // hashes. For now...
                                    // when timestamps are equal do nothing
                                    // this case shouldn't really occur...
                                    // for testing panic
                                    panic!("Differing hashes (dht vs local) but same timestamp")
                                }
                                if &current_hash == "no_file" {
                                    self.request_file(&peer.unwrap(), &key, dht_entry.clone());
                                    println!("Requesting file as no local file but dht and local db entries");
                                }
                            }
                        },
                        None => {
                            // TODO handle no local db entry for key
                            // this is in the setting of a DHT entry
                            // check if local file present
                            // if not then retrieve it and add DHT entry
                            // to local, If there is a local file then overwrite
                            // with DHT information and fetch file from peer

                        }
                    }
                    }
                    None => {
                        // TODO handle no responses from peers.
                        // should be the same as not found
                        // shouldn't really happen as would be a NotFound
                        // or QuorumFailed error
                    }
                };
            }
            Err(libp2p::kad::GetRecordError::NotFound {key, closest_peers}) => {
                println!("failed due to not found");
                // check local file present
                let local_fp = FilePath::new_from_key(key.to_vec());
                match path_to_hash(local_fp.clone()) {
                    Some(current_hash) => {
                        match self.db.get(key.to_vec().clone()) {
                            Some(entry) => {
                                let local_entry = DhtEntry::from_bytes(entry.to_vec()).unwrap();
                                let new_entry = DhtEntry {
                                    current: current_hash.clone(),
                                    previous: local_entry.previous,
                                    timestamp: local_entry.timestamp
                                };
                                let local_hash = current_hash.clone();
                                self.db.insert(key.to_vec(), new_entry.clone().to_bytes());
                                add_to_dht(
                                    &mut self.swarm.behaviour_mut().kademlia,
                                    local_fp.sub_home(),
                                    new_entry);
                                println!("Added to dht and local from local entry but no dht entry")
                            },
                            None => {
                                // no local db entry and file on disk
                                let new_entry = DhtEntry {
                                    current: current_hash.clone(),
                                    previous: None,
                                    timestamp: Utc::now().timestamp()
                                };
                                let local_hash = current_hash.clone();
                                self.db.insert(key.to_vec(), new_entry.clone().to_bytes());
                                add_to_dht(
                                    &mut self.swarm.behaviour_mut().kademlia,
                                    local_fp.sub_home(),
                                    new_entry);
                                println!("Added to dht and local from no entry in dht or local")
                            }
                        }
                    },
                    None => {
                        println!("No local file, and no record on DHT")
                    }
                };
            }
            Err(libp2p::kad::GetRecordError::QuorumFailed {key, records, quorum}) => {
                println!("failed due to quorum - should not happen with Quorum::One")
            }
            Err(libp2p::kad::GetRecordError::Timeout {key, records, quorum}) => {
                println!("failed due to timeout")
            }
        }
    }

    pub fn get_files_from_dirs(&mut self, dirs: Vec<PathBuf>) -> Vec<FilePath> {
        let home = std::env::var("HOME").unwrap();
        let mut paths: Vec<_> = vec![];
        for path in dirs.into_iter() {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
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
        paths
    }

    pub fn sync(&mut self) {
        //TODO check local and compare DHT. Any different files download from peer and
        // update local. Perform this for each new peer.

    }

    async fn handle_watch(&mut self, command: Command) {
        match command {
            // TODO write comand hooks
            Command::EditFileAdd{path: path} => {
                let path_string = path.to_str().unwrap().to_string();
            },
            _ => {println!("unhandled")}
        }
    }

    async fn handle_cli(&mut self, command: CliCommand) {
        match command {
            // TODO write comand hooks
            CliCommand {Push: path} => {
                println!("CLI command to push");
                self.add_peer_check();
                println!("{:?}", &path);
            },
            _ => {println!("unhandled")}
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            // TODO write comand hooks
            Command::NewPeer => {
                println!("Entered command newpeer");
                self.add_peer_check();
            },
            _ => {println!("unhandled")}
        }
    }

    async fn handle_swarm(&mut self, event: SwarmEvent<
            OrgBehaviourEvent,
            EitherError<EitherError<EitherError<GossipsubHandlerError, std::io::Error>, void::Void>, libp2p::ping::Failure>
            >) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {:?}", address);
            },
            SwarmEvent::Behaviour(OrgBehaviourEvent::Gossipsub(GossipsubEvent::Message {
                propagation_source: peer_id,
                message_id: id,
                message,
            })) => {
                let msg: nm::Messages = bendy::serde::from_bytes::<nm::Messages>(&message.data).unwrap();
                println!(
                    "Got message: {:?} with id: {} from peer: {:?}",
                    &msg,
                    id,
                    peer_id
                );
                // TODO add message logic
                match msg {
                    nm::Messages::Added { path, peerid } => {
                        // add file to transfer pending
                        // trigger file transfer request/response
                        // file transferred and saved to disk
                        // watcher triggers for new file
                        // handle_watcher checks transfer pending and removes entry (msg not sent)
                        // update entry to local / sync with dht
                    },
                    nm::Messages::Changed { path, patch, peerid } => {
                        // check previous hash is equal to current hash on disk
                        // add file to transfer pending
                        // if true apply patch and save to disk
                        // if false trigger file transfer
                        // watcher triggers new file or change
                        // handler check transfer pending and removes entry (msg not sent)
                        // update entry to local / sync with dht
                    },
                    nm::Messages::Pushed { path, peerid } => {
                        // this is a force push of state from peer
                        // add all files in pathlist to transfer pending
                        // request all files from peer
                        // watcher triggered and removes transfer pending
                    },
                    nm::Messages::Removed { path, .. } => {
                        // add file to transfer pending
                        // remove file
                        // watcher notes removed file and takes of transfer pending
                    },
                    nm::Messages::FileCheck { filepath, timestamp } => {
                        let fp = FilePath::struct_from_bytes(filepath).unwrap();
                        let key = Key::new(&fp.to_bytes());
                        self.swarm.behaviour_mut().kademlia.get_record(
                            key, Quorum::One);
                        println!("FileCheck msg for {:?}", fp.sub_home())
                    }
                    _ => {}
                }
            },
            SwarmEvent::Behaviour(OrgBehaviourEvent::Mdns(MdnsEvent::Discovered(list))) => {
                println!("Found peer(s)");
                for (peer_id, multiaddr) in list {
                    self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                    println!("Added peer: {:?}", peer_id)
                }
                self.commandsender.try_send(
                    Command::NewPeer
                ).expect("Command reciever not to be dropped");
            },
            SwarmEvent::Behaviour(OrgBehaviourEvent::Mdns(MdnsEvent::Expired(list))) => {
                for (peer_id, multiaddr) in list {
                    // self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    // self.swarm.behaviour_mut().kademlia.remove_address(&peer_id, &multiaddr);
                    println!("Removed peer: {:?}", peer_id)
                }
            },
            SwarmEvent::Behaviour(OrgBehaviourEvent::Kademlia(KademliaEvent::OutboundQueryCompleted { result, ..})) => {
            match result {
                QueryResult::GetProviders(Ok(ok)) => {
                    for peer in ok.providers {
                        println!(
                            "Peer {:?} provides key {:?}",
                            peer,
                            std::str::from_utf8(ok.key.as_ref()).unwrap()
                        );
                    }
                }
                QueryResult::GetProviders(Err(err)) => {
                    eprintln!("Failed to get providers: {:?}", err);
                }
                QueryResult::GetRecord(result) => {
                    self.get_record(result);}
                QueryResult::PutRecord(Ok(PutRecordOk { key })) => {
                    println!(
                        "Successfully put record {:?}",
                        std::str::from_utf8(key.as_ref()).unwrap()
                    );
                }
                QueryResult::PutRecord(Err(err)) => {
                    eprintln!("Failed to put record: {:?}", err);
                }
                QueryResult::StartProviding(Ok(AddProviderOk { key })) => {
                    println!(
                        "Successfully put provider record {:?}",
                        std::str::from_utf8(key.as_ref()).unwrap()
                    );
                }
                QueryResult::StartProviding(Err(err)) => {
                    eprintln!("Failed to put provider record: {:?}", err);
                }
                _ => {}
                }
            },
            // TODO include request response behaviour
            // SwarmEvent::Behaviour(OrgBehaviourEvent::RequestResponse)
            _ => {}
        }
    }
}
