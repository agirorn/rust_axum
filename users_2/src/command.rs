use eventsourced_core::Command;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum UserCommand {
    Create(Create),
    Delete(Delete),
    Enable(Enable),
    Disable(Disable),
    SetPassword(SetPassword),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Create {
    pub aggregate_id: uuid::Uuid,
    pub username: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Delete {
    pub aggregate_id: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Enable {
    pub aggregate_id: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Disable {
    pub aggregate_id: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SetPassword {
    pub aggregate_id: uuid::Uuid,
    pub password: String,
}

impl Command<uuid::Uuid> for UserCommand {
    fn aggregate_id(&self) -> uuid::Uuid {
        match self {
            UserCommand::Create(cmd) => cmd.aggregate_id,
            UserCommand::Delete(cmd) => cmd.aggregate_id,
            UserCommand::Enable(cmd) => cmd.aggregate_id,
            UserCommand::Disable(cmd) => cmd.aggregate_id,
            UserCommand::SetPassword(cmd) => cmd.aggregate_id,
        }
    }
}
