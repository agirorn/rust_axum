use crate::command::{self, UserCommand};
use crate::error::{Error, Result};
use crate::event::{self, UserEvent};
use crate::state::UserState;
// use crate::store::UserEventStore;
use async_trait::async_trait;
use eventsourced_core::{Aggregate, EventStore, EventStoreFor};
use uuid::Uuid;

#[derive(Default, Debug)]
pub struct User {
    events: Vec<UserEvent>,
    state: UserState,
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

    async fn load_from<ES>(
        _event_store: &mut ES,
        aggregate_id: Self::AggregateId,
    ) -> Self::LoadResult
    where
        ES: EventStoreFor<Self>,
    {
        let user = User::new(aggregate_id);
        // let stream = event_store.stream(aggregate_id).await;
        // while let Some(event) = stream.next().await {
        //     user.apply(event, false);
        // }
        Ok(user)
    }

    async fn save_to<ES>(&mut self, _event_store: &mut ES) -> Self::Result
    where
        ES: EventStoreFor<Self>,
    {
        // event_store.write(self.get_uncommitted_events().await?);
        Ok(())
    }

    async fn apply(&mut self, event: UserEvent, save: bool) -> Self::Result {
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
            state: UserState { aggregate_id },
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
}
