// [[file:../remote.note::92f27790][92f27790]]
use super::*;
use gut::fs::*;
// 92f27790 ends here

// [[file:../remote.note::99dad0b0][99dad0b0]]
/// Client of remote execution
pub struct Client {
    client: reqwest::blocking::Client,
    service_uri: String,
}

impl Client {
    /// The connection address like "localhost:12345"
    pub fn new(address: &str) -> Self {
        // NOTE: the default request timeout is 30 seconds. Here we disable
        // timeout using reqwest builder.
        let client = reqwest::blocking::Client::builder()
            .timeout(None)
            .build()
            .expect("reqwest client");
        let service_uri = format!("http://{}", address);
        Self { client, service_uri }
    }
}
// 99dad0b0 ends here

// [[file:../remote.note::e5fdc097][e5fdc097]]
impl Client {
    pub(crate) fn post(&self, end_point: &str, data: impl serde::Serialize) -> Result<String> {
        let uri = format!("{}/{end_point}", self.service_uri);
        let resp = self
            .client
            .post(&uri)
            .json(&data)
            .send()?
            .text()
            .context("client requests to create job")?;

        Ok(resp)
    }
}
// e5fdc097 ends here
