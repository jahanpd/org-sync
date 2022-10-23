use async_std::io;
use std::io::prelude::*;
use async_std::task::spawn;
use clap::Parser;
use env_logger::{Builder, Env};
use futures::{prelude::*, select};
use libp2p::gossipsub::{GossipsubEvent};
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{
    record::Key, AddProviderOk, Kademlia, KademliaEvent, PeerRecord, PutRecordOk, QueryResult,
    Quorum, Record,
};
use futures::channel::{mpsc};
use libp2p::mdns::{MdnsEvent};
use libp2p::{swarm::{SwarmEvent}};
use std::error::Error;
use std::{
    net::{TcpListener, TcpStream},
    io::{prelude::*, BufReader},
};


mod handlers;
use handlers::*;

mod types;

mod onload;
use onload::*;

mod watcher;
use watcher::*;

mod netbase;
mod netbehaviour;
use netbehaviour::*;
mod netclient;
mod netcommand;
use netcommand::*;
mod netexchange;
mod netevent;
use netevent::NetworkEvent;

mod db;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    match args.choice {
        CliCommand::Serve { } => {
            let (config, dirs, paths) = onload(args);
            println!("Dir list: {:?}", &dirs);
            println!("Files list: {:?}", &paths.into_iter().map(|x| x.sub_home()).collect::<Vec<String>>());

            // get network objects
            let (mut watcher_sender,
                 mut cli_sender,
                 mut command_sender,
                 mut netevent
            ): (mpsc::Sender<Command>,
                mpsc::Sender<Command>,
                mpsc::Sender<Command>,
                NetworkEvent
            ) = netbase::new().await?;

            // set up file watcher
            let mut watcher = WatcherSender{sender: watcher_sender};
            spawn(watcher.watch(dirs));

            // Listen on all interfaces and whatever port the OS assigns
            netevent.swarm
                .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
                .unwrap();

            spawn(netevent.run());
            // Read full lines from stdin
            let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

            let cliinterface = CliInterface { sender: cli_sender };

            let listener = TcpListener::bind("127.0.0.1:1324").unwrap();
            cliinterface.run(listener);
        },

        CliCommand::Query {
            push
        } => {
            let mut stream = TcpStream::connect("127.0.0.1:1324").unwrap();
            println!("stream input {:?}", "push".as_bytes());
            stream.write_all(b"push");
        },
        _ => {}
    }
    Ok(())

}

#[derive(Clone)]
pub struct CliInterface {
    sender: mpsc::Sender<Command>
}
impl CliInterface {
    fn run(mut self, listener: TcpListener) {
        loop {
            for stream in listener.incoming() {
                let stream = stream.unwrap();
                self.handle_connection(stream);
            }
        }
    }
    fn handle_connection(&mut self, mut stream: TcpStream) {
        let buf_reader = BufReader::new(&mut stream);
        let b: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();
        println!("stream input {:?}", b[0]);
        self.sender.try_send(Command::CliPush)
        .expect("Command receiver not to be dropped.")
    }
}


fn handle_input_line(sender: &mut mpsc::Sender<Command>, line: String) {
    let mut args = line.split(' ');

    match args.next() {
        Some("GET") => {
            let key = {
                match args.next() {
                    Some(key) => Key::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };
            // kademlia.get_record(key, Quorum::One);
        }
        Some("GET_PROVIDERS") => {
            let key = {
                match args.next() {
                    Some(key) => Key::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };
            // kademlia.get_providers(key);
        }
        Some("PUT") => {
            let key = {
                match args.next() {
                    Some(key) => Key::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };
            let value = {
                match args.next() {
                    Some(value) => value.as_bytes().to_vec(),
                    None => {
                        eprintln!("Expected value");
                        return;
                    }
                }
            };
            let record = Record {
                key,
                value,
                publisher: None,
                expires: None,
            };
            // kademlia
            //     .put_record(record, Quorum::One)
            //     .expect("Failed to store record locally.");
        }
        Some("PUT_PROVIDER") => {
            let key = {
                match args.next() {
                    Some(key) => Key::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };

            // kademlia
            //     .start_providing(key)
            //     .expect("Failed to start providing key");
        }
        _ => {
            eprintln!("expected GET, GET_PROVIDERS, PUT or PUT_PROVIDER");
        }
    }
}
