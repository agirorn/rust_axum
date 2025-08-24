use async_trait::async_trait;
use std::fmt::Debug;

/// An aggregate use in event sourcing to represent a hole state built up of multiple smaller events.
/// The aggregate is used to take business decision that produce events that forma a new hols state
/// for the aggreggate
#[async_trait]
pub trait Aggregate: Sized {
    type Command;
    type Event;
    type Error;
    type Result;
    type LoadResult;
    type State;
    type AggregateId;
    // The store must be a generic type so you can svap switch from testing to production store
    // type Store: EventStore<Event = Self::Event, State = Self::State>;

    async fn execute<ES>(event_store: &mut ES, cmd: Self::Command) -> Result<(), Self::Error>
    where
        ES: EventStore<Event = Self::Event, State = Self::State, Error = Self::Error>;

    async fn handle_command(&mut self, cmd: Self::Command) -> Self::Result;

    async fn load_from<ES>(
        event_store: &mut ES,
        aggregate_id: Self::AggregateId,
    ) -> Self::LoadResult
    where
        ES: EventStore<Event = Self::Event, State = Self::State>;

    async fn save_to<ES>(&mut self, event_store: &mut ES) -> Self::Result
    where
        ES: EventStore<Event = Self::Event, State = Self::State>;

    async fn apply(&mut self, event: Self::Event, save: bool) -> Self::Result;

    fn get_uncommitted_events(&mut self) -> Vec<Self::Event>;
}

/// EventStore stores all the events and the state for a particular aggregate.
#[async_trait]
pub trait EventStore: Send + Debug {
    type Event;
    type State;
    type Error;

    async fn save(
        &mut self,
        events: &mut Vec<Self::Event>,
        state: &Self::State,
    ) -> Result<(), Self::Error>;
}
