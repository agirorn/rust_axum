use crate::error::Result;
use crate::event::{Envelope, UserEvent};
use crate::state::UserState;
use async_trait::async_trait;
use deadpool_postgres::{Client, Pool};
use eventsourced_core::{BoxEventStream, EventStore};
// use tokio_postgres::types::ToSql;
// use futures::StreamExt;
use futures::{Stream, StreamExt, TryStreamExt};
use futures_util::stream;
use serde_json::Value as JsonValue;
use std::pin::Pin;
use tokio_postgres::types::Json;
use tokio_postgres::types::{FromSql, IsNull, ToSql, Type};
use uuid::Uuid; // from tokio-postgres

#[derive(Debug)]
pub struct UserEventStore {
    pub pool: Pool,
}

impl UserEventStore {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventStore for UserEventStore {
    type Event = Envelope;
    type State = UserState;
    type Error = crate::error::Error;
    type AggregateId = Uuid;

    // async fn save(&mut self, events: &mut Vec<Self::Event>, state: &Self::State) -> Result<()> {
    //     let mut client = self.pool.get().await.unwrap();
    //     let tx = client.build_transaction().start().await?;
    //
    //     // Build one SQL statement. We keep your two INSERTs but fuse them with CTEs
    //     // so it remains ONE statement (extended protocol requirement).
    //     let mut sql = String::from(
    //         r#"
    //         WITH upsert_state AS (
    //             INSERT INTO states (state)
    //             VALUES ($1)
    //             ON CONFLICT (aggregate_id)
    //             DO UPDATE SET
    //                 state = EXCLUDED.state
    //             RETURNING 1
    //         )
    //         "#,
    //     );
    //     // Keep Json wrappers alive here
    //     let mut owned_params: Vec<Json<&JsonValue>> = Vec::new();
    //     owned_params.push(Json(&serde_json::json!(&state)));
    //     // // Bind params: $1 is the state JSON, followed by one $n per event payload
    //     // let mut params: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(1 + events.len());
    //     // params.push(&Json(&state));
    //
    //     if !events.is_empty() {
    //         sql.push_str(
    //             r#"
    //             , insert_events AS (
    //                 INSERT INTO events (envelope)
    //                 VALUES
    //             "#,
    //         );
    //
    //         // Append "($2), ($3), ..." for all events
    //         use std::fmt::Write as _;
    //         let mut first = true;
    //         let mut next_idx = 2;
    //         for ev in events.iter() {
    //             if !first {
    //                 sql.push_str(", ");
    //             }
    //             first = false;
    //             write!(&mut sql, "(${})", next_idx).unwrap();
    //
    //             owned_params.push(Json(&serde_json::json!(&ev)));
    //             next_idx += 1;
    //         }
    //
    //         sql.push_str(
    //             r#"
    //                 RETURNING 1
    //             )
    //             SELECT 1
    //             "#,
    //         );
    //     } else {
    //         // No events this time; still return something so it's a valid statement.
    //         sql.push_str(" SELECT 1 ");
    //     }
    //
    //     // Execute the single statement with dynamic parameters
    //     tx.execute_raw(&sql, owned_params.iter().copied()).await?;
    //
    //     tx.commit().await?;
    //     events.clear(); // optional
    //     Ok(())
    // }

    async fn save(&mut self, events: &mut Vec<Self::Event>, state: &Self::State) -> Result<()> {
        // unimplemented!("EventStore.save is not implemented for EventStore");
        // for projection in projections {
        //     prjection(events, state).await?
        // }

        // Get client
        let mut client = self.pool.get().await.unwrap();

        // Begin transaction
        let transaction = client.build_transaction().start().await?;

        // Prepare statements
        let stmt_insert_event = transaction
            .prepare(
                r#"
                INSERT INTO user_events (envelope)
                VALUES ($1)
                "#,
            )
            .await?;

        let stmt_upsert_state = transaction
            .prepare(
                r#"
                INSERT INTO states (state)
                VALUES ($1)
                ON CONFLICT (aggregate_id)
                DO UPDATE SET
                    state    = EXCLUDED.state
                "#,
            )
            .await?;

        // Insert/Upsert state
        transaction
            .execute(&stmt_upsert_state, &[&Json(&state)])
            .await?;

        // Insert events
        println!("--->>> events");
        for event in events.iter() {
            println!("event {}", event.event_id);
            transaction
                .execute(&stmt_insert_event, &[&Json(&event)])
                .await?;
        }

        // Commit
        transaction.commit().await?;

        // Clear buffer (optional)
        events.clear();

        Ok(())
    }

    async fn event_stream(
        &self,
        id: Self::AggregateId,
    ) -> BoxEventStream<Self::Event, Self::Error> {
        // TODO: remove the unwrap and add the real error to the Self::Error object
        let db = self.pool.get().await.unwrap();
        let sql = r#"
            WITH s AS (
              SELECT jsonb_build_object(
                       'aggregate_id',   r.aggregate_id,
                       'event_id',       r.aggregate_id,
                       'timestamp',      r."timestamp",
                       'occ_version',    r.occ_version,
                       'aggregate_type', $2::text,
                       'data', jsonb_build_object(
                         'data', r.state,
                         'event_name', 'snapshot'
                       )
                     ) AS envelope,
                     r.aggregate_id,
                     r.occ_version
                FROM (
                       SELECT state, aggregate_id, occ_version, "timestamp"
                         FROM states
                        WHERE aggregate_id = $1
                        LIMIT 1
                     ) as r
            )
            SELECT x.envelope
              FROM (
                     -- SNAPSHOT ROW (only if state exists) → build the desired JSON shape
                     SELECT s.envelope, s.aggregate_id, s.occ_version FROM s
                     UNION ALL
                     -- EVENTS NEWER THAN THE SNAPSHOT (or > 0 if no snapshot)
                     SELECT e.envelope, e.aggregate_id, e.occ_version
                     FROM user_events e
                     LEFT JOIN s ON s.aggregate_id = e.aggregate_id
                     WHERE e.aggregate_id = $1
                       AND e.occ_version > COALESCE(s.occ_version, 0)
                   ) AS x
             ORDER BY occ_version;
        "#;
        let aggregate_type = "user";
        let args: Vec<&(dyn ToSql + Sync)> = vec![
            &id as &(dyn ToSql + Sync),
            &aggregate_type as &(dyn ToSql + Sync),
        ];
        Ok(db
            .query_raw(sql, args)
            .await
            .unwrap()
            .and_then(|row| async move {
                let j = row.get::<&str, Json<serde_json::Value>>("envelope");
                Ok(row.get::<&str, Json<Envelope>>("envelope").into())
            })
            .map_err(|e| e.into())
            .boxed())
    }
}
