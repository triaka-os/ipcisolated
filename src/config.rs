use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
}

fn process_defines(s: &str, defines: &[(String, String)]) -> PathBuf {
    let mut result = s.to_owned();
    for (k, v) in defines {
        result = result.replace(&format!("${}", k), v);
    }
    result.into()
}
