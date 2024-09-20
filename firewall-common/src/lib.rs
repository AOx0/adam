#![cfg_attr(not(feature = "user"), no_std)]

pub const MAX_RULES: u32 = 100;

pub use netp;
use netp::network::InetProtocol;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "user", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "user", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum FirewallEvent {
    Pass,
    Blocked {
        rule: u32,
        addr: core::net::SocketAddr,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "user", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "user", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum FirewallAction {
    Accept,
    Drop,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "user", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "user", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum FirewallMatch {
    Match(core::net::IpAddr),
    Socket(core::net::SocketAddr),
    Port(u16),
    Protocol(InetProtocol),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "user", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "user", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Direction {
    Source,
    Destination,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "user", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "user", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FirewallRule {
    /// id is set by the firewall controller when adding the new rule
    #[cfg_attr(feature = "user", serde(default))]
    pub id: u32,
    pub action: FirewallAction,
    pub matches: FirewallMatch,
    pub applies_to: Direction,
    /// All rules are disabled by default
    #[cfg_attr(feature = "user", serde(default))]
    pub enabled: bool,
    /// All added rules are marked as initialized
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
