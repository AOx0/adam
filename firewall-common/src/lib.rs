#![cfg_attr(not(feature = "serde"), no_std)]

pub const MAX_RULES: u32 = 100;

pub mod processor {
    pub const IPV4_TCP: u32 = 0;
}

pub use netp;
use netp::network::InetProtocol;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Event {
    Pass,
    Blocked {
        rule: [u8; 32],
        addr: core::net::SocketAddr,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Action {
    Accept,
    Drop,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Match {
    Match(core::net::IpAddr),
    Socket(core::net::SocketAddr),
    Port(u16),
    Protocol(InetProtocol),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Direction {
    Source,
    Destination,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Rule {
    pub action: Action,
    pub matches: Match,
    pub applies_to: Direction,
    #[cfg_attr(feature = "serde", serde(default))]
    pub enabled: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub init: bool,
}

#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StoredRuleDecoded {
    pub id: [u8; 32],
    pub name: String,
    pub description: String,
    pub rule: Rule,
}

#[cfg(feature = "serde")]
#[cfg(feature = "chrono")]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StoredEventDecoded {
    pub time: chrono::NaiveDateTime,
    pub event: Event,
}

#[cfg(feature = "aya")]
unsafe impl aya::Pod for Rule {}

#[cfg(feature = "bpf")]
impl From<Action> for u32 {
    fn from(value: Action) -> Self {
        match value {
            Action::Drop => aya_ebpf::bindings::xdp_action::XDP_DROP,
            Action::Accept => aya_ebpf::bindings::xdp_action::XDP_PASS,
        }
    }
}
