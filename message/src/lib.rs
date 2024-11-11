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
    Since(chrono::NaiveDateTime),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Log<K> {
    pub time: chrono::NaiveDateTime,
    pub kind: K,
}

#[cfg(test)]
#[cfg(feature = "schema")]
mod test {

    use super::*;
    use firewall_common::*;

    #[test]
    pub fn print_schamas() {
        use schemars::{schema_for, JsonSchema};
        use std::time::Duration;

        let schema = schema_for!(EventQuery);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());

        println!(
            "{}",
            serde_json::to_string_pretty(&EventQuery::All).unwrap()
        );

        let duration = Duration::from_secs(320);

        println!(
            "{}",
            serde_json::to_string_pretty(&EventQuery::Last(duration)).unwrap()
        );

        let since = chrono::Local::now().naive_utc() - Duration::from_secs(60 * 60 * 24 * 30);

        println!(
            "{}",
            serde_json::to_string_pretty(&EventQuery::Since(since)).unwrap()
        );
    }
}
