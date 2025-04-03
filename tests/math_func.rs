use exprimo::Evaluator;
use std::collections::HashMap;

#[cfg(test)]
#[test]
fn test_floor() {
    let evaluator = Evaluator::new(HashMap::new());

    let res1 = evaluator.evaluate("floor(1.5)").unwrap();
    let res2 = evaluator.evaluate("floor(1.0)").unwrap();
    let res3 = evaluator.evaluate("floor(1.9)").unwrap();
    let res4 = evaluator.evaluate("floor(-2.2)").unwrap();

    assert_eq!(res1, 1.0);
    assert_eq!(res2, 1.0);
    assert_eq!(res3, 1.0);
    assert_eq!(res4, -2.0);
}
#[test]
fn test_ceil() {
    let evaluator = Evaluator::new(HashMap::new());

    let res1 = evaluator.evaluate("ceil(1.5)").unwrap();
    let res2 = evaluator.evaluate("ceil(1.0)").unwrap();
    let res3 = evaluator.evaluate("ceil(1.9)").unwrap();
    let res4 = evaluator.evaluate("ceil(-2.2)").unwrap();

    assert_eq!(res1, 2.0);
    assert_eq!(res2, 1.0);
    assert_eq!(res3, 2.0);
    assert_eq!(res4, -2.0);
}
#[test]
fn test_round() {
    let evaluator = Evaluator::new(HashMap::new());

    let res1 = evaluator.evaluate("round(1.5)").unwrap();
    let res2 = evaluator.evaluate("round(1.0)").unwrap();
    let res3 = evaluator.evaluate("round(1.9)").unwrap();
    let res4 = evaluator.evaluate("round(-2.2)").unwrap();

    assert_eq!(res1, 2.0);
    assert_eq!(res2, 1.0);
    assert_eq!(res3, 2.0);
    assert_eq!(res4, -2.0);
}
#[test]
fn test_sin() {
    let evaluator = Evaluator::new(HashMap::new());

    let res1 = evaluator.evaluate("sin(0)").unwrap();
    let res2 = evaluator.evaluate("sin(1)").unwrap();
    let res3 = evaluator.evaluate("sin(2)").unwrap();
    let res4 = evaluator.evaluate("sin(3)").unwrap();

    assert_eq!(res1, 0.0);
    assert_eq!(res2, f64::sin(1.0));
    assert_eq!(res3, f64::sin(2.0));
    assert_eq!(res4, f64::sin(3.0));
}
#[test]
fn test_mod() {
    let evaluator = Evaluator::new(HashMap::new());

    let res1 = evaluator.evaluate("mod(10, 3)").unwrap();
    let res2 = evaluator.evaluate("mod(10, 5)").unwrap();
    let res3 = evaluator.evaluate("mod(-10, 3)").unwrap();
    let res4 = evaluator.evaluate("mod(10, -3)").unwrap();

    assert_eq!(res1, 1.0);
    assert_eq!(res2, 0.0);
    assert_eq!(res3, -1.0);
    assert_eq!(res4, 1.0);
}
#[test]
fn test_bitwise_and() {
    let evaluator = Evaluator::new(HashMap::new());

    let res1 = evaluator.evaluate("bitwiseAnd(5, 3)").unwrap(); // 5 = 101, 3 = 011 -> 001
    let res2 = evaluator.evaluate("bitwiseAnd(10, 7)").unwrap(); // 10 = 1010, 7 = 0111 -> 0010
    let res3 = evaluator.evaluate("bitwiseAnd(0, 15)").unwrap(); // 0 = 0000, 15 = 1111 -> 0000
    let res4 = evaluator.evaluate("bitwiseAnd(255, 128)").unwrap(); // 255 = 11111111, 128 = 10000000 -> 10000000

    assert_eq!(res1, 1.0);
    assert_eq!(res2, 2.0);
    assert_eq!(res3, 0.0);
    assert_eq!(res4, 128.0);
}
