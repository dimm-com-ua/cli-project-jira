use crate::io_utils::{get_user_input, print_line};
use crate::models::{Epic, Status, Story};

pub struct Prompts {
    pub create_epic: Box<dyn Fn() -> Epic>,
    pub create_story: Box<dyn Fn() -> Story>,
    pub delete_epic: Box<dyn Fn() -> bool>,
    pub delete_story: Box<dyn Fn() -> bool>,
    pub update_status: Box<dyn Fn() -> Option<Status>>
}

impl Prompts {
    pub fn new() -> Self {
        Self {
            create_epic: Box::new(create_epic_prompt),
            create_story: Box::new(create_story_prompt),
            delete_epic: Box::new(delete_epic_prompt),
            delete_story: Box::new(delete_story_prompt),
            update_status: Box::new(update_status_prompt),
        }
    }
}

fn create_epic_prompt() -> Epic {
    print_line();
    println!("Epic name: ");
    let epic_name = get_user_input();
    println!("Epic description: ");
    let epic_description = get_user_input();
    let epic = Epic::new(
        epic_name.trim().to_owned(), epic_description.trim().to_owned()
    );
    epic
}

fn create_story_prompt() -> Story {
    print_line();
    println!("Story name: ");
    let story_name = get_user_input();
    println!("Story description: ");
    let story_description = get_user_input();
    let story = Story::new(
        story_name.trim().to_owned(),
        story_description.trim().to_owned());
    story
}

fn delete_epic_prompt() -> bool {
    print_line();
    println!("Are you sure you want to delete this epic? All stories in this epic will be deleted too [Y/n]:");
    let input = get_user_input();
    if input.trim().eq("Y") {
        return true;
    }
    false
}

fn delete_story_prompt() -> bool {
    print_line();
    println!("Are you sure you want to delete this story? [Y/n]:");
    let input = get_user_input();
    if input.trim().eq("Y") {
        return true;
    }
    false
}

fn update_status_prompt() -> Option<Status> {
    print_line();
    println!("New status (1 - OPEN, 2 - IN-PROGRESS, 3 - RESOLVED, 4 - CLOSED): ");
    let status = get_user_input();
    let status = status.trim().parse::<u8>();

    if let Ok(status) = status {
        return match status {
            1 => {
                Some(Status::Open)
            }
            2 => {
                Some(Status::InProgress)
            }
            3 => {
                Some(Status::Resolved)
            }
            4 => {
                Some(Status::Closed)
            }
            _ => None
        }
    }
    None
}