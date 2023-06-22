use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub services: Vec<Service>,
}
impl Config {
    pub fn read_from(path: &str) -> anyhow::Result<Self> {
        let rdr: Box<dyn std::io::Read> = match path {
            "-" => Box::new(std::io::stdin()),
            x => Box::new(std::fs::File::open(x)?),
        };
        let service: Self = serde_json::from_reader(rdr)?;
        Ok(service)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    pub src: String,
    pub dst: String,
}
impl Service {
    pub fn src_in(&self, defines: &[(String, String)]) -> PathBuf {
        process_defines(&self.src, defines)
    }
    pub fn dst_in(&self, defines: &[(String, String)]) -> PathBuf {
        process_defines(&self.dst, defines)
    }
}

fn process_defines(s: &str, defines: &[(String, String)]) -> PathBuf {
    let mut result = s.to_owned();
    for (k, v) in defines {
        result = result.replace(&format!("${}", k), v);
    }
    result.into()
}
