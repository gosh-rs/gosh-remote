// [[file:../../remote.note::3d2c01c2][3d2c01c2]]
use super::Server;

use axum::Json;
use std::net::SocketAddr;
// 3d2c01c2 ends here

// [[file:../../remote.note::8be5152c][8be5152c]]
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

// Make our own error that wraps `anyhow::Error`.
pub(crate) struct AppError(Error);

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
// 8be5152c ends here

// [[file:../../remote.note::ad35d99c][ad35d99c]]
// type State = std::sync::Arc<TaskSender>;
type TaskState = super::TaskSender;
// ad35d99c ends here

// [[file:../../remote.note::7157f9ad][7157f9ad]]
use super::error;
use super::Computed;
use super::Error;
use super::Molecule;
use axum::extract::State;

#[axum::debug_handler]
async fn compute_mol(State(client): State<TaskState>, Json(mol): Json<Molecule>) -> Result<Json<Computed>, AppError> {
    let computed = client.remote_compute(mol).await?;
    Ok(Json(computed))
}
// 7157f9ad ends here

// [[file:../../remote.note::59c3364a][59c3364a]]
macro_rules! build_app_with_routes {
    ($state: expr) => {{
        use axum::routing::post;
        axum::Router::new().route("/mol", post(compute_mol)).with_state($state)
    }};
}
// 59c3364a ends here

// [[file:../../remote.note::415dc72b][415dc72b]]
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

// [[file:../../remote.note::f4a1566d][f4a1566d]]
impl Server {
    /// Start restful service
    ///
    /// # Parameters
    ///
    /// * addr: socket address to bind
    /// * state: shared state between route handlers
    pub(super) async fn run_restful(addr: impl Into<SocketAddr>, state: TaskState) {
        let app = build_app_with_routes!(state);
        let addr = addr.into();

        if let Err(err) = axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await
        {
            error!("error in restful serivce: {err:?}");
        }
    }
}
// f4a1566d ends here
