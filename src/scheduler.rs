// [[file:../remote.note::c07df478][c07df478]]
use super::*;

use base::{Job, Node};
// c07df478 ends here

// [[file:../remote.note::6730a02b][6730a02b]]
use crate::Client;
use std::path::Path;

impl Client {
    /// Request server to run `cmd` in directory `wrk_dir`.
    pub async fn run_cmd(&self, cmd: &str, wrk_dir: &Path) -> Result<String> {
        let wrk_dir = wrk_dir.shell_escape_lossy();
        #[rustfmt::skip]
        let script = format!("#! /usr/bin/env bash
set -x
cd {wrk_dir}
{cmd}
");
        let job = Job::new(script);
        let o = self.post("jobs", job).await?;

        Ok(o)
    }

    /// Request server to add a new node for remote computation.
    pub async fn add_node(&self, node: impl Into<Node>) -> Result<()> {
        self.post("nodes", node.into()).await?;
        Ok(())
    }

    #[tokio::main()]
    /// For non-async call
    pub async fn run_cmd_block(&self, cmd: &str, wrk_dir: &Path) -> Result<String> {
        let s = self.run_cmd(cmd, wrk_dir).await?;
        Ok(s)
    }
}
// 6730a02b ends here

// [[file:../remote.note::dec20ace][dec20ace]]
mod routes {
    use super::*;
    use crate::rest::server::AppError;
    use crate::worker::ComputationResult;
    use interactive::TaskClient;

    use axum::extract::State;
    use axum::Json;

    /// Handle request for adding a new node into `Nodes`
    #[axum::debug_handler]
    async fn add_node(State(task): State<TaskClient>, Json(node): Json<Node>) -> Result<(), AppError> {
        task.add_node(node).await?;
        Ok(())
    }

    /// Handle request for adding a new node into `Nodes`
    #[axum::debug_handler]
    async fn add_job(
        State(mut task): State<TaskClient>,
        Json(job): Json<Job>,
    ) -> Result<Json<ComputationResult>, AppError> {
        let r = task.interact(job).await?;
        let c = ComputationResult::parse_from_json(&r)?;
        Ok(Json(c))
    }

    pub(super) async fn run_restful(addr: impl Into<SocketAddr>, state: TaskClient) -> Result<()> {
        use axum::routing::post;

        let app = axum::Router::new()
            .route("/jobs", post(add_job))
            .with_state(state.clone())
            .route("/nodes", post(add_node))
            .with_state(state);
        let addr = addr.into();

        let x = axum::Server::bind(&addr).serve(app.into_make_service()).await?;
        Ok(())
    }
}
// dec20ace ends here

// [[file:../remote.note::63fb876f][63fb876f]]
use base::Nodes;
use server::Server;

impl Server {
    pub async fn serve_as_scheduler(addr: &str) {
        println!("listening on {addr:?}");
        let (mut task_server, task_client) = crate::interactive::new_interactive_task();
        let nodes: Vec<String> = vec![];
        let h1 = tokio::spawn(async move {
            if let Err(e) = task_server.run_and_serve(Nodes::new(nodes)).await {
                error!("task server: {e:?}");
            }
        });
        tokio::pin!(h1);

        let server = Self::bind(addr);
        let tc = task_client.clone();
        let h2 = tokio::spawn(async move {
            self::routes::run_restful(server.address, tc).await;
        });
        tokio::pin!(h2);

        let h3 = tokio::signal::ctrl_c();
        tokio::pin!(h3);
        loop {
            tokio::select! {
                _res = &mut h1 => {
                    log_dbg!();
                }
                _res = &mut h2 => {
                    log_dbg!();
                }
                _res = &mut h3 => {
                    info!("User interrupted. Shutting down ...");
                    let _ = task_client.abort().await;
                    break;
                }
            }
        }
        h1.abort();
        h2.abort();
    }
}
// 63fb876f ends here
