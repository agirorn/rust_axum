use crate::aggregate::User;
use crate::command::{self, UserCommand};
use crate::error::Result;
use crate::event::{self, UserEvent};
use crate::result::UserResult;
use crate::state::UserState;
use crate::store::UserEventStore;
use async_trait::async_trait;
use eventsourced_core::{Aggregate, EventStore};
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

const USER_ID_AGGREGATE_ID: Uuid = uuid::uuid!("aba80c9b-21c6-4fee-b046-7b069f8d9120");

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
