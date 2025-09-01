#![allow(unused)]
use deadpool_postgres::Client;
use eventsourced_core::EventStore;
use pretty_assertions::assert_eq;
// use futures_util::TryStreamExt;
// use futures::TryStreamExt;
use futures_util::{pin_mut, StreamExt, TryStreamExt};
// use futures::{StreamExt, TryStreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio_postgres::types::Json;
// use tokio_stream::StreamExt;
use users_2::event::{self, Envelope, UserEvent};
use users_2::state::UserState;
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
    let res = db
        .execute(
            r#"
        INSERT INTO user_events (envelope)
        VALUES ($1::json);
        "#,
            &[&Json(&envelope)],
        )
        .await
        .unwrap();
}

async fn insert_snapshot(db: &Client, state: &UserState) {
    let res = db
        .execute(
            r#"
        INSERT INTO states (state)
        VALUES ($1::json);
        "#,
            &[&Json(&state)],
        )
        .await
        .unwrap();
}

async fn print_state(db: &Client) {
    match db
        .query(
            r#"
            SELECT aggregate_id, state, timestamp
            FROM states
            "#,
            &[],
        )
        .await
    {
        Ok(rows) => {
            println!("============ STATE ==================");
            for row in rows {
                // println!("============ ROW -> {event_id} ==================");
                // let event_name: String = row.get("event_name");
                // let aggregate_type: String = row.get("aggregate_type");
                let aggregate_id: Uuid = row.get("aggregate_id");
                let state: Json<UserState> = row.get("state");
                let state: UserState = state.0;
                // println!(" ---->>>> event_id: {event_id:#?}");
                println!(" ---->>>> aggregate_id: {aggregate_id:#?}");
                // println!(" ---->>>> event_name: {event_name:#?}");
                // println!(" ---->>>> aggregate_type: {aggregate_type:#?}");
                println!(" ---->>>> state: {state:#?}");
                println!(" ---->>>> found");
            }
            println!("============ STATE END ==================");
        }
        Err(err) => {
            println!("Error: {err:#?}");
            panic!("did not get the event");
        }
    }
}

async fn print_events(db: &Client) {
    match db
        .query(
            r#"
            SELECT event_id,
                   aggregate_type,
                   aggregate_id,
                   envelope,
                   event_name
            FROM user_events
            "#,
            &[],
        )
        .await
    {
        Ok(rows) => {
            println!("============ USER_EVENTS ==================");
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
            println!("============ USER_EVENTS END ==================");
        }
        Err(err) => {
            println!("Error: {err:#?}");
            panic!("did not get the event");
        }
    }
}

async fn print_event_by_event_id(db: &Client, id: Uuid) {
    match db
        .query_one(
            r#"
            SELECT event_id,
                   aggregate_type,
                   aggregate_id,
                   envelope,
                   event_name
            FROM user_events
            WHERE event_id = $1
            LIMIT 1;
            "#,
            &[&id],
        )
        .await
    {
        Ok(row) => {
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
        Err(err) => {
            println!("Error: {err:#?}");
            panic!("did not get the event be eventy_id {id}");
        }
    }
}
