use libp2p::gossipsub::{
    GossipsubEvent
};
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{
    Kademlia, KademliaEvent
};
use libp2p::mdns::{Mdns, MdnsEvent};
use libp2p::{gossipsub, NetworkBehaviour};

// Logic for network behaviour
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OrgBehaviourEvent")]
pub struct OrgBehaviour {
    pub gossipsub: gossipsub::Gossipsub,
    pub kademlia: Kademlia<MemoryStore>,
    pub mdns: Mdns,
    pub ping: libp2p::ping::Behaviour
}

pub enum OrgBehaviourEvent {
    Gossipsub(GossipsubEvent),
    Kademlia(KademliaEvent),
    Mdns(MdnsEvent),
    Ping(libp2p::ping::Event)
}

impl From<KademliaEvent> for OrgBehaviourEvent {
    fn from(event: KademliaEvent) -> Self {
        OrgBehaviourEvent::Kademlia(event)
    }
}
impl From<GossipsubEvent> for OrgBehaviourEvent {
    fn from(event: GossipsubEvent) -> Self {
        OrgBehaviourEvent::Gossipsub(event)
    }
}

impl From<MdnsEvent> for OrgBehaviourEvent {
    fn from(event: MdnsEvent) -> Self {
        OrgBehaviourEvent::Mdns(event)
    }
}

impl From<libp2p::ping::Event> for OrgBehaviourEvent {
    fn from(event: libp2p::ping::Event) -> Self {
        OrgBehaviourEvent::Ping(event)
    }
}
