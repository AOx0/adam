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

#[cfg(feature = "user")]
unsafe impl aya::Pod for FirewallRule {}

#[cfg(feature = "bpf")]
impl From<FirewallAction> for u32 {
    fn from(value: FirewallAction) -> Self {
        match value {
            FirewallAction::Drop => aya_ebpf::bindings::xdp_action::XDP_DROP,
            FirewallAction::Accept => aya_ebpf::bindings::xdp_action::XDP_PASS,
        }
    }
}
