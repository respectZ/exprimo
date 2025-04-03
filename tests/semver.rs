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

#[test]
fn test_semver_obj() {
    let mut context = HashMap::new();

    let mut map1 = serde_json::Map::new();
    map1.insert("major".to_string(), Value::Number(0.into()));
    map1.insert("minor".to_string(), Value::Number(0.into()));
    map1.insert("patch".to_string(), Value::Number(1.into()));

    let mut map2 = serde_json::Map::new();
    map2.insert("major".to_string(), Value::Number(1.into()));
    map2.insert("minor".to_string(), Value::Number(0.into()));
    map2.insert("patch".to_string(), Value::Number(1.into()));

    context.insert("a".to_string(), ContextEntry::Variable(Value::Object(map1)));
    context.insert("b".to_string(), ContextEntry::Variable(Value::Object(map2)));
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
