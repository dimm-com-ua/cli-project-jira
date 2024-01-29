use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    NavigateToEpicDetail { epic_id: u32 },
    NavigateToStoryDetail { epic_id: u32, story_id: u32 },
    NavigateToPreviousPage,
    CreateEpic,
    UpdateEpicStatus { epic_id: u32 },
    DeleteEpic { epic_id: u32 },
    CreateStory { epic_id: u32 },
    UpdateStoryStatus { story_id: u32 },
    DeleteStory { epic_id: u32, story_id: u32 },
    Exit
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Status {
    Open,
    InProgress,
    Resolved,
    Closed,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Open => { write!(f, "OPEN") }
            Status::InProgress => { write!(f, "IN PROGRESS") }
            Status::Resolved => { write!(f, "RESOLVED") }
            Status::Closed => { write!(f, "CLOSED") }
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DBState {
    pub(crate) last_item_id: u32,
    pub(crate) epics: HashMap<u32, Epic>,
    pub(crate) stories: HashMap<u32, Story>
}
