// [[file:../remote.note::92f27790][92f27790]]
use super::*;
use gut::fs::*;
use std::io::{Read, Write};
use tokio::net::UnixStream;
// 92f27790 ends here

// [[file:../remote.note::*base][base:1]]
/// Client of remote execution
pub struct Client {
    address: String,
}
// base:1 ends here

// [[file:../remote.note::e5fdc097][e5fdc097]]
impl Client {
    /// Make connection to unix domain socket server
    pub async fn connect(server_address: &str) -> Result<Self> {
        debug!("Connect to remote server: {server_address:?}");
        todo!();
    }

    /// Request server to run cmd_line in working_dir and wait until complete.
    pub async fn interact(&mut self, cmd_line: &str, working_dir: &str) -> Result<String> {
        debug!("Request server to run {cmd_line:?} in {working_dir:?} ...");
        // let op = codec::ServerOp::Command((cmd_line.into(), working_dir.into()));
        // self.send_op(op).await?;

        // trace!("receiving output");
        // let txt = codec::recv_msg_decode(&mut self.stream).await?;
        // trace!("got {} bytes", txt.len());

        todo!();
    }

    /// Request server to add remote node into server list for remote computation.
    pub async fn add_node(&mut self, node: String) -> Result<()> {
        todo!();
    }
}
// e5fdc097 ends here
