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
async fn main() -> anyhow::Result<()> {
    let cmdline = Cmdline::parse();

    let mut services = Vec::with_capacity(cmdline.services.len());
    for i in cmdline.services {
        let blob = tokio::fs::read_to_string(i).await?;
        let service: config::Service = serde_json::from_str(&blob)?;
        services.push(service);
    }

    for service in services.iter() {
        match redirector::Node::from_config(service, &cmdline.defines) {
            Ok(node) => {tokio::spawn(async move {
                loop {
                    if let Ok(pair) = node.accept().await {
                        pair.run();
                    }
                }
            });},
            Err(err) => match cmdline.strict {
                true => {
                    clean(&services, &cmdline.defines).await;
                    Err(err)?
                },
                false => continue,
            },
        }
    }

    tokio::signal::ctrl_c().await?;
    clean(&services, &cmdline.defines).await;

    Ok(())
}

async fn clean(services: &[Service], defines: &[(String, String)]) {
    for service in services {
        tokio::fs::remove_file(service.isolated_path(defines)).await.ok();
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
