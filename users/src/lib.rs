#![allow(unused)]
#![allow(unused_variables)]
mod command;
mod error;
mod event;
use async_trait::async_trait;
use command::UserCommand;
use error::Result;
use event::UserEvent;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[pgmt::test(migrations = "../migrations")]
    async fn it_works(pool: pgmt::Pool) {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[tokio::test]
    async fn create_user() {
        let cmd = UserCommand::Create(command::Create {
            aggregate_id: uuid::uuid!("d8caf94a-a720-4e25-b592-096802e1e32b"),
            username: "username".to_string(),
        });
        let event_store = Arc::new(Mutex::new(TestEventStore::default()));
        Commander::new(event_store.clone())
            .execute(cmd)
            .await
            .unwrap();
        let event_store = event_store.lock().await;
        assert_eq!(event_store.events.len(), 1);
    }
}

#[derive(Deserialize, Serialize)]
pub struct User {
    id: uuid::Uuid,
    username: String,
}

#[async_trait]
trait UserCommander {
    async fn execute(&mut self, cmd: UserCommand) -> Result<()>;
}

struct Commander {
    store: Arc<Mutex<dyn EventStore>>,
}

impl Commander {
    pub fn new(store: Arc<Mutex<dyn EventStore>>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl UserCommander for Commander {
    async fn execute(&mut self, cmd: UserCommand) -> Result<()> {
        // Missing from this function
        // The aggregate id to load the aggregate on from the event store and the cache
        // The cached user state
        // The pipeline for other inline updates for on command execution updates.
        match cmd {
            UserCommand::Create(cmd) => {
                let events = vec![UserEvent::Created(event::Created {
                    aggregate_id: cmd.aggregate_id,
                    event_id: uuid::Uuid::new_v4(),
                    username: cmd.username,
                })];
                {
                    let mut store = self.store.lock().await;
                    store.write(events).await?;
                }
            }

            UserCommand::Delete(_cmd) => {}
            UserCommand::Enable(_cmd) => {}
            UserCommand::Disable(_cmd) => {}
            UserCommand::SetPassword(_cmd) => {}
        }
        Ok(())
    }
}

#[async_trait]
trait EventStore: Send + Sync {
    async fn read(&self, aggregate_id: uuid::Uuid) -> Result<User>;
    async fn write(&mut self, events: Vec<UserEvent>) -> Result<()>;
}

#[derive(Default)]
pub struct TestEventStore {
    pub events: Vec<UserEvent>,
}

#[async_trait]
impl EventStore for TestEventStore {
    async fn read(&self, _aggregate_id: uuid::Uuid) -> Result<User> {
        Ok(User {
            id: uuid::uuid!("6b3300ad-9761-43d2-8036-3e1c90cb60ee"),
            username: "username".to_string(),
        })
    }
    async fn write(&mut self, mut events: Vec<UserEvent>) -> Result<()> {
        self.events.append(&mut events);
        Ok(())
    }
}
