// [[file:../remote.note::ae9e9435][ae9e9435]]
use super::*;
use crate::task::Task;

use base::{Job, Node, Nodes};
// ae9e9435 ends here

// [[file:../remote.note::55bd52fb][55bd52fb]]
use crate::task::{RemoteIO, TaskReceiver, TaskSender};

/// The message sent from client for control running job
#[derive(Debug, Clone)]
enum Control {
    AddNode(Node),
    Abort,
}

type RxInteraction = TaskReceiver<Job, String>;
type TxInteraction = TaskSender<Job, String>;
type RxControl = TaskReceiver<Control, ()>;
type TxControl = TaskSender<Control, ()>;
// 55bd52fb ends here

// [[file:../remote.note::5a59464d][5a59464d]]
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
            let out = self.tx_int.send(job).await?;
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
// 5a59464d ends here

// [[file:../remote.note::062132e0][062132e0]]
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
                        tx_jobs.send(int)?;
                    }
                    Some(ctl) = rx_ctl.recv() => {
                        match ctl {
                            RemoteIO(Control::AddNode(node), _) => {
                                info!("client asked to add a new remote node: {node:?}");
                                let nodes = nodes.clone();
                                nodes.return_node(node.into())?;
                            }
                            RemoteIO(Control::Abort, _) => {
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
// 062132e0 ends here

// [[file:../remote.note::231ad4be][231ad4be]]
/// Create task server and client. The client can be cloned and used in
/// concurrent environment
pub(super) fn new_interactive_task() -> (TaskServer, TaskClient) {
    let (rx_int, tx_int) = Task::new().split();
    let (rx_ctl, tx_ctl) = Task::new().split();

    let server = TaskServer {
        rx_int: rx_int.into(),
        rx_ctl: rx_ctl.into(),
    };

    let client = TaskClient { tx_int, tx_ctl };

    (server, client)
}
// 231ad4be ends here
