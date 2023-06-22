pub mod config;
mod proxy;

use clap::Parser;
use config::{Config, Service};
use std::error::Error;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cmdline {
    /// Define <key> to <value>
    #[arg(short = 'D', value_parser = parse_key_val::<String, String>)]
    defines: Vec<(String, String)>,

    /// Filesystem path of the configuration, or `-` for reading from stdin
    #[arg(short, long)]
    config: String,

    /// Don't cleanup on exit
    #[arg(long, default_value_t)]
    no_cleanup: bool,
}

#[tokio::main]
async fn main() {
    // Initializes the logging system
    tracing_subscriber::fmt::init();

    // Parses command-line parameters
    let cmdline = Cmdline::parse();

    // Reads the configuration
    let config = Config::read_from(&cmdline.config).unwrap_or_else(|err| {
        tracing::error!("failed to read the configuration: {}", err);
        std::process::exit(1);
    });

    // Starts services
    for (count, service) in config.services.iter().enumerate() {
        match proxy::Node::from_config(service, &cmdline.defines) {
            Ok(node) => {
                tokio::spawn(async move {
                    loop {
                        match node.accept().await {
                            Ok(pair) => pair.run(),
                            Err(err) => {
                                tracing::warn!("service(id={}): accept() failed: {}", count, err);
                            }
                        }
                    }
                });
            }
            Err(err) => {
                tracing::error!("service(id={}) failed to start: {}", count, err);
                clean(&cmdline, &config.services, &cmdline.defines).await;
                std::process::exit(3);
            }
        }
    }

    // Wait for Ctrl-C signal to clean the environment up
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!("failed listening Ctrl-C signal: {}", err);
    }
    clean(&cmdline, &config.services, &cmdline.defines).await;
}

/// Removes files to prepare for exiting
async fn clean(cmdline: &Cmdline, services: &[Service], defines: &[(String, String)]) {
    if !cmdline.no_cleanup {
        tracing::trace!("cleaning up...");
        for service in services {
            if let Err(err) = tokio::fs::remove_file(service.dst_in(defines)).await {
                tracing::warn!("a service failed to cleanup: {}", err);
            }
        }
    }
}

/// Parses key-value command-line arguments
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
