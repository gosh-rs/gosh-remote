// [[file:../remote.note::ec61676b][ec61676b]]
use super::*;

use base::{Job, Node};
use server::Server;
// ec61676b ends here

// [[file:../remote.note::9f3b28d3][9f3b28d3]]
mod filters {
    use super::*;
    use interactive::TaskClient;
    use warp::Filter;

    /// Handle request for adding a new node into `Nodes`
    async fn add_node(node: Node, task: TaskClient) -> Result<impl warp::Reply, warp::Rejection> {
        task.add_node(node).await;
        Ok(warp::reply::json(&()))
    }

    /// Handle request for adding a new node into `Nodes`
    async fn add_job(job: Job, mut task: TaskClient) -> Result<impl warp::Reply, warp::Rejection> {
        // FIXME: remove unwrap
        let out = task.interact(job).await.unwrap();
        Ok(warp::reply::json(&out))
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
impl Server {
    pub async fn serve_as_scheduler(addr: &str) {
        let (mut task_server, task_client) = interactive::new_interactive_task();
        let nodes = vec![];
        let h1 = task_server.run_and_serve(nodes);
        tokio::pin!(h1);

        let server = Self::new(addr);
        let api = filters::api(task_client);
        let services = warp::serve(api);
        let (tx, rx) = tokio::sync::oneshot::channel();
        let (addr, h2) = services.bind_with_graceful_shutdown(server.address, async {
            rx.await.ok();
        });
        println!("listening on {addr:?}");
        tokio::pin!(h2);

        let ctrl_c = tokio::signal::ctrl_c();
        tokio::select! {
            _ = ctrl_c => {
                info!("User interrupted. Shutting down ...");
                let _ = tx.send(());
            },
            res = &mut h1 => {
                dbg!();
            }
            res = &mut h2 => {
                dbg!();
            }
        }
    }
}
// 63fb876f ends here
