use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::thread::spawn;
use anyhow::{anyhow, Result};

use crate::models::{DBState, Epic, Status, Story};

pub struct ProjectsDatabase {
    database: Box<dyn Database>
}

impl ProjectsDatabase {
    pub fn new(file_path: String) -> Self {
        ProjectsDatabase {
            database: Box::new(JSONFileDatabase {
                file_path,
            })
        }
    }

    pub fn read_db(&self) -> Result<DBState> {
        self.database.read_db()
    }

    pub fn create_epic(&self, epic: Epic) -> Result<u32> {
        let mut state = self.read_db()?;
        let current_id = state.last_item_id + 1;
        state.last_item_id = current_id;

        state.epics.insert(current_id, epic);
        self.database.write_db(&state)?;
        Ok(current_id)
    }

    pub fn create_story(&self, story: Story, epic_id: u32) -> Result<u32> {
        let mut state = self.read_db()?;
        let current_id = state.last_item_id + 1;

        state.last_item_id = current_id;
        state.stories.insert(current_id, story);
        state.epics
            .get_mut(&epic_id)
            .ok_or_else(|| anyhow!("Epic not found!"))?
            .stories
            .push(current_id);
        self.database.write_db(&state)?;
        Ok(current_id)
    }

    pub fn delete_epic(&self, epic_id: u32) -> Result<()> {
        let mut state = self.read_db()?;
        for id in state.epics
            .get(&epic_id)
            .ok_or_else(|| anyhow!("Epic with such id not found!"))?
            .stories {
            state.stories.remove(&id);
        }
        state.epics.remove(&epic_id);
        self.database.write_db(&state)?;
        Ok(())
    }

    pub fn delete_story(&self, epic_id: u32, story_id: u32) -> Result<()> {
        let mut state = self.read_db()?;
        let epic = state
            .epics
            .get_mut(&epic_id)
            .ok_or_else(|| anyhow!("Epic with such id not found!"))?;
        let story_idx = epic
            .stories
            .iter()
            .position(|id| id == &story_id)
            .ok_or_else(|| anyhow!("Story not found"))?;

        epic.stories.remove(story_idx);

        state.stories.remove(&story_id);

        self.database.write_db(&state)?;
        Ok(())
    }

    pub fn update_epic_status(&self, epic_id: u32, status: Status) -> Result<()> {
        let mut state = self.read_db()?;
        state
            .epics
            .get_mut(&epic_id)
            .ok_or_else(|| anyhow!("Epic with such id not found!"))?
            .status = status;

        self.database.write_db(&state)?;
        Ok(())
    }

    pub fn update_story_status(&self, story_id: u32, status: Status) -> Result<()> {
        let mut state = self.read_db()?;
        state
            .stories
            .get_mut(&story_id)
            .ok_or_else(|| anyhow!("Story with such id not found!"))?
            .status = status;

        self.database.write_db(&state)?;
        Ok(())
    }
}

trait Database {
    fn read_db(&self) -> Result<DBState>;
    fn write_db(&self, db_state: &DBState) -> Result<()>;
}

struct JSONFileDatabase {
    pub file_path: String
}

impl Database for JSONFileDatabase {
    fn read_db(&self) -> Result<DBState> {
        let mut file = File::open(&self.file_path)?;
        let mut data: String = "".to_owned();
        file.read_to_string(&mut data)?;

        let db_state: DBState = serde_json::from_str(&data)?;
        Ok(db_state)
    }

    fn write_db(&self, db_state: &DBState) -> Result<()> {
        let data = serde_json::to_string(db_state)?;
        fs::write(&self.file_path, data)
            .expect("Can't write to file");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod database {
        use std::collections::HashMap;
        use crate::models::{Epic, Status, Story};
        use super::*;

        #[test]
        fn read_db_should_fail_with_invalid_path() {
            let db = JSONFileDatabase { file_path: "WRONG_PATH".to_owned() };
            assert_eq!(db.read_db().is_err(), true);
        }

        #[test]
        fn read_db_should_fail_with_invalid_json() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
            let file_content = r#"{"last_item_id": 0 epics: {} stories {}}"#;
            write!(tmpfile, "{}", file_content).unwrap();

            let db = JSONFileDatabase { file_path: tmpfile.path().to_str()
                .expect("Failed to convert tmpfile path to str").to_string() };

            let result = db.read_db();

            assert_eq!(result.is_err(), true);
        }

        #[test]
        fn read_db_should_parse_json_file() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
            let file_content = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
            write!(tmpfile, "{}", file_content).unwrap();

            let db = JSONFileDatabase { file_path: tmpfile.path().to_str()
                .expect("Failed to convert tmpfile path to str").to_string() };

            let result = db.read_db();

            assert_eq!(result.is_ok(), true);
        }

        #[test]
        fn write_db_should_work() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_content = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
            write!(tmpfile, "{}", file_content).unwrap();

            let db = JSONFileDatabase { file_path: tmpfile.path().to_str()
                .expect("Failed to convert tmpfile path to str").to_string()};

            let story = Story { name: "epic 1".to_owned(), description: "epic 1".to_owned(), status: Status::Open };
            let epic = Epic { name: "epic 1".to_owned(), description: "epic 1".to_owned(), status: Status::Open, stories: vec![2] };

            let mut stories = HashMap::new();
            stories.insert(2, story);

            let mut epics = HashMap::new();
            epics.insert(1, epic);

            let state = DBState { last_item_id: 2, epics, stories };

            let write_result = db.write_db(&state);
            let read_result = db.read_db().unwrap();

            assert_eq!(write_result.is_ok(), true);
            assert_eq!(read_result, state);

        }
    }
}