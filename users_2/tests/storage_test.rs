#![allow(unused)]
use chrono::{DateTime, Utc};
use deadpool_postgres::Client;
use eventsourced_core::EventStore;
use pretty_assertions::assert_eq;
// use futures_util::TryStreamExt;
// use futures::TryStreamExt;
use futures_util::{pin_mut, StreamExt, TryStreamExt};
use serde::de::{value, Expected};
// use futures::{StreamExt, TryStreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio_postgres::types::Json;
// use tokio_stream::StreamExt;
use users_2::event::{self, Envelope, UserEvent};
use users_2::state::{self, UserState};
use users_2::UserEventStore;
use uuid::Uuid;

const USER_ID_AGGREGATE_ID: Uuid = uuid::uuid!("aba80c9b-21c6-4fee-b046-7b069f8d9120");

#[test]
fn test_this() {
    let v = true;
    assert!(v, "the variable v should be true");
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

// #[pgmt::test(migrations = "../migrations")]
// async fn it_works(pool: pgmt::Pool) {
//     let db: Client = pool.get().await.unwrap();
//     let result = add(2, 2);
//     assert_eq!(result, 4);
//     let sql = r#"
//         SELECT 1 as num;
//     "#;
//     if let Ok(rows) = db.query(sql, &[]).await {
//         let v = rows
//             .into_iter()
//             .map(|row| row.get("num"))
//             .collect::<Vec<i32>>();
//         println!("{v:#?}");
//     }
//
//     if let Ok(row) = db.query_one(sql, &[]).await {
//         let v: i32 = row.get("num");
//         println!("{v:#?}");
//     }
//
//     let id = Uuid::new_v4();
//     let name = "event_name";
//     let data = json!({
//         "name": "Alice",
//         "age": 30
//     });
//
//     // let values = vec[&id, &Json(&data)];
//
//     println!("#################################");
//     println!("data: {data}");
//     println!("#################################");
//     let res = db
//         .execute(
//             r#"
//         INSERT INTO events (id, name, data)
//         VALUES ($1, $2, $3::json);
//         "#,
//             &[&id, &name, &Json(&data)],
//         )
//         .await
//         .unwrap();
//
//     #[derive(Deserialize, Debug)]
//     struct S {
//         name: String,
//         age: i32,
//     }
//
//     if let Ok(row) = db
//         .query_one("SELECT id, name, data from events limit 1;", &[])
//         .await
//     {
//         let id: Uuid = row.get("id");
//         let name: String = row.get("name");
//         let data: Json<S> = row.get("data");
//         let data: S = data.0;
//         println!("id: {id:#?}, name: {name:#?}, data: {data:#?}");
//     }
// }

#[pgmt::test(migrations = "../migrations")]
async fn stream_no_snap_shot(pool: pgmt::Pool) {
    let db = pool.get().await.unwrap();

    let event_created = Envelope::new(
        USER_ID_AGGREGATE_ID,
        1,
        event::Created {
            username: "username".to_lowercase(),
        }
        .into(),
    );
    let event_enabled = Envelope::new(USER_ID_AGGREGATE_ID, 2, UserEvent::Enabled);
    insert_event(&db, &event_created).await;
    insert_event(&db, &event_enabled).await;
    let store = UserEventStore::new(pool);
    let mut stream = store.event_stream(USER_ID_AGGREGATE_ID).await.unwrap();
    let mut rows: Vec<_> = store
        .event_stream(USER_ID_AGGREGATE_ID)
        .await
        .unwrap()
        .take(2)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(vec![event_created.clone(), event_enabled.clone()], rows);
}

// #[pgmt::test(migrations = "../migrations")]
// async fn stream_snap(pool: pgmt::Pool) {
//     // Insert Snapshot on offset
//     // assert get snapshot events
//     // No other events
// }

#[pgmt::test(migrations = "../migrations")]
async fn stream_snap_and_created(pool: pgmt::Pool) {
    // Insert Created
    // Insert Snapshot on offset 1
    // Stream from EventStore
    // assert get snapshot event
    // No other events

    let db = pool.get().await.unwrap();
    let created_event = Envelope::new(
        USER_ID_AGGREGATE_ID,
        1,
        event::Created {
            username: "username".to_lowercase(),
        }
        .into(),
    );
    let state = UserState {
        event_name: "snapshot".to_string(),
        aggregate_id: USER_ID_AGGREGATE_ID,
        username: "username".to_string(),
        has_password: false,
        exists: true,
        enabled: true,
        password_hash: None,
        occ_version: 1,
    };
    insert_event(&db, &created_event).await;
    insert_snapshot(&db, &state).await;

    let store = UserEventStore::new(pool);
    let mut stream = store.event_stream(USER_ID_AGGREGATE_ID).await.unwrap();
    let mut rows: Vec<_> = store
        .event_stream(USER_ID_AGGREGATE_ID)
        .await
        .unwrap()
        .take(2)
        .try_collect()
        .await
        .unwrap();
    let mut snapshot_event = Envelope::new(USER_ID_AGGREGATE_ID, 1, UserEvent::Snapshot(state));
    // This timestamp  should not by like this.
    snapshot_event.timestamp = rows[0].timestamp;
    // Is is a geed ides to re-use the aggregate_id as the event id.
    // Might is be better to just generate a uuid or simply store an id on the snapshot.
    // does any of this matter
    snapshot_event.event_id = snapshot_event.aggregate_id;
    assert_eq!(vec![snapshot_event.clone()], rows);
}

#[pgmt::test(migrations = "../migrations")]
async fn save_events_test(pool: pgmt::Pool) {
    let db = pool.get().await.unwrap();
    let created_event = Envelope::new(
        USER_ID_AGGREGATE_ID,
        1,
        event::Created {
            username: "username".to_lowercase(),
        }
        .into(),
    );
    let state = UserState {
        event_name: "snapshot".to_string(),
        aggregate_id: USER_ID_AGGREGATE_ID,
        username: "username".to_string(),
        has_password: false,
        exists: true,
        enabled: true,
        password_hash: None,
        occ_version: 1,
    };
    let mut store = UserEventStore::new(pool.clone());
    let mut events = vec![created_event.clone()];
    store.save(&mut events, &state).await.unwrap();
    let db = pool.get().await.unwrap();
    assert_eq!(
        get_events(&db).await.unwrap(),
        vec![EventRow {
            event_id: created_event.event_id,
            event_name: "created".to_string(),
            aggregate_type: "user".to_string(),
            aggregate_id: USER_ID_AGGREGATE_ID,
            envelope: created_event
        }]
    );

    let states = get_states(&db).await.unwrap();
    let expected = vec![StateRow {
        aggregate_id: USER_ID_AGGREGATE_ID,
        occ_version: 1,
        timestamp: states[0].timestamp,
        state: UserState {
            event_name: "snapshot".to_string(),
            aggregate_id: USER_ID_AGGREGATE_ID,
            username: "username".to_string(),
            has_password: false,
            occ_version: 1,
            exists: true,
            enabled: true,
            password_hash: None,
        },
    }];
    assert_eq!(states, expected);
}

#[pgmt::test(migrations = "../migrations")]
async fn stream_snap_and_no_event(pool: pgmt::Pool) {
    // Insert Created
    // Insert Enabled
    // Insert Snapshot on offset of enabled
    // Stream from EventStore
    // assert get snapshot event
    // No other events
}

#[pgmt::test(migrations = "../migrations")]
async fn stream_snap_and_one_moer_event(pool: pgmt::Pool) {
    // Insert Created
    // Insert Enabled
    // Insert Snapshot on offset of Created
    // Stream from EventStore
    // assert get snapshot event
    // assert next event is Enabled
    // No other events
}

async fn insert_event(db: &Client, envelope: &Envelope) {
    let sql = r#"
        INSERT INTO events (envelope)
        VALUES ($1::json);
    "#;
    db.execute(sql, &[&Json(&envelope)]).await.unwrap();
}

async fn insert_snapshot(db: &Client, state: &UserState) {
    let sql = r#"
        INSERT INTO states (state)
        VALUES ($1::json);
    "#;
    db.execute(sql, &[&Json(&state)]).await.unwrap();
}

async fn print_states(db: &Client) {
    let sql = r#"
        SELECT aggregate_id, state, timestamp
        FROM states
    "#;
    let rows = db.query(sql, &[]).await.unwrap();
    println!("============ STATE ==================");
    for row in rows {
        let aggregate_id: Uuid = row.get("aggregate_id");
        let state: Json<UserState> = row.get("state");
        let state: UserState = state.0;
        println!(" ---->>>> aggregate_id: {aggregate_id:#?}");
        println!(" ---->>>> state: {state:#?}");
        println!(" ---->>>> found");
    }
    println!("============ STATE END ==================");
}

#[derive(Debug, PartialEq)]
struct EventRow {
    event_id: Uuid,
    aggregate_type: String,
    aggregate_id: Uuid,
    envelope: Envelope,
    event_name: String,
    // occ_version: u64
}

impl From<tokio_postgres::Row> for EventRow {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            event_name: row.get("event_name"),
            aggregate_type: row.get("aggregate_type"),
            aggregate_id: row.get("aggregate_id"),
            event_id: row.get("event_id"),
            envelope: row.get::<&str, Json<Envelope>>("envelope").0,
        }
    }
}

async fn get_events(db: &Client) -> users_2::error::Result<Vec<EventRow>> {
    let sql = r#"
        SELECT event_id, aggregate_type, aggregate_id, envelope, event_name
        FROM events
    "#;
    Ok(db
        .query(sql, &[])
        .await?
        .into_iter()
        .map(|row| row.into())
        .collect())
}

#[derive(Debug, PartialEq)]
struct StateRow {
    aggregate_id: Uuid,
    occ_version: i64,
    timestamp: DateTime<Utc>,
    state: UserState,
}

impl From<tokio_postgres::Row> for StateRow {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            aggregate_id: row.get("aggregate_id"),
            occ_version: row.get("occ_version"),
            timestamp: row.get("timestamp"),
            state: row.get::<&str, Json<UserState>>("state").0,
        }
    }
}

async fn get_states(db: &Client) -> users_2::error::Result<Vec<StateRow>> {
    let sql = r#"
        SELECT aggregate_id, occ_version, timestamp, state
        FROM states
    "#;
    Ok(db
        .query(sql, &[])
        .await?
        .into_iter()
        .map(|row| row.into())
        .collect())
}

async fn print_events(db: &Client) {
    let sql = r#"
        SELECT event_id, aggregate_type, aggregate_id, envelope, event_name
        FROM events
    "#;
    let rows = db.query(sql, &[]).await.unwrap();
    println!("============ events ==================");
    for row in rows {
        let event_id: Uuid = row.get("event_id");
        println!("============ ROW -> {event_id} ==================");
        let event_name: String = row.get("event_name");
        let aggregate_type: String = row.get("aggregate_type");
        let aggregate_id: Uuid = row.get("aggregate_id");
        let envelope: Json<Envelope> = row.get("envelope");
        let envelope: Envelope = envelope.0;
        println!(" ---->>>> event_id: {event_id:#?}");
        println!(" ---->>>> aggregate_id: {aggregate_id:#?}");
        println!(" ---->>>> event_name: {event_name:#?}");
        println!(" ---->>>> aggregate_type: {aggregate_type:#?}");
        println!(" ---->>>> data: {envelope:#?}");
        println!(" ---->>>> found");
    }
    println!("============ events END ==================");
}

async fn print_event_by_event_id(db: &Client, id: Uuid) {
    let sql = r#"
        SELECT event_id, aggregate_type, aggregate_id, envelope, event_name
        FROM events
        WHERE event_id = $1
        LIMIT 1;
    "#;
    let row = db.query_one(sql, &[&id]).await.unwrap();
    let event_id: Uuid = row.get("event_id");
    let event_name: String = row.get("event_name");
    let aggregate_type: String = row.get("aggregate_type");
    let aggregate_id: Uuid = row.get("aggregate_id");
    let envelope: Json<Envelope> = row.get("envelope");
    let envelope: Envelope = envelope.0;
    println!(" ---->>>> event_id: {event_id:#?}");
    println!(" ---->>>> aggregate_id: {aggregate_id:#?}");
    println!(" ---->>>> event_name: {event_name:#?}");
    println!(" ---->>>> aggregate_type: {aggregate_type:#?}");
    println!(" ---->>>> data: {envelope:#?}");
    println!(" ---->>>> found");
}
