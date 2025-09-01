use crate::error::Result;
use crate::event::{Envelope, UserEvent};
use crate::state::UserState;
use async_trait::async_trait;
use deadpool_postgres::{Client, Pool};
use eventsourced_core::{BoxEventStream, EventStore};
// use futures::StreamExt;
use futures::{Stream, StreamExt, TryStreamExt};
use futures_util::stream;
use std::pin::Pin;
use tokio_postgres::types::Json;
use uuid::Uuid;

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

    async fn save(&mut self, _events: &mut Vec<Self::Event>, _state: &Self::State) -> Result<()> {
        unimplemented!("EventStore.save is not implemented for EventStore");
    }

    async fn event_stream(
        &self,
        id: Self::AggregateId,
    ) -> BoxEventStream<Self::Event, Self::Error> {
        let db = self.pool.get().await.unwrap();
        let sql = r#"
            WITH s AS (
              SELECT jsonb_build_object(
                       'aggregate_id',   r.aggregate_id,
                       'event_id',       r.aggregate_id,
                       'timestamp',      r."timestamp",
                       'occ_version',    r.occ_version,
                       'aggregate_type', 'user',
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
        Ok(db
            .query_raw(sql, &[&id])
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
