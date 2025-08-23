use async_trait::async_trait;
use std::fmt::Debug;

/// An aggregate use in event sourcing to represent a hole state built up of multiple smaller events.
/// The aggregate is used to take business decision that produce events that forma a new hols state
/// for the aggreggate
#[async_trait]
pub trait Aggregate: Sized {
    type Command;
    type Event;
    type Result;
    type LoadResult;
    type State;
    type AggregateId;
    type Store: EventStore<Event = Self::Event, State = Self::State>;

    async fn execute(event_store: &mut Self::Store, cmd: Self::Command) -> Self::Result;
    async fn handle_command(&mut self, cmd: Self::Command) -> Self::Result;
    async fn load_from(
        event_store: &mut Self::Store,
        aggregate_id: Self::AggregateId,
    ) -> Self::LoadResult;
    async fn save_to(&mut self, event_store: &mut Self::Store) -> Self::Result;
    async fn apply(&mut self, event: Self::Event, save: bool) -> Self::Result;
    fn get_uncommitted_events(&mut self) -> Vec<Self::Event>;
}

/// EventStore stores all the events and the state for a particular aggregate.
#[async_trait]
pub trait EventStore: Send + Debug {
    type Event;
    type State;
    type Result;
    async fn save(&mut self, events: &mut Vec<Self::Event>, state: &Self::State) -> Self::Result;
}
