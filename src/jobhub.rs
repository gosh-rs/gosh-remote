// [[file:../remote.note::c67d342c][c67d342c]]
use crate::base::{Job, Node};
use crate::client::Client;
use crate::common::*;
use gut::rayon;

use std::path::{Path, PathBuf};
// c67d342c ends here

// [[file:../remote.note::a3bb4770][a3bb4770]]
/// A job hub for parallel running of multiple jobs over remote
/// computational nodes
pub struct JobHub {
    client: Client,
    jobs: Vec<(String, PathBuf)>,
}

impl JobHub {
    /// Create a job hub for background scheduler specified in
    /// `scheduler_address`.
    pub fn new(scheduler_address: &str) -> Self {
        let client = Client::connect(&scheduler_address);
        Self { client, jobs: vec![] }
    }

    /// Return the number of parallel threads.
    pub fn num_threads() -> usize {
        rayon::current_num_threads()
    }

    /// Set up number of threads for parallel run.
    pub fn set_num_threads(n: usize) -> Result<()> {
        rayon::ThreadPoolBuilder::new().num_threads(n).build_global()?;
        Ok(())
    }

    /// Add a new node into node pool for execution.
    pub fn add_node(&mut self, node: Node) -> Result<()> {
        self.client.add_node(node)?;
        Ok(())
    }

    /// Add a new job into job hub for scheduling.
    pub fn add_job(&mut self, cmd: String, wrk_dir: PathBuf) {
        self.jobs.push((cmd, wrk_dir));
    }

    /// Run all scheduled jobs with nodes in pool.
    pub fn run(mut self) -> Result<()> {
        let results: Vec<_> = self
            .jobs
            .into_par_iter()
            .map(|(cmd, wrk_dir)| self.client.clone().run_cmd(&cmd, &wrk_dir))
            .collect();

        for (i, x) in results.iter().enumerate() {
            println!("job {i}: {x:?}");
        }

        Ok(())
    }
}
// a3bb4770 ends here
