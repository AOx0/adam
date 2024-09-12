pub use bincode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Terminate = 1,
    Start = 2,
}
