use crate::netbehaviour::{OrgBehaviour, OrgBehaviourEvent};
use crate::netcommand::Command;
use crate::db::Database;
use futures::channel::{mpsc};
use futures::{prelude::*, select};
use libp2p::{Swarm};
use libp2p::swarm::{SwarmEvent};
use libp2p::gossipsub::error::GossipsubHandlerError;
use libp2p::mdns::{MdnsEvent};
use libp2p::gossipsub::{GossipsubEvent};
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{
    record::Key, AddProviderOk, Kademlia, KademliaEvent, PeerRecord, PutRecordOk, QueryResult,
    Quorum, Record,
};
use async_std::io;
use std::error::Error;
use libp2p::core::either::EitherError;
use void;

pub struct NetworkEvent {
    pub swarm: Swarm<OrgBehaviour>,
    pub watchreceiver: mpsc::Receiver<Command>,
    pub clireceiver: mpsc::Receiver<Command>,
    pub commandreceiver: mpsc::Receiver<Command>,
    pub db: Database
}

impl NetworkEvent {
    pub fn new (
        swarm: Swarm<OrgBehaviour>,
        watchreceiver: mpsc::Receiver<Command>,
        clireceiver: mpsc::Receiver<Command>,
        commandreceiver: mpsc::Receiver<Command>,
        db: Database
    ) -> Self {
        Self {
            swarm,
            watchreceiver,
            clireceiver,
            commandreceiver,
            db
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
        // update local

    }

    pub fn force_push(&mut self) {
        //TODO check pending hash map for force push type
        // one of all or file
        // If all push local to DHT
        // If a file push that file to DHT
    }

    async fn handle_watch(&mut self, command: Command) {
        match command {
            // TODO write comand hooks
            Command::EditFileAdd{path: path} => {
                println!("Success with {:?}", path)
            },
            _ => {println!("unhandled")}
        }
    }

    async fn handle_cli(&mut self, command: Command) {
        match command {
            // TODO write comand hooks
            Command::CliPush => {
                println!("Success with push")
            },
            _ => {println!("unhandled")}
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            // TODO write comand hooks
            Command::EditFileAdd{path: path} => {
                println!("Success with {:?}", path)
            },
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
            })) => println!(
                "Got message: {} with id: {} from peer: {:?}",
                String::from_utf8_lossy(&message.data),
                id,
                peer_id
            ),
            SwarmEvent::Behaviour(OrgBehaviourEvent::Mdns(MdnsEvent::Discovered(list))) => {
                for (peer_id, multiaddr) in list {
                    self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                    println!("Added peer: {:?}", peer_id)
                }
                // TODO perform version check and sync
            },
            SwarmEvent::Behaviour(OrgBehaviourEvent::Mdns(MdnsEvent::Expired(list))) => {
                for (peer_id, _) in list {
                    self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
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
