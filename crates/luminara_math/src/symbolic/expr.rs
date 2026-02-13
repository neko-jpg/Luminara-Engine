//! Symbolic expression AST and basic operations.
//!
//! Represents mathematical expressions as an abstract syntax tree.

use std::fmt;
use std::rc::Rc;

/// A node in the symbolic expression tree.
#[derive(Clone, Debug, PartialEq)]
pub enum SymExpr {
    Const(f64),
    Var(String),
    Add(Rc<SymExpr>, Rc<SymExpr>),
    Sub(Rc<SymExpr>, Rc<SymExpr>),
    Mul(Rc<SymExpr>, Rc<SymExpr>),
    Div(Rc<SymExpr>, Rc<SymExpr>),
    Pow(Rc<SymExpr>, Rc<SymExpr>),
    Neg(Rc<SymExpr>),
    Sin(Rc<SymExpr>),
    Cos(Rc<SymExpr>),
    Exp(Rc<SymExpr>),
    Ln(Rc<SymExpr>),
}

impl SymExpr {
    pub fn constant(val: f64) -> Rc<Self> {
        Rc::new(Self::Const(val))
    }

    pub fn var(name: &str) -> Rc<Self> {
        Rc::new(Self::Var(name.to_string()))
    }

    pub fn add(lhs: Rc<Self>, rhs: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Add(lhs, rhs))
    }

    pub fn sub(lhs: Rc<Self>, rhs: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Sub(lhs, rhs))
    }

    pub fn mul(lhs: Rc<Self>, rhs: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Mul(lhs, rhs))
    }

    pub fn div(lhs: Rc<Self>, rhs: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Div(lhs, rhs))
    }

    pub fn pow(base: Rc<Self>, exp: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Pow(base, exp))
    }

    pub fn neg(expr: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Neg(expr))
    }

    pub fn sin(expr: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Sin(expr))
    }

    pub fn cos(expr: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Cos(expr))
    }

    pub fn exp(expr: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Exp(expr))
    }

    pub fn ln(expr: Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Ln(expr))
    }
}

// Helper trait for constructing expressions more easily
pub trait ToSymExpr {
    fn to_sym(&self) -> Rc<SymExpr>;
}

impl ToSymExpr for f64 {
    fn to_sym(&self) -> Rc<SymExpr> {
        SymExpr::constant(*self)
    }
}

impl ToSymExpr for Rc<SymExpr> {
    fn to_sym(&self) -> Rc<SymExpr> {
        self.clone()
    }
}

impl fmt::Display for SymExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymExpr::Const(v) => write!(f, "{}", v),
            SymExpr::Var(n) => write!(f, "{}", n),
            SymExpr::Add(l, r) => write!(f, "({} + {})", l, r),
            SymExpr::Sub(l, r) => write!(f, "({} - {})", l, r),
            SymExpr::Mul(l, r) => write!(f, "({} * {})", l, r),
            SymExpr::Div(l, r) => write!(f, "({} / {})", l, r),
            SymExpr::Pow(b, e) => write!(f, "pow({}, {})", b, e),
            SymExpr::Neg(e) => write!(f, "-{}", e),
            SymExpr::Sin(e) => write!(f, "sin({})", e),
            SymExpr::Cos(e) => write!(f, "cos({})", e),
            SymExpr::Exp(e) => write!(f, "exp({})", e),
            SymExpr::Ln(e) => write!(f, "ln({})", e),
        }
    }
}
