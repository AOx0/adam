pub use bincode;
pub use firewall_common;

use firewall_common::FirewallRule;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Message {
    Terminate,
    Start,
    Halt,
    Firewall(FirewallRequest),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum FirewallResponse {
    Id(u32),
    ListFull,
    Rules(Vec<FirewallRule>),
    Rule(FirewallRule),
    DoesNotExist,
    Status(FirewallStatus),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum FirewallStatus {
    Stopped,
    Running,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum FirewallRequest {
    AddRule(FirewallRule),
    DeleteRule(u32),
    EnableRule(u32),
    DisableRule(u32),
    GetRule(u32),
    GetRules,
    Status,
}

#[cfg(test)]
#[cfg(feature = "schema")]
mod test {
    use super::*;

    #[test]
    pub fn print_schamas() {
        use schemars::{schema_for, JsonSchema};

        let schema = schema_for!(FirewallRule);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }
}
