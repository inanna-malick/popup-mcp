use anyhow::{anyhow, Result};
use pest::Parser;
use pest_derive::Parser;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "condition.pest"]
struct ConditionParser;

/// AST for condition expressions
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionExpr {
    // Logical operators
    Or(Vec<ConditionExpr>),
    And(Vec<ConditionExpr>),
    Not(Box<ConditionExpr>),

    // Comparison
    Compare {
        op: CompareOp,
        left: Box<ConditionExpr>,
        right: Box<ConditionExpr>,
    },

    // Values
    Ref(String),           // @id reference
    Number(f64),           // Numeric literal
    String(String),        // String literal
    Boolean(bool),         // Boolean literal

    // Functions
    Count(Box<ConditionExpr>),                          // count(@field)
    Selected(Box<ConditionExpr>, Box<ConditionExpr>),  // selected(@field, value)
    Any(Vec<ConditionExpr>),                            // any(expr1, expr2, ...)
    All(Vec<ConditionExpr>),                            // all(expr1, expr2, ...)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompareOp {
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    Equal,
    NotEqual,
}

/// Parse a condition expression string into AST
pub fn parse_condition(input: &str) -> Result<ConditionExpr> {
    let pairs = ConditionParser::parse(Rule::expr, input)
        .map_err(|e| anyhow!("Failed to parse condition: {}", e))?;

    let pair = pairs.into_iter().next()
        .ok_or_else(|| anyhow!("Empty condition expression"))?;

    build_ast(pair)
}

fn build_ast(pair: pest::iterators::Pair<Rule>) -> Result<ConditionExpr> {
    match pair.as_rule() {
        Rule::expr => {
            let inner = pair.into_inner().next().unwrap();
            build_ast(inner)
        }

        Rule::or => {
            let parts: Vec<_> = pair.into_inner().collect();
            if parts.len() == 1 {
                build_ast(parts[0].clone())
            } else {
                let exprs: Result<Vec<_>> = parts.into_iter().map(build_ast).collect();
                Ok(ConditionExpr::Or(exprs?))
            }
        }

        Rule::and => {
            let parts: Vec<_> = pair.into_inner().collect();
            if parts.len() == 1 {
                build_ast(parts[0].clone())
            } else {
                let exprs: Result<Vec<_>> = parts.into_iter().map(build_ast).collect();
                Ok(ConditionExpr::And(exprs?))
            }
        }

        Rule::comp => {
            let mut parts = pair.into_inner();
            let left = build_ast(parts.next().unwrap())?;

            if let Some(op_pair) = parts.next() {
                let op = match op_pair.as_str() {
                    ">" => CompareOp::Greater,
                    "<" => CompareOp::Less,
                    ">=" => CompareOp::GreaterEqual,
                    "<=" => CompareOp::LessEqual,
                    "==" => CompareOp::Equal,
                    "!=" => CompareOp::NotEqual,
                    _ => return Err(anyhow!("Unknown comparison operator: {}", op_pair.as_str())),
                };
                let right = build_ast(parts.next().unwrap())?;
                Ok(ConditionExpr::Compare {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            } else {
                Ok(left)
            }
        }

        Rule::value => {
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::expr => build_ast(inner), // Parenthesized expression
                Rule::value => {
                    // Negation: "!" ~ value
                    let negated = inner.into_inner().next().unwrap();
                    Ok(ConditionExpr::Not(Box::new(build_ast(negated)?)))
                }
                _ => build_ast(inner),
            }
        }

        Rule::r#ref => {
            let id = pair.as_str().trim_start_matches('@');
            Ok(ConditionExpr::Ref(id.to_string()))
        }

        Rule::func => {
            let mut parts = pair.into_inner();
            let func_name = parts.next().unwrap().as_str();

            // Next might be args rule or nothing
            let args: Result<Vec<_>> = if let Some(args_pair) = parts.next() {
                if args_pair.as_rule() == Rule::args {
                    args_pair.into_inner().map(build_ast).collect()
                } else {
                    // Single argument without args wrapper
                    vec![build_ast(args_pair)].into_iter().collect()
                }
            } else {
                Ok(vec![])
            };
            let args = args?;

            match func_name {
                "count" => {
                    if args.len() != 1 {
                        return Err(anyhow!("count() expects exactly 1 argument"));
                    }
                    Ok(ConditionExpr::Count(Box::new(args[0].clone())))
                }
                "selected" => {
                    if args.len() != 2 {
                        return Err(anyhow!("selected() expects exactly 2 arguments"));
                    }
                    Ok(ConditionExpr::Selected(
                        Box::new(args[0].clone()),
                        Box::new(args[1].clone()),
                    ))
                }
                "any" => {
                    if args.is_empty() {
                        return Err(anyhow!("any() expects at least 1 argument"));
                    }
                    Ok(ConditionExpr::Any(args))
                }
                "all" => {
                    if args.is_empty() {
                        return Err(anyhow!("all() expects at least 1 argument"));
                    }
                    Ok(ConditionExpr::All(args))
                }
                _ => Err(anyhow!("Unknown function: {}", func_name)),
            }
        }

        Rule::ident => {
            // Bare identifier = implicit string literal
            Ok(ConditionExpr::String(pair.as_str().to_string()))
        }

        Rule::number => {
            let num = pair.as_str().parse::<f64>()
                .map_err(|e| anyhow!("Failed to parse number: {}", e))?;
            Ok(ConditionExpr::Number(num))
        }

        Rule::string => {
            // Strip quotes from string literal
            let s = pair.as_str();
            let unquoted = &s[1..s.len()-1]; // Remove first and last char (quotes)
            Ok(ConditionExpr::String(unquoted.to_string()))
        }

        _ => Err(anyhow!("Unexpected rule: {:?}", pair.as_rule())),
    }
}

/// Evaluate a condition expression against popup state
pub fn evaluate_condition(
    expr: &ConditionExpr,
    state: &HashMap<String, Value>,
) -> bool {
    match expr {
        ConditionExpr::Or(exprs) => {
            // Short-circuit: return true if any is truthy
            exprs.iter().any(|e| evaluate_condition(e, state))
        }

        ConditionExpr::And(exprs) => {
            // Short-circuit: return false if any is falsy
            exprs.iter().all(|e| evaluate_condition(e, state))
        }

        ConditionExpr::Not(inner) => {
            !evaluate_condition(inner, state)
        }

        ConditionExpr::Compare { op, left, right } => {
            let left_val = eval_to_value(left, state);
            let right_val = eval_to_value(right, state);
            compare_values(&left_val, op, &right_val)
        }

        ConditionExpr::Ref(id) => {
            // Reference to field - check truthiness of value
            state.get(id).map(is_truthy).unwrap_or(false)
        }

        ConditionExpr::Number(n) => *n != 0.0, // Non-zero is truthy

        ConditionExpr::String(s) => !s.is_empty(), // Non-empty is truthy

        ConditionExpr::Boolean(b) => *b,

        ConditionExpr::Count(field_ref) => {
            if let ConditionExpr::Ref(id) = &**field_ref {
                let count = count_selections(state, id);
                count > 0
            } else {
                false
            }
        }

        ConditionExpr::Selected(field_ref, value_expr) => {
            if let (ConditionExpr::Ref(id), value_str) = (&**field_ref, eval_to_string(value_expr, state)) {
                is_selected(state, id, &value_str)
            } else {
                false
            }
        }

        ConditionExpr::Any(exprs) => {
            exprs.iter().any(|e| evaluate_condition(e, state))
        }

        ConditionExpr::All(exprs) => {
            exprs.iter().all(|e| evaluate_condition(e, state))
        }
    }
}

/// Evaluate expression to JSON Value
fn eval_to_value(expr: &ConditionExpr, state: &HashMap<String, Value>) -> Value {
    match expr {
        ConditionExpr::Ref(id) => state.get(id).cloned().unwrap_or(Value::Null),
        ConditionExpr::Number(n) => Value::Number(serde_json::Number::from_f64(*n).unwrap()),
        ConditionExpr::String(s) => Value::String(s.clone()),
        ConditionExpr::Boolean(b) => Value::Bool(*b),
        ConditionExpr::Count(field_ref) => {
            if let ConditionExpr::Ref(id) = &**field_ref {
                Value::Number(count_selections(state, id).into())
            } else {
                Value::Number(0.into())
            }
        }
        _ => Value::Null,
    }
}

/// Evaluate expression to string
fn eval_to_string(expr: &ConditionExpr, state: &HashMap<String, Value>) -> String {
    match expr {
        ConditionExpr::String(s) => s.clone(),
        ConditionExpr::Number(n) => n.to_string(),
        ConditionExpr::Ref(id) => {
            state.get(id)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        }
        _ => String::new(),
    }
}

/// Compare two JSON values
fn compare_values(left: &Value, op: &CompareOp, right: &Value) -> bool {
    // Try numeric comparison first
    if let (Some(l), Some(r)) = (left.as_f64(), right.as_f64()) {
        return match op {
            CompareOp::Greater => l > r,
            CompareOp::Less => l < r,
            CompareOp::GreaterEqual => l >= r,
            CompareOp::LessEqual => l <= r,
            CompareOp::Equal => (l - r).abs() < f64::EPSILON,
            CompareOp::NotEqual => (l - r).abs() >= f64::EPSILON,
        };
    }

    // Try string comparison
    if let (Some(l), Some(r)) = (left.as_str(), right.as_str()) {
        return match op {
            CompareOp::Equal => l == r,
            CompareOp::NotEqual => l != r,
            CompareOp::Greater => l > r,
            CompareOp::Less => l < r,
            CompareOp::GreaterEqual => l >= r,
            CompareOp::LessEqual => l <= r,
        };
    }

    // Fall back to equality check
    match op {
        CompareOp::Equal => left == right,
        CompareOp::NotEqual => left != right,
        _ => false,
    }
}

/// Check if value is truthy
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
        Value::Null => false,
    }
}

/// Count selections in multiselect or checkbox
fn count_selections(state: &HashMap<String, Value>, id: &str) -> i64 {
    state.get(id).map(|v| {
        match v {
            Value::Bool(b) => if *b { 1 } else { 0 },
            Value::Array(arr) => arr.iter().filter(|v| {
                // Count booleans (true), non-empty strings, and positive numbers
                v.as_bool().unwrap_or(false)
                    || v.as_str().map(|s| !s.is_empty()).unwrap_or(false)
                    || v.as_i64().map(|n| n > 0).unwrap_or(false)
            }).count() as i64,
            Value::Number(n) => if n.as_i64().unwrap_or(0) > 0 { 1 } else { 0 },
            _ => 0,
        }
    }).unwrap_or(0)
}

/// Check if specific value is selected
fn is_selected(state: &HashMap<String, Value>, id: &str, value: &str) -> bool {
    state.get(id).map(|v| {
        match v {
            Value::Bool(b) => *b && id == value, // Checkbox: match if checked and label matches
            Value::String(s) => s == value,      // Choice: selected option text
            Value::Array(arr) => {
                // Multiselect: check if value is in selected options
                arr.iter().any(|v| v.as_str().map(|s| s == value).unwrap_or(false))
            }
            _ => false,
        }
    }).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_ref() {
        let ast = parse_condition("@enabled").unwrap();
        assert_eq!(ast, ConditionExpr::Ref("enabled".to_string()));
    }

    #[test]
    fn test_parse_comparison() {
        let ast = parse_condition("@cpu > 80").unwrap();
        match ast {
            ConditionExpr::Compare { op, .. } => assert_eq!(op, CompareOp::Greater),
            _ => panic!("Expected comparison"),
        }
    }

    #[test]
    fn test_parse_and() {
        let ast = parse_condition("@cpu > 80 && @mem > 80").unwrap();
        match ast {
            ConditionExpr::And(exprs) => assert_eq!(exprs.len(), 2),
            _ => panic!("Expected AND"),
        }
    }

    #[test]
    fn test_parse_function_count() {
        let ast = parse_condition("count(@items) >= 3").unwrap();
        match ast {
            ConditionExpr::Compare { left, .. } => {
                match left.as_ref() {
                    ConditionExpr::Count(_) => {},
                    _ => panic!("Expected count function"),
                }
            }
            _ => panic!("Expected comparison"),
        }
    }

    #[test]
    fn test_parse_unquoted_string() {
        let ast = parse_condition("selected(@theme, Dark)").unwrap();
        match ast {
            ConditionExpr::Selected(_, value) => {
                match value.as_ref() {
                    ConditionExpr::String(s) => assert_eq!(s, "Dark"),
                    _ => panic!("Expected string"),
                }
            }
            _ => panic!("Expected selected function"),
        }
    }

    #[test]
    fn test_evaluate_truthiness() {
        let mut state = HashMap::new();
        state.insert("enabled".to_string(), Value::Bool(true));
        state.insert("disabled".to_string(), Value::Bool(false));

        let ast = parse_condition("@enabled").unwrap();
        assert!(evaluate_condition(&ast, &state));

        let ast = parse_condition("@disabled").unwrap();
        assert!(!evaluate_condition(&ast, &state));
    }

    #[test]
    fn test_evaluate_comparison() {
        let mut state = HashMap::new();
        state.insert("cpu".to_string(), Value::Number(85.into()));

        let ast = parse_condition("@cpu > 80").unwrap();
        assert!(evaluate_condition(&ast, &state));

        let ast = parse_condition("@cpu < 80").unwrap();
        assert!(!evaluate_condition(&ast, &state));
    }

    #[test]
    fn test_slider_comparisons_all_operators() {
        // Test slider comparison with all operators (for Phase 5: slider comparisons in when clauses)
        let mut state = HashMap::new();
        // Use float value like sliders produce
        state.insert("severity".to_string(), Value::Number(serde_json::Number::from_f64(7.5).unwrap()));

        // Greater than
        assert!(evaluate_condition(&parse_condition("@severity > 5").unwrap(), &state));
        assert!(!evaluate_condition(&parse_condition("@severity > 8").unwrap(), &state));

        // Less than
        assert!(evaluate_condition(&parse_condition("@severity < 10").unwrap(), &state));
        assert!(!evaluate_condition(&parse_condition("@severity < 7").unwrap(), &state));

        // Greater or equal
        assert!(evaluate_condition(&parse_condition("@severity >= 7.5").unwrap(), &state));
        assert!(evaluate_condition(&parse_condition("@severity >= 7").unwrap(), &state));
        assert!(!evaluate_condition(&parse_condition("@severity >= 8").unwrap(), &state));

        // Less or equal
        assert!(evaluate_condition(&parse_condition("@severity <= 7.5").unwrap(), &state));
        assert!(evaluate_condition(&parse_condition("@severity <= 8").unwrap(), &state));
        assert!(!evaluate_condition(&parse_condition("@severity <= 7").unwrap(), &state));

        // Equality
        assert!(evaluate_condition(&parse_condition("@severity == 7.5").unwrap(), &state));
        assert!(!evaluate_condition(&parse_condition("@severity == 7").unwrap(), &state));

        // Not equal
        assert!(evaluate_condition(&parse_condition("@severity != 8").unwrap(), &state));
        assert!(!evaluate_condition(&parse_condition("@severity != 7.5").unwrap(), &state));
    }

    #[test]
    fn test_slider_with_logical_operators() {
        // Test combining slider comparisons with logical operators
        let mut state = HashMap::new();
        state.insert("severity".to_string(), Value::Number(serde_json::Number::from_f64(8.0).unwrap()));
        state.insert("debug".to_string(), Value::Bool(true));

        // Combined conditions (typical use case: show warning when severity high AND debug enabled)
        assert!(evaluate_condition(&parse_condition("@severity >= 8 && @debug").unwrap(), &state));

        // Turn off debug
        state.insert("debug".to_string(), Value::Bool(false));
        assert!(!evaluate_condition(&parse_condition("@severity >= 8 && @debug").unwrap(), &state));

        // OR logic
        assert!(evaluate_condition(&parse_condition("@severity >= 8 || @debug").unwrap(), &state));

        // Lower severity
        state.insert("severity".to_string(), Value::Number(serde_json::Number::from_f64(5.0).unwrap()));
        assert!(!evaluate_condition(&parse_condition("@severity >= 8 || @debug").unwrap(), &state));
    }

    #[test]
    fn test_slider_range_conditions() {
        // Test range conditions like "5 <= severity <= 8"
        // (Since we don't have direct range syntax, this uses AND)
        let mut state = HashMap::new();
        state.insert("level".to_string(), Value::Number(serde_json::Number::from_f64(6.0).unwrap()));

        // Value in range [5, 8]
        assert!(evaluate_condition(&parse_condition("@level >= 5 && @level <= 8").unwrap(), &state));

        // Value at boundary
        state.insert("level".to_string(), Value::Number(serde_json::Number::from_f64(5.0).unwrap()));
        assert!(evaluate_condition(&parse_condition("@level >= 5 && @level <= 8").unwrap(), &state));

        // Value outside range
        state.insert("level".to_string(), Value::Number(serde_json::Number::from_f64(4.0).unwrap()));
        assert!(!evaluate_condition(&parse_condition("@level >= 5 && @level <= 8").unwrap(), &state));
    }
}
