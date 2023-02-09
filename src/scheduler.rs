// [[file:../remote.note::ec61676b][ec61676b]]
use super::*;

use base::{Job, Node};
use server::Server;
// ec61676b ends here

// [[file:../remote.note::568e244d][568e244d]]
use client::Client;
use std::path::Path;

impl Client {
    /// Request server to run `cmd` in directory `wrk_dir`.
    pub fn run_cmd(&self, cmd: &str, wrk_dir: &Path) -> Result<String> {
        let wrk_dir = wrk_dir.shell_escape_lossy();
        #[rustfmt::skip]
        let script = format!("#! /usr/bin/env bash
set -x
cd {wrk_dir}
{cmd}
");
        let job = Job::new(script);
        let o = self.post("jobs", job)?;

        Ok(o)
    }

    /// Request server to add a new node for remote computation.
    pub fn add_node(&self, node: impl Into<Node>) -> Result<()> {
        self.post("nodes", node.into())?;
        Ok(())
    }
}
// 568e244d ends here

// [[file:../remote.note::9f3b28d3][9f3b28d3]]
mod filters {
    use super::*;
    use interactive::TaskClient;
    use warp::Filter;

    /// Handle request for adding a new node into `Nodes`
    async fn add_node(node: Node, task: TaskClient) -> Result<impl warp::Reply, warp::Rejection> {
        let o = format!("{:?}", task.add_node(node).await);
        Ok(warp::reply::json(&o))
    }

    /// Handle request for adding a new node into `Nodes`
    async fn add_job(job: Job, mut task: TaskClient) -> Result<impl warp::Reply, warp::Rejection> {
        use crate::worker::ComputationResult;

        // FIXME: do not know how to return warp error
        let r = task.interact(job).await;
        match r.and_then(|r| ComputationResult::parse_from_json(&r)) {
            Ok(o) => Ok(warp::reply::json(&o)),
            Err(e) => Ok(warp::reply::json(&format!("{e:?}"))),
        }
    }

    /// POST /nodes with JSON body
    pub fn api(task: TaskClient) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let db = warp::any().map(move || task.clone());
        let jobs = warp::path!("jobs")
            .and(warp::post())
            .and(warp::body::json())
            .and(db.clone())
            .and_then(add_job);
        let nodes = warp::path!("nodes")
            .and(warp::post())
            .and(warp::body::json())
            .and(db.clone())
            .and_then(add_node);

        jobs.or(nodes)
    }
}
// 9f3b28d3 ends here

// [[file:../remote.note::63fb876f][63fb876f]]
use base::Nodes;

impl Server {
    pub async fn serve_as_scheduler(addr: &str) {
        println!("listening on {addr:?}");
        let (mut task_server, task_client) = interactive::new_interactive_task();
        let nodes: Vec<String> = vec![];
        // let h1 = task_server.run_and_serve(nodes);
        let h1 = tokio::spawn(async move {
            if let Err(e) = task_server.run_and_serve(Nodes::new(nodes)).await {
                error!("task server: {e:?}");
            }
        });
        tokio::pin!(h1);

        let server = Self::new(addr);
        let api = filters::api(task_client.clone());
        let h2 = tokio::spawn(async move {
            warp::serve(api).run(server.address).await;
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
        log_dbg!();
        h1.abort();
        log_dbg!();
        h2.abort();
        log_dbg!();
    }
}
// 63fb876f ends here
