use crate::error::Result;
use crate::event::UserEvent;
use crate::state::UserState;
use async_trait::async_trait;
use eventsourced_core::{BoxEventStream, EventStore};
use futures_util::stream;
use uuid::Uuid;

#[derive(Default, Debug)]
pub struct UserEventStore {}

impl UserEventStore {}

#[async_trait]
impl EventStore for UserEventStore {
    type Event = UserEvent;
    type State = UserState;
    type Error = crate::error::Error;
    type AggregateId = Uuid;

    async fn save(&mut self, _events: &mut Vec<Self::Event>, _state: &Self::State) -> Result<()> {
        unimplemented!("EventStore.save is not implemented for EventStore");
    }

    fn stream_events(&self, _id: Self::AggregateId) -> BoxEventStream<Self::Event, Self::Error> {
        let items: Vec<_> = vec![];
        Box::pin(stream::iter(items))
    }
}
