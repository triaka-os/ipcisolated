use crate::config::Service;
use std::path::PathBuf;
use tokio::net::{UnixListener, UnixStream};

/// An isolation node.
#[derive(Debug)]
pub struct Node {
    raw_path: PathBuf,
    isolated: UnixListener,
}
impl Node {
    /// Creates a new `Node` from configuration.
    pub fn from_config(service: &Service, defines: &[(String, String)]) -> std::io::Result<Self> {
        Ok(Self {
            raw_path: service.src_in(defines),
            isolated: UnixListener::bind(service.dst_in(defines))?,
        })
    }

    /// Accepts an connection, returning a [Pair].
    pub async fn accept(&self) -> std::io::Result<Pair> {
        let client = self.isolated.accept().await?.0;
        let server = UnixStream::connect(&self.raw_path).await?;
        Ok(Pair { client, server })
    }
}

/// A pair of isolated connection.
#[derive(Debug)]
pub struct Pair {
    client: UnixStream,
    server: UnixStream,
}
impl Pair {
    /// Runs the pair.
    pub fn run(mut self) {
        tokio::spawn(async move {
            tokio::io::copy_bidirectional(&mut self.server, &mut self.client)
                .await
                .ok();
        });
    }
}
