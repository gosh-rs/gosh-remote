// [[file:../remote.note::3a532d42][3a532d42]]
use super::*;
use gut::cli::*;
use gut::fs::*;

pub use gut::prelude::*;
// 3a532d42 ends here

// [[file:../remote.note::5f9971ad][5f9971ad]]
#[derive(Parser)]
enum Cli {
    Client(ClientCli),
    Server(ServerCli),
}

#[tokio::main]
pub async fn remote_enter_main() -> Result<()> {
    match Cli::from_args() {
        Cli::Client(client) => {
            client.enter_main().await?;
        }
        Cli::Server(server) => {
            debug!("Run VASP for interactive calculation ...");
            server.enter_main().await?;
        }
    }

    Ok(())
}
// 5f9971ad ends here

// [[file:../remote.note::512e88e7][512e88e7]]
// use crate::remote::{Client, Server};

/// A client of a unix domain socket server for interacting with the program
/// run in background
#[derive(StructOpt)]
struct ClientCli {
    /// The remote execution service address, e.g. localhost:3030
    #[structopt(long, default_value = "localhost:3030")]
    address: String,

    #[clap(subcommand)]
    action: ClientAction,
}

#[derive(Subcommand)]
enum ClientAction {
    Run {
        #[clap(flatten)]
        run: ClientRun,
    },
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
        // let mut stream = Client::connect(&self.address).await?;
        // match self.action {
        //     ClientAction::Run { run } => {
        //         let wrk_dir = run.wrk_dir.canonicalize()?;
        //         let wrk_dir = wrk_dir.to_string_lossy();
        //         stream.interact_with_remote_session(&run.cmd, &wrk_dir).await?;
        //     }
        //     ClientAction::AddNode { node } => {
        //         stream.add_node(node).await?;
        //     }
        // }

        Ok(())
    }
}
// 512e88e7 ends here

// [[file:../remote.note::674c2404][674c2404]]
use crate::server::Server;

/// A helper program to run VASP calculation in remote node
#[derive(Parser, Debug)]
struct ServerCli {
    #[structopt(flatten)]
    verbose: gut::cli::Verbosity,

    /// Bind on the address for providing remote execution service
    #[clap(default_value = "localhost:3030")]
    address: String,

    /// Start server as job scheduler
    #[clap(long, conflicts_with = "as-worker")]
    as_scheduler: bool,

    /// Start server as a worker for remote computation
    #[clap(long, conflicts_with = "as-scheduler")]
    as_worker: bool,
}

impl ServerCli {
    async fn enter_main(self) -> Result<()> {
        let args = ServerCli::parse();
        args.verbose.setup_logger();

        if args.as_scheduler {
            Server::serve_as_scheduler(&self.address).await;
        } else {
            Server::serve_as_worker(&self.address).await;
        }

        Ok(())
    }
}
// 674c2404 ends here
