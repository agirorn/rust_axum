use crate::error::Result;
use crate::event::UserEvent;
use crate::state::UserState;
use async_trait::async_trait;
use eventsourced_core::EventStore;

#[derive(Default, Debug)]
pub struct UserEventStore {}

impl UserEventStore {}

#[async_trait]
impl EventStore for UserEventStore {
    type Event = UserEvent;
    type State = UserState;
    type Error = crate::error::Error;

    async fn save(&mut self, _events: &mut Vec<Self::Event>, _state: &Self::State) -> Result<()> {
        unimplemented!("EventStore.save is not implemented for EventStore");
    }
}
