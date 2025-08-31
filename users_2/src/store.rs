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
                SELECT aggregate_id,
                       occ_version,
                       state AS envelope,
                       "timestamp"
                  FROM states
                 WHERE aggregate_id = $1
                 LIMIT 1
            )
            SELECT x.envelope
            FROM (
                   -- 1) snapshot row (only if state exists) → build the desired JSON shape
                   SELECT jsonb_build_object(
                            'aggregate_id',   s.aggregate_id,
                            'event_id',       s.aggregate_id,
                            'timestamp',      s."timestamp",
                            'occ_version',    s.occ_version,
                            'aggregate_type', 'user',
                            'data',           jsonb_build_object(
                                   'data', s.envelope, -- see note below if not jsonb
                                   'event_name', 'snapshot'
                                   )
                          ) AS envelope,
                          s.aggregate_id,
                          s.occ_version
                   FROM s

                   UNION ALL

                   -- 2) events newer than the snapshot (or > 0 if no snapshot)
                   SELECT e.envelope,      -- leave events as-is
                          e.aggregate_id,
                          e.occ_version
                   FROM user_events e
                   LEFT JOIN s ON s.aggregate_id = e.aggregate_id
                   WHERE e.aggregate_id = $1
                     AND e.occ_version >  COALESCE(s.occ_version, 0)
                 ) AS x
            ORDER BY occ_version;
        "#;

        // let sql = r#"
        //     SELECT envelope
        //       FROM user_events
        //      WHERE aggregate_id = $1
        // "#;
        Ok(db
            .query_raw(sql, &[&id])
            .await
            .unwrap()
            .and_then(|row| async move {
                // println!("---->>> row: {:#?}", row);
                let j = row.get::<&str, Json<serde_json::Value>>("envelope");
                println!("---->>> row: {:#?}", j);
                // Ok(row.get::<&str, Json<Envelope>>("envelope").into())
                Ok(row.get::<&str, Json<Envelope>>("envelope").into())
            })
            .map_err(|e| e.into())
            .boxed())
    }
}
