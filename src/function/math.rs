use anyhow::{anyhow, bail, Result};
use serde_json::{Number, Value};

pub fn unary_function<F>(value: &Number, f: F) -> Result<Value>
where
    F: Fn(f64) -> f64,
{
    let value = value
        .as_f64()
        .ok_or_else(|| anyhow!("Invalid number format"))?;
    let result = f(value);
    Ok(Value::Number(Number::from_f64(result).unwrap()))
}

pub fn binary_function<F>(value: &Number, second: &Value, f: F) -> Result<Value>
where
    F: Fn(f64, f64) -> f64,
{
    if second.is_null() {
        bail!("Second argument is null")
    }
    let first = value
        .as_f64()
        .ok_or_else(|| anyhow!("Invalid number format"))?;
    let second = second
        .as_f64()
        .ok_or_else(|| anyhow!("Invalid number format"))?;
    let result = f(first, second);
    Ok(Value::Number(Number::from_f64(result).unwrap()))
}
