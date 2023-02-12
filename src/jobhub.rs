// [[file:../remote.note::c67d342c][c67d342c]]
use crate::common::*;
use crate::Client;

use gchemol::Molecule;
use gosh_model::Computed;

use std::path::{Path, PathBuf};
// c67d342c ends here

// [[file:../remote.note::a3bb4770][a3bb4770]]
/// A job hub for parallel running of multiple jobs over remote
/// computational nodes
pub struct JobHub {
    // the client for sending requests
    client: Client,
    // the molcules to be computed
    jobs: Vec<Molecule>,
    // the computed results
    results: Vec<Result<Computed>>,
}
// a3bb4770 ends here

// [[file:../remote.note::b2b2f089][b2b2f089]]
use gut::rayon;

impl JobHub {
    /// Create a job hub for background scheduler specified in
    /// `scheduler_address`.
    pub fn new(scheduler_address: &str) -> Self {
        let client = Client::connect(&scheduler_address);
        Self {
            client,
            jobs: vec![],
            results: vec![],
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
}
// b2b2f089 ends here

// [[file:../remote.note::c3d41589][c3d41589]]
impl JobHub {
    /// Add a new mol into job hub for later computation. Return associated
    /// jobid which can be used to retrive computation result.
    pub fn add_job(&mut self, mol: Molecule) -> usize {
        self.jobs.push(mol);
        self.jobs.len() - 1
    }

    /// Return the numbrer of pending jobs.
    pub fn njobs(&self) -> usize {
        self.jobs.len()
    }

    /// Run all scheduled jobs with nodes in pool. Call this method
    /// will overwrite computed results and clear pending jobs.
    pub fn run(&mut self) -> Result<()> {
        self.results = self
            .jobs
            .par_iter()
            .map(|mol| self.client.compute_molecule(mol))
            .collect();

        // clear pending jobs
        self.jobs.clear();

        Ok(())
    }

    /// Return computed result for job `jobid`.
    pub fn get_computed(&mut self, jobid: usize) -> Result<Computed> {
        let computed = self.results.get(jobid).ok_or(anyhow!("no such job {jobid}"))?;
        match computed {
            Err(e) => bail!("job {jobid} failed with error: {e:?}"),
            Ok(r) => Ok(r.to_owned()),
        }
    }
}
// c3d41589 ends here
