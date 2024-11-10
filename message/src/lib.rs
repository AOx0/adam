pub use async_bincode;
pub use firewall_common;

use serde::{Deserialize, Serialize};

pub mod firewall;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Message {
    Terminate,
    Start,
    Halt,
    Firewall(firewall::Request),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum EventQuery {
    All,
    Last(std::time::Duration),
}

#[cfg(test)]
#[cfg(feature = "schema")]
mod test {
    use super::*;
    use firewall_common::*;

    #[test]
    pub fn print_schamas() {
        use schemars::{schema_for, JsonSchema};

        let schema = schema_for!(Rule);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }
}
