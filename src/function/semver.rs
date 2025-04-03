use anyhow::{anyhow, bail, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct SemverParser {
    pub version: Version,
}

impl SemverParser {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        SemverParser {
            version: Version::new(major, minor, patch),
        }
    }
    pub fn from_vec(vec: &Vec<u64>) -> Self {
        SemverParser {
            version: Version::new(vec[0], vec[1], vec[2]),
        }
    }
    pub fn from_str(version: &str) -> Self {
        SemverParser {
            version: Version::parse(version).unwrap(),
        }
    }
    pub fn from_object(obj: &serde_json::Map<String, Value>) -> Result<Self> {
        Ok(serde_json::from_value(Value::Object(obj.clone()))
            .map_err(|_| anyhow!("Invalid semver format"))?)
    }
    pub fn parse(values: Vec<Value>) -> Result<Self> {
        let result = match values.len() {
            1 => match &values[0] {
                Value::String(s) => Self::from_str(s),
                Value::Array(arr) => Self::from_vec(
                    &arr.iter()
                        .map(|v| Self::to_number(v))
                        .collect::<Result<Vec<u64>>>()?,
                ),
                Value::Object(obj) => Self::from_object(obj)?,
                _ => {
                    bail!("Invalid semver format")
                }
            },
            3 => {
                let major = Self::to_number(&values[0])?;
                let minor = Self::to_number(&values[1])?;
                let patch = Self::to_number(&values[2])?;
                Self::new(major, minor, patch)
            }
            _ => {
                bail!("Invalid semver format")
            }
        };
        Ok(result)
    }

    fn to_number(value: &Value) -> Result<u64> {
        match value {
            Value::Number(num) => num.as_u64().ok_or_else(|| anyhow!("Invalid number format")),
            Value::String(s) => s
                .parse::<u64>()
                .map_err(|_| anyhow!("Invalid number format")),
            _ => bail!("Invalid number format"),
        }
    }
}
