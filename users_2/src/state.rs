use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Default, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct UserState {
    pub event_name: String,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub username: String,
    pub has_password: bool,
    pub occ_version: u64,
    pub exists: bool,
    pub enabled: bool,
    pub password_hash: Option<String>,
}
