// [[file:../remote.note::77b43c27][77b43c27]]
use std::fmt::Debug;
use std::net::{SocketAddr, ToSocketAddrs};
// 77b43c27 ends here

// [[file:../remote.note::0b562a75][0b562a75]]
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
// 0b562a75 ends here
