use libp2p::kad::RoutingUpdate;
use libp2p::kad::{
    store::MemoryStore as KademliaInMemory, Behaviour as KademliaBehavior, Event as KademliaEvent,
};
use libp2p::swarm::NetworkBehaviour;
use libp2p::{Multiaddr, PeerId};

use libp2p::identify::{Behaviour as IdentifyBehavior, Event as IdentifyEvent};

use libp2p::request_response::cbor::Behaviour as RequestResponseBehavior;
use libp2p::request_response::{
    Event as RequestResponseEvent, OutboundRequestId, ResponseChannel as RequestResponseChannel,
};

pub use crate::message::{GreetRequest, GreetResponse};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event")]
//pub(crate) struct Behavior {
pub struct Behavior {
    identify: IdentifyBehavior,
    kad: KademliaBehavior<KademliaInMemory>,
    rr: RequestResponseBehavior<GreetRequest, GreetResponse>,
}

impl Behavior {
    pub fn new(
        kad: KademliaBehavior<KademliaInMemory>,
        identify: IdentifyBehavior,
        rr: RequestResponseBehavior<GreetRequest, GreetResponse>,
    ) -> Self {
        Self { kad, identify, rr }
    }

    pub fn register_addr_kad(&mut self, peer_id: &PeerId, addr: Multiaddr) -> RoutingUpdate {
        self.kad.add_address(peer_id, addr)
    }

    pub fn register_addr_rr(&mut self, peer_id: &PeerId, addr: Multiaddr) -> bool {
        self.rr.add_address(peer_id, addr)
    }

    pub fn send_message(&mut self, peer_id: &PeerId, message: GreetRequest) -> OutboundRequestId {
        self.rr.send_request(peer_id, message)
    }

    pub fn send_response(
        &mut self,
        ch: RequestResponseChannel<GreetResponse>,
        rs: GreetResponse,
    ) -> Result<(), GreetResponse> {
        self.rr.send_response(ch, rs)
    }

    pub fn set_server_mode(&mut self) {
        self.kad.set_mode(Some(libp2p::kad::Mode::Server))
    }
}

#[derive(Debug)]
//pub(crate) enum Event {
pub enum Event {
    Identify(IdentifyEvent),
    Kad(KademliaEvent),
    RequestResponse(RequestResponseEvent<GreetRequest, GreetResponse>),
}

impl From<IdentifyEvent> for Event {
    fn from(value: IdentifyEvent) -> Self {
        Self::Identify(value)
    }
}

impl From<KademliaEvent> for Event {
    fn from(value: KademliaEvent) -> Self {
        Self::Kad(value)
    }
}

impl From<RequestResponseEvent<GreetRequest, GreetResponse>> for Event {
    fn from(value: RequestResponseEvent<GreetRequest, GreetResponse>) -> Self {
        Self::RequestResponse(value)
    }
}
