use libp2p::gossipsub::{
    GossipsubEvent
};
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{
    Kademlia, KademliaEvent
};
use libp2p::mdns::{Mdns, MdnsEvent};
use libp2p::{gossipsub, NetworkBehaviour};
use crate::netexchange::*;
use libp2p::request_response::{
    ProtocolSupport, RequestId, RequestResponse, RequestResponseCodec, RequestResponseEvent,
    RequestResponseMessage, ResponseChannel,
};

// Logic for network behaviour
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OrgBehaviourEvent")]
pub struct OrgBehaviour {
    pub gossipsub: gossipsub::Gossipsub,
    pub kademlia: Kademlia<MemoryStore>,
    pub mdns: Mdns,
    pub ping: libp2p::ping::Behaviour,
    pub request_response: RequestResponse<FileExchangeCodec>,
}

pub enum OrgBehaviourEvent {
    Gossipsub(GossipsubEvent),
    Kademlia(KademliaEvent),
    Mdns(MdnsEvent),
    Ping(libp2p::ping::Event),
    RequestResponse(RequestResponseEvent<FileRequest, FileResponse>)

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

impl From<RequestResponseEvent<FileRequest, FileResponse>> for OrgBehaviourEvent {
    fn from(event: RequestResponseEvent<FileRequest, FileResponse>) -> Self {
        OrgBehaviourEvent::RequestResponse(event)
    }
}
