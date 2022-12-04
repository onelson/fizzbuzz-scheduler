//! Mostly lifted from <https://github.com/tokio-rs/axum/blob/main/examples/tokio-postgres>

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
pub use bb8::{Pool, PooledConnection};
pub use bb8_postgres::PostgresConnectionManager;
pub use common::tokio_postgres::NoTls;

pub struct DbConn(pub PooledConnection<'static, PostgresConnectionManager<NoTls>>);
type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

#[async_trait]
impl<S> FromRequestParts<S> for DbConn
where
    ConnectionPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = ConnectionPool::from_ref(state);
        let conn = pool
            .get_owned()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Self(conn))
    }
}
