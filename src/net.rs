use crate::{config::Config, dispatch::Dispatcher, metrics::MESSAGES_SENT};
use bytes::Bytes;
use log::info;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::UdpSocket;

pub async fn start_multicast_listener(
    config: &Config,
    dispatcher: Dispatcher,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listen_addr: Ipv4Addr = "0.0.0.0".parse()?;

    let addr = SocketAddrV4::new(listen_addr, config.multicast_port);
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&addr.into())?;
    socket.join_multicast_v4(&config.multicast_group, &Ipv4Addr::UNSPECIFIED)?;
    let socket = UdpSocket::from_std(socket.into())?;

    info!(
        "Multicast listener started on {}:{}",
        config.multicast_group, config.multicast_port
    );

    let mut buf = vec![0u8; 1500];
    loop {
        if let Ok((size, peer)) = socket.recv_from(&mut buf).await {
            let data = &buf[..size];
            info!("Received {} bytes from {}", size, peer);
            dispatcher.dispatch(data);
        }
    }
}

pub async fn send_multicast(config: &Config, data: Bytes) -> std::io::Result<()> {
    let addr = SocketAddrV4::new(config.multicast_group, config.multicast_port);
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.send_to(&data, addr).await?;
    MESSAGES_SENT.inc();
    Ok(())
}
