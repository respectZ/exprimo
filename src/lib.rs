pub mod function;
pub mod method;
use function::{math, semver::SemverParser};
use method::{array::ArrayMethod, str::StrMethod};
use rslint_parser::{
    ast::{BinExpr, BinOp, CallExpr, CondExpr, DotExpr, Expr, Name, NameRef, UnaryExpr, UnaryOp},
    parse_text, AstNode, SyntaxKind, SyntaxNode,
};

use anyhow::{bail, Result};
use thiserror::Error;

use serde_json::{json, Value};

use std::{collections::HashMap, f64};

type BoxFunction = Box<dyn Fn(Vec<Value>) -> Value>;

#[derive(Error, Debug)]
#[error("Evaluation error")]
pub struct EvaluationError {
    #[from]
    source: NodeError,
}

#[derive(Error, Debug)]
#[error("Node error {message}, node: {node:?}")]
pub struct NodeError {
    message: String,
    node: Option<SyntaxNode>,
}

pub enum ContextEntry {
    Variable(Value),
    Function(BoxFunction),
}

#[cfg(feature = "logging")]
use scribe_rust::Logger;
#[cfg(feature = "logging")]
use std::sync::Arc;

pub struct Evaluator {
    context: HashMap<String, ContextEntry>,
    #[cfg(feature = "logging")]
    logger: Arc<Logger>,
}

impl Evaluator {
    pub fn new(
        context: HashMap<String, ContextEntry>,
        #[cfg(feature = "logging")] logger: Arc<Logger>,
    ) -> Self {
        Evaluator {
            context,
            #[cfg(feature = "logging")]
            logger,
        }
    }

    pub fn evaluate(&self, expression: &str) -> Result<Value> {
        let ast = parse_text(expression, 0).syntax();
        let untyped_expr_node = match ast.first_child() {
            Some(node) => node,
            None => {
                return Err(NodeError {
                    message: "Empty expression".to_string(),
                    node: None,
                }
                .into())
            }
        };

        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Expression AST:\n\n{:#?}\n-----------------",
            untyped_expr_node
        ));

        let result = self.evaluate_node(&untyped_expr_node)?;

        #[cfg(feature = "logging")]
        self.logger.trace(&format!("Result: {}", result));

        Ok(result)
    }

    fn evaluate_node(&self, node: &SyntaxNode) -> Result<Value, NodeError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating NodeKind: {:#?}, {:?}",
            node.kind(),
            node.to_string()
        ));

        let res = match node.kind() {
            SyntaxKind::EXPR_STMT => {
                let expr = node.first_child().ok_or_else(|| NodeError {
                    message: "[Empty expression]".to_string(),
                    node: None,
                })?;
                self.evaluate_node(&expr)
            }
            SyntaxKind::DOT_EXPR => self.evaluate_dot_expr(&DotExpr::cast(node.clone()).unwrap()),
            SyntaxKind::NAME_REF => self.evaluate_name_ref(&NameRef::cast(node.clone()).unwrap()),
            SyntaxKind::NAME => self.evaluate_name(&Name::cast(node.clone()).unwrap()),
            SyntaxKind::BIN_EXPR => self.evaluate_bin_expr(&BinExpr::cast(node.clone()).unwrap()),
            SyntaxKind::LITERAL => self.evaluate_literal(&Expr::cast(node.clone()).unwrap()),
            SyntaxKind::COND_EXPR => {
                self.evaluate_cond_expr(&CondExpr::cast(node.clone()).unwrap())
            }
            SyntaxKind::IDENT => self.evaluate_identifier(&Expr::cast(node.clone()).unwrap()),
            SyntaxKind::UNARY_EXPR => {
                self.evaluate_prefix_expr(&UnaryExpr::cast(node.clone()).unwrap())
            }
            SyntaxKind::CALL_EXPR => {
                self.evaluate_call_expr(&CallExpr::cast(node.clone()).unwrap())
            }
            SyntaxKind::GROUPING_EXPR => {
                let expr = node.first_child().ok_or_else(|| NodeError {
                    message: "[Empty grouping expression]".to_string(),
                    node: None,
                })?;
                self.evaluate_node(&expr)
            }
            _ => Err(NodeError {
                message: format!("Unsupported syntax kind: {:?}", node.kind()),
                node: Some(node.clone()),
            }),
        };

        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "NodeKind: {:?} => {:#?}",
            node.kind(),
            res.as_ref()
        ));

        res
    }

    fn evaluate_bin_expr(&self, bin_expr: &BinExpr) -> Result<Value, NodeError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Binary Expression: {:#?}",
            bin_expr.to_string()
        ));

        let left = bin_expr.lhs().ok_or_else(|| NodeError {
            message: "[Empty BinExpr Left Expression]".to_string(),
            node: Some(bin_expr.syntax().clone()),
        })?;
        let right = bin_expr.rhs().ok_or_else(|| NodeError {
            message: "[Empty BinExpr Right Expression]".to_string(),
            node: Some(bin_expr.syntax().clone()),
        })?;

        let left_value = self.evaluate_node(left.syntax())?;
        let right_value = self.evaluate_node(right.syntax())?;

        let op = bin_expr.op_details();

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("BinaryOp left_value {:?}", left_value));

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("BinaryOp right_value {:?}", right_value));

        #[cfg(feature = "logging")]
        self.logger.trace(&format!("BinaryOp op_details {:?}", op));

        if let (Value::Object(l), Value::Object(r)) = (&left_value, &right_value) {
            if let (Ok(l_obj), Ok(r_obj)) =
                (SemverParser::from_object(l), SemverParser::from_object(r))
            {
                let cmp = l_obj.version.cmp_precedence(&r_obj.version);
                let res = match op {
                    Some((_, BinOp::Equality | BinOp::StrictEquality)) => cmp.is_eq(),
                    Some((_, BinOp::Inequality | BinOp::StrictInequality)) => !cmp.is_eq(),
                    Some((_, BinOp::GreaterThan)) => cmp.is_gt(),
                    Some((_, BinOp::LessThan)) => cmp.is_lt(),
                    Some((_, BinOp::GreaterThanOrEqual)) => cmp.is_ge(),
                    Some((_, BinOp::LessThanOrEqual)) => cmp.is_le(),
                    _ => {
                        return Err(NodeError {
                            message: "Unsupported binary operator for semver".to_string(),
                            node: Some(bin_expr.syntax().clone()),
                        });
                    }
                };
                return Ok(Value::Bool(res));
            }
        }

        let result = match op {
            Some((_, BinOp::Plus)) => self.add_values(left_value, right_value),
            Some((_, BinOp::Minus)) => self.subtract_values(left_value, right_value),
            Some((_, BinOp::Times)) => self.multiply_values(left_value, right_value),
            Some((_, BinOp::Divide)) => self.divide_values(left_value, right_value),
            Some((_, BinOp::Remainder)) => self.modulo_values(left_value, right_value),
            Some((_, BinOp::LogicalAnd)) => Ok(Value::Bool(
                self.to_boolean(&left_value)? && self.to_boolean(&right_value)?,
            )),
            Some((_, BinOp::LogicalOr)) => Ok(Value::Bool(
                self.to_boolean(&left_value)? || self.to_boolean(&right_value)?,
            )),
            Some((_, BinOp::Equality)) | Some((_, BinOp::StrictEquality)) => Ok(Value::Bool(
                self.abstract_equality(&left_value, &right_value),
            )),
            Some((_, BinOp::Inequality)) | Some((_, BinOp::StrictInequality)) => Ok(Value::Bool(
                !self.abstract_equality(&left_value, &right_value),
            )),
            Some((_, BinOp::GreaterThan)) => {
                self.compare_values(&left_value, &right_value, |a, b| a > b)
            }
            Some((_, BinOp::LessThan)) => {
                self.compare_values(&left_value, &right_value, |a, b| a < b)
            }
            Some((_, BinOp::GreaterThanOrEqual)) => {
                self.compare_values(&left_value, &right_value, |a, b| a >= b)
            }
            Some((_, BinOp::LessThanOrEqual)) => {
                self.compare_values(&left_value, &right_value, |a, b| a <= b)
            }
            _ => Err(NodeError {
                message: "Unsupported binary operator".to_string(),
                node: Some(bin_expr.syntax().clone()),
            }),
        }?;

        #[cfg(feature = "logging")]
        self.logger.trace(&format!("Binary Result: {:?}", result));

        Ok(result)
    }

    fn add_values(&self, left: Value, right: Value) -> Result<Value, NodeError> {
        match (left.clone(), right.clone()) {
            (Value::Number(l), Value::Number(r)) => {
                let sum = l.as_f64().unwrap() + r.as_f64().unwrap();
                Ok(Value::Number(serde_json::Number::from_f64(sum).unwrap()))
            }
            (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
            (Value::String(l), r) => Ok(Value::String(l + &self.value_to_string(&r))),
            (l, Value::String(r)) => Ok(Value::String(self.value_to_string(&l) + &r)),
            _ => {
                // Type coercion similar to JavaScript
                let l_str = self.value_to_string(&left);
                let r_str = self.value_to_string(&right);
                Ok(Value::String(l_str + &r_str))
            }
        }
    }

    fn subtract_values(&self, left: Value, right: Value) -> Result<Value, NodeError> {
        let l_num = self.to_number(&left)?;
        let r_num = self.to_number(&right)?;
        Ok(Value::Number(
            serde_json::Number::from_f64(l_num - r_num).unwrap(),
        ))
    }

    fn multiply_values(&self, left: Value, right: Value) -> Result<Value, NodeError> {
        let l_num = self.to_number(&left)?;
        let r_num = self.to_number(&right)?;
        Ok(Value::Number(
            serde_json::Number::from_f64(l_num * r_num).unwrap(),
        ))
    }

    fn divide_values(&self, left: Value, right: Value) -> Result<Value, NodeError> {
        let l_num = self.to_number(&left)?;
        let r_num = self.to_number(&right)?;
        if r_num == 0.0 {
            return Err(NodeError {
                message: "Division by zero".to_string(),
                node: None,
            });
        }
        Ok(Value::Number(
            serde_json::Number::from_f64(l_num / r_num).unwrap(),
        ))
    }

    fn modulo_values(&self, left: Value, right: Value) -> Result<Value, NodeError> {
        let l_num = self.to_number(&left)?;
        let r_num = self.to_number(&right)?;
        Ok(Value::Number(
            serde_json::Number::from_f64(l_num % r_num).unwrap(),
        ))
    }

    fn compare_values<F>(&self, left: &Value, right: &Value, cmp: F) -> Result<Value, NodeError>
    where
        F: Fn(f64, f64) -> bool,
    {
        let l_num = self.to_number(left)?;
        let r_num = self.to_number(right)?;
        Ok(Value::Bool(cmp(l_num, r_num)))
    }

    fn evaluate_prefix_expr(&self, prefix_expr: &UnaryExpr) -> Result<Value, NodeError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Prefix Expression: {:#?}",
            prefix_expr.to_string()
        ));
        let expr = prefix_expr.expr().ok_or_else(|| NodeError {
            message: "[Empty PrefixExpr Expression]".to_string(),
            node: Some(prefix_expr.syntax().clone()),
        })?;
        let expr_value = self.evaluate_node(expr.syntax())?;

        let op = prefix_expr.op_details();

        let result = match op {
            Some((_, UnaryOp::LogicalNot)) => Value::Bool(!self.to_boolean(&expr_value)?),
            Some((_, UnaryOp::Minus)) => {
                let num = self.to_number(&expr_value)?;
                Value::Number(serde_json::Number::from_f64(-num).unwrap())
            }
            Some((_, UnaryOp::Plus)) => {
                let num = self.to_number(&expr_value)?;
                Value::Number(serde_json::Number::from_f64(num).unwrap())
            }
            _ => {
                return Err(NodeError {
                    message: "Unsupported unary operator".to_string(),
                    node: Some(prefix_expr.syntax().clone()),
                })
            }
        };

        #[cfg(feature = "logging")]
        self.logger.trace(&format!("Prefix Result: {:?}", result));

        Ok(result)
    }

    fn evaluate_cond_expr(&self, cond_expr: &CondExpr) -> Result<Value, NodeError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Conditional Expression: {:#?}",
            cond_expr.to_string()
        ));
        let cond = cond_expr.test().ok_or_else(|| NodeError {
            message: "[Empty CondExpr Test Expression]".to_string(),
            node: Some(cond_expr.syntax().clone()),
        })?;
        let true_expr = cond_expr.cons().ok_or_else(|| NodeError {
            message: "[Empty CondExpr Consequent Expression]".to_string(),
            node: Some(cond_expr.syntax().clone()),
        })?;
        let false_expr = cond_expr.alt().ok_or_else(|| NodeError {
            message: "[Empty CondExpr Alternate Expression]".to_string(),
            node: Some(cond_expr.syntax().clone()),
        })?;

        let cond_value = self.evaluate_node(cond.syntax())?;
        let cond_bool = self.to_boolean(&cond_value)?;

        let result = if cond_bool {
            self.evaluate_node(true_expr.syntax())?
        } else {
            self.evaluate_node(false_expr.syntax())?
        };

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Conditional Result: {:?}", result));

        Ok(result)
    }

    fn evaluate_dot_expr(&self, dot_expr: &DotExpr) -> Result<Value, NodeError> {
        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Evaluating Dot Expression: {:#?}", dot_expr));

        // Start with the leftmost expression
        let mut current_expr = dot_expr.clone();
        let mut property_chain = Vec::new();

        // Collect all identifiers in the dot expression
        loop {
            let prop = current_expr.prop();
            let obj = current_expr.object();

            if let Some(prop) = prop {
                let prop_name = prop.syntax().text().to_string();
                property_chain.push(prop_name);
            } else {
                return Err(NodeError {
                    message: "Missing property in dot expression".to_string(),
                    node: Some(current_expr.syntax().clone()),
                });
            }

            if let Some(obj_expr) = obj {
                let obj_syntax = obj_expr.syntax().clone();
                if let Some(prev_dot_expr) = DotExpr::cast(obj_syntax.clone()) {
                    current_expr = prev_dot_expr;
                } else if let Some(name_ref) = NameRef::cast(obj_syntax.clone()) {
                    let obj_name = name_ref.syntax().text().to_string();
                    property_chain.push(obj_name);
                    break;
                } else if let Some(name) = Name::cast(obj_syntax.clone()) {
                    let obj_name = name.syntax().text().to_string();
                    property_chain.push(obj_name);
                    break;
                } else {
                    return Err(NodeError {
                        message: "Unsupported object type in dot expression".to_string(),
                        node: Some(obj_expr.syntax().clone()),
                    });
                }
            } else {
                break;
            }
        }

        // Reverse the property chain to get the correct order
        property_chain.reverse();

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Property Chain: {:?}", property_chain));

        // Start from the top-level context
        let mut value = match self.context.get(&property_chain[0]) {
            Some(ContextEntry::Variable(v)) => v.clone(),
            _ => Value::Null,
        };

        // Navigate through the nested properties
        for prop in &property_chain[1..] {
            match &value {
                Value::Object(map) => {
                    value = map.get(prop).cloned().unwrap_or(Value::Null);
                }
                _ => {
                    // Return Null when the value is not an object or property is missing
                    value = Value::Null;
                    break;
                }
            }
        }

        Ok(value)
    }

    // Implement abstract equality similar to JavaScript
    fn abstract_equality(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Null, Value::Null) => true,
            (Value::Number(l), Value::Number(r)) => l.as_f64() == r.as_f64(),
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Bool(l), Value::Bool(r)) => l == r,
            _ => false,
        }
    }

    fn evaluate_by_name(&self, identifier_name: String) -> Result<Value, NodeError> {
        let identifier_value = match self.context.get(&identifier_name) {
            Some(ContextEntry::Variable(v)) => Some(v.clone()),
            _ => None,
        };

        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Identifier Value: {:#?}", identifier_value));

        match identifier_value {
            Some(value) => Ok(value.clone()),
            None => Err(NodeError {
                message: format!("Identifier '{}' not found in context.", identifier_name),
                node: None,
            }),
        }
    }

    fn evaluate_name(&self, name: &Name) -> Result<Value, NodeError> {
        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Evaluating Name: {:#?}", name.to_string()));
        let identifier_name = name
            .ident_token()
            .ok_or_else(|| NodeError {
                message: "[Empty Name]".to_string(),
                node: Some(name.syntax().clone()),
            })?
            .to_string();

        self.evaluate_by_name(identifier_name)
    }

    fn evaluate_name_ref(&self, name_ref: &NameRef) -> Result<Value, NodeError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Name Reference: {:#?}",
            name_ref.to_string()
        ));
        let identifier_name = name_ref
            .ident_token()
            .ok_or_else(|| NodeError {
                message: "[Empty NameRef]".to_string(),
                node: Some(name_ref.syntax().clone()),
            })?
            .to_string();

        self.evaluate_by_name(identifier_name)
    }

    fn evaluate_identifier(&self, identifier: &Expr) -> Result<Value, NodeError> {
        #[cfg(feature = "logging")]
        self.logger.trace(&format!(
            "Evaluating Identifier: {:#?}",
            identifier.to_string()
        ));
        let identifier_name = identifier.to_string();

        self.evaluate_by_name(identifier_name)
    }

    fn evaluate_literal(&self, literal: &Expr) -> Result<Value, NodeError> {
        #[cfg(feature = "logging")]
        self.logger
            .trace(&format!("Evaluating Literal: {:#?}", literal.to_string()));

        let literal_str = literal.to_string();

        // Handle numeric literals
        if let Ok(number) = literal_str.parse::<f64>() {
            return Ok(Value::Number(serde_json::Number::from_f64(number).unwrap()));
        }

        // Handle string literals
        if literal_str.starts_with('"') || literal_str.starts_with('\'') {
            let unquoted = literal_str
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string();
            return Ok(Value::String(unquoted));
        }

        // Handle boolean literals
        match literal_str.as_str() {
            "true" => return Ok(Value::Bool(true)),
            "false" => return Ok(Value::Bool(false)),
            "null" => return Ok(Value::Null),
            _ => {}
        }

        Err(NodeError {
            message: format!("Unknown literal type: {}", literal_str),
            node: Some(literal.syntax().clone()),
        })
    }

    fn evaluate_call_expr(&self, expr: &CallExpr) -> Result<Value, NodeError> {
        let callee = expr.callee().ok_or_else(|| NodeError {
            message: "Empty callee in call expression".to_string(),
            node: Some(expr.syntax().clone()),
        })?;
        let args = match expr.arguments() {
            Some(args) => {
                let args = args.args();
                let mut arg_values = Vec::new();
                for arg in args {
                    let arg_value = self.evaluate_node(arg.syntax())?;
                    arg_values.push(arg_value);
                }
                arg_values
            }
            None => vec![],
        };
        if callee.syntax().kind() == SyntaxKind::DOT_EXPR {
            let dot_expr = DotExpr::cast(callee.syntax().clone()).ok_or_else(|| NodeError {
                message: "Invalid callee in call expression".to_string(),
                node: Some(expr.syntax().clone()),
            })?;
            let obj = dot_expr.object().ok_or_else(|| NodeError {
                message: "Empty object in dot expression".to_string(),
                node: Some(expr.syntax().clone()),
            })?;
            let prop = dot_expr.prop().ok_or_else(|| NodeError {
                message: "Empty property in dot expression".to_string(),
                node: Some(expr.syntax().clone()),
            })?;
            let obj_value = self.evaluate_node(obj.syntax())?;
            let prop_name = prop.syntax().text().to_string();
            let result = match obj_value {
                Value::String(s) => Self::str_method(&s, &prop_name, args),
                Value::Array(arr) => Self::array_method(&arr, &prop_name, args),
                _ => {
                    return Err(NodeError {
                        message: format!(
                            "Unsupported object type for method call: {}",
                            obj_value.to_string()
                        ),
                        node: Some(expr.syntax().clone()),
                    })
                }
            };
            return result.map_err(|e| NodeError {
                message: format!(
                    "Error calling method '{}' on object: {}. {}",
                    prop_name,
                    obj.text().to_string(),
                    e
                ),
                node: Some(expr.syntax().clone()),
            });
        }
        match self.context.get(&callee.to_string()) {
            Some(ContextEntry::Function(f)) => Ok(f(args)),
            _ => {
                let name = callee.syntax().text().to_string();
                if name == "semver" {
                    SemverParser::parse(args)
                        .map(|v| json!(v))
                        .map_err(|e| NodeError {
                            message: format!("Error parsing semver: {}", e),
                            node: Some(expr.syntax().clone()),
                        })
                } else if let Some(first) = args.get(0) {
                    let rest = args.get(1..).unwrap_or(&[]);
                    match first {
                        Value::String(s) => Self::str_method(s, &name, rest.to_vec()),
                        Value::Array(arr) => Self::array_method(arr, &name, rest.to_vec()),
                        Value::Number(value) => {
                            let second = rest.get(0).unwrap_or(&Value::Null);
                            match name.as_str() {
                                "floor" => math::unary_function(value, f64::floor),
                                "ceil" => math::unary_function(value, f64::ceil),
                                "round" => math::unary_function(value, f64::round),
                                "sin" => math::unary_function(value, f64::sin),
                                "cos" => math::unary_function(value, f64::cos),
                                "tan" => math::unary_function(value, f64::tan),
                                "asin" => math::unary_function(value, f64::asin),
                                "acos" => math::unary_function(value, f64::acos),
                                "atan" => math::unary_function(value, f64::atan),
                                "sqrt" => math::unary_function(value, f64::sqrt),
                                "abs" => math::unary_function(value, f64::abs),
                                "clamp" => math::unary_function(value, |x| x.clamp(0.0, 1.0)),
                                "bitiwseNot" => math::unary_function(value, |x| {
                                    !((f64::round(x)) as i64) as f64
                                }),
                                "atan2" => math::binary_function(value, second, f64::atan2),
                                "min" => math::binary_function(value, second, f64::min),
                                "max" => math::binary_function(value, second, f64::max),
                                "mod" => math::binary_function(value, second, |x, y| x % y),
                                "pow" => math::binary_function(value, second, f64::powf),
                                "bitwiseAnd" => math::binary_function(value, second, |x, y| {
                                    (((f64::round(x)) as i64) & ((f64::round(y)) as i64)) as f64
                                }),
                                "bitwiseOr" => math::binary_function(value, second, |x, y| {
                                    (((f64::round(x)) as i64) | ((f64::round(y)) as i64)) as f64
                                }),
                                "bitshiftLeft" => math::binary_function(value, second, |x, y| {
                                    (((f64::round(x)) as i64) << ((f64::round(y)) as i64)) as f64
                                }),
                                "bitshiftRight" => math::binary_function(value, second, |x, y| {
                                    (((f64::round(x)) as i64) >> ((f64::round(y)) as i64)) as f64
                                }),
                                _ => {
                                    return Err(NodeError {
                                        message: format!(
                                            "Function '{}' not found in context",
                                            name
                                        ),
                                        node: Some(expr.syntax().clone()),
                                    });
                                }
                            }
                        }
                        _ => Err(NodeError {
                            message: format!(
                                "Function '{}' not found in context.",
                                callee.to_string()
                            ),
                            node: Some(expr.syntax().clone()),
                        })?,
                    }
                    .map_err(|e| NodeError {
                        message: format!(
                            "Error calling method '{}' on object: {}. {}",
                            name,
                            first.to_string(),
                            e
                        ),
                        node: Some(expr.syntax().clone()),
                    })
                } else {
                    Err(NodeError {
                        message: format!("Function '{}' not found in context", callee.to_string()),
                        node: Some(expr.syntax().clone()),
                    })
                }
            }
        }
    }

    fn to_number(&self, value: &Value) -> Result<f64, NodeError> {
        match value {
            Value::Number(n) => Ok(n.as_f64().unwrap()),
            Value::String(s) => s.parse::<f64>().map_err(|_| NodeError {
                message: format!("Cannot convert string '{}' to number", s),
                node: None,
            }),
            Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            Value::Null => Ok(0.0),
            _ => Err(NodeError {
                message: "Cannot convert value to number".to_string(),
                node: None,
            }),
        }
    }

    fn to_boolean(&self, value: &Value) -> Result<bool, NodeError> {
        let result = match value {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Number(n) => {
                let num = n.as_f64().unwrap();
                num != 0.0 && !num.is_nan()
            }
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
        };
        Ok(result)
    }

    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(_) => "[Array]".to_string(),
            Value::Object(_) => "[Object]".to_string(),
        }
    }

    fn str_method(value: &str, method: &str, args: Vec<Value>) -> Result<Value> {
        let str_method = StrMethod::new(args);
        let result = match method {
            "replace" => str_method.replace(value),
            "contains" => str_method.contains(value),
            "split" => str_method.split(value),
            "indexOf" => str_method.index_of(value),
            "lastIndexOf" => str_method.last_index_of(value),
            "toUpperCase" => str_method.to_upper_case(value),
            "toLowerCase" => str_method.to_lower_case(value),
            "substring" => str_method.substring(value),
            "startsWith" => str_method.starts_with(value),
            "endsWith" => str_method.ends_with(value),
            "regexReplace" => str_method.regex_replace(value),
            "length" => str_method.length(value),
            "trim" => str_method.trim(value),
            _ => {
                bail!("Unknown string method: {}", method);
            }
        };
        result
    }

    fn array_method(value: &Vec<Value>, method: &str, args: Vec<Value>) -> Result<Value> {
        let array_method = ArrayMethod::new(args);
        let result = match method {
            "join" => array_method.join(value),
            _ => {
                bail!("Unknown array method: {}", method);
            }
        };
        result
    }
}
