#![allow(unused)]
#![allow(unused_variables)]
mod command;
mod error;
mod event;
mod result;
use async_trait::async_trait;
use command::UserCommand;
use error::Result;
use event::UserEvent;
use result::UserResult;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[derive(Clone, Default, Deserialize, Serialize, Debug, PartialEq, Eq)]
struct UserState {
    aggregate_id: Uuid,
}

mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const USER_ID_AGGREGATE_ID: Uuid = uuid::uuid!("aba80c9b-21c6-4fee-b046-7b069f8d9120");

    #[pgmt::test(migrations = "../migrations")]
    async fn it_works(pool: pgmt::Pool) {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    async fn creat_user(event_store: &mut UserEventStore) -> Result<()> {
        User::execute(
            event_store,
            UserCommand::Create(command::Create {
                aggregate_id: USER_ID_AGGREGATE_ID,
                username: "username".to_string(),
            }),
        )
        .await
        .unwrap();
        Ok(())
    }

    #[tokio::test]
    async fn create_user() {
        let mut event_store = UserEventStore::default();

        creat_user(&mut event_store).await.unwrap();
        assert_eq!(event_store.event_count(), 1);
        let event = event_store.get_event(0).unwrap();
        assert_eq!(
            event,
            &UserEvent::Created(event::Created {
                aggregate_id: USER_ID_AGGREGATE_ID,
                event_id: event.get_event_id(),
                username: "username".to_string(),
            })
        );
        assert_eq!(
            event_store.get_state_for(&USER_ID_AGGREGATE_ID),
            &UserState {
                aggregate_id: USER_ID_AGGREGATE_ID,
            },
        );
    }

    #[tokio::test]
    async fn delete_user() {
        let mut event_store = UserEventStore::default();
        creat_user(&mut event_store).await.unwrap();
        User::execute(
            &mut event_store,
            UserCommand::Delete(command::Delete {
                aggregate_id: USER_ID_AGGREGATE_ID,
            }),
        )
        .await
        .unwrap();
        assert_eq!(event_store.event_count(), 2);
        let event = event_store.get_event(1).unwrap();
        assert_eq!(
            event,
            &UserEvent::Deleted(event::Deleted {
                aggregate_id: USER_ID_AGGREGATE_ID,
                event_id: event.get_event_id(),
            })
        );
    }
}

#[async_trait]
trait Aggregate: Sized {
    type Command;
    type Event;
    type Result;
    type State;
    type Store: EventStore<Event = Self::Event, State = Self::State>;

    async fn execute(event_store: &mut Self::Store, cmd: Self::Command) -> Result<Self::Result>;
    async fn handle_command(&mut self, cmd: Self::Command) -> Result<Self::Result>;
    async fn load_from(event_store: &mut Self::Store, aggregate_id: Uuid) -> Result<Self>;
    async fn save_to(&mut self, event_store: &mut Self::Store) -> Result<()>;
    async fn apply(&mut self, event: Self::Event, save: bool) -> Result<()>;
    fn get_uncommitted_events(&mut self) -> Vec<Self::Event>;
}

#[async_trait]
trait EventStore: Send + Debug {
    type Event;
    type State;
    async fn save(&mut self, events: &mut Vec<Self::Event>, state: &Self::State) -> Result<()>;
}

#[derive(Default, Debug, Deserialize, Serialize)]
struct User {
    events: Vec<UserEvent>,
    state: UserState,
}

impl User {
    pub fn new(aggregate_id: Uuid) -> Self {
        Self {
            events: vec![],
            state: UserState { aggregate_id },
        }
    }
}

#[async_trait]
impl Aggregate for User {
    type Command = UserCommand;
    type Result = ();
    type Event = UserEvent;
    type State = UserState;
    type Store = UserEventStore;

    async fn execute(event_store: &mut Self::Store, cmd: Self::Command) -> Result<Self::Result> {
        let mut aggregate = User::load_from(event_store, cmd.aggregate_id()).await?;
        let result = aggregate.handle_command(cmd).await?;
        event_store
            .save(&mut aggregate.get_uncommitted_events(), &aggregate.state)
            .await?;
        Ok(())
    }

    // The result here should probably be Self::CommandResult
    async fn handle_command(&mut self, cmd: Self::Command) -> Result<Self::Result> {
        match cmd {
            UserCommand::Create(cmd) => {
                self.apply(
                    UserEvent::Created(event::Created {
                        aggregate_id: self.state.aggregate_id,
                        event_id: Uuid::new_v4(),
                        username: "username".to_string(),
                    }),
                    true,
                )
                .await?;
                // uuid::uuid!("aba80c9b-21c6-4fee-b046-7b069f8d9120")
            }
            UserCommand::Delete(cmd) => {
                self.apply(
                    UserEvent::Deleted(event::Deleted {
                        aggregate_id: self.state.aggregate_id,
                        event_id: Uuid::new_v4(),
                    }),
                    true,
                )
                .await?;
            }
            UserCommand::Enable(cmd) => {
                unimplemented!("UserCommand::Delete");
            }
            UserCommand::Disable(cmd) => {
                unimplemented!("UserCommand::Delete");
            }
            UserCommand::SetPassword(cmd) => {
                unimplemented!("UserCommand::Delete");
            }
        }
        Ok(())
    }

    async fn load_from(event_store: &mut Self::Store, aggregate_id: Uuid) -> Result<User> {
        let user = User::new(aggregate_id);
        // let stream = event_store.stream(aggregate_id).await;
        // while let Some(event) = stream.next().await {
        //     user.apply(event, false);
        // }
        Ok(user)
    }
    async fn save_to(&mut self, event_store: &mut Self::Store) -> Result<()> {
        // event_store.write(self.get_uncommitted_events().await?);
        Ok(())
    }
    async fn apply(&mut self, event: UserEvent, save: bool) -> Result<()> {
        if save {
            self.events.push(event);
        }
        Ok(())
    }

    // UserEvent should probably be Self::Event here
    fn get_uncommitted_events(&mut self) -> Vec<Self::Event> {
        self.events.clone()
    }
}

#[derive(Default, Debug)]
struct UserEventStore {
    events: Vec<UserEvent>,
    states: std::collections::HashMap<Uuid, UserState>,
}

impl UserEventStore {
    fn get_events(&self) -> Vec<UserEvent> {
        self.events.clone()
    }

    fn event_count(&self) -> usize {
        self.events.len()
    }

    fn get_event(&self, index: usize) -> Option<&UserEvent> {
        self.events.get(index)
    }
    fn get_state_for(&self, aggregate_id: &Uuid) -> &UserState {
        self.states.get(aggregate_id).unwrap()
    }
}

#[async_trait]
impl EventStore for UserEventStore {
    type Event = UserEvent;
    type State = UserState;
    async fn save(&mut self, events: &mut Vec<UserEvent>, state: &UserState) -> Result<()> {
        self.events.append(events);
        let state = state.clone();
        self.states.insert(state.aggregate_id, state);
        Ok(())
    }
}
