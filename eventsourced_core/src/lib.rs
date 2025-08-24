use async_trait::async_trait;
use futures_core::Stream;
use std::fmt::Debug;
use std::pin::Pin;

/// An aggregate use in event sourcing to represent a hole state built up of multiple smaller events.
/// The aggregate is used to take business decision that produce events that forma a new hols state
/// for the aggregate
#[async_trait]
pub trait Aggregate: Sized {
    type Command;
    type Event;
    type Error;
    type Result;
    type LoadResult;
    type State;
    type AggregateId;

    async fn execute<ES>(event_store: &mut ES, cmd: Self::Command) -> Result<(), Self::Error>
    where
        ES: EventStoreFor<Self>;

    async fn handle_command(&mut self, cmd: Self::Command) -> Self::Result;

    async fn load_from<ES>(event_store: &ES, aggregate_id: Self::AggregateId) -> Self::LoadResult
    where
        ES: EventStoreFor<Self>;

    async fn save_to<ES>(&mut self, event_store: &mut ES) -> Self::Result
    where
        ES: EventStoreFor<Self>;

    async fn apply(&mut self, event: Self::Event, save: bool) -> Self::Result;

    fn get_uncommitted_events(&mut self) -> Vec<Self::Event>;
}

pub type BoxEventStream<E, Err> = Pin<Box<dyn Stream<Item = Result<E, Err>> + Send + 'static>>;

/// EventStore stores all the events and the state for a particular aggregate.
#[async_trait]
pub trait EventStore: Send + Sync + Debug {
    type Event;
    type State;
    type Error;
    // Why does AggregateId have to be Send + Sync + 'static; ?
    type AggregateId: Send + Sync + 'static;

    async fn save(
        &mut self,
        events: &mut Vec<Self::Event>,
        state: &Self::State,
    ) -> Result<(), Self::Error>;

    // Stream all events for an aggregate
    fn event_stream(&self, id: Self::AggregateId) -> BoxEventStream<Self::Event, Self::Error>;
}

/// EventStoreFor is a nice alias for the EventStore
///
///
/// Place it anywhere you'd otherwise do
///
/// where
///   ES: EventStore<Event = Self::Event, State = Self::State, Error = Self::Error>
///
pub trait EventStoreFor<A: Aggregate>:
    EventStore<Event = A::Event, State = A::State, Error = A::Error, AggregateId = A::AggregateId>
{
}

impl<A, ES> EventStoreFor<A> for ES
where
    A: Aggregate,
    ES: EventStore<
        Event = A::Event,
        State = A::State,
        Error = A::Error,
        AggregateId = A::AggregateId,
    >,
{
}
