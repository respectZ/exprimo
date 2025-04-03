use anyhow::{bail, Result};
use serde_json::Value;

pub struct ArrayMethod {
    args: Vec<Value>,
}

impl ArrayMethod {
    pub fn new(args: Vec<Value>) -> Self {
        ArrayMethod { args }
    }
    pub fn join(&self, arr: &Vec<Value>) -> Result<Value> {
        if self.args.len() != 1 {
            bail!("join method requires 1 argument")
        }
        let delimiter = self.args[0].as_str().unwrap_or("");
        let result: Vec<String> = arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();
        let joined = result.join(delimiter);
        Ok(Value::String(joined))
    }
}
