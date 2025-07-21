use clap::{Args, Parser};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Parser, Debug, Clone)]
#[command(name = "homelabd", about = "Peer daemon for your homelab")]
pub struct Config {
    /// Multicast group address (IPv4)
    #[arg(long, default_value = "239.255.0.1")]
    pub multicast_group: Ipv4Addr,

    /// Multicast port
    #[arg(long, default_value_t = 44044)]
    pub multicast_port: u16,

    /// HTTP bind address
    #[arg(long, default_value = "0.0.0.0")]
    pub http_bind_ip: IpAddr,

    /// HTTP port
    #[arg(long, default_value_t = 8800)]
    pub http_port: u16,

    /// Override the hostname
    #[arg(long)]
    pub hostname_override: Option<String>,
}
