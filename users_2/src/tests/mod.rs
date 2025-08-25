mod store;
use crate::aggregate::User;
use crate::command::{self, UserCommand};
use crate::error::Result;
use crate::event::{self, Envelope, UserEvent};
use crate::state::UserState;
use crate::tests::store::TestUserEventStore as Users;
use eventsourced_core::{Aggregate, EventStoreFor};
use pretty_assertions::assert_eq;
use uuid::Uuid;

const USER_ID_AGGREGATE_ID: Uuid = uuid::uuid!("aba80c9b-21c6-4fee-b046-7b069f8d9120");

async fn creat_user(event_store: &mut impl EventStoreFor<User>) -> Result<()> {
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
    let mut store = Users::default();
    creat_user(&mut store).await.unwrap();
    assert_eq!(store.event_count(), 1);
    let event = store.get_event(0).unwrap();
    // TODO: assert that the event.timestamp is close to now and in the past
    // TODO: assert that the event.event_id is av4 UUID

    assert_eq!(event.event_id, event.event_id);
    assert_eq!(event.timestamp, event.timestamp);
    assert_eq!(event.aggregate_id, USER_ID_AGGREGATE_ID);
    assert_eq!(
        event.data,
        UserEvent::Created(event::Created {
            username: "username".to_string(),
        })
    );
    let expected_state = UserState {
        aggregate_id: USER_ID_AGGREGATE_ID,
        username: "username".to_string(),
        has_password: false,
        exists: true,
        enabled: true,
        password_hash: None,
    };
    let aggregate = User::load(&store, USER_ID_AGGREGATE_ID).await.unwrap();
    assert_eq!(expected_state, aggregate.state);
    assert_eq!(expected_state, store.get_state_for(&USER_ID_AGGREGATE_ID));
}

#[tokio::test]
async fn delete_user() {
    let mut store = Users::default();
    creat_user(&mut store).await.unwrap();
    User::execute(
        &mut store,
        UserCommand::Delete(command::Delete {
            aggregate_id: USER_ID_AGGREGATE_ID,
        }),
    )
    .await
    .unwrap();
    assert_eq!(store.event_count(), 2);
    let event = store.get_event(1).unwrap();
    assert_eq!(event.aggregate_id, USER_ID_AGGREGATE_ID);
    assert_eq!(event.data, UserEvent::Deleted);
    let expected_state = UserState {
        aggregate_id: USER_ID_AGGREGATE_ID,
        username: "username".to_string(),
        has_password: false,
        exists: false,
        enabled: true,
        password_hash: None,
    };
    let aggregate = User::load(&store, USER_ID_AGGREGATE_ID).await.unwrap();
    assert_eq!(expected_state, aggregate.state);
    assert_eq!(expected_state, store.get_state_for(&USER_ID_AGGREGATE_ID));
}

#[tokio::test]
async fn set_password() {
    let mut store = Users::default();
    creat_user(&mut store).await.unwrap();
    User::execute(
        &mut store,
        UserCommand::SetPassword(command::SetPassword {
            aggregate_id: USER_ID_AGGREGATE_ID,
            password: "password".to_string(),
        }),
    )
    .await
    .unwrap();
    assert_eq!(store.event_count(), 2);
    let event = store.get_event(1).unwrap();
    let mut password_hash = "".to_string();
    if let UserEvent::NewPassword(event) = event.clone().data {
        password_hash = event.password_hash;
    }
    assert_eq!(event.aggregate_id, USER_ID_AGGREGATE_ID);
    assert_eq!(
        event.data,
        UserEvent::NewPassword(event::NewPassword {
            password_hash: password_hash.clone(),
        })
    );
    assert!(bcrypt::verify("password", &password_hash).unwrap());
    let expected_state = UserState {
        aggregate_id: USER_ID_AGGREGATE_ID,
        username: "username".to_string(),
        has_password: true,
        exists: true,
        enabled: true,
        password_hash: Some(password_hash),
    };
    let aggregate = User::load(&store, USER_ID_AGGREGATE_ID).await.unwrap();
    assert_eq!(expected_state, aggregate.state);
    assert_eq!(expected_state, store.get_state_for(&USER_ID_AGGREGATE_ID));
}

#[tokio::test]
async fn disable_then_enable_user() {
    let mut store = Users::default();
    creat_user(&mut store).await.unwrap();
    User::execute(
        &mut store,
        UserCommand::Disable(command::Disable {
            aggregate_id: USER_ID_AGGREGATE_ID,
        }),
    )
    .await
    .unwrap();
    assert_eq!(store.event_count(), 2);
    let event = store.get_last_event().unwrap();
    assert_eq!(event.aggregate_id, USER_ID_AGGREGATE_ID);
    assert_eq!(event.data, UserEvent::Disabled);
    let expected_state = UserState {
        aggregate_id: USER_ID_AGGREGATE_ID,
        username: "username".to_string(),
        has_password: false,
        exists: true,
        enabled: false,
        password_hash: None,
    };
    assert_eq!(expected_state, store.get_state_for(&USER_ID_AGGREGATE_ID));

    User::execute(
        &mut store,
        UserCommand::Enable(command::Enable {
            aggregate_id: USER_ID_AGGREGATE_ID,
        }),
    )
    .await
    .unwrap();
    assert_eq!(store.event_count(), 3);
    let event = store.get_last_event().unwrap();
    assert_eq!(event.aggregate_id, USER_ID_AGGREGATE_ID);
    assert_eq!(event.data, UserEvent::Enabled);
    let expected_state = UserState {
        aggregate_id: USER_ID_AGGREGATE_ID,
        username: "username".to_string(),
        has_password: false,
        exists: true,
        enabled: true,
        password_hash: None,
    };

    assert_eq!(expected_state, store.get_state_for(&USER_ID_AGGREGATE_ID));
}
