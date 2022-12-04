//! Worker process for executing pending tasks.

use common::{storage, tokio, tokio_postgres, TaskState};
use tracing::{event, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");

    let (mut client, conn) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            event!(Level::ERROR, "connection error: {}", e);
        }
    });

    let row = client
        .query_one(
            "SELECT COUNT(id) FROM tasks WHERE execution_time <= now() AND state = $1",
            &[&TaskState::Pending.as_sql()],
        )
        .await?;
    let backlog: i64 = row.get(0);
    event!(Level::DEBUG, "backlog size: {}", backlog);
    event!(Level::DEBUG, "polling...");
    loop {
        let tx = client.transaction().await?;
        storage::execute_task(tx).await?;
    }
}
