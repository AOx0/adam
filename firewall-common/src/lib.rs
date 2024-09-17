#![no_std]

use netp::network::InetProtocol;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "user", derive(serde::Serialize, serde::Deserialize))]
pub enum FirewallEvent {
    Blocked(u32, core::net::SocketAddr),
    Pass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "user", derive(serde::Serialize, serde::Deserialize))]
pub enum FirewallAction {
    Accept,
    Drop,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "user", derive(serde::Serialize, serde::Deserialize))]
pub enum FirewallMatch {
    Match(core::net::IpAddr),
    Socket(core::net::SocketAddr),
    Port(u16),
    Protocol(InetProtocol),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "user", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    Source,
    Destination,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "user", derive(serde::Serialize, serde::Deserialize))]
pub struct FirewallRule {
    pub action: FirewallAction,
    pub matches: FirewallMatch,
    pub applies_to: Direction,
    #[cfg_attr(feature = "user", serde(default))]
    pub enabled: bool,
    #[cfg_attr(feature = "user", serde(default))]
    pub init: bool,
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
