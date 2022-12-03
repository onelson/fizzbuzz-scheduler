//! Common code shared by the worker and http service.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub type ID = i32;
pub type Timestamp = DateTime<Utc>;

#[derive(Debug, Serialize)]
pub struct Task {
    pub id: ID,
    pub kind: TaskType,
    pub execution_time: Timestamp,
    pub state: TaskState,
    pub created: Timestamp,
    pub updated: Timestamp,
}

impl Task {
    pub fn run(&self) {
        let sleep_secs = match self.kind {
            TaskType::Fizz => 3,
            TaskType::Buzz => 5,
            TaskType::FizzBuzz => 15,
        };

        std::thread::sleep(std::time::Duration::from_secs(sleep_secs));
        println!("{} {}", self.kind, self.id);
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TaskType {
    Fizz,
    Buzz,
    FizzBuzz,
}

impl Display for TaskType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TaskType::Fizz => "Fizz",
                TaskType::Buzz => "Buzz",
                TaskType::FizzBuzz => "Fizz Buzz",
            }
        )
    }
}

// N.b. we don't really have any fallible tasks, but if we did we could
// represent that as a state.
#[derive(Debug, Deserialize, Serialize)]
pub enum TaskState {
    Pending,
    Completed,
}

#[derive(Debug, Default, Deserialize)]
pub struct Filters {
    pub state: Option<TaskState>,
    #[serde(rename = "type")]
    pub kind: Option<TaskType>,
}

pub mod storage;
