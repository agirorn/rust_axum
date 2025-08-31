use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::types::Json;
use uuid::Uuid;

use crate::{command::SetPassword, state::UserState};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "event_name", content = "data", rename_all = "snake_case")]
pub enum UserEvent {
    Snapshot(UserState),
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
    pub occ_version: u64,
    pub aggregate_type: String,
    pub data: UserEvent,
}

impl Envelope {
    pub fn new(aggregate_id: Uuid, occ_version: u64, data: UserEvent) -> Self {
        Self {
            aggregate_id,
            event_id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            aggregate_type: "user".to_string(),
            occ_version,
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

impl From<Json<Envelope>> for Envelope {
    fn from(value: Json<Envelope>) -> Envelope {
        value.0
    }
}
