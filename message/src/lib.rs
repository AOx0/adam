pub use async_bincode;
pub use firewall_common;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Message {
    Terminate,
    Start,
    Halt,
    Firewall(firewall::Request),
}

pub mod firewall {

    use serde::{Deserialize, Serialize};
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    pub enum Response {
        Id(u32),
        ListFull,
        Rules(Vec<firewall_common::StoredRuleDecoded>),
        Rule(firewall_common::StoredRuleDecoded),
        DoesNotExist,
        Status(Status),
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    pub enum Status {
        Stopped,
        Running,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
    pub enum Request {
        AddRule(firewall_common::StoredRuleDecoded),
        DeleteRule(u32),
        EnableRule(u32),
        DisableRule(u32),
        GetRule(u32),
        GetRules,
        Status,
    }
}

#[cfg(test)]
#[cfg(feature = "schema")]
mod test {
    use super::*;

    #[test]
    pub fn print_schamas() {
        use schemars::{schema_for, JsonSchema};

        let schema = schema_for!(Rule);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }
}
