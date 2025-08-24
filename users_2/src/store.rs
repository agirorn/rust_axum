use crate::error::Result;
use crate::event::UserEvent;
use crate::state::UserState;
use async_trait::async_trait;
use eventsourced_core::EventStore;
use uuid::Uuid;

#[derive(Default, Debug)]
pub struct UserEventStore {
    events: Vec<UserEvent>,
    states: std::collections::HashMap<Uuid, UserState>,
}

impl UserEventStore {
    #[cfg(test)]
    pub fn get_events(&self) -> Vec<UserEvent> {
        self.events.clone()
    }

    #[cfg(test)]
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    #[cfg(test)]
    pub fn get_event(&self, index: usize) -> Option<&UserEvent> {
        self.events.get(index)
    }

    #[cfg(test)]
    pub fn get_state_for(&self, aggregate_id: &Uuid) -> &UserState {
        self.states.get(aggregate_id).unwrap()
    }
}

#[async_trait]
impl EventStore for UserEventStore {
    type Event = UserEvent;
    type State = UserState;
    type Result = Result<()>;

    async fn save(&mut self, events: &mut Vec<UserEvent>, state: &UserState) -> Self::Result {
        self.events.append(events);
        let state = state.clone();
        self.states.insert(state.aggregate_id, state);
        Ok(())
    }
}
