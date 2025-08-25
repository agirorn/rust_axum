use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::command::SetPassword;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum UserEvent {
    Created(Created),
    Deleted,
    Enabled,
    Disabled,
    NewPassword(NewPassword),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Created {
    pub username: String,
}

impl From<Created> for UserEvent {
    fn from(value: Created) -> Self {
        UserEvent::Created(value)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Deleted {}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Enabled {}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Disabled {}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct NewPassword {
    /// A bcrypt hased password
    pub password_hash: String,
}

impl From<NewPassword> for UserEvent {
    fn from(value: NewPassword) -> Self {
        UserEvent::NewPassword(value)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Envelope {
    pub aggregate_id: uuid::Uuid,
    pub event_id: uuid::Uuid,
    pub timestamp: DateTime<Utc>,
    pub data: UserEvent,
}

impl Envelope {
    pub fn new(aggregate_id: Uuid, data: UserEvent) -> Self {
        Self {
            aggregate_id,
            event_id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            data,
        }
    }

    pub fn get_aggregate_id(&self) -> uuid::Uuid {
        self.aggregate_id
    }

    pub fn get_event_id(&self) -> uuid::Uuid {
        self.event_id
    }
}
