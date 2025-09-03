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

    async fn save(&mut self, events: &mut Vec<Self::Event>, state: &Self::State) -> Result<()> {
        let mut state_values: Vec<Json<JsonValue>> = vec![Json(serde_json::json!(&state))];
        let mut state_sql = String::from(
            r#"
                INSERT INTO states (state) VALUES ($1)
                ON CONFLICT (aggregate_id)
                DO UPDATE SET state = EXCLUDED.state
            "#,
        );
        let mut events_values: Vec<Json<JsonValue>> = vec![];
        let mut events_sql = "".to_string();
        if !events.is_empty() {
            events_sql.push_str(
                r#"
                INSERT INTO user_events (envelope)
                VALUES
                "#,
            );

            // Append "($2), ($3), ..." for all events
            use std::fmt::Write as _;
            let mut first = true;
            let mut idx = 1;
            for ev in events.iter() {
                if !first {
                    events_sql.push_str(", ");
                }
                first = false;
                write!(&mut events_sql, "(${})", idx).unwrap();
                events_values.push(Json(serde_json::json!(&ev)));
                idx += 1;
            }
        }
        let mut client = self.pool.get().await.unwrap();
        let transaction = client.build_transaction().start().await?;
        transaction.execute_raw(&state_sql, &state_values).await?;
        if !events_sql.is_empty() {
            transaction.execute_raw(&events_sql, &events_values).await?;
        }
        transaction.commit().await?;
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
