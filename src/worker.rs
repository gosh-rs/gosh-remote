// [[file:../remote.note::4b6cf6fa][4b6cf6fa]]
use super::*;
use base::{Job, Node};
// 4b6cf6fa ends here

// [[file:../remote.note::0688d573][0688d573]]
use gosh_model::Computed;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ComputationResult {
    JobCompleted(String),
    JobFailed(String),
}

impl ComputationResult {
    pub(crate) fn parse_from_json(x: &str) -> Result<Self> {
        let computed = serde_json::from_str(&x).with_context(|| format!("invalid json str: {x:?}"))?;
        Ok(computed)
    }

    pub fn get_computed_from_str(s: &str) -> Result<Computed> {
        match Self::parse_from_json(s)? {
            Self::JobCompleted(s) => {
                let computed = s.parse()?;
                Ok(computed)
            }
            Self::JobFailed(s) => {
                bail!("job failed with error message:\n{s:?}");
            }
        }
    }
}
// 0688d573 ends here

// [[file:../remote.note::a2266f5f][a2266f5f]]
mod handlers {
    use super::*;
    use crate::rest::server::AppError;
    use axum::Json;

    /// Run `job` locally and return stdout on success.
    #[axum::debug_handler]
    pub(super) async fn create_job(Json(job): Json<Job>) -> Result<Json<ComputationResult>, AppError> {
        match job.submit() {
            Ok(mut comput) => match comput.wait_for_output().await {
                Ok(out) => {
                    let ret = ComputationResult::JobCompleted(out);
                    debug!("computation done with: {ret:?}");
                    Ok(Json(ret))
                }
                Err(err) => {
                    let msg = format!("{err:?}");
                    let ret = ComputationResult::JobFailed(msg);
                    debug!("computation failed with: {ret:?}");
                    Ok(Json(ret))
                }
            },
            Err(err) => {
                let msg = format!("failed to create job: {err:?}");
                error!("{msg}");
                let ret = ComputationResult::JobFailed(msg);
                Ok(Json(ret))
            }
        }
    }
}
// a2266f5f ends here

// [[file:../remote.note::d6f1b9d7][d6f1b9d7]]
use crate::Client;

/// Submit job remotely using REST api service
pub struct RemoteComputation {
    job: Job,
    client: Client,
}

impl RemoteComputation {
    pub async fn wait_for_output(&self) -> Result<String> {
        debug!("wait output for job {:?}", self.job);
        let resp = self.client.post("jobs", &self.job).await?;
        Ok(resp)
    }
}

impl Job {
    /// Remote submission using RESTful service
    pub fn submit_remote(self, node: &Node) -> Result<RemoteComputation> {
        let client = Client::connect(node);
        let comput = RemoteComputation { job: self, client };

        Ok(comput)
    }
}
// d6f1b9d7 ends here

// [[file:../remote.note::9407c3be][9407c3be]]
use axum::Router;

impl server::Server {
    /// Serve as a worker running on local node.
    pub async fn serve_as_worker(&self) -> Result<()> {
        use crate::rest::shutdown_signal;

        let addr = self.address;
        println!("listening on {addr:?}");

        let signal = shutdown_signal();
        let server = axum::Server::bind(&addr).serve(app().into_make_service());
        let (tx, rx) = tokio::sync::oneshot::channel();
        tokio::select! {
            _ = server => {
                eprintln!("server closed");
            }
            _ = signal => {
                let _ = tx.send(());
                eprintln!("user interruption");
            }
        }

        Ok(())
    }
}

fn app() -> Router {
    use self::handlers::create_job;
    use axum::routing::post;

    Router::new().route("/jobs", post(create_job))
}
// 9407c3be ends here
