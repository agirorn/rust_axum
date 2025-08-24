use crate::command::{self, UserCommand};
use crate::error::{Error, Result};
use crate::event::{self, UserEvent};
use crate::state::UserState;
// use crate::store::UserEventStore;
use async_trait::async_trait;
use eventsourced_core::{Aggregate, EventStoreFor};
// use futures_util::StreamExt;
use futures_util::StreamExt;
use uuid::Uuid;

#[derive(Default, Debug)]
pub struct User {
    events: Vec<UserEvent>,
    pub state: UserState,
}

#[async_trait]
impl Aggregate for User {
    type Command = UserCommand;
    type Result = Result<()>;
    type Error = Error;
    type LoadResult = Result<Self>;
    type Event = UserEvent;
    type State = UserState;
    type AggregateId = Uuid;

    async fn execute<ES>(
        event_store: &mut ES,
        cmd: Self::Command,
    ) -> std::result::Result<(), Self::Error>
    where
        ES: EventStoreFor<Self>,
    {
        let mut aggregate = User::load_from(event_store, cmd.aggregate_id()).await?;
        aggregate.handle_command(cmd).await?;
        event_store
            .save(&mut aggregate.get_uncommitted_events(), &aggregate.state)
            .await?;
        Ok(())
    }

    async fn handle_command(&mut self, cmd: Self::Command) -> Self::Result {
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

    async fn load_from<ES>(event_store: &ES, aggregate_id: Self::AggregateId) -> Self::LoadResult
    where
        ES: EventStoreFor<Self>,
    {
        let mut user = User::new(aggregate_id);
        let mut events = event_store.event_stream(aggregate_id);
        while let Some(event) = events.next().await {
            let event = event?;
            user.apply(event, false).await?;
        }
        Ok(user)
    }

    async fn save_to<ES>(&mut self, _event_store: &mut ES) -> Self::Result
    where
        ES: EventStoreFor<Self>,
    {
        // event_store.write(self.get_uncommitted_events().await?);
        Ok(())
    }

    // This function should not be async
    async fn apply(&mut self, event: UserEvent, save: bool) -> Self::Result {
        self.proccess_event(&event);
        if save {
            self.events.push(event);
        }
        Ok(())
    }

    fn get_uncommitted_events(&mut self) -> Vec<Self::Event> {
        self.events.clone()
    }
}

impl User {
    pub fn new(aggregate_id: Uuid) -> Self {
        Self {
            events: vec![],
            state: UserState {
                aggregate_id,
                exists: false,
            },
        }
    }

    async fn handle_create(&mut self, _cmd: command::Create) -> Result<()> {
        self.apply(
            UserEvent::Created(event::Created {
                aggregate_id: self.state.aggregate_id,
                event_id: Uuid::new_v4(),
                username: "username".to_string(),
            }),
            true,
        )
        .await?;
        Ok(())
    }

    async fn handle_delete(&mut self, _cmd: command::Delete) -> Result<()> {
        self.apply(
            UserEvent::Deleted(event::Deleted {
                aggregate_id: self.state.aggregate_id,
                event_id: Uuid::new_v4(),
            }),
            true,
        )
        .await?;
        Ok(())
    }

    fn proccess_event(&mut self, event: &UserEvent) {
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
