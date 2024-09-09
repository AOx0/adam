#![no_std]

#[derive(Debug, Clone, Copy)]
pub enum FirewallEvent {
    Blocked(core::net::SocketAddr),
    Pass,
}

#[derive(Debug, Clone, Copy)]
pub enum FirewallAction {
    Accept,
    Pass,
}

#[derive(Debug, Clone, Copy)]
pub enum FirewallMatch {
    Mask(core::net::IpAddr),
    Lit(core::net::IpAddr),
    Socket(core::net::SocketAddr),
    Port(u16),
}

#[derive(Debug, Clone, Copy)]
pub struct FirewallRule {
    action: FirewallAction,
    matches: FirewallMatch,
}
