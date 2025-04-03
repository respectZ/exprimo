use anyhow::{anyhow, bail, Result};
use regex::Regex;
use serde_json::Value;

pub struct StrMethod {
    args: Vec<Value>,
}

impl StrMethod {
    pub fn new(args: Vec<Value>) -> Self {
        StrMethod { args }
    }
    pub fn replace(&self, s: &str) -> Result<Value> {
        if self.args.len() != 2 {
            bail!("replace method requires 2 arguments")
        }
        let old = self.args[0].as_str().unwrap_or("");
        let new = self.args[1].as_str().unwrap_or("");
        let result = s.replace(old, new);
        Ok(Value::String(result))
    }
    pub fn contains(&self, s: &str) -> Result<Value> {
        if self.args.len() != 1 {
            bail!("contains method requires 1 argument")
        }
        let substring = self.args[0].as_str().unwrap_or("");
        let result = s.contains(substring);
        Ok(Value::Bool(result))
    }
    pub fn split(&self, s: &str) -> Result<Value> {
        if self.args.len() != 1 {
            bail!("split method requires 1 argument")
        }
        let delimiter = self.args[0].as_str().unwrap_or("");
        let result: Vec<String> = match delimiter {
            "" => s.chars().map(|c| c.to_string()).collect(),
            _ => s.split(delimiter).map(|s| s.to_string()).collect(),
        };
        Ok(Value::Array(
            result.into_iter().map(Value::String).collect(),
        ))
    }
    pub fn index_of(&self, s: &str) -> Result<Value> {
        if self.args.len() != 1 {
            bail!("index_of method requires 1 argument")
        }
        let substring = self.args[0].as_str().unwrap_or("");
        let result = match s.find(substring) {
            Some(index) => index as i32,
            None => -1,
        };
        Ok(Value::Number(result.into()))
    }
    pub fn last_index_of(&self, s: &str) -> Result<Value> {
        if self.args.len() != 1 {
            bail!("last_index_of method requires 1 argument")
        }
        let substring = self.args[0].as_str().unwrap_or("");
        let result = match s.rfind(substring) {
            Some(index) => index as i32,
            None => -1,
        };
        Ok(Value::Number(result.into()))
    }
    pub fn to_upper_case(&self, s: &str) -> Result<Value> {
        if self.args.len() != 0 {
            bail!("to_upper_case method requires no arguments")
        }
        let result = s.to_uppercase();
        Ok(Value::String(result))
    }
    pub fn to_lower_case(&self, s: &str) -> Result<Value> {
        if self.args.len() != 0 {
            bail!("to_lower_case method requires no arguments")
        }
        let result = s.to_lowercase();
        Ok(Value::String(result))
    }
    pub fn substring(&self, s: &str) -> Result<Value> {
        let result = match self.args.len() {
            1 => {
                let start = match &self.args[0] {
                    Value::Number(num) => num.as_f64().unwrap_or(0.0) as usize,
                    Value::String(s) => s.parse::<usize>().unwrap_or(0),
                    _ => bail!("substring method requires a number as the first argument"),
                };
                let end = s.len();
                let result = &s[start..end];
                result.to_string()
            }
            2 => {
                let start = match &self.args[0] {
                    Value::Number(num) => num.as_f64().unwrap_or(0.0) as usize,
                    Value::String(s) => s.parse::<usize>().unwrap_or(0),
                    _ => bail!("substring method requires a number as the first argument"),
                };
                let end = match &self.args[1] {
                    Value::Number(num) => num.as_f64().unwrap_or(0.0) as usize,
                    Value::String(s) => s.parse::<usize>().unwrap_or(0),
                    _ => bail!("substring method requires a number as the second argument"),
                };
                let result = &s[start..end];
                result.to_string()
            }
            0 => "".to_string(),
            _ => bail!("substring method requires 1 or 2 arguments"),
        };
        Ok(Value::String(result))
    }
    pub fn starts_with(&self, s: &str) -> Result<Value> {
        if self.args.len() != 1 {
            bail!("starts_with method requires 1 argument")
        }
        let prefix = self.args[0].as_str().unwrap_or("");
        let result = s.starts_with(prefix);
        Ok(Value::Bool(result))
    }
    pub fn ends_with(&self, s: &str) -> Result<Value> {
        if self.args.len() != 1 {
            bail!("ends_with method requires 1 argument")
        }
        let suffix = self.args[0].as_str().unwrap_or("");
        let result = s.ends_with(suffix);
        Ok(Value::Bool(result))
    }
    pub fn regex_replace(&self, s: &str) -> Result<Value> {
        if self.args.len() != 2 {
            bail!("regex_replace method requires 2 arguments")
        }
        let pattern = self.args[0].as_str().unwrap_or("");
        let replacement = self.args[1].as_str().unwrap_or("");
        let re = Regex::new(pattern).map_err(|e| anyhow!(e))?;
        let result = re.replace_all(s, replacement);
        Ok(Value::String(result.to_string()))
    }
    pub fn length(&self, s: &str) -> Result<Value> {
        if self.args.len() != 0 {
            bail!("length method requires no arguments")
        }
        let result = s.chars().count();
        Ok(Value::Number(result.into()))
    }
    pub fn trim(&self, s: &str) -> Result<Value> {
        if self.args.len() != 0 {
            bail!("trim method requires no arguments")
        }
        let result = s.trim();
        Ok(Value::String(result.to_string()))
    }
}
