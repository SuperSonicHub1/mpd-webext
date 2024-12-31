use axum::http::StatusCode;

use axum::response::Response;

use axum::response::IntoResponse;
use tokio::signal;

/// https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs
pub(crate) async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

/// Make our own error that wraps `anyhow::Error`.
/// https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs
pub(crate) struct AppError(anyhow::Error);

pub(crate) type AppResult<T> = Result<T, AppError>;

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "Something went wrong: {}\n{}\n{}",
                self.0,
                self.0.root_cause(),
                self.0.backtrace()
            ),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
