//! Symbolic differentiation with chain rule.
//!
//! Provides automatic differentiation for symbolic expressions.

use super::expr::SymExpr;
use std::rc::Rc;

/// Differentiate an expression with respect to a variable.
pub fn differentiate(expr: &Rc<SymExpr>, var: &str) -> Rc<SymExpr> {
    match &**expr {
        SymExpr::Const(_) => SymExpr::constant(0.0),
        SymExpr::Var(name) => {
            if name == var {
                SymExpr::constant(1.0)
            } else {
                SymExpr::constant(0.0)
            }
        }
        SymExpr::Add(lhs, rhs) => SymExpr::add(differentiate(lhs, var), differentiate(rhs, var)),
        SymExpr::Sub(lhs, rhs) => SymExpr::sub(differentiate(lhs, var), differentiate(rhs, var)),
        SymExpr::Mul(lhs, rhs) => {
            // Product rule: d(uv) = u dv + v du
            SymExpr::add(
                SymExpr::mul(lhs.clone(), differentiate(rhs, var)),
                SymExpr::mul(rhs.clone(), differentiate(lhs, var)),
            )
        }
        SymExpr::Div(lhs, rhs) => {
            // Quotient rule: d(u/v) = (v du - u dv) / v^2
            SymExpr::div(
                SymExpr::sub(
                    SymExpr::mul(rhs.clone(), differentiate(lhs, var)),
                    SymExpr::mul(lhs.clone(), differentiate(rhs, var)),
                ),
                SymExpr::pow(rhs.clone(), SymExpr::constant(2.0)),
            )
        }
        SymExpr::Pow(base, exp) => {
            // Check if exponent is constant for simpler power rule (avoids singularity at base=0)
            if let SymExpr::Const(c) = **exp {
                // d(u^c) = c * u^(c-1) * du
                return SymExpr::mul(
                    SymExpr::mul(
                        SymExpr::constant(c),
                        SymExpr::pow(base.clone(), SymExpr::constant(c - 1.0)),
                    ),
                    differentiate(base, var),
                );
            }

            // Power rule generalized: d(u^v) = u^v * (v/u du + ln(u) dv)

            let term1 = SymExpr::div(
                SymExpr::mul(exp.clone(), differentiate(base, var)),
                base.clone(),
            );

            let term2 = SymExpr::mul(SymExpr::ln(base.clone()), differentiate(exp, var));

            SymExpr::mul(
                expr.clone(), // u^v
                SymExpr::add(term1, term2),
            )
        }
        SymExpr::Neg(inner) => SymExpr::neg(differentiate(inner, var)),
        SymExpr::Sin(inner) => {
            // d(sin(u)) = cos(u) du
            SymExpr::mul(SymExpr::cos(inner.clone()), differentiate(inner, var))
        }
        SymExpr::Cos(inner) => {
            // d(cos(u)) = -sin(u) du
            SymExpr::mul(
                SymExpr::neg(SymExpr::sin(inner.clone())),
                differentiate(inner, var),
            )
        }
        SymExpr::Exp(inner) => {
            // d(exp(u)) = exp(u) du
            SymExpr::mul(expr.clone(), differentiate(inner, var))
        }
        SymExpr::Ln(inner) => {
            // d(ln(u)) = 1/u du
            SymExpr::div(differentiate(inner, var), inner.clone())
        }
    }
}

/// Compute the Jacobian matrix of a vector of expressions with respect to a list of variables.
/// Returns a flattened row-major vector of expressions.
pub fn jacobian(exprs: &[Rc<SymExpr>], vars: &[&str]) -> Vec<Rc<SymExpr>> {
    let mut jac = Vec::with_capacity(exprs.len() * vars.len());

    for expr in exprs {
        for var in vars {
            jac.push(differentiate(expr, var));
        }
    }

    jac
}
