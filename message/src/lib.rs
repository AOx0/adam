pub use bincode;
pub use firewall_common;

use firewall_common::FirewallRule;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Terminate,
    Start,
    Halt,
    Firewall(FirewallRequest),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FirewallResponse {
    Id(u32),
    ListFull,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FirewallRequest {
    AddRule(FirewallRule),
    DeleteRule(u32),
    EnableRule(u32),
    DisableRule(u32),
}
