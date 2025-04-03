use exprimo::{ContextEntry, Evaluator};
use std::collections::HashMap;

#[cfg(test)]
#[test]
fn test_method() {
    let mut context = HashMap::new();

    context.insert(
        "a".to_string(),
        ContextEntry::Variable(serde_json::Value::String("hello".to_string())),
    );
    context.insert(
        "b".to_string(),
        ContextEntry::Variable(serde_json::Value::String("world".to_string())),
    );
    context.insert(
        "c".to_string(),
        ContextEntry::Variable(serde_json::Value::String("hello12345".to_string())),
    );
    context.insert(
        "d".to_string(),
        ContextEntry::Variable(serde_json::Value::String("hello\n".to_string())),
    );
    context.insert(
        "e".to_string(),
        ContextEntry::Variable(serde_json::Value::Array(vec![
            serde_json::Value::String("h".to_string()),
            serde_json::Value::String("e".to_string()),
            serde_json::Value::String("l".to_string()),
            serde_json::Value::String("l".to_string()),
            serde_json::Value::String("o".to_string()),
        ])),
    );
    let mut map = serde_json::Map::new();
    map.insert(
        "a".to_string(),
        serde_json::Value::String("hello".to_string()),
    );
    map.insert(
        "b".to_string(),
        serde_json::Value::String("world".to_string()),
    );
    context.insert(
        "f".to_string(),
        ContextEntry::Variable(serde_json::Value::Object(map)),
    );

    let evaluator = Evaluator::new(context);

    let res1 = evaluator
        .evaluate("replace(a, 'h', 'H').replace('llo', 'ok')")
        .unwrap();
    let res2 = evaluator.evaluate("contains('hello', 'h')").unwrap();
    let res3 = evaluator.evaluate("indexOf('hello', 'l')").unwrap();
    let res4 = evaluator.evaluate("split(a, 'l')").unwrap();
    let res4 = res4.as_array().unwrap();
    let res5 = evaluator.evaluate("indexOf(a, 'l')").unwrap();
    let res6 = evaluator.evaluate("lastIndexOf(a, 'l')").unwrap();
    let res7 = evaluator.evaluate("toUpperCase(a, )").unwrap();
    let res8 = evaluator.evaluate("toLowerCase(a, )").unwrap();
    let res9 = evaluator.evaluate("substring(a, 1, 3)").unwrap();
    let res10 = evaluator.evaluate("substring(a, 1)").unwrap();
    let res11 = evaluator.evaluate("substring(a, 0, 1)").unwrap();
    let res12 = evaluator.evaluate("substring(a, 0)").unwrap();
    let res13 = evaluator.evaluate("startsWith(a, 'h')").unwrap();
    let res14 = evaluator.evaluate("startsWith(a, 'H')").unwrap();
    let res15 = evaluator.evaluate("endsWith(a, 'o')").unwrap();
    let res16 = evaluator.evaluate("endsWith(a, 'H')").unwrap();
    let res17 = evaluator.evaluate("regexReplace(a, 'h', 'H')").unwrap();
    let res18 = evaluator.evaluate("regexReplace(c, '[a-z]', 'L')").unwrap();
    let res19 = evaluator.evaluate("length(a)").unwrap();
    let res20 = evaluator.evaluate("trim(d)").unwrap();
    let res21 = evaluator.evaluate("join(e, '')").unwrap();
    let res22 = evaluator.evaluate("join(e, '-')").unwrap();
    let res23 = evaluator.evaluate("split(a, '').join('.')").unwrap();
    let res24 = evaluator.evaluate("replace(f.a, 'h', 'H')").unwrap();
    assert_eq!(res1, "Heok".to_string());
    assert_eq!(res2, true);
    assert_eq!(res3, 2);
    assert_eq!(res4.len(), 3);
    assert_eq!(res4[0], "he".to_string());
    assert_eq!(res4[1], "".to_string());
    assert_eq!(res4[2], "o".to_string());
    assert_eq!(res5, 2);
    assert_eq!(res6, 3);
    assert_eq!(res7, "HELLO".to_string());
    assert_eq!(res8, "hello".to_string());
    assert_eq!(res9, "el".to_string());
    assert_eq!(res10, "ello".to_string());
    assert_eq!(res11, "h".to_string());
    assert_eq!(res12, "hello".to_string());
    assert_eq!(res13, true);
    assert_eq!(res14, false);
    assert_eq!(res15, true);
    assert_eq!(res16, false);
    assert_eq!(res17, "Hello".to_string());
    assert_eq!(res18, "LLLLL12345".to_string());
    assert_eq!(res19, 5);
    assert_eq!(res20, "hello".to_string());
    assert_eq!(res21, "hello".to_string());
    assert_eq!(res22, "h-e-l-l-o".to_string());
    assert_eq!(res23, "h.e.l.l.o".to_string());
    assert_eq!(res24, "Hello".to_string());
}
