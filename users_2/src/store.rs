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
        // let items: Vec<_> = vec![];
        // Box::pin(stream::iter(items))

        let db = self.pool.get().await.unwrap();
        let mut it = db
            .query_raw(
                r#"
        SELECT envelope
          FROM user_events
         WHERE aggregate_id = $1
        "#,
                &[&id],
            )
            .await
            .unwrap();
        Ok(it
            .and_then(|row| async move {
                let p: Json<Envelope> = row.get("envelope");
                Ok(p.0)
            })
            .map_err(|e| e.into())
            .boxed())
    }
}
