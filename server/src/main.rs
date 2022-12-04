//! HTTP service for creating tasks and checking their status.

use crate::{
    database::{DbConn, NoTls, Pool, PostgresConnectionManager},
    errors::HandlerError,
};
use anyhow::Context;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response, Result},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use tracing::{event, Level};

mod database;
mod errors;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");

    let manager =
        PostgresConnectionManager::new_from_stringlike(&db_url, NoTls).expect("db conn manager");
    let pool = Pool::builder().build(manager).await.expect("db pooler");

    let conn = pool.get().await.expect("init connection");
    common::storage::init_schema(&conn)
        .await
        .with_context(|| "failed to configure database schema")
        .expect("init schema");

    drop(conn);

    let app = Router::new()
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/:task_id", get(read_task).delete(destroy_task))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    event!(Level::DEBUG, "listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("server run");
}

type Resp = Result<Response, HandlerError>;

#[derive(Deserialize)]
struct CreateTaskRequest {
    #[serde(rename = "type")]
    kind: common::TaskType,
    execution_time: common::Timestamp,
}

async fn create_task(DbConn(conn): DbConn, Json(payload): Json<CreateTaskRequest>) -> Resp {
    let task_id = common::storage::create(&conn, &payload.kind, &payload.execution_time).await?;
    Ok((StatusCode::CREATED, Json(json!({ "id": task_id }))).into_response())
}

async fn read_task(DbConn(conn): DbConn, Path(task_id): Path<common::ID>) -> Resp {
    let task = common::storage::read(&conn, task_id).await?;
    Ok(match task {
        None => StatusCode::NOT_FOUND.into_response(),
        Some(_) => Json(task).into_response(),
    })
}

async fn destroy_task(DbConn(conn): DbConn, Path(task_id): Path<common::ID>) -> Resp {
    // Deletions should be idempotent in so much as deleting an already
    // non-existent task should give a 204, same as a regular deletion.
    common::storage::destroy(&conn, task_id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn list_tasks(DbConn(conn): DbConn, Query(filters): Query<common::Filters>) -> Resp {
    let tasks = common::storage::list(&conn, filters).await?;
    Ok(Json(json!({ "results": tasks })).into_response())
}
