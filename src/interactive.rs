// [[file:../remote.note::*docs][docs:1]]
//! This mod is for VASP interactive calculations.
// docs:1 ends here

// [[file:../remote.note::ae9e9435][ae9e9435]]
use super::*;
use base::{Job, Node, Nodes};

use tokio::sync::oneshot;
// ae9e9435 ends here

// [[file:../remote.note::e899191b][e899191b]]
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

// [[file:../remote.note::d88217da][d88217da]]
#[derive(Clone)]
/// Manage client requests in threading environment
pub struct TaskClient {
    // for send client request for pause, resume, stop computation on server side
    tx_ctl: TxControl,
    // for interaction with child process on server side
    tx_int: TxInteraction,
}

mod client {
    use super::*;

    impl TaskClient {
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

// [[file:../remote.note::7b4ac45b][7b4ac45b]]
pub struct TaskServer {
    // for receiving interaction message for child process
    rx_int: Option<RxInteraction>,
    // for controlling child process
    rx_ctl: Option<RxControl>,
}

mod server {
    use super::*;

    type Jobs = (Job, oneshot::Sender<String>);
    type RxJobs = spmc::Receiver<Jobs>;
    async fn handle_client_interaction(jobs: RxJobs, node: &Node) -> Result<()> {
        let (job, tx_resp) = jobs.recv()?;
        let name = job.name();
        info!("Request remote node {node:?} to compute job {name} ...");
        // FIXME: potentially deadlock
        let mut comput = job.submit_remote(node)?;
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
                    log_dbg!();
                    let jobs = rx_jobs.clone();
                    let nodes = nodes.clone();
                    tokio::spawn(async move {
                        info!("thread {i}: wait for remote node to compute incoming job");
                        info!("thread {i}: we have {} nodes available for computations");
                        borrow_node_and_compute(nodes, jobs).await;
                        log_dbg!();
                    })
                };
                log_dbg!();
                // handle logic in main thread
                tokio::select! {
                    Ok(x) = &mut join_handler => {
                        log_dbg!();
                        dbg!(x);
                    }
                    Some(int) = rx_int.recv() => {
                        log_dbg!();
                        let Interaction(job, tx_resp) = int;
                        tx_jobs.send((job, tx_resp))?;
                    }
                    Some(ctl) = rx_ctl.recv() => {
                        log_dbg!();
                        match ctl {
                            Control::AddNode(node) => {
                                info!("client asked to add a new remote node: {node:?}");
                                let nodes = nodes.clone();
                                nodes.return_node(node.into())?;
                            }
                            Control::Abort => {
                                log_dbg!();
                                join_handler.abort();
                                log_dbg!();
                                break;
                            },
                            _ => log_dbg!(),
                        }
                    }
                    else => {
                        bail!("Unexpected branch: the communication channels broken?");
                    }
                }
                log_dbg!();
            }
            Ok(())
        }
    }
}
// 7b4ac45b ends here

// [[file:../remote.note::8408786a][8408786a]]
/// Create task server and client. The client can be cloned and used in
/// concurrent environment
pub fn new_interactive_task() -> (TaskServer, TaskClient) {
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
