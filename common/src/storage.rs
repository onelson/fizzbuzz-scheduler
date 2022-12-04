use crate::{Filters, Task, TaskState, TaskType, Timestamp, ID};
use anyhow::Result;
use tokio_postgres::{Client, Row, Transaction};
use tracing::{event, Level};

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

pub async fn list(db: &Client, filters: Filters) -> Result<Vec<Task>> {
    // Base query, without optional filters
    let unfiltered = "SELECT * FROM tasks";
    // XXX: Looked at trying to build a vec of params so I could dynamically
    // build up the WHERE clause, but ran into type-level issues (being unable
    // to satisfy `&(dyn ToSql + Sync)` with my `Vec<&str>`.
    // This sort of thing was trivial with `diesel` but complicated here, it seems.
    // The most direct route which seems workable is to have a separate query
    // invocation per combination of filters, but this will not scale well as
    // new filters are added.
    // FIXME: look at somehow implementing `ToSql` for `Filters`.
    let rows = match (filters.kind, filters.state) {
        (Some(kind), Some(state)) => {
            db.query(
                &format!("{} WHERE type = $1 AND state = $2", unfiltered),
                &[&kind.as_sql(), &state.as_sql()],
            )
            .await?
        }
        (Some(kind), None) => {
            db.query(
                &format!("{} WHERE type = $1", unfiltered),
                &[&kind.as_sql()],
            )
            .await?
        }
        (None, Some(state)) => {
            db.query(
                &format!("{} WHERE state = $1", unfiltered),
                &[&state.as_sql()],
            )
            .await?
        }
        (None, None) => db.query(unfiltered, &[]).await?,
    };
    rows.into_iter().map(Task::try_from).collect()
}

pub async fn destroy(db: &Client, id: ID) -> Result<()> {
    db.execute("DELETE FROM tasks WHERE id = $1", &[&id])
        .await?;
    Ok(())
}

/// Picks the next `Pending` task which is ready to execute, runs it, then marks
/// it as `Completed`.
///
/// ## Queuing Details
///
/// Row-level locking are used to drive task selection such that tasks which are
/// already locked will be excluded by the query while whatever task ends up
/// being selected will be locked until the transaction is either committed or
/// rolled back.
///
/// All these details are handled within, and as such the caller should only
/// need to be responsible for starting a new transaction and handing it off
/// here.
pub async fn execute_task(tx: Transaction<'_>) -> Result<()> {
    let maybe_row = tx
        .query_opt(
            r#"
        SELECT * FROM tasks
        WHERE execution_time <= now() AND state = $1
        LIMIT 1
        -- lock the selected row so no other workers will be able to claim it
        FOR UPDATE SKIP LOCKED
        "#,
            &[&TaskState::Pending.as_sql()],
        )
        .await?;
    match maybe_row {
        Some(row) => {
            let task: Task = row.try_into()?;
            event!(Level::DEBUG, "executing task: id={}", task.id);
            task.run().await;
            event!(Level::DEBUG, "marking task completed: id={}", task.id);
            tx.execute(
                "UPDATE tasks SET state = $2, updated_at = now() WHERE id = $1",
                &[&task.id, &TaskState::Completed.as_sql()],
            )
            .await?;
        }
        None => {
            event!(Level::DEBUG, "no pending tasks found");
        }
    }
    tx.commit().await?;
    Ok(())
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
