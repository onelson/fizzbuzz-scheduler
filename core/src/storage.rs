use crate::{Filters, TaskType, Timestamp, ID};
use anyhow::Result;

/// Stand-in for a client connection from a yet-to-be-decided SQL crate/
pub struct Connection {}

impl Connection {
    pub fn new(_dsn: &str) -> Result<Self> {
        todo!()
    }
}

pub fn create(_conn: &Connection, _kind: TaskType, _execution_time: Timestamp) -> Result<ID> {
    todo!()
}

pub fn read(_conn: &Connection, _id: ID) -> Result<Option<crate::Task>> {
    todo!()
}

pub fn list(_conn: &Connection, _filters: Filters) -> Result<Vec<crate::Task>> {
    todo!()
}

pub fn destroy(_conn: &Connection, _id: ID) -> Result<()> {
    todo!()
}

pub fn execute_task(_conn: &Connection) -> Result<()> {
    // TODO: Use row locking to select the next pending task, run it, then
    //   update the status.
    todo!()
}
