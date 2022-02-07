// [[file:../remote.note::3a532d42][3a532d42]]
use super::*;
use gut::cli::*;
use gut::fs::*;

pub use gut::prelude::*;
// 3a532d42 ends here

// [[file:../remote.note::512e88e7][512e88e7]]
// use crate::remote::{Client, Server};

/// The client side for running program concurrently distributed over multiple
/// remote nodes
#[derive(StructOpt)]
struct ClientCli {
    /// The remote execution service address, e.g. localhost:3030
    #[structopt(long = "scheduler")]
    scheduler_address: String,

    #[clap(subcommand)]
    action: ClientAction,
}

#[derive(Subcommand)]
enum ClientAction {
    Run(ClientRun),
    /// Request server to add a new node for remote computation.
    AddNode {
        /// The node to be added into node list for remote computation.
        node: String,
    },
}

#[derive(StructOpt)]
/// request server to run a cmd
struct ClientRun {
    /// The cmd to run in remote session
    cmd: String,

    /// The working dir to run the cmd
    #[structopt(long, default_value = ".")]
    wrk_dir: PathBuf,
}

impl ClientCli {
    async fn enter_main(self) -> Result<()> {
        use crate::client::Client;

        let client = Client::connect(&self.scheduler_address);
        match self.action {
            ClientAction::Run(run) => {
                let wrk_dir = run.wrk_dir.canonicalize()?;
                let o = client.run_cmd(&run.cmd, &wrk_dir)?;
                println!("{o}");
            }
            ClientAction::AddNode { node } => {
                client.add_node(&node)?;
            }
        }

        Ok(())
    }
}
// 512e88e7 ends here

// [[file:../remote.note::674c2404][674c2404]]
use crate::server::Server;

#[derive(Debug, Clone, ArgEnum)]
enum ServerMode {
    AsScheduler,
    AsWorker,
}

/// The server side for running program concurrently distributed over multiple remote nodes
#[derive(Parser, Debug)]
struct ServerCli {
    /// Bind on the address for providing remote execution service
    #[clap(long)]
    address: String,

    /// The server mode to start.
    #[clap(arg_enum)]
    mode: ServerMode,
}

impl ServerCli {
    async fn enter_main(self) -> Result<()> {
        let address = &self.address;
        match self.mode {
            ServerMode::AsScheduler => {
                println!("Start scheduler serivce at {address:?}");
                Server::serve_as_scheduler(address).await;
            }
            ServerMode::AsWorker => {
                println!("Start worker serivce at {address:?}");
                Server::serve_as_worker(address).await?;
            }
        }

        Ok(())
    }
}
// 674c2404 ends here

// [[file:../remote.note::001e63a1][001e63a1]]
use base::wait_file;

#[derive(StructOpt)]
struct MpiCli {}

impl MpiCli {
    async fn enter_main(&self) -> Result<()> {
        run_scheduler_or_worker_dwim().await?;
        Ok(())
    }
}

/// Run scheduler or worker according to MPI local rank ID
async fn run_scheduler_or_worker_dwim() -> Result<()> {
    let node = hostname();
    let lock_file_scheduler = format!("gosh-remote-scheduler-{node}.lock");
    let lock_file_worker = format!("gosh-remote-worker-{node}.lock");

    match mpi::get_local_rank_id() {
        Some(0) => {
            info!("run as a scheduler on {node} ...");
            let address = format!("{node}:3030");
            let server = ServerCli {
                address: address.clone(),
                mode: ServerMode::AsScheduler,
            };
            let lock = LockFile::new(lock_file_scheduler.as_ref(), &address);
            server.enter_main().await?;
        }
        Some(rank) => {
            info!("run as worker on {node} in rank {rank} ...");
            let address = format!("{node}:3031");
            let server = ServerCli {
                address: address.clone(),
                mode: ServerMode::AsWorker,
            };
            wait_file(lock_file_scheduler.as_ref(), 2.0)?;
            let o = gut::fs::read_file(lock_file_scheduler)?;
            let client = crate::client::Client::connect(o.trim());
            if address_available(&address) {
                client.add_node(&address)?;
                server.enter_main().await?;
            } else {
                error!("{address} is not available for binding");
            }
        }
        None => {
            bail!("no relevant MPI env var found!");
        }
    };

    Ok(())
}
// 001e63a1 ends here

// [[file:../remote.note::5f9971ad][5f9971ad]]
/// A helper program for running program concurrently distributed over multiple
/// remote nodes
#[derive(Parser)]
struct Cli {
    #[structopt(flatten)]
    verbose: gut::cli::Verbosity,

    #[clap(subcommand)]
    command: RemoteCommand,
}

#[derive(Subcommand)]
enum RemoteCommand {
    Client(ClientCli),
    Server(ServerCli),
    Mpi(MpiCli),
}

#[tokio::main]
pub async fn remote_enter_main() -> Result<()> {
    let args = Cli::parse();
    args.verbose.setup_logger();

    match args.command {
        RemoteCommand::Client(client) => {
            client.enter_main().await?;
        }
        RemoteCommand::Server(server) => {
            debug!("Run VASP for interactive calculation ...");
            server.enter_main().await?;
        }
        RemoteCommand::Mpi(mpi) => {
            mpi.enter_main().await?;
        }
    }

    Ok(())
}
// 5f9971ad ends here
