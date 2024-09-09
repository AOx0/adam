#![no_std]

use netp::network::InetProtocol;

#[derive(Debug, Clone, Copy)]
pub enum FirewallEvent {
    Blocked(core::net::SocketAddr),
    Pass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirewallAction {
    Accept,
    Drop,
}

#[derive(Debug, Clone, Copy)]
pub enum FirewallMatch {
    Match(core::net::IpAddr),
    Socket(core::net::SocketAddr),
    Port(u16),
    Protocol(InetProtocol),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Source,
    Destination,
}

#[derive(Debug, Clone, Copy)]
pub struct FirewallRule {
    pub action: FirewallAction,
    pub matches: FirewallMatch,
    pub applies_to: Direction,
}

#[cfg(feature = "aya")]
unsafe impl aya::Pod for FirewallRule {}
