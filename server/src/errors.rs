//! Error handling glue, largely lifted from the axum examples...

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub struct HandlerError(anyhow::Error);

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        // XXX: Should be possible to "downcast" here to switch up the status
        // code if needed.
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for HandlerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
