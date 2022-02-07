// [[file:../remote.note::dba9de5e][dba9de5e]]
//! Distributed parallel computing over multiple nodes.
// dba9de5e ends here

// [[file:../remote.note::963f5eb8][963f5eb8]]
use gosh_core::*;
use gut::prelude::*;
// 963f5eb8 ends here

// [[file:../remote.note::b21b77b4][b21b77b4]]
mod base;
mod client;
mod interactive;
mod mpi;
mod scheduler;
mod server;
mod worker;

pub mod cli;
// b21b77b4 ends here

// [[file:../remote.note::5c33a18a][5c33a18a]]
/// Return system host name
fn hostname() -> String {
    let mut buf = [0u8; 512];
    nix::unistd::gethostname(&mut buf)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

/// Test if `address` available for socket binding
fn address_available(address: &str) -> bool {
    match std::net::TcpListener::bind(address) {
        Ok(_) => true,
        Err(_) => false,
    }
}
// 5c33a18a ends here

// [[file:../remote.note::9b7911ae][9b7911ae]]
use fs2::*;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct LockFile {
    file: std::fs::File,
    path: PathBuf,
}

impl LockFile {
    fn create(path: &Path) -> Result<LockFile> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .context("Could not create ID file")?;

        // https://docs.rs/fs2/0.4.3/fs2/trait.FileExt.html
        file.try_lock_exclusive()
            .context("Could not lock ID file; Is the daemon already running?")?;

        Ok(LockFile {
            file,
            path: path.to_owned(),
        })
    }

    fn write_msg(&mut self, msg: &str) -> Result<()> {
        writeln!(&mut self.file, "{msg}").context("Could not write ID file")?;
        self.file.flush().context("Could not flush ID file")
    }

    /// Create a pidfile for process `pid`
    pub fn new(path: &Path, msg: &str) -> Result<Self> {
        let mut pidfile = Self::create(path)?;
        pidfile.write_msg(msg);

        Ok(pidfile)
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
// 9b7911ae ends here

// [[file:../remote.note::56d334b5][56d334b5]]
#[cfg(feature = "adhoc")]
/// Docs for local mods
pub mod docs {
    macro_rules! export_doc {
        ($l:ident) => {
            pub mod $l {
                pub use crate::$l::*;
            }
        };
    }

    export_doc!(base);
    export_doc!(interactive);
    export_doc!(worker);
}
// 56d334b5 ends here
