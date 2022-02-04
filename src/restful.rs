// [[file:../remote.note::b8081727][b8081727]]
use super::*;
use base::{Job, Node};

use warp::Filter;
// b8081727 ends here

// [[file:../remote.note::08048436][08048436]]
use std::sync::atomic;

static SERVER_BUSY: atomic::AtomicBool = atomic::AtomicBool::new(false);

fn server_busy() -> bool {
    SERVER_BUSY.load(atomic::Ordering::SeqCst)
}

fn server_mark_busy() {
    if !server_busy() {
        SERVER_BUSY.store(true, atomic::Ordering::SeqCst);
    } else {
        panic!("server is already busy")
    }
}

fn server_mark_free() {
    if server_busy() {
        SERVER_BUSY.store(false, atomic::Ordering::SeqCst);
    } else {
        panic!("server is already free")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum ComputationResult {
    JobCompleted(String),
    JobFailed(String),
    NotStarted(String),
}
// 08048436 ends here

// [[file:../remote.note::07c5146c][07c5146c]]
mod handlers {
    use super::*;

    /// POST /jobs with JSON body
    pub async fn create_job(job: Job) -> Result<impl warp::Reply, warp::Rejection> {
        if !server_busy() {
            server_mark_busy();
            let comput = job.submit();
            match comput {
                Ok(mut comput) => match comput.wait_for_output().await {
                    Ok(out) => {
                        server_mark_free();
                        let ret = ComputationResult::JobCompleted(out);
                        Ok(warp::reply::json(&ret))
                    }
                    Err(err) => {
                        server_mark_free();
                        let msg = format!("{err:?}");
                        let ret = ComputationResult::JobFailed(msg);
                        Ok(warp::reply::json(&ret))
                    }
                },
                Err(err) => {
                    server_mark_free();
                    let msg = format!("failed to create job: {err:?}");
                    error!("{msg}");
                    let ret = ComputationResult::JobFailed(msg);
                    Ok(warp::reply::json(&ret))
                }
            }
        } else {
            server_mark_free();
            let msg = format!("Server is busy");
            let ret = ComputationResult::NotStarted(msg);
            Ok(warp::reply::json(&ret))
        }
    }
}
// 07c5146c ends here

// [[file:../remote.note::a5b61fa9][a5b61fa9]]
mod filters {
    use super::*;

    /// POST /jobs with JSON body
    async fn job_run() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("jobs")
            .and(warp::post())
            .and(warp::body::json())
            .and_then(handlers::create_job)
    }

    pub async fn jobs() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        job_run().await
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
        info!("wait output for job {:?}", self.job);
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
    pub async fn serve_as_worker(addr: &str) {
        let server = Self::new(addr);
        let api = filters::jobs().await;
        server.serve_api(api).await;
    }

    /// Serve warp api service
    pub async fn serve_api<F>(&self, api: F)
    where
        F: Filter + Clone + Send + Sync + 'static,
        F::Extract: warp::Reply,
    {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let services = warp::serve(api.with(warp::log("gosh-remote")));
        let (addr, server) = services.bind_with_graceful_shutdown(self.address, async {
            rx.await.ok();
        });
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
    }
}
// 62b9ac23 ends here
