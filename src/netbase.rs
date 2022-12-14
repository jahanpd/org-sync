use libp2p::gossipsub::MessageId;
use libp2p::gossipsub::{
    GossipsubMessage, IdentTopic as Topic, MessageAuthenticity, ValidationMode,
};
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{Kademlia};
use libp2p::mdns::{Mdns, MdnsConfig};
use libp2p::{gossipsub, identity, PeerId};
use futures::channel::{mpsc};
use std::time::{Duration};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use libp2p::request_response::{
    ProtocolSupport, RequestId, RequestResponse, RequestResponseCodec, RequestResponseEvent,
    RequestResponseMessage, ResponseChannel,
};

use crate::netbehaviour::*;
use crate::netcommand::*;
use crate::netevent::NetworkEvent;
use crate::db;
use crate::netexchange::*;

/// Function for creating new network components
pub async fn new() -> Result<(
    mpsc::Sender<Command>,
    mpsc::Sender<CliCommand>,
    NetworkEvent), Box<dyn Error>> {
    // Create a random PeerId
    // TODO deterministic peerid but specific to machine
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Set up an encrypted TCP Transport over the Mplex and Yamux protocols
    // let transport = libp2p::development_transport(local_key.clone()).await?;
    let transport = libp2p::development_transport(local_key.clone()).await?;

    // Create a Gossipsub topic
    let topic = Topic::new("org-files");

    // Create a Swarm to manage peers and events
    let mut swarm = {
        // To content-address message, we can take the hash of message and use it as an ID.
        let message_id_fn = |message: &GossipsubMessage| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            MessageId::from(s.finish().to_string())
        };

        // Set a custom gossipsub
        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
            .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            .message_id_fn(message_id_fn) // content-address messages. No two messages of the
            // same content will be propagated.
            .build()
            .expect("Valid config");
        // build a gossipsub network behaviour
        let mut gossipsub: gossipsub::Gossipsub =
            gossipsub::Gossipsub::new(MessageAuthenticity::Signed(local_key), gossipsub_config)
                .expect("Correct configuration");

        // subscribes to our topic
        gossipsub.subscribe(&topic).unwrap();

        // add an explicit peer if one was provided
        if let Some(explicit) = std::env::args().nth(2) {
            match explicit.parse() {
                Ok(id) => gossipsub.add_explicit_peer(&id),
                Err(err) => println!("Failed to parse explicit peer id: {:?}", err),
            }
        }

        let mdns = Mdns::new(MdnsConfig::default()).unwrap();
        let store = MemoryStore::new(local_peer_id);
        let kademlia = Kademlia::new(local_peer_id, store);
        let ping = libp2p::ping::Behaviour::new(
            libp2p::ping::Config::new().with_keep_alive(true)
                .with_interval(Duration::from_secs(60))
        );
        let request_response = libp2p::request_response::RequestResponse::new(
            FileExchangeCodec(),
            std::iter::once((FileExchangeProtocol(), ProtocolSupport::Full)),
            Default::default(),
        );
        let behaviour = OrgBehaviour { gossipsub, kademlia, mdns, ping, request_response};
        // build the swarm
        libp2p::Swarm::new(transport, behaviour, local_peer_id)
    };

    // for sending and recieving commands across async processes
    let (watcher_sender, watcher_receiver) = mpsc::channel(0);
    let (cli_sender, cli_receiver) = mpsc::channel(0);
    let (command_sender, command_receiver) = mpsc::channel(0);

    // set up database object
    let database: db::Database = db::new();

    Ok((
        watcher_sender,
        cli_sender,
        NetworkEvent::new(
            swarm,
            watcher_receiver,
            cli_receiver,
            command_receiver,
            command_sender,
            database,
            topic,
            HashMap::new(),
            HashMap::new(),
        )
    )
    )
}
