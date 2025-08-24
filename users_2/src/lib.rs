#![allow(unused)]
#![allow(unused_variables)]
mod aggregate;
mod command;
mod error;
mod event;
mod result;
mod state;
mod store;
#[cfg(test)]
mod tests;
use aggregate::User;
use async_trait::async_trait;
use command::UserCommand;
use error::Result;
use event::UserEvent;
use eventsourced_core::{Aggregate, EventStore};
use result::UserResult;
use serde::{Deserialize, Serialize};
use state::UserState;
use std::{fmt::Debug, sync::Arc};
use store::UserEventStore;
use tokio::sync::Mutex;
use uuid::Uuid;
