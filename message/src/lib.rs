pub use bincode;
use firewall_common::FirewallRule;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Terminate,
    Start,
    Halt,
    Firewall(Firewall),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Firewall {
    AddRule(FirewallRule),
    DeleteRule(u32),
    EnableRule(u32),
    DisableRule(u32),
}
