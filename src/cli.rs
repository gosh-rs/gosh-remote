// [[file:../remote.note::3a532d42][3a532d42]]
use super::*;
use gut::cli::*;
use gut::fs::*;
use base::wait_file;

pub use gut::prelude::*;
// 3a532d42 ends here

// [[file:../remote.note::bdfa3d68][bdfa3d68]]
const GOSH_SCHEDULER_FILE: &str = "gosh-remote-scheduler.lock";

fn read_scheduler_address_from_lock_file(scheduler_address_file: &Path, timeout: f64) -> Result<String> {
    debug!("reading scheduler address from file: {scheduler_address_file:?}");
    wait_file(scheduler_address_file, timeout)?;
    let o = gut::fs::read_file(scheduler_address_file)?.trim().to_string();
    Ok(o)
}
// bdfa3d68 ends here

// [[file:../remote.note::512e88e7][512e88e7]]
// use crate::remote::{Client, Server};

/// The client side for running program concurrently distributed over multiple
/// remote nodes
#[derive(StructOpt)]
struct ClientCli {
    /// The remote execution service address, e.g. localhost:3031
    #[structopt(long = "address", conflicts_with = "scheduler-address-file")]
    scheduler_address: Option<String>,

    /// The scheduler address to be read from file `scheduler_address_file`
    #[structopt(short = 'w', default_value = GOSH_SCHEDULER_FILE)]
    scheduler_address_file: PathBuf,

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
        let scheduler_address = if let Some(a) = self.scheduler_address {
            a
        } else {
            read_scheduler_address_from_lock_file(&self.scheduler_address_file, 2.0)?
        };

        let client = Client::connect(&scheduler_address);
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
use base::LockFile;
use server::Server;

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

    async fn run_as_scheduler(address: String) -> Result<()> {
        let server = ServerCli {
            address: address,
            mode: ServerMode::AsScheduler,
        };
        server.enter_main().await?;
        Ok(())
    }

    async fn run_as_worker(address: String) -> Result<()> {
        let server = ServerCli {
            address: address,
            mode: ServerMode::AsWorker,
        };
        server.enter_main().await?;
        Ok(())
    }
}
// 674c2404 ends here

// [[file:../remote.note::001e63a1][001e63a1]]
/// Start scheduler and worker services automatically when run in MPI
/// environment (to be called with mpirun command)
#[derive(StructOpt)]
struct BootstrapCli {
    /// The scheduler address will be wrote into `address_file`
    #[structopt(short = 'w', default_value = GOSH_SCHEDULER_FILE)]
    address_file: PathBuf,

    #[structopt(long, default_value = "2.0")]
    timeout: f64,

    /// The server mode to start.
    #[clap(arg_enum)]
    mode: ServerMode,
}

impl BootstrapCli {
    async fn enter_main(&self) -> Result<()> {
        let node = hostname();
        let address = default_server_address();
        let address_file = self.address_file.to_owned();
        let timeout = self.timeout;
        match self.mode {
            ServerMode::AsScheduler => {
                info!("install scheduler on {node}");
                let _lock = LockFile::new(&address_file, &address)?;
                ServerCli::run_as_scheduler(address).await?;
            }
            ServerMode::AsWorker => {
                info!("install worker on {node}");
                let o = read_scheduler_address_from_lock_file(&address_file, timeout)?;
                // tell the scheduler add this worker
                crate::client::Client::connect(o).add_node(&address)?;
                ServerCli::run_as_worker(address).await?;
            }
        }
        Ok(())
    }
}

fn default_server_address() -> String {
    match get_free_tcp_address().expect("tcp address") {
        std::net::SocketAddr::V4(addr) => addr.to_string(),
        std::net::SocketAddr::V6(_) => panic!("IPV6 is not supported"),
    }
}
// 001e63a1 ends here

// [[file:../remote.note::5f9971ad][5f9971ad]]
/// A helper program for running program concurrently distributed over multiple
/// remote nodes
#[derive(Parser)]
#[clap(author, version, about)]
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
    Bootstrap(BootstrapCli),
}

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
        RemoteCommand::Bootstrap(bootstrap) => {
            bootstrap.enter_main().await?;
        }
    }

    Ok(())
}
// 5f9971ad ends here
