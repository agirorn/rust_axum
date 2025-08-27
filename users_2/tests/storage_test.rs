#![allow(unused)]
use deadpool_postgres::Client;
use serde::Deserialize;
use serde_json::json;
use tokio_postgres::types::Json;
use users_2::event::{self, Envelope, UserEvent};
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

#[pgmt::test(migrations = "../migrations")]
async fn it_works(pool: pgmt::Pool) {
    let db: Client = pool.get().await.unwrap();
    let result = add(2, 2);
    assert_eq!(result, 4);
    let sql = r#"
        SELECT 1 as num;
    "#;
    if let Ok(rows) = db.query(sql, &[]).await {
        let v = rows
            .into_iter()
            .map(|row| row.get("num"))
            .collect::<Vec<i32>>();
        println!("{v:#?}");
    }

    if let Ok(row) = db.query_one(sql, &[]).await {
        let v: i32 = row.get("num");
        println!("{v:#?}");
    }

    let id = Uuid::new_v4();
    let name = "event_name";
    let data = json!({
        "name": "Alice",
        "age": 30
    });

    // let values = vec[&id, &Json(&data)];

    println!("#################################");
    println!("data: {data}");
    println!("#################################");
    let res = db
        .execute(
            r#"
        INSERT INTO events (id, name, data)
        VALUES ($1, $2, $3::json);
        "#,
            &[&id, &name, &Json(&data)],
        )
        .await
        .unwrap();

    #[derive(Deserialize, Debug)]
    struct S {
        name: String,
        age: i32,
    }

    if let Ok(row) = db
        .query_one("SELECT id, name, data from events limit 1;", &[])
        .await
    {
        let id: Uuid = row.get("id");
        let name: String = row.get("name");
        let data: Json<S> = row.get("data");
        let data: S = data.0;
        println!("id: {id:#?}, name: {name:#?}, data: {data:#?}");
    }
}

#[pgmt::test(migrations = "../migrations")]
async fn stream_no_snap_shot(pool: pgmt::Pool) {
    let db = pool.get().await.unwrap();

    let event_created = Envelope::new(
        USER_ID_AGGREGATE_ID,
        event::Created {
            username: "username".to_lowercase(),
        }
        .into(),
    );
    let event_enabled = Envelope::new(USER_ID_AGGREGATE_ID, UserEvent::Enabled);
    insert_event(&db, &event_created).await;
    insert_event(&db, &event_enabled).await;

    // print_event_by_event_id(&db, event_created.event_id).await;
    print_events(&db).await;
    // Insert Created
    // Insert Enabled
    // Stream from EventStore
    // assert first is Created
    // assert second is Enabled
    // No snap shot

    use futures_util::{pin_mut, StreamExt, TryStreamExt};

    let mut it = db
        .query_raw(
            r#"
        SELECT envelope
          FROM user_events
         WHERE aggregate_id = $1
        "#,
            &[&USER_ID_AGGREGATE_ID],
        )
        .await
        .unwrap();

    pin_mut!(it);
    let rows = it.take(2);
    // let rows = it.take(2).await.unwrap();

    let rows = rows
        .map(|r| -> Envelope {
            match r {
                Ok(r) => {
                    let p: Json<Envelope> = r.get("envelope");
                    p.0
                }
                Err(err) => panic!("Error {err}"),
            }
        })
        .collect::<Vec<_>>()
        .await;
    println!("-x-x-x-x-x-x-x-x-x-x->>> data: {:#?}", rows);
    assert_eq!(vec![event_created, event_enabled], rows);

    // while let Some(row) = it.try_next().await.unwrap() {
    // while let Some(row) = it.try_next().await.unwrap() {
    //     let data: Json<Envelope> = row.get("envelope");
    //     let data: Envelope = data.0;
    //     println!("-=-=->>> data: {:#?}", data);
    // }

    println!(" ============== DONE ====================");
}

#[pgmt::test(migrations = "../migrations")]
async fn stream_snap(pool: pgmt::Pool) {
    // Insert Snapshot on offset
    // assert get snapshot events
    // No other events
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
            println!("============ ROWS ==================");
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
