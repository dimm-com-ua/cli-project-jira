use std::fs;
use std::fs::File;
use std::io::{Read, Write};

use anyhow::{anyhow, Result};

use crate::models::{DBState, Epic, Status, Story};

pub struct ProjectsDatabase {
    pub database: Box<dyn Database>
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
        for id in &state.epics
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

pub trait Database {
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

pub mod test_utils {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use super::*;

    pub struct MockDb {
        last_written_state: RefCell<DBState>
    }

    impl MockDb {
        pub fn new() -> Self {
            Self {
                last_written_state: RefCell::new(DBState {
                    last_item_id: 0,
                    epics: HashMap::new(),
                    stories: HashMap::new()
                })
            }
        }
    }
    
    impl Database for MockDb {
        fn read_db(&self) -> anyhow::Result<DBState> {
            let state = self.last_written_state.borrow().clone();
            Ok(state)
        }

        fn write_db(&self, db_state: &DBState) -> anyhow::Result<()> {
            let latest_state = &self.last_written_state;
            *latest_state.borrow_mut() = db_state.clone();
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_utils::MockDb;

    #[test]
    fn create_epic_should_work() {
        let db = ProjectsDatabase { database: Box::new(MockDb::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());

        // TODO: fix this error by deriving the appropriate traits for Epic
        let result = db.create_epic(epic.clone());

        assert_eq!(result.is_ok(), true);

        let id = result.unwrap();
        let db_state = db.read_db().unwrap();

        let expected_id = 1;

        assert_eq!(id, expected_id);
        assert_eq!(db_state.last_item_id, expected_id);
        assert_eq!(db_state.epics.get(&id), Some(&epic));
    }

    #[test]
    fn create_story_should_error_if_invalid_epic_id() {
        let db = ProjectsDatabase { database: Box::new(MockDb::new()) };
        let story = Story::new("".to_owned(), "".to_owned());

        let non_existent_epic_id = 999;

        let result = db.create_story(story, non_existent_epic_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn create_story_should_work() {
        let db = ProjectsDatabase { database: Box::new(MockDb::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        // TODO: fix this error by deriving the appropriate traits for Story
        let result = db.create_story(story.clone(), epic_id);
        assert_eq!(result.is_ok(), true);

        let id = result.unwrap();
        let db_state = db.read_db().unwrap();

        let expected_id = 2;

        assert_eq!(id, expected_id);
        assert_eq!(db_state.last_item_id, expected_id);
        assert_eq!(db_state.epics.get(&epic_id).unwrap().stories.contains(&id), true);
        assert_eq!(db_state.stories.get(&id), Some(&story));
    }

    #[test]
    fn delete_epic_should_error_if_invalid_epic_id() {
        let db = ProjectsDatabase { database: Box::new(MockDb::new()) };

        let non_existent_epic_id = 999;

        let result = db.delete_epic(non_existent_epic_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_epic_should_work() {
        let db = ProjectsDatabase { database: Box::new(MockDb::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let story_id = result.unwrap();

        let result = db.delete_epic(epic_id);
        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        let expected_last_id = 2;

        assert_eq!(db_state.last_item_id, expected_last_id);
        assert_eq!(db_state.epics.get(&epic_id), None);
        assert_eq!(db_state.stories.get(&story_id), None);
    }

    #[test]
    fn delete_story_should_error_if_invalid_epic_id() {
        let db = ProjectsDatabase { database: Box::new(MockDb::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let story_id = result.unwrap();

        let non_existent_epic_id = 999;

        let result = db.delete_story(non_existent_epic_id, story_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_story_should_error_if_story_not_found_in_epic() {
        let db = ProjectsDatabase { database: Box::new(MockDb::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let non_existent_story_id = 999;

        let result = db.delete_story(epic_id, non_existent_story_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_story_should_work() {
        let db = ProjectsDatabase { database: Box::new(MockDb::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let story_id = result.unwrap();

        let result = db.delete_story(epic_id, story_id);
        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        let expected_last_id = 2;

        assert_eq!(db_state.last_item_id, expected_last_id);
        assert_eq!(db_state.epics.get(&epic_id).unwrap().stories.contains(&story_id), false);
        assert_eq!(db_state.stories.get(&story_id), None);
    }

    #[test]
    fn update_epic_status_should_error_if_invalid_epic_id() {
        let db = ProjectsDatabase { database: Box::new(MockDb::new()) };

        let non_existent_epic_id = 999;

        let result = db.update_epic_status(non_existent_epic_id, Status::Closed);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn update_epic_status_should_work() {
        let db = ProjectsDatabase { database: Box::new(MockDb::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);

        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.update_epic_status(epic_id, Status::Closed);

        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        assert_eq!(db_state.epics.get(&epic_id).unwrap().status, Status::Closed);
    }

    #[test]
    fn update_story_status_should_error_if_invalid_story_id() {
        let db = ProjectsDatabase { database: Box::new(MockDb::new()) };

        let non_existent_story_id = 999;

        let result = db.update_story_status(non_existent_story_id, Status::Closed);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn update_story_status_should_work() {
        let db = ProjectsDatabase { database: Box::new(MockDb::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);

        let story_id = result.unwrap();

        let result = db.update_story_status(story_id, Status::Closed);

        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        assert_eq!(db_state.stories.get(&story_id).unwrap().status, Status::Closed);
    }

    mod database {
        use std::collections::HashMap;
        use std::io::Write;

        use super::*;

        #[test]
        fn read_db_should_fail_with_invalid_path() {
            let db = JSONFileDatabase { file_path: "INVALID_PATH".to_owned() };
            assert_eq!(db.read_db().is_err(), true);
        }

        #[test]
        fn read_db_should_fail_with_invalid_json() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_contents = r#"{ "last_item_id": 0 epics: {} stories {} }"#;
            write!(tmpfile, "{}", file_contents).unwrap();

            let db = JSONFileDatabase { file_path: tmpfile.path().to_str()
                .expect("failed to convert tmpfile path to str").to_string() };

            let result = db.read_db();

            assert_eq!(result.is_err(), true);
        }

        #[test]
        fn read_db_should_parse_json_file() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_contents = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
            write!(tmpfile, "{}", file_contents).unwrap();

            let db = JSONFileDatabase { file_path: tmpfile.path().to_str()
                .expect("failed to convert tmpfile path to str").to_string() };

            let result = db.read_db();

            assert_eq!(result.is_ok(), true);
        }

        #[test]
        fn write_db_should_work() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_contents = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
            write!(tmpfile, "{}", file_contents).unwrap();

            let db = JSONFileDatabase { file_path: tmpfile.path().to_str()
                .expect("failed to convert tmpfile path to str").to_string() };

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