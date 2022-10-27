use libp2p::core::{Multiaddr, PeerId};
use libp2p::request_response::{ResponseChannel};
use futures::channel::{oneshot};

use std::error::Error;
use std::collections::{HashSet};

use futures::channel::{mpsc};
use crate::netexchange::*;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Command {
    StartListening {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    Dial {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    StartProviding {
        file_name: String,
        sender: oneshot::Sender<()>,
    },
    GetProviders {
        file_name: String,
        sender: oneshot::Sender<HashSet<PeerId>>,
    },
    RequestFile {
        file_name: String,
        peer: PeerId,
        sender: oneshot::Sender<Result<Vec<u8>, Box<dyn Error + Send>>>,
    },
    RespondFile {
        file: Vec<u8>,
        channel: ResponseChannel<FileResponse>,
    },
    EditFileChange {
        path: PathBuf
    },
    EditFileAdd {
        path: PathBuf
    },
    EditFileDelete {
        path: PathBuf
    },
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct CliCommand {
    pub Push: String,
}

pub struct Commanders {
    pub sender: mpsc::Sender<Command>,
    pub reciever: mpsc::Receiver<Command>
}

