// [[file:../remote.note::dba9de5e][dba9de5e]]
//! Distributed parallel computing over multiple nodes.
// dba9de5e ends here

// [[file:../remote.note::963f5eb8][963f5eb8]]
use gosh_core::*;
use gut::prelude::*;

use std::path::{Path, PathBuf};
// 963f5eb8 ends here

// [[file:../remote.note::b21b77b4][b21b77b4]]
mod base;
mod client;
mod rest;
mod scheduler;
mod server;
mod worker;

pub mod cli;
pub mod task;

// experimental
mod jobhub;
// b21b77b4 ends here

// [[file:../remote.note::5f427630][5f427630]]
mod common {
    pub use gosh_core::gut::prelude::*;
    pub use gosh_core::*;
}
// 5f427630 ends here

// [[file:../remote.note::5c33a18a][5c33a18a]]
/// Return system host name
fn hostname() -> String {
    nix::unistd::gethostname().unwrap().to_str().unwrap().to_string()
}
// 5c33a18a ends here

// [[file:../remote.note::92bf67b7][92bf67b7]]
use std::net::{SocketAddr, TcpListener, ToSocketAddrs};

/// Test if `address` available for socket binding
pub fn address_available<A: ToSocketAddrs>(address: A) -> bool {
    match TcpListener::bind(address) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Return the address available for binding with the OS assigns port.
pub fn get_free_tcp_address() -> Option<SocketAddr> {
    let host = hostname();
    TcpListener::bind(format!("{host}:0")).ok()?.local_addr().ok()
}

#[test]
fn test_addr() {
    let addr = get_free_tcp_address().unwrap();
    assert!(address_available(dbg!(addr)));
}
// 92bf67b7 ends here

// [[file:../remote.note::0a725e9c][0a725e9c]]
pub use base::LockFile;

pub use crate::client::Client;
pub use crate::server::Server;
pub use jobhub::JobHub;
// 0a725e9c ends here

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
    export_doc!(worker);
    export_doc!(jobhub);
    export_doc!(rest);
    export_doc!(task);
}
// 56d334b5 ends here
