use std::net::{IpAddr, UdpSocket};
use tokio_multi::*;

use get_if_addrs::{get_if_addrs, IfAddr};

use std::collections::HashMap;
use std::env::args;
use std::error::Error;
use std::time::Duration;

use env_logger::{Builder, Env};
use log::{debug, error, info, trace, warn};
use tokio;

use libp2p::{
    identity, tcp::Config as TcpConfig, yamux::Config as YamuxConfig, Multiaddr, PeerId,
    StreamProtocol, SwarmBuilder,
};

use libp2p::futures::StreamExt;
use libp2p::noise::Config as NoiceConfig;
use libp2p::swarm::SwarmEvent;

use libp2p::identify::{
    Behaviour as IdentifyBehavior, Config as IdentifyConfig, Event as IdentifyEvent,
};

use libp2p::kad::{
    store::MemoryStore as KadInMemory, Behaviour as KadBehavior, Config as KadConfig,
    Event as KadEvent, RoutingUpdate,
};

use libp2p::request_response::{
    Config as RequestResponseConfig, Event as RequestResponseEvent,
    Message as RequestResponseMessage, ProtocolSupport as RequestResponseProtocolSupport,
};

use libp2p::request_response::cbor::Behaviour as RequestResponseBehavior;

//mod behavior;
use tokio_multi::behavior::{Behavior as AgentBehavior, Event as AgentEvent};

//mod message;
use tokio_multi::message::{GreetRequest, GreetResponse};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args_vec: Vec<String> = env::args().collect();

    if args_vec.len() < 2 {
        info!("Please provide at least one argument.");
        ()
    }

    trace!("Arguments:");
    for (index, arg) in args_vec.iter().enumerate() {
        if Some(index) == Some(0) {
            trace!("Some(index) = Some(0):  {}: {}", index, arg);
        } else {
            trace!("  {}: {}", index, arg);
        }
    }

    if let Some(whoami) = Some(args_vec[0].clone()) {
        //let whoami: String = whoami.parse()?;
        info!("whoami: {whoami}");
    } else {
        //let whoami: String = whoami.parse()?;
        //info!("whoami: {whoami}");
    }

    if let Some(log_level) = args().nth(2) {
        Builder::from_env(Env::default().default_filter_or(log_level)).init();
    } else {
        Builder::from_env(Env::default().default_filter_or("info")).init();
    }

    let local_key = identity::Keypair::generate_ed25519();

    let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(TcpConfig::default(), NoiceConfig::new, YamuxConfig::default)?
        .with_behaviour(|key| {
            let local_peer_id = PeerId::from(key.clone().public());
            info!("LocalPeerID: {local_peer_id}");

            let mut kad_config = KadConfig::default();
            kad_config.set_protocol_names(vec![StreamProtocol::new("/agent/connection/1.0.0")]);

            let kad_memory = KadInMemory::new(local_peer_id);
            let kad = KadBehavior::with_config(local_peer_id, kad_memory, kad_config);

            let identity_config =
                IdentifyConfig::new("/agent/connection/1.0.0".to_string(), key.clone().public())
                    .with_push_listen_addr_updates(true)
                    .with_interval(Duration::from_secs(1)); //TODO cli arg

            let rr_config = RequestResponseConfig::default();
            let rr_protocol = StreamProtocol::new("/agent/message/1.0.0");
            let rr_behavior = RequestResponseBehavior::<GreetRequest, GreetResponse>::new(
                [(rr_protocol, RequestResponseProtocolSupport::Full)],
                rr_config,
            );

            let identify = IdentifyBehavior::new(identity_config);
            AgentBehavior::new(kad, identify, rr_behavior)
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(10)))
        .build();

    swarm.behaviour_mut().set_server_mode();
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let interfaces = get_if_addrs()?;

    if let Some(addr) = args().nth(1) {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        info!("\n\nDialed to: {addr}\n");
    } else {
        info!("Act as bootstrap node");
        for iface in interfaces {
            debug!("Interface: {}", iface.name);
            match iface.addr {
                IfAddr::V4(ipv4) => {
                    trace!("\n\nipv4.ip={}\n\n", ipv4.ip);
                    let multi_addr: Multiaddr = format!("/ip4/{}/tcp/6102", ipv4.ip).parse()?;
                    swarm.listen_on(format!("/ip4/{}/tcp/6102", ipv4.ip).parse()?)?;
                    let _ = swarm.dial(multi_addr)?;
                }
                IfAddr::V6(ipv6) => {
                    trace!("\n\nipv6.ip={}\n\n", ipv6.ip);
                    let multi_addr: Multiaddr = format!("/ip6/{}/tcp/6102", ipv6.ip).parse()?;
                    swarm.listen_on(format!("/ip6/{}/tcp/6102", ipv6.ip).parse()?)?;
                    let _ = swarm.dial(multi_addr)?;
                }
            }
        }
    }

    let mut peers: HashMap<PeerId, Vec<Multiaddr>> = HashMap::new();
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { listener_id, address } => info!("NewListenAddr: {listener_id:?} | {address:?}"),
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                num_established,
                concurrent_dial_errors,
                established_in } => info!("\n\nConnectionEstablished: {peer_id} | {connection_id} | {endpoint:?} | {num_established} | {concurrent_dial_errors:?} | {established_in:?}\n"),
            SwarmEvent::Dialing { peer_id, connection_id } => info!("Dialing: {peer_id:?} | {connection_id}"),
            SwarmEvent::Behaviour(AgentEvent::Identify(event)) => match event {
                IdentifyEvent::Sent { peer_id, .. } => info!("IdentifyEvent:Sent: {peer_id}"),
                IdentifyEvent::Pushed { peer_id, info, .. } => info!("IdentifyEvent:Pushed: {peer_id} | {info:?}"),
                //IdentifyEvent::Received { peer_id, info } => {
				IdentifyEvent::Received { peer_id, info, connection_id } => {
				//IdentifyEvent::Received { peer_id, info, .. } => {
                    info!("\n\nIdentifyEvent:Received:\n{peer_id} | {info:?}\n\n");
                    peers.insert(peer_id, info.clone().listen_addrs);

                    for addr in info.clone().listen_addrs {
                    info!("\n\naddr:\n{addr} | {info:?}\n");
                    info!("\n\ninfo:\n{info:?}\n");
                        let agent_routing = swarm.behaviour_mut().register_addr_kad(&peer_id, addr.clone());
                        match agent_routing {
                            RoutingUpdate::Failed => error!("IdentifyReceived: Failed to register address to Kademlia"),
                            RoutingUpdate::Pending => warn!("IdentifyReceived: Register address pending"),
                            RoutingUpdate::Success => {
                                info!("\n\nIdentifyReceived: {addr}: Success register address\n");
                            }
                        }

                        _ = swarm.behaviour_mut().register_addr_rr(&peer_id, addr.clone());

                        let local_peer_id = local_key.public().to_peer_id();
						//GreetRequest
                        let message = GreetRequest{ message: format!("Send message from: {local_peer_id}: Hello gnostr!!!") };
                        let request_id = swarm.behaviour_mut().send_message(&peer_id, message);
                        info!("\n\nRequestID: {request_id}\n")
                    }

                    info!("\n\nAvailable peers:\n{peers:?}\n\n");

                },
                _ => {}
            },
            SwarmEvent::Behaviour(AgentEvent::RequestResponse(event)) => match event {
                RequestResponseEvent::Message { peer, message } => {
                    match message {
                        RequestResponseMessage::Request { request_id, request, channel} => {


                            info!("\n\nRequestResponseEvent::Message::Request -> PeerID: {peer} | RequestID: {request_id} | RequestMessage:\n{request:?}\n\n");


                            let local_peer_id = local_key.public().to_peer_id();
                            let response = GreetResponse{ message: format!("Response from: {local_peer_id}: hello gnostr too!!!").to_string() };


                            let result = swarm.behaviour_mut().send_response(channel, response);
                            if result.is_err() {
                                let err = result.unwrap_err();
                                error!("Error sending response: {err:?}")
                            } else {


                                info!("\n\n\nSending a gnostr message was success!!!\n\n")


                            }
                        },
                        RequestResponseMessage::Response { request_id, response } => {


                            info!("\n\n\nRequestResponseEvent::Message::Response -> PeerID:\n{peer} | RequestID: {request_id} | Response:\n{response:?}\n\n")


                        }
                    }
                },
                RequestResponseEvent::InboundFailure { peer, request_id, error } => {
                    warn!("\n\nRequestResponseEvent::InboundFailure -> PeerID:\n{peer} | RequestID: {request_id} | Error: {error}\n")
                },
                RequestResponseEvent::ResponseSent { peer, request_id } => {


                    info!("\n\n\nRequestResponseEvent::ResponseSent -> PeerID:\n{peer} | RequestID:\n{request_id}\n\n")


                },
                RequestResponseEvent::OutboundFailure { peer, request_id, error } => {
                    warn!("\n\n\nRequestResponseEvent::OutboundFailure -> PeerID:\n{peer} | RequestID:\n{request_id} | Error: {error}")
                }
            },
            SwarmEvent::Behaviour(AgentEvent::Kad(event)) => match event {
                KadEvent::ModeChanged { new_mode } => info!("\n\nKadEvent:ModeChanged:\n{new_mode}\n"),
                KadEvent::RoutablePeer { peer, address } => info!("\n\nKadEvent:RoutablePeer:\n{peer} | {address}\n"),
                KadEvent::PendingRoutablePeer { peer, address } => info!("\n\nKadEvent:PendingRoutablePeer:\n{peer} | {address}\n"),
                KadEvent::InboundRequest { request } => info!("\n\nKadEvent:InboundRequest:\n{request:?}\n"),
                KadEvent::RoutingUpdated {
                    peer,
                    is_new_peer,
                    addresses,
                    bucket_range,
                    old_peer } => {

                        info!("\n\nKadEvent:RoutingUpdated:\n{peer} | IsNewPeer? {is_new_peer} | {addresses:?} | {bucket_range:?} | OldPeer: {old_peer:?}\n\n");

                    },
                KadEvent::OutboundQueryProgressed {
                    id,
                    result,
                    stats,
                    step } => {

                    info!("\n\nKadEvent:OutboundQueryProgressed:\nID: {id:?} | Result:\n{result:?} | Stats:\n{stats:?} | Step:\n{step:?}\n\n")

                },
                _ => {}
            }
            _ => {}
        }
    }
}
