use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum UserEvent {
    Created(Created),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Created {
    pub aggregate_id: uuid::Uuid,
    pub event_id: uuid::Uuid,
    pub username: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Deleted {
    pub aggregate_id: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Enabled {
    pub aggregate_id: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Disabled {
    pub aggregate_id: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NewPassword {
    pub aggregate_id: uuid::Uuid,
    pub password: String,
}
