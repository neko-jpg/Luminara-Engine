//! WGSL shader code generation from symbolic expressions.
//!
//! Compiles symbolic expressions to WGSL compute shader code.

use super::expr::SymExpr;
use std::fmt::Write;
use std::rc::Rc;

/// Generate WGSL code for a symbolic expression.
pub fn compile_to_wgsl(expr: &Rc<SymExpr>) -> String {
    let mut buffer = String::new();
    generate_recursive(expr, &mut buffer);
    buffer
}

fn generate_recursive(expr: &SymExpr, buffer: &mut String) {
    match expr {
        SymExpr::Const(v) => {
            // Ensure float formatting
            if v.fract() == 0.0 {
                write!(buffer, "{:.1}", v).unwrap();
            } else {
                write!(buffer, "{}", v).unwrap();
            }
        }
        SymExpr::Var(name) => {
            write!(buffer, "{}", name).unwrap();
        }
        SymExpr::Add(l, r) => {
            write!(buffer, "(").unwrap();
            generate_recursive(l, buffer);
            write!(buffer, " + ").unwrap();
            generate_recursive(r, buffer);
            write!(buffer, ")").unwrap();
        }
        SymExpr::Sub(l, r) => {
            write!(buffer, "(").unwrap();
            generate_recursive(l, buffer);
            write!(buffer, " - ").unwrap();
            generate_recursive(r, buffer);
            write!(buffer, ")").unwrap();
        }
        SymExpr::Mul(l, r) => {
            write!(buffer, "(").unwrap();
            generate_recursive(l, buffer);
            write!(buffer, " * ").unwrap();
            generate_recursive(r, buffer);
            write!(buffer, ")").unwrap();
        }
        SymExpr::Div(l, r) => {
            write!(buffer, "(").unwrap();
            generate_recursive(l, buffer);
            write!(buffer, " / ").unwrap();
            generate_recursive(r, buffer);
            write!(buffer, ")").unwrap();
        }
        SymExpr::Pow(b, e) => {
            write!(buffer, "pow(").unwrap();
            generate_recursive(b, buffer);
            write!(buffer, ", ").unwrap();
            generate_recursive(e, buffer);
            write!(buffer, ")").unwrap();
        }
        SymExpr::Neg(inner) => {
            write!(buffer, "-(").unwrap();
            generate_recursive(inner, buffer);
            write!(buffer, ")").unwrap();
        }
        SymExpr::Sin(inner) => {
            write!(buffer, "sin(").unwrap();
            generate_recursive(inner, buffer);
            write!(buffer, ")").unwrap();
        }
        SymExpr::Cos(inner) => {
            write!(buffer, "cos(").unwrap();
            generate_recursive(inner, buffer);
            write!(buffer, ")").unwrap();
        }
        SymExpr::Exp(inner) => {
            write!(buffer, "exp(").unwrap();
            generate_recursive(inner, buffer);
            write!(buffer, ")").unwrap();
        }
        SymExpr::Ln(inner) => {
            write!(buffer, "log(").unwrap(); // WGSL uses log for natural logarithm
            generate_recursive(inner, buffer);
            write!(buffer, ")").unwrap();
        }
    }
}
