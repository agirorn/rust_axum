use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum UserEvent {
    Created(Created),
    Deleted(Deleted),
    Enabled(Enabled),
    Disabled(Disabled),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Created {
    pub aggregate_id: uuid::Uuid,
    pub event_id: uuid::Uuid,
    pub username: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Deleted {
    pub aggregate_id: uuid::Uuid,
    pub event_id: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Enabled {
    pub aggregate_id: uuid::Uuid,
    pub event_id: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Disabled {
    pub aggregate_id: uuid::Uuid,
    pub event_id: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct NewPassword {
    pub aggregate_id: uuid::Uuid,
    pub event_id: uuid::Uuid,
    pub password: String,
}

impl UserEvent {
    #[cfg(test)]
    pub fn get_event_id(&self) -> uuid::Uuid {
        match self {
            UserEvent::Created(e) => e.event_id,
            UserEvent::Deleted(e) => e.event_id,
            UserEvent::Enabled(e) => e.event_id,
            UserEvent::Disabled(e) => e.event_id,
        }
    }
}
