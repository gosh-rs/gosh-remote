// [[file:../remote.note::fed8a9d3][fed8a9d3]]
use super::*;
// fed8a9d3 ends here

// [[file:../remote.note::49f269df][49f269df]]
use gut::utils::sleep;

/// Wait until file `f` available for max time of `timeout`.
///
/// # Parameters
/// * timeout: timeout in seconds
/// * f: the file to wait for available
pub fn wait_file(f: &std::path::Path, timeout: f64) -> Result<()> {
    // wait a moment for socke file ready
    let interval = 0.1;
    let mut t = 0.0;
    loop {
        if f.exists() {
            trace!("Elapsed time during waiting: {:.2} seconds ", t);
            return Ok(());
        }
        t += interval;
        sleep(interval);

        if t > timeout {
            bail!("file {:?} doest exist for {} seconds", f, timeout);
        }
    }
}
// 49f269df ends here

// [[file:../remote.note::50e6ed5a][50e6ed5a]]
/// Represents a computational job inputted by user.
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Job {
    // FIXME: remove pub
    /// The content of running script
    pub(crate) script: String,

    /// A unique random name
    name: String,

    /// Path to a file for saving output stream of computation.
    pub out_file: PathBuf,

    /// Path to a file for saving error stream of computation.
    pub err_file: PathBuf,

    /// Path to a script file that defining how to start computation
    pub run_file: PathBuf,
}

impl Default for Job {
    fn default() -> Self {
        Self {
            script: "pwd".into(),
            name: random_name(),
            out_file: "job.out".into(),
            err_file: "job.err".into(),
            run_file: "run".into(),
        }
    }
}

impl Job {
    /// Construct a Job running shell script.
    ///
    /// # Parameters
    ///
    /// * script: the content of the script for running the job.
    ///
    pub fn new(script: impl Into<String>) -> Self {
        Self {
            script: script.into(),
            ..Default::default()
        }
    }

    /// Set job name.
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.into();
        self
    }

    /// Return the job name
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

fn random_name() -> String {
    use rand::distributions::Alphanumeric;
    use rand::Rng;

    let mut rng = rand::thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(6)
        .collect()
}
// 50e6ed5a ends here

// [[file:../remote.note::769262a8][769262a8]]
mod node {
    use super::*;
    use crossbeam_channel::{unbounded, Receiver, Sender};

    /// Represents a remote node for computation
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Node {
        name: String,
    }

    impl Node {
        /// Return the name of remote node
        pub fn name(&self) -> &str {
            &self.name
        }
    }

    impl<T: Into<String>> From<T> for Node {
        fn from(node: T) -> Self {
            let name = node.into();
            assert!(!name.is_empty(), "node name cannot be empty!");
            Self { name }
        }
    }

    impl std::fmt::Display for Node {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let name = &self.name;
            write!(f, "{name}")
        }
    }

    /// Represents a list of remote nodes allocated for computation
    #[derive(Clone)]
    pub struct Nodes {
        rx: Receiver<Node>,
        tx: Sender<Node>,
    }

    impl Nodes {
        /// Construct `Nodes` from a list of nodes.
        pub fn new<T: Into<Node>>(nodes: impl IntoIterator<Item = T>) -> Self {
            let (tx, rx) = unbounded();
            let nodes = nodes.into_iter().collect_vec();
            let n = nodes.len();
            info!("We have {n} nodes in totoal for computation.");
            for node in nodes {
                tx.send(node.into()).unwrap();
            }
            Self { rx, tx }
        }

        /// Return the number of nodes
        pub fn len(&self) -> usize {
            self.rx.len()
        }

        /// Borrow one node from `Nodes`
        pub fn borrow_node(&self) -> Result<Node> {
            let node = self.rx.recv()?;
            let name = &node.name;
            info!("client borrowed one node: {name:?}");
            Ok(node)
        }

        /// Return one `node` to `Nodes`
        pub fn return_node(&self, node: Node) -> Result<()> {
            let name = &node.name;
            info!("client returned node {name:?}");
            self.tx.send(node)?;
            Ok(())
        }
    }
}
// 769262a8 ends here

// [[file:../remote.note::e19bce71][e19bce71]]
use std::path::{Path, PathBuf};

use gosh_runner::prelude::SpawnSessionExt;
use gosh_runner::process::Session;

use tempfile::TempDir;
// e19bce71 ends here

// [[file:../remote.note::955c926a][955c926a]]
/// Computation represents a submitted `Job`
pub struct Computation {
    job: Job,

    /// command session. The drop order is above Tempdir
    session: Option<Session<tokio::process::Child>>,

    /// The working directory of computation
    wrk_dir: TempDir,
}
// 955c926a ends here

// [[file:../remote.note::a65e6dae][a65e6dae]]
impl Computation {
    /// The full path to the working directory for running the job.
    fn wrk_dir(&self) -> &Path {
        self.wrk_dir.path()
    }

    /// The full path to computation output file (stdout).
    fn out_file(&self) -> PathBuf {
        self.wrk_dir().join(&self.job.out_file)
    }

    /// The full path to computation error file (stderr).
    fn err_file(&self) -> PathBuf {
        self.wrk_dir().join(&self.job.err_file)
    }

    /// The full path to the script for running the job.
    fn run_file(&self) -> PathBuf {
        self.wrk_dir().join(&self.job.run_file)
    }
}
// a65e6dae ends here

// [[file:../remote.note::f8672e0c][f8672e0c]]
impl Job {
    /// Submit the job and turn it into Computation.
    pub fn submit(self) -> Result<Computation> {
        Computation::try_run(self)
    }
}

impl Computation {
    /// create run file and make sure it executable later
    fn create_run_file(&self) -> Result<()> {
        let run_file = &self.run_file();
        gut::fs::write_script_file(run_file, &self.job.script)?;
        wait_file(&run_file, 2.0)?;

        Ok(())
    }

    /// Construct `Computation` of user inputted `Job`.
    fn try_run(job: Job) -> Result<Self> {
        // create working directory in scratch space.
        let wdir = tempfile::TempDir::new_in(".").expect("temp dir");
        let session = Self {
            job,
            wrk_dir: wdir.into(),
            session: None,
        };

        session.create_run_file()?;

        Ok(session)
    }

    /// Wait for background command to complete.
    async fn wait(&mut self) -> Result<()> {
        if let Some(s) = self.session.as_mut() {
            let ecode = s.child.wait().await?;
            info!("job session exited: {}", ecode);
            if !ecode.success() {
                error!("job exited unsuccessfully!");
                let txt = gut::fs::read_file(self.run_file())?;
                let run = format!("run file: {txt:?}");
                let txt = gut::fs::read_file(self.err_file())?;
                let err = format!("stderr: {txt:?}");
                bail!("Job failed with error:\n{run:?}{err:?}");
            }
            Ok(())
        } else {
            bail!("Job not started yet.");
        }
    }

    /// Run command in background.
    async fn start(&mut self) -> Result<()> {
        let program = self.run_file();
        let wdir = self.wrk_dir();
        info!("job work direcotry: {}", wdir.display());

        let mut session = tokio::process::Command::new(&program)
            .current_dir(wdir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn_session()?;

        let mut stdout = session.child.stdout.take().expect("child did not have a handle to stdout");
        let mut stderr = session.child.stderr.take().expect("child did not have a handle to stderr");

        // redirect stdout and stderr to files for user inspection.
        let mut fout = tokio::fs::File::create(self.out_file()).await?;
        let mut ferr = tokio::fs::File::create(self.err_file()).await?;
        tokio::io::copy(&mut stdout, &mut fout).await?;
        tokio::io::copy(&mut stderr, &mut ferr).await?;

        let sid = session.handler().id();
        info!("command running in session {:?}", sid);
        self.session = session.into();

        Ok(())
    }

    /// Start computation, and wait and return its standard output
    pub async fn wait_for_output(&mut self) -> Result<String> {
        self.start().await?;
        self.wait().await?;
        let txt = gut::fs::read_file(self.out_file())?;
        Ok(txt)
    }

    /// Return true if session already has been started.
    fn is_started(&self) -> bool {
        self.session.is_some()
    }
}
// f8672e0c ends here

// [[file:../remote.note::34c67980][34c67980]]
impl Computation {
    /// Check if job has been done correctly.
    fn is_done(&self) -> bool {
        let runfile = self.run_file();
        let outfile = self.out_file();

        if self.wrk_dir().is_dir() {
            if outfile.is_file() && runfile.is_file() {
                if let Ok(time2) = outfile.metadata().and_then(|m| m.modified()) {
                    if let Ok(time1) = runfile.metadata().and_then(|m| m.modified()) {
                        if time2 >= time1 {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Update file timestamps to make sure `is_done` call return true.
    fn fake_done(&self) {
        todo!()
    }
}
// 34c67980 ends here

// [[file:../remote.note::4a28f1b7][4a28f1b7]]
pub use node::{Node, Nodes};
// 4a28f1b7 ends here

// [[file:../remote.note::f725ca9b][f725ca9b]]
#[test]
fn test_node() {
    let localhost: Node = "localhost".into();
    assert_eq!(localhost.name(), "localhost");
}
// f725ca9b ends here
