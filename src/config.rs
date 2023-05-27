use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    pub raw_path: String,
    pub isolated_path: String,
}
impl Service {
    pub fn raw_path(&self, defines: &[(String, String)]) -> PathBuf {
        process_defines(&self.raw_path, defines)
    }
    pub fn isolated_path(&self, defines: &[(String, String)]) -> PathBuf {
        process_defines(&self.isolated_path, defines)
    }
    pub async fn from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let blob = tokio::fs::read_to_string(path).await?;
        let service: Self = serde_json::from_str(&blob)?;
        Ok(service)
    }
}

fn process_defines(s: &str, defines: &[(String, String)]) -> PathBuf {
    let mut result = s.to_owned();
    for (k, v) in defines {
        result = result.replace(&format!("${}", k), v);
    }
    result.into()
}
