// [[file:../remote.note::c67d342c][c67d342c]]
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
    job_results: Vec<Result<String>>,
}

impl JobHub {
    /// Create a job hub for background scheduler specified in
    /// `scheduler_address`.
    pub fn new(scheduler_address: &str) -> Self {
        let client = Client::connect(&scheduler_address);
        Self {
            client,
            jobs: vec![],
            job_results: vec![],
        }
    }

    /// Return the number of parallel threads.
    pub fn num_threads() -> usize {
        rayon::current_num_threads()
    }

    // FIXME: do not work, as already set
    // /// Set up number of threads for parallel run.
    // pub fn set_num_threads(n: usize) -> Result<()> {
    //     rayon::ThreadPoolBuilder::new().num_threads(n).build_global()?;
    //     Ok(())
    // }

    /// Add a new job into job hub for scheduling.
    pub fn add_job(&mut self, cmd: String) -> usize {
        self.jobs.push((cmd, ".".into()));
        self.jobs.len() - 1
    }

    /// Add a new job into job hub for scheduling.
    pub fn get_job_out(&mut self, jobid: usize) -> Result<String> {
        match self.job_results.get(jobid).unwrap() {
            Err(e) => bail!("job {jobid} failed with error: {e:?}"),
            Ok(r) => Ok(r.to_owned()),
        }
    }

    /// Run all scheduled jobs with nodes in pool.
    pub fn run(&mut self) -> Result<()> {
        self.job_results = self
            .jobs
            .par_iter()
            .map(|(cmd, wrk_dir)| self.client.clone().run_cmd(&cmd, &wrk_dir))
            .collect();

        // clear pending jobs
        self.jobs.clear();

        Ok(())
    }
}
// a3bb4770 ends here
