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

mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const USER_ID_AGGREGATE_ID: Uuid = uuid::uuid!("aba80c9b-21c6-4fee-b046-7b069f8d9120");

    #[pgmt::test(migrations = "../migrations")]
    async fn it_works(pool: pgmt::Pool) {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    async fn creat_user(
        event_store: &mut (impl EventStore<UserEvent> + Send + Debug),
    ) -> Result<()> {
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

    async fn execute(
        event_store: &mut (impl EventStore<Self::Event> + Send + Debug),
        cmd: Self::Command,
    ) -> Result<Self::Result>;
    async fn handle_command(&mut self, cmd: Self::Command) -> Result<Self::Result>;
    async fn load_from(
        event_store: &mut (impl EventStore<Self::Event> + Send + Debug),
        aggregate_id: Uuid,
    ) -> Result<Self>;
    async fn save_to(
        &mut self,
        event_store: &mut (impl EventStore<Self::Event> + Send + Debug),
    ) -> Result<()>;
    async fn apply(&mut self, event: Self::Event, save: bool) -> Result<()>;
    fn get_uncommitted_events(&mut self) -> Vec<Self::Event>;
}

#[async_trait]
trait EventStore<Event> {
    async fn save(&mut self, events: &mut Vec<Event>) -> Result<()>;
}

#[derive(Default, Debug, Deserialize, Serialize)]
struct User {
    aggregate_id: Uuid,
    events: Vec<UserEvent>,
}

impl User {
    pub fn new(aggregate_id: Uuid) -> Self {
        Self {
            events: vec![],
            aggregate_id,
        }
    }
}

#[async_trait]
impl Aggregate for User
where
    UserEvent: Debug,
    UserCommand: Debug,
{
    type Command = UserCommand;
    type Result = ();
    type Event = UserEvent;

    async fn execute(
        event_store: &mut (impl EventStore<UserEvent> + Send + Debug),
        cmd: Self::Command,
    ) -> Result<Self::Result> {
        let mut aggregate = User::load_from(event_store, cmd.aggregate_id()).await?;
        let result = aggregate.handle_command(cmd).await?;
        event_store
            .save(&mut aggregate.get_uncommitted_events())
            .await?;
        Ok(())
    }

    // The result here should probably be Self::CommandResult
    async fn handle_command(&mut self, cmd: Self::Command) -> Result<Self::Result> {
        match cmd {
            UserCommand::Create(cmd) => {
                self.apply(
                    UserEvent::Created(event::Created {
                        aggregate_id: self.aggregate_id,
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
                        aggregate_id: self.aggregate_id,
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

    async fn load_from(
        event_store: &mut (impl EventStore<UserEvent> + Send + Debug),
        aggregate_id: Uuid,
    ) -> Result<User> {
        let user = User::new(aggregate_id);
        // let stream = event_store.stream(aggregate_id).await;
        // while let Some(event) = stream.next().await {
        //     user.apply(event, false);
        // }
        Ok(user)
    }
    async fn save_to(
        &mut self,
        event_store: &mut (impl EventStore<UserEvent> + Send + Debug),
    ) -> Result<()> {
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
}

#[async_trait]
impl EventStore<UserEvent> for UserEventStore {
    async fn save(&mut self, events: &mut Vec<UserEvent>) -> Result<()> {
        self.events.append(events);
        Ok(())
    }
}

// #[derive(Deserialize, Serialize)]
// pub struct User {
//     id: Uuid,
//     username: String,
// }

// #[async_trait]
// trait UserCommander {
//     async fn execute(&mut self, cmd: UserCommand) -> Result<()>;
// }
//
// struct Commander {
//     store: Arc<Mutex<dyn EventStore>>,
// }
//
// impl Commander {
//     pub fn new(store: Arc<Mutex<dyn EventStore>>) -> Self {
//         Self { store }
//     }
// }
//
// #[async_trait]
// impl UserCommander for Commander {
//     async fn execute(&mut self, cmd: UserCommand) -> Result<()> {
//         // Missing from this function
//         // The aggregate id to load the aggregate on from the event store and the cache
//         // The cached user state
//         // The pipeline for other inline updates for on command execution updates.
//         match cmd {
//             UserCommand::Create(cmd) => {
//                 let events = vec![UserEvent::Created(event::Created {
//                     aggregate_id: cmd.aggregate_id,
//                     event_id: Uuid::new_v4(),
//                     username: cmd.username,
//                 })];
//                 {
//                     let mut store = self.store.lock().await;
//                     store.write(events).await?;
//                 }
//             }
//
//             UserCommand::Delete(_cmd) => {}
//             UserCommand::Enable(_cmd) => {}
//             UserCommand::Disable(_cmd) => {}
//             UserCommand::SetPassword(_cmd) => {}
//         }
//         Ok(())
//     }
// }
//
// #[async_trait]
// trait EventStore: Send + Sync {
//     async fn read(&self, aggregate_id: Uuid) -> Result<User>;
//     async fn write(&mut self, events: Vec<UserEvent>) -> Result<()>;
// }
//
// #[derive(Default)]
// pub struct TestEventStore {
//     pub events: Vec<UserEvent>,
// }
//
// #[async_trait]
// impl EventStore for TestEventStore {
//     async fn read(&self, _aggregate_id: Uuid) -> Result<User> {
//         Ok(User {
//             id: uuid::uuid!("6b3300ad-9761-43d2-8036-3e1c90cb60ee"),
//             username: "username".to_string(),
//         })
//     }
//     async fn write(&mut self, mut events: Vec<UserEvent>) -> Result<()> {
//         self.events.append(&mut events);
//         Ok(())
//     }
// }
