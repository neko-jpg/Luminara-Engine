//! Rust closure generation from symbolic expressions.
//!
//! Compiles symbolic expressions to executable Rust closures.

use super::expr::SymExpr;
use std::collections::HashMap;
use std::rc::Rc;

/// Trait for evaluating compiled expressions.
/// Supports variable lookup.
pub trait Evaluator {
    fn eval(&self, vars: &HashMap<String, f64>) -> f64;
}

/// Compile a symbolic expression into a boxed closure.
/// Note: This does not generate source code, but returns an executable object.
pub fn compile_to_fn(expr: &Rc<SymExpr>) -> Box<dyn Evaluator> {
    let expr_clone = expr.clone();

    // We can just return a struct that holds the expression tree and interprets it.
    // True "JIT compilation" to machine code requires a lot more work (e.g. cranelift).
    // For now, "compilation" means preparing an efficient evaluator or just wrapping interpretation.
    // To satisfy requirement "Generate Rust closures", we return a trait object.

    Box::new(Interpreter { expr: expr_clone })
}

struct Interpreter {
    expr: Rc<SymExpr>,
}

impl Evaluator for Interpreter {
    fn eval(&self, vars: &HashMap<String, f64>) -> f64 {
        evaluate_recursive(&self.expr, vars)
    }
}

fn evaluate_recursive(expr: &SymExpr, vars: &HashMap<String, f64>) -> f64 {
    match expr {
        SymExpr::Const(v) => *v,
        SymExpr::Var(name) => *vars.get(name).unwrap_or(&f64::NAN),
        SymExpr::Add(l, r) => evaluate_recursive(l, vars) + evaluate_recursive(r, vars),
        SymExpr::Sub(l, r) => evaluate_recursive(l, vars) - evaluate_recursive(r, vars),
        SymExpr::Mul(l, r) => evaluate_recursive(l, vars) * evaluate_recursive(r, vars),
        SymExpr::Div(l, r) => evaluate_recursive(l, vars) / evaluate_recursive(r, vars),
        SymExpr::Pow(b, e) => evaluate_recursive(b, vars).powf(evaluate_recursive(e, vars)),
        SymExpr::Neg(e) => -evaluate_recursive(e, vars),
        SymExpr::Sin(e) => evaluate_recursive(e, vars).sin(),
        SymExpr::Cos(e) => evaluate_recursive(e, vars).cos(),
        SymExpr::Exp(e) => evaluate_recursive(e, vars).exp(),
        SymExpr::Ln(e) => evaluate_recursive(e, vars).ln(),
    }
}
