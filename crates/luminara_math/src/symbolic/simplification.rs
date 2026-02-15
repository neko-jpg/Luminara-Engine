//! Symbolic expression simplification.
//!
//! Applies algebraic identities and constant folding.

use super::expr::SymExpr;
use std::rc::Rc;

/// Simplify an expression.
pub fn simplify(expr: &Rc<SymExpr>) -> Rc<SymExpr> {
    match &**expr {
        SymExpr::Const(_) | SymExpr::Var(_) => expr.clone(),

        SymExpr::Add(l, r) => {
            let l_s = simplify(l);
            let r_s = simplify(r);
            match (&*l_s, &*r_s) {
                (SymExpr::Const(a), SymExpr::Const(b)) => SymExpr::constant(a + b),
                (SymExpr::Const(c), _) if *c == 0.0 => r_s,
                (_, SymExpr::Const(c)) if *c == 0.0 => l_s,
                _ => {
                    if l_s == r_s {
                        // x + x = 2x
                        SymExpr::mul(SymExpr::constant(2.0), l_s)
                    } else {
                        SymExpr::add(l_s, r_s)
                    }
                }
            }
        }

        SymExpr::Sub(l, r) => {
            let l_s = simplify(l);
            let r_s = simplify(r);
            match (&*l_s, &*r_s) {
                (SymExpr::Const(a), SymExpr::Const(b)) => SymExpr::constant(a - b),
                (_, SymExpr::Const(c)) if *c == 0.0 => l_s,
                _ => {
                    if l_s == r_s {
                        SymExpr::constant(0.0)
                    } else {
                        SymExpr::sub(l_s, r_s)
                    }
                }
            }
        }

        SymExpr::Mul(l, r) => {
            let l_s = simplify(l);
            let r_s = simplify(r);
            match (&*l_s, &*r_s) {
                (SymExpr::Const(a), SymExpr::Const(b)) => SymExpr::constant(a * b),
                (SymExpr::Const(c), _) if *c == 0.0 => SymExpr::constant(0.0),
                (_, SymExpr::Const(c)) if *c == 0.0 => SymExpr::constant(0.0),
                (SymExpr::Const(c), _) if *c == 1.0 => r_s,
                (_, SymExpr::Const(c)) if *c == 1.0 => l_s,
                _ => SymExpr::mul(l_s, r_s),
            }
        }

        SymExpr::Div(l, r) => {
            let l_s = simplify(l);
            let r_s = simplify(r);
            match (&*l_s, &*r_s) {
                (SymExpr::Const(a), SymExpr::Const(b)) if *b != 0.0 => SymExpr::constant(a / b),
                (SymExpr::Const(c), _) if *c == 0.0 => SymExpr::constant(0.0),
                (_, SymExpr::Const(c)) if *c == 1.0 => l_s,
                _ => {
                    if l_s == r_s {
                        SymExpr::constant(1.0)
                    } else {
                        SymExpr::div(l_s, r_s)
                    }
                }
            }
        }

        SymExpr::Pow(b, e) => {
            let b_s = simplify(b);
            let e_s = simplify(e);
            match (&*b_s, &*e_s) {
                (SymExpr::Const(base), SymExpr::Const(exp)) => SymExpr::constant(base.powf(*exp)),
                (_, SymExpr::Const(c)) if *c == 0.0 => SymExpr::constant(1.0),
                (_, SymExpr::Const(c)) if *c == 1.0 => b_s,
                (SymExpr::Const(c), _) if *c == 0.0 => SymExpr::constant(0.0), // 0^x = 0
                (SymExpr::Const(c), _) if *c == 1.0 => SymExpr::constant(1.0), // 1^x = 1
                _ => SymExpr::pow(b_s, e_s),
            }
        }

        SymExpr::Neg(inner) => {
            let s = simplify(inner);
            match &*s {
                SymExpr::Const(c) => SymExpr::constant(-c),
                SymExpr::Neg(inner_inner) => inner_inner.clone(), // -(-x) = x
                _ => SymExpr::neg(s),
            }
        }

        // Transcendental functions
        SymExpr::Sin(inner) => SymExpr::sin(simplify(inner)),
        SymExpr::Cos(inner) => SymExpr::cos(simplify(inner)),
        SymExpr::Exp(inner) => SymExpr::exp(simplify(inner)),
        SymExpr::Ln(inner) => SymExpr::ln(simplify(inner)),
    }
}
