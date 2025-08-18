use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum UserResult {
    Created(Created),
    Deleted,
    Enabled,
    Disabled,
    PasswordSet,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct Created {
    pub aggregate_id: uuid::Uuid,
}

// #[derive(Deserialize, Serialize, Debug)]
// pub struct Delete {
//     pub aggregate_id: uuid::Uuid,
// }
//
// #[derive(Deserialize, Serialize, Debug)]
// pub struct Enable {
//     pub aggregate_id: uuid::Uuid,
// }
//
// #[derive(Deserialize, Serialize, Debug)]
// pub struct Disable {
//     pub aggregate_id: uuid::Uuid,
// }
//
// #[derive(Deserialize, Serialize, Debug)]
// pub struct SetPassword {
//     pub aggregate_id: uuid::Uuid,
//     pub password: String,
// }
