//! HTTP service for creating tasks and checking their status.

use anyhow::Context;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use errors::HandlerResult;
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use tracing::{event, Level};

mod errors;

// FIXME: naive to blindly reach for a fresh connection every time.
//   Better would be to use a connection pooler and supply it to handlers
//   via shared state.
fn get_db() -> anyhow::Result<core::storage::Connection> {
    // FIXME: better to expose config via CLI (with optional env vars).
    let dsn =
        std::env::var("DATABASE_URL").with_context(|| "DATABASE_URL is required but unset")?;
    core::storage::Connection::new(&dsn).with_context(|| "failed to connect to database")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/{}", get(read_task).delete(destroy_task));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    event!(Level::DEBUG, "listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct CreateTaskRequest {
    #[serde(rename = "type")]
    kind: core::TaskType,
    execution_time: core::Timestamp,
}

async fn create_task(Json(payload): Json<CreateTaskRequest>) -> HandlerResult {
    let conn = get_db()?;
    let task_id = core::storage::create(&conn, payload.kind, payload.execution_time)?;
    Ok((StatusCode::CREATED, Json(json!({ "id": task_id }))).into_response())
}

async fn read_task(Path(task_id): Path<core::ID>) -> HandlerResult {
    let conn = get_db()?;
    // TODO: this handler should 404 if the task is None
    let task = core::storage::read(&conn, task_id)?;
    Ok((StatusCode::OK, Json(task)).into_response())
}

async fn destroy_task(Path(task_id): Path<core::ID>) -> HandlerResult {
    let conn = get_db()?;
    // TODO: deletions should be idempotent in so much as deleting an already
    //   non-existent task should give a 204, same as a regular deletion.
    core::storage::destroy(&conn, task_id)?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

async fn list_tasks(Query(filters): Query<core::Filters>) -> HandlerResult {
    let conn = get_db()?;
    let tasks = core::storage::list(&conn, filters)?;
    Ok((StatusCode::OK, Json(json!({ "results": tasks }))).into_response())
}
