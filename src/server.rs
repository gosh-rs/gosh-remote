// [[file:../remote.note::aa689ac9][aa689ac9]]
use super::*;
use std::fmt::Debug;
use std::net::{SocketAddr, ToSocketAddrs};

/// Computation server.
pub struct Server {
    pub address: SocketAddr,
}

impl Server {
    pub fn new(addr: impl ToSocketAddrs + Debug) -> Self {
        let addrs: Vec<_> = addr.to_socket_addrs().expect("bad address").collect();
        assert!(addrs.len() > 0, "invalid server address: {addr:?}");
        Self { address: addrs[0] }
    }
}
// aa689ac9 ends here
