use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Default, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct UserState {
    pub aggregate_id: Uuid,
}
