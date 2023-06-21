use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub services: Vec<Service>,
}
impl Config {
    pub async fn read_from<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let blob = tokio::fs::read_to_string(path).await?;
        let service: Self = serde_json::from_str(&blob)?;
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
