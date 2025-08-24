use crate::command::{self, UserCommand};
use crate::error::{Error, Result};
use crate::event::{self, UserEvent};
use crate::state::UserState;
use async_trait::async_trait;
// use eventsourced_core::{Aggregate, EventStoreFor};
use eventsourced_core::Aggregate;
use uuid::Uuid;

#[derive(Default, Debug)]
pub struct User {
    events: Vec<UserEvent>,
    pub state: UserState,
}

#[async_trait]
impl Aggregate for User {
    type Command = UserCommand;
    type CommandResult = ();
    type Error = Error;
    type Event = UserEvent;
    type State = UserState;
    type AggregateId = Uuid;

    async fn handle(&mut self, cmd: Self::Command) -> Result<Self::CommandResult> {
        match cmd {
            UserCommand::Create(cmd) => self.handle_create(cmd).await?,
            UserCommand::Delete(cmd) => self.handle_delete(cmd).await?,
            UserCommand::Enable(_cmd) => {
                unimplemented!("UserCommand::Delete");
            }
            UserCommand::Disable(_cmd) => {
                unimplemented!("UserCommand::Delete");
            }
            UserCommand::SetPassword(_cmd) => {
                unimplemented!("UserCommand::Delete");
            }
        }
        Ok(())
    }

    fn apply(&mut self, event: UserEvent, save: bool) -> Result<()> {
        self.apply_event(&event);
        if save {
            self.events.push(event);
        }
        Ok(())
    }

    fn get_uncommitted_events(&mut self) -> Vec<Self::Event> {
        self.events.clone()
    }

    fn get_state(&self) -> &Self::State {
        &self.state
    }

    fn new(aggregate_id: &Uuid) -> Self {
        Self {
            events: vec![],
            state: UserState {
                aggregate_id: *aggregate_id,
                exists: false,
            },
        }
    }
}

impl User {
    async fn handle_create(&mut self, _cmd: command::Create) -> Result<()> {
        self.apply(
            UserEvent::Created(event::Created {
                aggregate_id: self.state.aggregate_id,
                event_id: Uuid::new_v4(),
                username: "username".to_string(),
            }),
            true,
        )?;
        Ok(())
    }

    async fn handle_delete(&mut self, _cmd: command::Delete) -> Result<()> {
        self.apply(
            UserEvent::Deleted(event::Deleted {
                aggregate_id: self.state.aggregate_id,
                event_id: Uuid::new_v4(),
            }),
            true,
        )?;
        Ok(())
    }

    fn apply_event(&mut self, event: &UserEvent) {
        match event {
            UserEvent::Created(_event) => {
                self.state.exists = true;
            }

            UserEvent::Deleted(_event) => {
                self.state.exists = false;
            }

            UserEvent::Enabled(_event) => {
                println!("--> UserEvent::Enabled");
            }

            UserEvent::Disabled(_event) => {
                println!("--> UserEvent::Disabled");
            }
        }
    }
}
