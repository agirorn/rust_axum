use crate::error::{Error, Result};
use crate::event::UserEvent;
use crate::state::UserState;
use async_trait::async_trait;
use eventsourced_core::{BoxEventStream, EventStore};
use futures_util::stream;
use uuid::Uuid;

#[derive(Default, Debug)]
pub struct TestUserEventStore {
    events: Vec<UserEvent>,
    states: std::collections::HashMap<Uuid, UserState>,
}

impl TestUserEventStore {
    // pub fn get_events(&self) -> Vec<UserEvent> {
    //     self.events.clone()
    // }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn get_event(&self, index: usize) -> Option<&UserEvent> {
        self.events.get(index)
    }

    pub fn get_last_event(&self) -> Option<&UserEvent> {
        self.events.last()
    }

    pub fn get_state_for(&self, aggregate_id: &Uuid) -> UserState {
        self.states.get(aggregate_id).unwrap().clone()
    }
}

#[async_trait]
impl EventStore for TestUserEventStore {
    type Event = UserEvent;
    type State = UserState;
    type Error = Error;
    type AggregateId = Uuid;

    async fn save(&mut self, events: &mut Vec<Self::Event>, state: &Self::State) -> Result<()> {
        self.events.append(events);
        let state = state.clone();
        self.states.insert(state.aggregate_id, state);
        Ok(())
    }

    fn event_stream(&self, id: Self::AggregateId) -> BoxEventStream<Self::Event, Self::Error> {
        let items: Vec<_> = self
            .events
            .iter()
            .filter(move |a| id == a.get_aggregate_id())
            .map(|e| Ok(e.clone())) // requires MyEvent: Clone
            .collect();

        Box::pin(stream::iter(items))
    }
}
