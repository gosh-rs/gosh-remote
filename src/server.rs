// [[file:../remote.note::77b43c27][77b43c27]]
use crate::common::*;
use std::fmt::Debug;
use std::net::{SocketAddr, ToSocketAddrs};
// 77b43c27 ends here

// [[file:../remote.note::0b562a75][0b562a75]]
use crate::get_free_tcp_address;

/// Computation server.
pub struct Server {
    pub address: SocketAddr,
}

impl Server {
    /// Create a `Server` binding to `addr`.
    pub fn bind(addr: impl ToSocketAddrs + Debug) -> Self {
        let addrs: Vec<_> = addr.to_socket_addrs().expect("bad address").collect();
        assert!(addrs.len() > 0, "invalid server address: {addr:?}");
        Self { address: addrs[0] }
    }

    /// Create a `Server` binding to a free available address automatically.
    pub fn try_bind_auto() -> Result<Self> {
        let address = get_free_tcp_address().ok_or(format_err!("no free tcp addr"))?;
        Ok(Self { address })
    }
}
// 0b562a75 ends here
