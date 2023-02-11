// [[file:../remote.note::3d2c01c2][3d2c01c2]]
use super::*;
// 3d2c01c2 ends here

// [[file:../remote.note::415dc72b][415dc72b]]
/// Handle unix/linux signals for graceful shutdown of server
pub async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
// 415dc72b ends here

// [[file:../remote.note::8be5152c][8be5152c]]
mod app_error {
    use crate::common::Error;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};

    // Make our own error that wraps `anyhow::Error`.
    pub struct AppError(Error);

    impl<E> From<E> for AppError
    where
        E: Into<Error>,
    {
        fn from(err: E) -> Self {
            Self(err.into())
        }
    }

    // Tell axum how to convert `AppError` into a response.
    impl IntoResponse for AppError {
        fn into_response(self) -> Response {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {}", self.0),
            )
                .into_response()
        }
    }
}
// 8be5152c ends here

// [[file:../remote.note::908a93c5][908a93c5]]
pub use self::app_error::AppError;
// 908a93c5 ends here
