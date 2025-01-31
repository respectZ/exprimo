use std::collections::HashMap;
use std::error::Error;

#[cfg(feature = "logging")]
use scribe_rust;

use exprimo;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "logging")]
    let logger = scribe_rust::Logger::default();

    let mut ctx = HashMap::new();
    ctx.insert(
        "x".to_string(),
        exprimo::ContextEntry::Variable(serde_json::Value::Number(5.into())),
    );
    let engine = exprimo::Evaluator::new(
        ctx,
        #[cfg(feature = "logging")]
        logger,
    );

    let result = engine.evaluate("x == 5")?;

    println!("x = {}", result);

    Ok(())
}
