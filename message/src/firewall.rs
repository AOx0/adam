use firewall_common::StoredEventDecoded;
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
    RuleChange(RuleChange),
    Events(Vec<firewall_common::StoredEventDecoded>),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum LogKind {
    Event(StoredEventDecoded),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum RuleChange {
    NoSuchRule,
    NoChangeRequired(RuleStatus),
    Change(RuleStatus),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum RuleStatus {
    Active,
    Inactive,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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
    ToggleRule(u32),
    GetRule(u32),
    GetRules,
    Status,
    GetEvents(crate::EventQuery),
}
