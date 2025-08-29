use async_trait::async_trait;
use futures_core::Stream;
use futures_util::StreamExt;
use std::fmt::Debug;
use std::pin::Pin;

/// An aggregate use in event sourcing to represent a hole state built up of multiple smaller events.
/// The aggregate is used to take business decision that produce events that forma a new hols state
/// for the aggregate
#[async_trait]
pub trait Aggregate: Sized {
    type Command: Command<Self::AggregateId> + Send + 'static;
    type CommandResult: Send + 'static;
    type Event: Send + 'static;
    type Error;
    type State;
    type AggregateId: Send + 'static;

    /// Implement this function to handle any command that should be executed on the Aggregate.
    async fn handle(&mut self, cmd: Self::Command) -> Result<Self::CommandResult, Self::Error>;

    /// Applies each event on to the aggregate.
    ///
    /// This function is only applying facts on to the aggregate and should need other systems to
    /// get to the connect state and should should not call out to other systems to get any
    /// information. All the facts should be in the events that are applied.
    fn apply(&mut self, event: Self::Event, save: bool) -> Result<(), Self::Error>;

    /// Implement this function to return any event that needs to be persisted after a successfully
    /// handling of a command.
    fn get_uncommitted_events(&mut self) -> Vec<Self::Event>;

    /// Return the current state of the Aggregate.
    fn get_state(&self) -> &Self::State;

    /// Create a new fresh Aggregate by aggregate_id
    fn new(aggregate_id: &Self::AggregateId) -> Self;

    /// Full execution of a command with persistence.
    ///
    /// The default implementation loads the Aggregate into memory and then streams in all the
    /// saved events for the aggregate and applies them one after the other to get the aggregate
    /// state up to date before executing the command using the handle function and then storing
    /// the any new events in the event store.
    async fn execute<ES>(
        event_store: &mut ES,
        cmd: Self::Command,
    ) -> Result<Self::CommandResult, Self::Error>
    where
        ES: EventStoreFor<Self>,
    {
        // Default implementation for the aggregate execute so that all aggregates don't have to,
        let mut aggregate = Self::load(event_store, cmd.aggregate_id()).await?;
        let result = aggregate.handle(cmd).await?;
        event_store
            .save(
                &mut aggregate.get_uncommitted_events(),
                aggregate.get_state(),
            )
            .await?;
        Ok(result)
    }

    /// Loads a single aggregate form the event store.
    ///
    /// The default implementation does this by constructing a new Aggregate and the applying all
    /// events stored in the event store to the Aggregate
    async fn load<ES>(
        event_store: &ES,
        aggregate_id: Self::AggregateId,
    ) -> Result<Self, Self::Error>
    where
        ES: EventStoreFor<Self>,
    {
        // Default implementation for loading the aggregate from the EventStore
        let mut user = Self::new(&aggregate_id);
        let mut events = event_store.event_stream(aggregate_id).await?;
        while let Some(event) = events.next().await {
            let event = event?;
            user.apply(event, false)?;
        }
        Ok(user)
    }
}

pub trait Command<Id> {
    fn aggregate_id(&self) -> Id; // or &Id if you prefer borrowing
}

pub type BoxEventStream<E, Err> =
    Result<Pin<Box<dyn Stream<Item = Result<E, Err>> + Send + 'static>>, Err>;

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
    async fn event_stream(&self, id: Self::AggregateId)
        -> BoxEventStream<Self::Event, Self::Error>;
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
