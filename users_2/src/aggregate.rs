use crate::command::{self, UserCommand};
use crate::error::{Error, Result};
use crate::event::{self, Envelope, UserEvent};
use crate::state::UserState;
use async_trait::async_trait;
// use eventsourced_core::{Aggregate, EventStoreFor};
use eventsourced_core::Aggregate;
use uuid::Uuid;

#[derive(Default, Debug)]
pub struct User {
    events: Vec<Envelope>,
    pub state: UserState,
}

#[async_trait]
impl Aggregate for User {
    type Command = UserCommand;
    type CommandResult = ();
    type Error = Error;
    type Event = Envelope;
    type State = UserState;
    type AggregateId = Uuid;

    fn new(aggregate_id: &Uuid) -> Self {
        Self {
            events: vec![],
            state: UserState {
                aggregate_id: *aggregate_id,
                username: "".to_string(),
                has_password: false,
                exists: false,
                enabled: true,
                password_hash: None,
            },
        }
    }

    fn get_state(&self) -> &Self::State {
        &self.state
    }

    fn get_uncommitted_events(&mut self) -> Vec<Self::Event> {
        self.events.clone()
    }

    fn apply(&mut self, event: Envelope, save: bool) -> Result<()> {
        self.apply_event(&event);
        if save {
            self.events.push(event);
        }
        Ok(())
    }

    async fn handle(&mut self, cmd: Self::Command) -> Result<Self::CommandResult> {
        match cmd {
            UserCommand::Create(cmd) => self.handle_create(cmd).await?,
            UserCommand::Delete(cmd) => self.handle_delete(cmd).await?,
            UserCommand::Enable(cmd) => self.handle_enable(cmd).await?,
            UserCommand::Disable(cmd) => self.handle_disable(cmd).await?,
            UserCommand::SetPassword(cmd) => self.handle_set_password(cmd).await?,
        }
        Ok(())
    }
}

impl User {
    async fn handle_create(&mut self, _cmd: command::Create) -> Result<()> {
        self.apply(
            Envelope::new(
                self.state.aggregate_id,
                event::Created {
                    username: "username".to_string(),
                }
                .into(),
            ),
            true,
        )?;
        Ok(())
    }

    async fn handle_delete(&mut self, _cmd: command::Delete) -> Result<()> {
        self.apply(
            Envelope::new(self.state.aggregate_id, UserEvent::Deleted),
            true,
        )?;
        Ok(())
    }

    async fn handle_enable(&mut self, _cmd: command::Enable) -> Result<()> {
        self.apply(
            Envelope::new(self.state.aggregate_id, UserEvent::Enabled),
            true,
        )?;
        Ok(())
    }

    async fn handle_disable(&mut self, _cmd: command::Disable) -> Result<()> {
        self.apply(
            Envelope::new(self.state.aggregate_id, UserEvent::Disabled),
            true,
        )?;
        Ok(())
    }

    async fn handle_set_password(&mut self, cmd: command::SetPassword) -> Result<()> {
        self.apply(
            Envelope::new(
                self.state.aggregate_id,
                event::NewPassword {
                    password_hash: bcrypt::hash(cmd.password, bcrypt::DEFAULT_COST)?,
                }
                .into(),
            ),
            true,
        )?;
        Ok(())
    }
}

impl User {
    fn apply_event(&mut self, event: &Envelope) {
        match event.data.clone() {
            UserEvent::Created(event) => {
                self.state.exists = true;
                self.state.username = event.username.clone();
            }

            UserEvent::Deleted => {
                self.state.exists = false;
            }

            UserEvent::Enabled => {
                self.state.enabled = true;
            }

            UserEvent::Disabled => {
                self.state.enabled = false;
            }

            UserEvent::NewPassword(event) => {
                self.state.has_password = true;
                self.state.password_hash = Some(event.password_hash);
            }
        }
    }
}
