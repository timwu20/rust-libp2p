use futures::StreamExt;
use libp2p::{
    core::{muxing::StreamMuxerBox, Transport},
    multiaddr::Multiaddr,
    ping,
    swarm::SwarmEvent,
};
use libp2p_webrtc as webrtc;
use rand::thread_rng;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_other_transport(|id_keys| {
            Ok(webrtc::tokio::Transport::new(
                id_keys.clone(),
                webrtc::tokio::Certificate::generate(&mut thread_rng())?,
            )
            .map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn))))
        })?
        .with_behaviour(|_| ping::Behaviour::default())?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    let listen_addr = "/ip4/192.168.68.115/udp/0/webrtc-direct".parse()?;
    swarm.listen_on(listen_addr)?;

    let libp2p_endpoint = "/ip4/192.168.68.115/udp/49255/webrtc-direct/certhash/uEiCeII60YATfKm535q2oaGlJMbHsw8XIym0u9JWhvWko0Q/p2p/12D3KooWEFm8af9RqnCaUYr1NtPb7yjunGrixwCqeLxpQWcbK2Ch";

    let addr = libp2p_endpoint.parse::<Multiaddr>()?;
    tracing::info!("Dialing {addr}");
    swarm.dial(addr)?;

    loop {
        match swarm.next().await.unwrap() {
            SwarmEvent::Behaviour(ping::Event { result: Err(e), .. }) => {
                tracing::error!("Ping failed: {:?}", e);

                break;
            }
            SwarmEvent::Behaviour(ping::Event {
                peer,
                result: Ok(rtt),
                ..
            }) => {
                tracing::info!("Ping successful: RTT: {rtt:?}, from {peer}");
            }
            SwarmEvent::ConnectionClosed {
                cause: Some(cause), ..
            } => {
                tracing::info!("Connection closed due to: {:?}", cause);
            }
            evt => tracing::info!("Swarm event: {:?}", evt),
        }
    }

    Ok(())
}
