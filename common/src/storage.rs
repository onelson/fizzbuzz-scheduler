use crate::{Filters, Task, TaskState, TaskType, Timestamp, ID};
use anyhow::Result;
use tokio_postgres::{Client, Row};

/// (Optionally) create tables needed for the scheduler.
///
/// This is a bit of a hack in so much as irl you'd want a proper migration
/// system, separate from all the application concerns.
/// Since this is "just a test" more similar to a toy/proof of concept project
/// we can settle for using `IF NOT EXISTS` guards and run schema creation
/// DDL during process startup.
pub async fn init_schema(db: &Client) -> Result<()> {
    db.batch_execute(
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

pub async fn create(db: &Client, kind: &TaskType, execution_time: &Timestamp) -> Result<ID> {
    let row = db
        .query_one(
            r#"
    INSERT INTO tasks (type, execution_time, state, created_at, updated_at)
    VALUES ($1, $2, $3, now(), now())
    RETURNING id
    "#,
            &[&kind.as_sql(), execution_time, &TaskState::Pending.as_sql()],
        )
        .await?;
    Ok(row.get(0))
}

pub async fn read(db: &Client, id: ID) -> Result<Option<Task>> {
    let maybe_row = db
        .query_opt(r#"SELECT * from tasks WHERE id = $1"#, &[&id])
        .await?;

    Ok(match maybe_row {
        Some(row) => {
            let task = row.try_into()?;
            Some(task)
        }
        None => None,
    })
}

pub async fn list(db: &Client, _filters: Filters) -> Result<Vec<Task>> {
    // FIXME: build WHERE clause for `Filters`
    let rows = db.query(r#"SELECT * FROM tasks"#, &[]).await?;
    rows.into_iter().map(Task::try_from).collect()
}

pub async fn destroy(db: &Client, id: ID) -> Result<()> {
    db.execute("DELETE FROM tasks WHERE id = $1", &[&id])
        .await?;
    Ok(())
}

pub async fn execute_task(_db: &Client) -> Result<()> {
    // TODO: Use row locking to select the next pending task, run it, then
    //   update the status.
    todo!()
}

impl TryFrom<Row> for Task {
    type Error = anyhow::Error;

    fn try_from(value: Row) -> std::result::Result<Self, Self::Error> {
        let kind: &str = value.try_get("type")?;
        let state: &str = value.try_get("state")?;

        Ok(Task {
            id: value.try_get("id")?,
            kind: kind.parse()?,
            execution_time: value.try_get("execution_time")?,
            state: state.parse()?,
            created_at: value.try_get("created_at")?,
            updated_at: value.try_get("updated_at")?,
        })
    }
}
