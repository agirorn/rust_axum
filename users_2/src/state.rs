use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Default, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct UserState {
    pub aggregate_id: Uuid,
    pub username: String,
    pub has_password: bool,
    pub exists: bool,
    pub enabled: bool,
}
