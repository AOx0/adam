use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    #[serde(skip)]
    token: String,
    id: Thing,
    first_name: String,
    last_name: String,
    username: String,
    mail: String,
}
