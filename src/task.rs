// [[file:../remote.note::d03e6166][d03e6166]]
#![deny(warnings)]
//! Task for remote execution
//!
//! # Example
//!
//! ```ignore
//! let (rx, tx) = Task::new().split();
//! 
//! // client side
//! tx1 = tx.clone();
//! tx2 = tx.clone();
//! let out1 = tx1.remote_compute("test input 1")?;
//! let out2 = tx2.remote_compute("test input 2")?;
//! 
//! // server side
//! if let Some(RemoteIO(input, tx_out)) = rx.recv() {
//!     // compute with job input
//!     let output = compute_with(input)?;
//!     // send job output to client side
//!     tx_out.send(output)?;
//! } else {
//!     // task channel closed
//!     // ...
//! }
//! ```
// d03e6166 ends here

// [[file:../remote.note::475dbc7d][475dbc7d]]
use super::*;

use tokio::sync::{mpsc, oneshot};
// 475dbc7d ends here

// [[file:../remote.note::214790a9][214790a9]]
type Computed = String;
type TxComputed = oneshot::Sender<Computed>;
/// RemoteIO contains input and output for remote execution. The first field in tuple
/// is job input, and the second is for writing job output.
pub type RemoteIO = (String, TxComputed);
type TxInput = mpsc::Sender<RemoteIO>;
/// The receiver of jobs for remote execution
pub type RxInput = mpsc::Receiver<RemoteIO>;
// 214790a9 ends here

// [[file:../remote.note::b55affa9][b55affa9]]
/// The client side for remote execution
#[derive(Debug, Clone, Default)]
pub struct TaskSender {
    tx_inp: Option<TxInput>,
}

impl TaskSender {
    /// Ask remote side compute with `input` and return the computed.
    pub async fn remote_compute(&self, input: impl Into<String>) -> Result<Computed> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx_inp
            .as_ref()
            .expect("task input")
            .send((input.into(), tx))
            .await?;
        let computed = rx.await?;
        Ok(computed)
    }
}
// b55affa9 ends here

// [[file:../remote.note::f45eafe9][f45eafe9]]
/// The server side for remote execution
#[derive(Debug)]
pub struct TaskReceiver {
    rx_inp: RxInput,
}

impl TaskReceiver {
    /// Receives the next task for this receiver.
    pub async fn recv(&mut self) -> Option<RemoteIO> {
        self.rx_inp.recv().await
    }
}

fn new_interactive_task() -> (TaskReceiver, TaskSender) {
    let (tx_inp, rx_inp) = tokio::sync::mpsc::channel(1);

    let server = TaskReceiver { rx_inp };
    let client = TaskSender { tx_inp: tx_inp.into() };

    (server, client)
}
// f45eafe9 ends here

// [[file:../remote.note::3f19ae12][3f19ae12]]
/// A Task channel for remote execution (multi-producer, single-consumer)
pub struct Task {
    sender: TaskSender,
    receiver: TaskReceiver,
}

impl Task {
    /// Create a task channel for computation of molecule in client/server
    /// architecture
    pub fn new() -> Self {
        let (receiver, sender) = new_interactive_task();
        Self { sender, receiver }
    }

    /// Splits a single task into separate read and write half
    pub fn split(self) -> (TaskReceiver, TaskSender) {
        match self {
            Self {
                sender: tx,
                receiver: rx,
            } => (rx, tx),
        }
    }
}
// 3f19ae12 ends here
