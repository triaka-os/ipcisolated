use crate::config::Service;
use std::path::PathBuf;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

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
            raw_path: service.raw_path(defines),
            isolated: UnixListener::bind(service.isolated_path(defines))?,
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
    pub fn run(self) {
        let (mut client_r, mut client_w) = self.client.into_split();
        let (mut server_r, mut server_w) = self.server.into_split();
        tokio::spawn(async move {
            redirect(&mut client_r, &mut server_w).await.ok();
        });
        tokio::spawn(async move {
            redirect(&mut server_r, &mut client_w).await.ok();
        });
    }
}

/// Redirects a pair of [AsyncRead] and [AsyncWrite]
async fn redirect(
    read: &mut (dyn AsyncRead + Send + Unpin),
    write: &mut (dyn AsyncWrite + Send + Unpin),
) -> std::io::Result<()> {
    let mut buf = [0u8];
    loop {
        read.read_exact(&mut buf).await?;
        write.write_all(&buf).await?;
    }
}
