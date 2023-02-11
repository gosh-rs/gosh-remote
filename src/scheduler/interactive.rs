// [[file:../../remote.note::ae9e9435][ae9e9435]]
#![deny(warnings)]

use super::*;
use base::{Job, Node, Nodes};

use tokio::sync::oneshot;
// ae9e9435 ends here

// [[file:../../remote.note::e899191b][e899191b]]
#[derive(Debug)]
struct Interaction(Job, oneshot::Sender<InteractionOutput>);

/// The message sent from client for controlling running job
#[derive(Debug, Clone)]
enum Control {
    AddNode(Node),
    Abort,
}

type InteractionOutput = String;
type RxInteraction = tokio::sync::mpsc::Receiver<Interaction>;
type TxInteraction = tokio::sync::mpsc::Sender<Interaction>;
type RxControl = tokio::sync::mpsc::Receiver<Control>;
type TxControl = tokio::sync::mpsc::Sender<Control>;
// e899191b ends here

// [[file:../../remote.note::d88217da][d88217da]]
#[derive(Clone)]
/// Manage client requests in threading environment
pub(super) struct TaskClient {
    // for send client request for pause, resume, stop computation on server side
    tx_ctl: TxControl,
    // for interaction with child process on server side
    tx_int: TxInteraction,
}

mod client {
    use super::*;

    impl TaskClient {
        /// Send `job` to target and wait for output. Return the
        /// computed result.
        pub async fn interact(&mut self, job: Job) -> Result<String> {
            // FIXME: refactor
            let (tx_resp, rx_resp) = oneshot::channel();
            self.tx_int.send(Interaction(job, tx_resp)).await?;
            let out = rx_resp.await?;
            Ok(out)
        }

        /// Add one remote node into list for computation
        pub async fn add_node(&self, node: Node) -> Result<()> {
            trace!("send add_node ctl msg");
            self.tx_ctl.send(Control::AddNode(node)).await?;
            Ok(())
        }

        /// Notify main thread to exit
        pub async fn abort(&self) -> Result<()> {
            debug!("send abort ctrl msg");
            // FIXME: there is a blocking issue if nodes is empty
            self.tx_ctl.send(Control::AddNode("localhost:3030".into())).await?;
            self.tx_ctl.send(Control::Abort).await?;
            Ok(())
        }
    }
}
// d88217da ends here

// [[file:../../remote.note::7b4ac45b][7b4ac45b]]
pub(super) struct TaskServer {
    // for receiving interaction message for child process
    rx_int: Option<RxInteraction>,
    // for controlling child process
    rx_ctl: Option<RxControl>,
}

mod server {
    use super::*;
    use crate::task::RemoteIO;

    type RxJobs = spmc::Receiver<RemoteIO<Job, String>>;

    /// compute job from `jobs` using `node`
    async fn handle_client_interaction(jobs: RxJobs, node: &Node) -> Result<()> {
        let RemoteIO(job, tx_resp) = jobs.recv()?;
        let name = job.name();
        info!("Request remote node {node:?} to compute job {name} ...");
        // FIXME: potentially deadlock
        let comput = job.submit_remote(node)?;
        // if computation failed, we should tell the client to exit
        match comput.wait_for_output().await {
            Ok(out) => {
                info!("Job {name} completed, sending stdout to the client ...");
                if let Err(_) = tx_resp.send(out) {
                    error!("the client has been dropped");
                }
            }
            Err(err) => {
                let msg = format!("Job {name:?} failed with error: {err:?}");
                tx_resp.send(msg).ok();
            }
        }

        Ok(())
    }

    /// ask a node from `nodes` to compute one job from `jobs`
    async fn borrow_node_and_compute(nodes: Nodes, jobs: RxJobs) {
        match nodes.borrow_node() {
            Ok(node) => {
                if let Err(err) = handle_client_interaction(jobs, &node).await {
                    error!("found error when running job: {err:?}");
                }
                // return node back when job done
                if let Err(err) = nodes.return_node(node) {
                    error!("found error when return node: {err:?}");
                }
            }
            Err(err) => {
                error!("found error when borrowing node: {err:?}");
            }
        }
    }

    impl TaskServer {
        /// Run child process in new session, and serve requests for interactions.
        pub async fn run_and_serve(&mut self, nodes: Nodes) -> Result<()> {
            let mut rx_int = self.rx_int.take().context("no rx_int")?;
            let mut rx_ctl = self.rx_ctl.take().context("no rx_ctl")?;

            let (mut tx_jobs, rx_jobs) = spmc::channel();
            for i in 0.. {
                // make sure run in parallel
                let mut join_handler = {
                    let jobs = rx_jobs.clone();
                    let nodes = nodes.clone();
                    tokio::spawn(async move {
                        let n = nodes.len();
                        info!("task {i}: wait for remote node to compute incoming job");
                        info!("task {i}: we have {n} nodes available for computations");
                        borrow_node_and_compute(nodes, jobs).await;
                    })
                };
                // handle logic in main thread
                tokio::select! {
                    Ok(_) = &mut join_handler => {
                        log_dbg!();
                    }
                    Some(int) = rx_int.recv() => {
                        let Interaction(job, tx_resp) = int;
                        tx_jobs.send(RemoteIO(job, tx_resp))?;
                    }
                    Some(ctl) = rx_ctl.recv() => {
                        match ctl {
                            Control::AddNode(node) => {
                                info!("client asked to add a new remote node: {node:?}");
                                let nodes = nodes.clone();
                                nodes.return_node(node.into())?;
                            }
                            Control::Abort => {
                                join_handler.abort();
                                break;
                            },
                        }
                    }
                    else => {
                        bail!("Unexpected branch: the communication channels broken?");
                    }
                }
            }
            Ok(())
        }
    }
}
// 7b4ac45b ends here

// [[file:../../remote.note::8408786a][8408786a]]
/// Create task server and client. The client can be cloned and used in
/// concurrent environment
pub(super) fn new_interactive_task() -> (TaskServer, TaskClient) {
    let (tx_int, rx_int) = tokio::sync::mpsc::channel(1);
    let (tx_ctl, rx_ctl) = tokio::sync::mpsc::channel(1);

    let server = TaskServer {
        rx_int: rx_int.into(),
        rx_ctl: rx_ctl.into(),
    };

    let client = TaskClient {
        tx_int,
        tx_ctl,
    };

    (server, client)
}
// 8408786a ends here
