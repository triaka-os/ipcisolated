pub mod config;
mod redirector;

use clap::Parser;
use config::Service;
use std::{error::Error, path::PathBuf};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cmdline {
    #[arg(short = 'D', value_parser = parse_key_val::<String, String>)]
    defines: Vec<(String, String)>,

    #[arg(long = "service")]
    services: Vec<PathBuf>,

    /// Make `ipcisolated` fail if some services fails to start
    #[arg(long)]
    strict: bool,
}

#[tokio::main]
async fn main() {
    // Initializes the logging system
    tracing_subscriber::fmt::init();

    // Parses command-line parameters
    let cmdline = Cmdline::parse();

    // Reads service files
    let mut services = Vec::with_capacity(cmdline.services.len());
    for i in cmdline.services {
        let service = match config::Service::from_path(&i).await {
            Ok(x) => x,
            Err(err) => {
                tracing::error!(
                    "failed to read service at `{}`: {}",
                    i.to_string_lossy(),
                    err
                );
                std::process::exit(1);
            }
        };
        services.push(service);
    }

    // Starts services
    for (count, service) in services.iter().enumerate() {
        match redirector::Node::from_config(service, &cmdline.defines) {
            Ok(node) => {
                tokio::spawn(async move {
                    loop {
                        match node.accept().await {
                            Ok(pair) => pair.run(),
                            Err(err) => {
                                tracing::warn!("service(id={}): accept() failed: {}", count, err);
                            },
                        }
                    }
                });
            }
            Err(err) => match cmdline.strict {
                true => {
                    tracing::error!("service(id={}) failed to start: {}", count, err);
                    clean(&services, &cmdline.defines).await;
                    std::process::exit(3);
                }
                false => {
                    tracing::warn!("service(id={}) failed to start: {}", count, err);
                    continue;
                },
            },
        }
    }

    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!("failed listening Ctrl-C signal: {}", err);
    }
    clean(&services, &cmdline.defines).await;
}

/// Removes files to prepare for exiting
async fn clean(services: &[Service], defines: &[(String, String)]) {
    tracing::trace!("cleaning up...");
    for service in services {
        if let Err(err) = tokio::fs::remove_file(service.isolated_path(defines)).await {
            tracing::warn!("a service failed to cleanup: {}", err);
        }
    }
}

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
