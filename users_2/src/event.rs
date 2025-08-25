use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum UserEvent {
    Created(Created),
    Deleted(Deleted),
    Enabled(Enabled),
    Disabled(Disabled),
    NewPassword(NewPassword),
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
    /// A bcrypt hased password
    pub password_hash: String,
}

impl UserEvent {
    #[cfg(test)]
    pub fn get_event_id(&self) -> uuid::Uuid {
        match self {
            UserEvent::Created(e) => e.event_id,
            UserEvent::Deleted(e) => e.event_id,
            UserEvent::Enabled(e) => e.event_id,
            UserEvent::Disabled(e) => e.event_id,
            UserEvent::NewPassword(e) => e.event_id,
        }
    }

    pub fn get_aggregate_id(&self) -> uuid::Uuid {
        match self {
            UserEvent::Created(e) => e.aggregate_id,
            UserEvent::Deleted(e) => e.aggregate_id,
            UserEvent::Enabled(e) => e.aggregate_id,
            UserEvent::Disabled(e) => e.aggregate_id,
            UserEvent::NewPassword(e) => e.aggregate_id,
        }
    }
}
