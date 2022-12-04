use crate::{Filters, TaskType, Timestamp, ID};
use anyhow::Result;
use tokio_postgres::Client;

/// (Optionally) create tables needed for the scheduler.
///
/// This is a bit of a hack in so much as irl you'd want a proper migration
/// system, separate from all the application concerns.
/// Since this is "just a test" more similar to a toy/proof of concept project
/// we can settle for using `IF NOT EXISTS` guards and run schema creation
/// DDL during process startup.
pub async fn init_schema(db: &Client) -> Result<()> {
    let _ = db
        .batch_execute(
            r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id SERIAL PRIMARY KEY,
            -- Sized for the larger type seen today: `FizzBuzz`.
            type VARCHAR(8) NOT NULL,
            -- Sized for the larger states seen today: `Completed`
            state VARCHAR(9) NOT NULL,
            execution_time TIMESTAMPTZ NOT NULL,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        );

        -- Effectively this drives worker task selection.
        CREATE INDEX IF NOT EXISTS queue_idx ON tasks (state, execution_time);
        "#,
        )
        .await?;
    Ok(())
}

pub async fn create(_db: &Client, _kind: TaskType, _execution_time: Timestamp) -> Result<ID> {
    todo!()
}

pub async fn read(_db: &Client, _id: ID) -> Result<Option<crate::Task>> {
    todo!()
}

pub async fn list(_db: &Client, _filters: Filters) -> Result<Vec<crate::Task>> {
    todo!()
}

pub async fn destroy(_db: &Client, _id: ID) -> Result<()> {
    todo!()
}

pub async fn execute_task(_db: &Client) -> Result<()> {
    // TODO: Use row locking to select the next pending task, run it, then
    //   update the status.
    todo!()
}
