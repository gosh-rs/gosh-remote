// [[file:../remote.note::b8081727][b8081727]]
use super::*;
use base::{Job, Node};

use warp::Filter;
// b8081727 ends here

// [[file:../remote.note::08048436][08048436]]
use gosh_model::Computed;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ComputationResult {
    JobCompleted(String),
    JobFailed(String),
}

impl ComputationResult {
    fn parse_from_json(x: &str) -> Result<Self> {
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
// 08048436 ends here

// [[file:../remote.note::07c5146c][07c5146c]]
mod handlers {
    use super::*;

    /// Run `job` locally and return stdout on success.
    pub async fn create_job(job: Job) -> Result<impl warp::Reply, warp::Rejection> {
        match job.submit() {
            Ok(mut comput) => match comput.wait_for_output().await {
                Ok(out) => {
                    let ret = ComputationResult::JobCompleted(out);
                    debug!("computation done with: {ret:?}");
                    Ok(warp::reply::json(&ret))
                }
                Err(err) => {
                    let msg = format!("{err:?}");
                    let ret = ComputationResult::JobFailed(msg);
                    debug!("computation failed with: {ret:?}");
                    Ok(warp::reply::json(&ret))
                }
            },
            Err(err) => {
                let msg = format!("failed to create job: {err:?}");
                error!("{msg}");
                let ret = ComputationResult::JobFailed(msg);
                Ok(warp::reply::json(&ret))
            }
        }
    }
}
// 07c5146c ends here

// [[file:../remote.note::a5b61fa9][a5b61fa9]]
mod filters {
    use super::*;

    /// POST /jobs with JSON body
    pub async fn jobs() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("jobs")
            .and(warp::post())
            .and(warp::body::json())
            .and_then(handlers::create_job)
    }
}
// a5b61fa9 ends here

// [[file:../remote.note::e324852d][e324852d]]
use client::Client;

/// Submit job remotely using REST api service
pub struct RemoteComputation {
    job: Job,
    client: Client,
}

impl RemoteComputation {
    pub async fn wait_for_output(&self) -> Result<String> {
        debug!("wait output for job {:?}", self.job);
        let resp = self.client.post("jobs", &self.job)?;
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
// e324852d ends here

// [[file:../remote.note::62b9ac23][62b9ac23]]
impl server::Server {
    /// Serve as a worker running on local node.
    pub async fn serve_as_worker(addr: &str) -> Result<()> {
        let server = Self::new(addr);
        let api = filters::jobs().await;
        server.serve_api(api).await?;
        Ok(())
    }

    /// Serve warp api service
    async fn serve_api<F>(&self, api: F) -> Result<()>
    where
        F: Filter + Clone + Send + Sync + 'static,
        F::Extract: warp::Reply,
    {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let services = warp::serve(api.with(warp::log("gosh-remote")));
        let (addr, server) = services.try_bind_with_graceful_shutdown(self.address, async {
            rx.await.ok();
        })?;
        println!("listening on {addr:?}");

        let ctrl_c = tokio::signal::ctrl_c();
        tokio::select! {
            _ = server => {
                eprintln!("server closed");
            }
            _ = ctrl_c => {
                let _ = tx.send(());
                eprintln!("user interruption");
            }
        }

        Ok(())
    }
}
// 62b9ac23 ends here
