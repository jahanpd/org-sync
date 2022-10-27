use crate::netbehaviour::{OrgBehaviour, OrgBehaviourEvent};
use crate::netcommand::{Command, CliCommand};
use crate::db::Database;
use crate::netmessages as nm;
use futures::channel::{mpsc};
use futures::{prelude::*, select};
use libp2p::{Swarm};
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
use libp2p::core::either::EitherError;
use void;

pub struct NetworkEvent {
    pub swarm: Swarm<OrgBehaviour>,
    pub watchreceiver: mpsc::Receiver<Command>,
    pub clireceiver: mpsc::Receiver<CliCommand>,
    pub commandreceiver: mpsc::Receiver<Command>,
    pub db: Database,
    pub topic: Topic,
    pub watch_pending: HashMap<String, String>
}

impl NetworkEvent {
    pub fn new (
        swarm: Swarm<OrgBehaviour>,
        watchreceiver: mpsc::Receiver<Command>,
        clireceiver: mpsc::Receiver<CliCommand>,
        commandreceiver: mpsc::Receiver<Command>,
        db: Database,
        topic: Topic,
        watch_pending: HashMap<String, String>

    ) -> Self {
        Self {
            swarm,
            watchreceiver,
            clireceiver,
            commandreceiver,
            db,
            topic,
            watch_pending
        }
    }

    pub async fn run(mut self) {
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
            }
        }
    }

    pub fn startup_check(&mut self) {
        //TODO check local files with db and DHT, alert for hook potential
    }

    pub fn sync(&mut self) {
        //TODO check local and compare DHT. Any different files download from peer and
        // update local. Perform this for each new peer.

    }

    pub fn force_push(&mut self, path: String) {
        //TODO check pending hash map for force push type
        // one of all or file
        // If all push local to DHT
        // If a file push that file to DHT
    }

    pub fn get_file_from_peer(&mut self, path: String, peerid: Vec<u8>) {
        // TODO add logic to get file from peer
    }

    pub fn patch_file_else_get_from_peer(&mut self, path: String, patch: String, peerid: Vec<u8>) {
        // TODO patch file or else
        // Check current verion is the DHT previous version and then patch
        // else get file from peer
    }

    pub fn remove_file(&mut self, path: String) {

    }

    async fn handle_watch(&mut self, command: Command) {
        match command {
            // TODO write comand hooks
            Command::EditFileAdd{path: path} => {
                // TODO check hashmap is empty
                let path_string = path.to_str().unwrap().to_string();
                if self.watch_pending.get(&path_string).is_none() {
                    println!("{:?} IS NOT registered in hashmap, sending message", path);
                    let msg = nm::Messages::Added {
                        path: path_string,
                        peerid: self.swarm.local_peer_id().to_bytes()
                    };
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(
                            self.topic.clone(),
                            bendy::serde::to_bytes(&msg).unwrap()
                        );
                    println!("Sent message that {:?} added", path)
                } else {
                    println!("{:?} IS registered in hashmap, not sending message", path);
                }
            },
            _ => {println!("unhandled")}
        }
    }

    async fn handle_cli(&mut self, command: CliCommand) {
        match command {
            // TODO write comand hooks
            CliCommand {Push: path} => {
                println!("CLI command to push");
                println!("{:?}", &path);
                self.force_push(path);
            },
            _ => {println!("unhandled")}
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            // TODO write comand hooks
            _ => {println!("unhandled")}
        }
    }


    async fn handle_swarm(&mut self, event: SwarmEvent<
            OrgBehaviourEvent,
            EitherError<EitherError<GossipsubHandlerError, std::io::Error>, void::Void>
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
                match msg {
                    nm::Messages::Added { path, peerid } => {
                        self.watch_pending.insert(path.clone(), "".into());
                        self.get_file_from_peer(path, peerid);
                    },
                    nm::Messages::Changed { path, patch, peerid } => {
                        self.watch_pending.insert(path.clone(), "".into());
                        self.patch_file_else_get_from_peer(path, patch, peerid);
                    },
                    nm::Messages::Pushed { path, peerid } => {
                        self.watch_pending.insert(path.clone(), "".into());
                        self.get_file_from_peer(path, peerid);
                    },
                    nm::Messages::Removed { path, .. } => {
                        self.watch_pending.insert(path.clone(), "".into());
                        self.remove_file(path);
                    },
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
                // TODO perform version check and sync with each new peer
            },
            SwarmEvent::Behaviour(OrgBehaviourEvent::Mdns(MdnsEvent::Expired(list))) => {
                for (peer_id, multiaddr) in list {
                    self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    self.swarm.behaviour_mut().kademlia.remove_address(&peer_id, &multiaddr);
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
                QueryResult::GetRecord(Ok(ok)) => {
                    // handle_get_request(&ok.records);
                    for PeerRecord {
                        record: Record { key, value, .. },
                        peer: peer
                    } in ok.records
                    {
                        let key_ = std::str::from_utf8(key.as_ref()).unwrap();
                        let value_ = std::str::from_utf8(&value).unwrap();
                        if key_ == "version" {
                            // TODO implement version check
                            // try and import version from file
                            // if no file, get latest version number by timestamp and save to disk
                            // if on file compare timestamp to DHT and take more recent timestamp
                            // publish version number to DHT as {peer:version}
                            println!("version: {:?} {:?}",
                                value_,
                                peer
                            )
                        }
                    }
                }
                QueryResult::GetRecord(Err(err)) => {
                    eprintln!("Failed to get record: {:?}", err);
                }
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
            }
            _ => {}
        }
    }
}
