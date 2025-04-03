use exprimo::{ContextEntry, Evaluator};
use serde_json::Value;
use std::collections::HashMap;

#[cfg(test)]
#[test]
fn test_semver_str() {
    let mut context = HashMap::new();
    context.insert(
        "a".to_string(),
        ContextEntry::Variable(Value::String("1.0.0".to_string())),
    );
    context.insert(
        "b".to_string(),
        ContextEntry::Variable(Value::String("1.0.1".to_string())),
    );
    let evaluator = Evaluator::new(context);
    let res1 = evaluator.evaluate("semver(a) < semver(b)").unwrap();
    let res2 = evaluator.evaluate("semver(a) > semver(b)").unwrap();
    let res3 = evaluator.evaluate("semver(a) == semver(b)").unwrap();
    let res4 = evaluator.evaluate("semver(a) != semver(b)").unwrap();
    assert_eq!(res1, true);
    assert_eq!(res2, false);
    assert_eq!(res3, false);
    assert_eq!(res4, true);
}

#[test]
fn test_semver_arr() {
    let mut context = HashMap::new();
    context.insert(
        "a".to_string(),
        ContextEntry::Variable(Value::Array(vec![
            Value::Number(1.into()),
            Value::Number(0.into()),
            Value::Number(0.into()),
        ])),
    );
    context.insert(
        "b".to_string(),
        ContextEntry::Variable(Value::Array(vec![
            Value::Number(1.into()),
            Value::Number(0.into()),
            Value::Number(1.into()),
        ])),
    );
    let evaluator = Evaluator::new(context);
    let res1 = evaluator.evaluate("semver(a) < semver(b)").unwrap();
    let res2 = evaluator.evaluate("semver(a) > semver(b)").unwrap();
    let res3 = evaluator.evaluate("semver(a) == semver(b)").unwrap();
    let res4 = evaluator.evaluate("semver(a) != semver(b)").unwrap();
    assert_eq!(res1, true);
    assert_eq!(res2, false);
    assert_eq!(res3, false);
    assert_eq!(res4, true);
}

#[test]
fn test_semver_int() {
    let mut context = HashMap::new();
    context.insert(
        "a".to_string(),
        ContextEntry::Variable(Value::Number(0.into())),
    );
    context.insert(
        "b".to_string(),
        ContextEntry::Variable(Value::Number(1.into())),
    );
    let evaluator = Evaluator::new(context);
    let res1 = evaluator.evaluate("semver(a,a,b) < semver(b,a,a)").unwrap();
    let res2 = evaluator.evaluate("semver(a,a,b) > semver(b,a,a)").unwrap();
    let res3 = evaluator
        .evaluate("semver(a,a,b) == semver(b,a,a)")
        .unwrap();
    let res4 = evaluator
        .evaluate("semver(a,a,b) != semver(b,a,a)")
        .unwrap();
    assert_eq!(res1, true);
    assert_eq!(res2, false);
    assert_eq!(res3, false);
    assert_eq!(res4, true);
}
