use async_std::io;
use std::io::prelude::*;
use async_std::task::spawn;
use clap::Parser;
use env_logger::{Builder, Env};
use futures::{prelude::*, select};
use libp2p::kad::{
    record::Key, Record,
};
use futures::channel::{mpsc};
use std::error::Error;
use std::{
    net::{TcpListener, TcpStream},
    io::{BufReader},
};

use bendy;

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
mod netmessages;

mod dht;
mod db;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    match args.choice {
        CliArgs::Serve { } => {
            let (config, dirs, paths) = onload(args);
            println!("Dir list: {:?}", &dirs);
            println!("Files list: {:?}", &paths.into_iter().map(|x| x.sub_home()).collect::<Vec<String>>());

            // get network objects
            let (mut watcher_sender,
                 mut cli_sender,
                 mut netevent
            ): (mpsc::Sender<Command>,
                mpsc::Sender<CliCommand>,
                NetworkEvent
            ) = netbase::new().await?;

            // set up file watcher
            let mut watcher = WatcherSender{sender: watcher_sender};
            spawn(watcher.watch(dirs.clone()));

            // Listen on all interfaces and whatever port the OS assigns
            netevent.swarm
                .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
                .unwrap();

            netevent.dirs = dirs;
            netevent.startup_check();
            spawn(netevent.run());
            // Read full lines from stdin
            let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

            let cliinterface = CliInterface { sender: cli_sender };

            let listener = TcpListener::bind("127.0.0.1:1324").unwrap();
            cliinterface.run(listener);
        },

        CliArgs::Query {
            push
        } => {
            let PushPath::Push {path} = push;

            let comm: CliCommand;
            if path.is_none() {
                comm = CliCommand {
                    Push: "all".into()
                };
            } else {
                comm = CliCommand {
                    Push: path.unwrap()
                };
            }
            let mut stream = TcpStream::connect("127.0.0.1:1324").unwrap();
            let bencode = bendy::serde::to_bytes(&comm).unwrap();
            println!("stream input {:?}", &bencode);
            stream.write_all(&bencode);
        },
        _ => {}
    }
    Ok(())

}

#[derive(Clone)]
pub struct CliInterface {
    sender: mpsc::Sender<CliCommand>
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
        // dbg!("stream input {:?}", &b);
        let comm: CliCommand = bendy::serde::from_bytes::<CliCommand>(b[0].as_bytes()).unwrap();
        self.sender.try_send(comm)
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
