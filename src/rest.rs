// [[file:../remote.note::6d0794c1][6d0794c1]]
//! For remote computation of molecules with RESTful web services
// 6d0794c1 ends here

// [[file:../remote.note::3d2c01c2][3d2c01c2]]
use super::*;
use gchemol::Molecule;
use gosh_model::{ChemicalModel, Computed};
use task::RemoteIO;
// 3d2c01c2 ends here

// [[file:../remote.note::ccbf3ca9][ccbf3ca9]]
type Task = task::Task<Molecule, Computed>;
type TaskReceiver = task::TaskReceiver<Molecule, Computed>;
type TaskSender = task::TaskSender<Molecule, Computed>;
type TxOutput = task::TxOutput<Computed>;
// ccbf3ca9 ends here

// [[file:../remote.note::aa8d1d68][aa8d1d68]]
pub(crate) mod server;
// aa8d1d68 ends here

// [[file:../remote.note::285a8db0][285a8db0]]
impl Client {
    #[tokio::main]
    /// Request remote server compute `mol` and return computed results.
    pub async fn compute_molecule(&self, mol: &Molecule) -> Result<Computed> {
        info!("Request server to compute molecule {}", mol.title());
        let x = self.post("mol", &mol).await?;
        let mol = serde_json::from_str(&x).with_context(|| format!("invalid json str: {x:?}"))?;
        Ok(mol)
    }
}
// 285a8db0 ends here

// [[file:../remote.note::389c909a][389c909a]]
fn compute_mol_and_send_out(mol: &Molecule, model: &mut impl ChemicalModel, tx: TxOutput) -> Result<()> {
    let mp = model.compute(mol)?;
    tx.send(mp).map_err(|err| format_err!("send task out error: {err:?}"))?;
    Ok(())
}

impl Server {
    /// Wait for incoming task and compute received Molecule using ChemicalModel
    async fn serve_incoming_task_with(mut task: TaskReceiver, mut model: impl ChemicalModel) {
        loop {
            debug!("wait for new molecule to compute ...");
            if let Some(RemoteIO(mol, tx_out)) = task.recv().await {
                debug!("ask client to compute molecule {}", mol.title());
                if let Err(err) = compute_mol_and_send_out(&mol, &mut model, tx_out) {
                    error!("{err:?}");
                }
            } else {
                info!("Task channel closed for some reason");
                break;
            }
        }
    }

    /// Serve as a computation server for chemical model.
    pub async fn serve_as_chemical_model(&self, model: impl ChemicalModel + Send + 'static) -> Result<()> {
        let addr = self.address;
        println!("chemical model computation server listening on {addr:?}");

        let (task_rx, task_tx) = Task::new().split();
        let h1 = tokio::spawn(async move { Self::run_restful(addr, task_tx).await });
        let h2 = tokio::spawn(async move { Self::serve_incoming_task_with(task_rx, model).await });
        tokio::try_join!(h1, h2)?;
        Ok(())
    }

    // FIXME: better name
    #[tokio::main]
    /// Enter point for command line usage.
    ///
    /// The server binding address will be wrote in `lock_file` available for
    /// client connections.
    pub async fn enter_main(lock_file: &Path, model: impl ChemicalModel + Send + 'static) -> Result<()> {
        let server = Self::try_bind_auto()?;
        let _lock = LockFile::new(lock_file, server.address);
        server.serve_as_chemical_model(model).await?;
        Ok(())
    }
}
// 389c909a ends here

// [[file:../remote.note::908a93c5][908a93c5]]
pub use crate::client::Client;

pub use crate::server::Server;
pub use self::server::shutdown_signal;
// 908a93c5 ends here
