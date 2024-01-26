use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Status {
    InProgress,
    Closed,
    Open
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Epic {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) status: Status,
    pub(crate) stories: Vec<u32>
}

impl Epic {
    pub fn new(name: String, description: String) -> Self {
        Epic {
            name, description, status: Status::Open, stories: vec![]
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Story {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) status: Status
}

impl Story {
    pub fn new(name: String, description: String) -> Self {
        Story {
            name, description, status: Status::Open
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DBState {
    pub(crate) last_item_id: u32,
    pub(crate) epics: HashMap<u32, Epic>,
    pub(crate) stories: HashMap<u32, Story>
}
