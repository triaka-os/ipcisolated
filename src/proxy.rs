use crate::config::Service;
use std::path::PathBuf;
use tokio::net::{UnixListener, UnixStream};

/// An isolation node.
#[derive(Debug)]
pub struct Node {
    src: PathBuf,
    dst: UnixListener,
}
impl Node {
    /// Creates a new `Node` from configuration.
    pub fn from_config(service: &Service, defines: &[(String, String)]) -> std::io::Result<Self> {
        Ok(Self {
            src: service.src_in(defines),
            dst: UnixListener::bind(service.dst_in(defines))?,
        })
    }

    /// Accepts an connection, returning a [Pair].
    pub async fn accept(&self) -> std::io::Result<Pair> {
        let client = self.dst.accept().await?.0;
        let server = UnixStream::connect(&self.src).await?;
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
