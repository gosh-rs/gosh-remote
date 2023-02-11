// [[file:../remote.note::6d0794c1][6d0794c1]]
//! For remote computation of molecules with RESTful web services
// 6d0794c1 ends here

// [[file:../remote.note::3d2c01c2][3d2c01c2]]
#![deny(warnings)]

use crate::common::*;
use gchemol::Molecule;
use gosh_model::Computed;
// 3d2c01c2 ends here

// [[file:../remote.note::ccbf3ca9][ccbf3ca9]]
use crate::task::Task;
// type Task = crate::task::Task<Molecule, Computed>;
type TaskReceiver = crate::task::TaskReceiver<Molecule, Computed>;
type TxOutput = crate::task::TxOutput<Computed>;
// ccbf3ca9 ends here

// [[file:../remote.note::ad35d99c][ad35d99c]]
type TaskState = crate::task::TaskSender<Molecule, Computed>;
// ad35d99c ends here

// [[file:../remote.note::7157f9ad][7157f9ad]]
use crate::rest::AppError;
use axum::extract::State;
use axum::Json;
use gosh_model::ChemicalModel;

fn compute_mol_and_send_out(mol: &Molecule, model: &mut impl ChemicalModel, tx: TxOutput) -> Result<()> {
    let mp = model.compute(mol)?;
    tx.send(mp).map_err(|err| format_err!("send task out error: {err:?}"))?;
    Ok(())
}

#[axum::debug_handler]
/// Handle compute molecule request from client side
pub(self) async fn compute_mol(
    State(client): State<TaskState>,
    Json(mol): Json<Molecule>,
) -> Result<Json<Computed>, AppError> {
    let computed = client.remote_compute(mol).await?;
    Ok(Json(computed))
}

/// Wait for incoming task and compute received Molecule using ChemicalModel
pub(self) async fn serve_incoming_task_with(mut task: TaskReceiver, mut model: impl ChemicalModel) {
    use crate::task::RemoteIO;

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
// 7157f9ad ends here

// [[file:../remote.note::59c3364a][59c3364a]]
macro_rules! build_app_with_routes {
    ($state: expr) => {{
        use axum::routing::post;
        axum::Router::new().route("/mols", post(compute_mol)).with_state($state)
    }};
}
// 59c3364a ends here

// [[file:../remote.note::285a8db0][285a8db0]]
use crate::client::Client;

impl Client {
    #[tokio::main]
    /// Request remote server compute `mol` and return computed results.
    pub async fn compute_molecule(&self, mol: &Molecule) -> Result<Computed> {
        info!("Request server to compute molecule {}", mol.title());
        let x = self.post("mols", mol).await?;
        let mol = serde_json::from_str(&x).with_context(|| format!("invalid json str: {x:?}"))?;
        Ok(mol)
    }
}
// 285a8db0 ends here

// [[file:../remote.note::389c909a][389c909a]]
use crate::Server;
use std::net::SocketAddr;

/// Start restful service for molecule computation
///
/// # Parameters
///
/// * addr: socket address to bind
/// * state: shared state between route handlers
pub(self) async fn serve_mol_comput_requests(addr: impl Into<SocketAddr>, state: TaskState) {
    use crate::rest::shutdown_signal;

    let app = build_app_with_routes!(state);
    if let Err(err) = axum::Server::bind(&addr.into())
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        error!("error in restful serivce: {err:?}");
    }
}
// 389c909a ends here

// [[file:../remote.note::f4a1566d][f4a1566d]]
impl Server {
    /// Serve as a computation server for chemical model.
    pub async fn serve_as_chemical_model(&self, model: impl ChemicalModel + Send + 'static) -> Result<()> {
        let addr = self.address;
        println!("chemical model computation server listening on {addr:?}");

        let (task_rx, task_tx) = Task::new().split();
        // serve incoming requests for computation of mol
        let h1 = tokio::spawn(async move { serve_mol_comput_requests(addr, task_tx).await });
        // handle real computation using chemical model
        let h2 = tokio::spawn(async move { serve_incoming_task_with(task_rx, model).await });
        tokio::try_join!(h1, h2)?;
        Ok(())
    }
}
// f4a1566d ends here
