//! MathDesignCommand DSL for unified AI interface.
//!
//! Provides high-level mathematical operations for AI systems.

use glam::Vec3;
use std::rc::Rc;
use crate::symbolic::{SymExpr, differentiate};

#[derive(Clone, Debug)]
pub enum SpacingMethod {
    Uniform,
    Chebyshev,
    Random,
}

#[derive(Clone, Debug)]
pub enum TopologyMetric {
    Euclidean,
    Geodesic,
}

#[derive(Clone, Debug)]
pub enum MathDesignCommand {
    /// Create a trajectory from a parametric equation.
    /// Param: position(t)
    Trajectory {
        equation: Rc<SymExpr>,
        variable: String,
        t_start: f64,
        t_end: f64,
        steps: usize,
    },
    /// Distribute points on a manifold.
    DistributePoints {
        count: usize,
        method: SpacingMethod,
    },
    /// Compute distance matrix.
    ComputeDistance {
        metric: TopologyMetric,
    },
}

pub struct MathCommandExecutor;

impl MathCommandExecutor {
    /// Execute a command.
    /// Returns a generic result (e.g. Vec<Vec3> for points).
    /// For simplicity, we return a Result with possible output.
    pub fn execute(&self, cmd: &MathDesignCommand) -> Result<MathCommandOutput, String> {
        match cmd {
            MathDesignCommand::Trajectory { equation, variable, t_start, t_end, steps } => {
                self.execute_trajectory(equation, variable, *t_start, *t_end, *steps)
            }
            MathDesignCommand::DistributePoints { .. } => {
                // Placeholder
                Ok(MathCommandOutput::Points(vec![]))
            }
            MathDesignCommand::ComputeDistance { .. } => {
                // Placeholder
                Ok(MathCommandOutput::Matrix(vec![]))
            }
        }
    }

    fn execute_trajectory(
        &self,
        equation: &Rc<SymExpr>,
        var: &str,
        t_start: f64,
        t_end: f64,
        steps: usize
    ) -> Result<MathCommandOutput, String> {
        // Automatic differentiation to get velocity and acceleration
        let velocity_expr = differentiate(equation, var);
        let acceleration_expr = differentiate(&velocity_expr, var);

        // Evaluate
        let mut points = Vec::with_capacity(steps);
        let dt = (t_end - t_start) / (steps as f64 - 1.0);

        for i in 0..steps {
            let t = t_start + dt * (i as f64);
            let p = self.eval_vec3(equation, var, t);
            let v = self.eval_vec3(&velocity_expr, var, t);
            let a = self.eval_vec3(&acceleration_expr, var, t);

            points.push(TrajectoryPoint {
                time: t,
                position: p,
                velocity: v,
                acceleration: a,
            });
        }

        Ok(MathCommandOutput::Trajectory(points))
    }

    // Helper to evaluate SymExpr as Vec3?
    // SymExpr currently returns scalar.
    // If equation represents a vector, it should be a vector of SymExprs?
    // The current SymExpr is scalar.
    // Let's assume the user provided an expression that evaluates to a scalar for 1D trajectory,
    // or we extend MathDesignCommand to take [SymExpr; 3] for 3D.
    // For now, let's treat it as 1D trajectory packed into x component of Vec3.

    fn eval_vec3(&self, expr: &SymExpr, var: &str, val: f64) -> Vec3 {
        // Simple evaluation logic (duplicated from rust_codegen for now to avoid dep loops or complexity)
        // Actually, we can use a helper or just implement it.
        let scalar = evaluate_scalar(expr, var, val);
        Vec3::new(scalar as f32, 0.0, 0.0)
    }
}

fn evaluate_scalar(expr: &SymExpr, var: &str, val: f64) -> f64 {
    match expr {
        SymExpr::Const(v) => *v,
        SymExpr::Var(name) => if name == var { val } else { f64::NAN },
        SymExpr::Add(l, r) => evaluate_scalar(l, var, val) + evaluate_scalar(r, var, val),
        SymExpr::Sub(l, r) => evaluate_scalar(l, var, val) - evaluate_scalar(r, var, val),
        SymExpr::Mul(l, r) => evaluate_scalar(l, var, val) * evaluate_scalar(r, var, val),
        SymExpr::Div(l, r) => evaluate_scalar(l, var, val) / evaluate_scalar(r, var, val),
        SymExpr::Pow(b, e) => evaluate_scalar(b, var, val).powf(evaluate_scalar(e, var, val)),
        SymExpr::Neg(e) => -evaluate_scalar(e, var, val),
        SymExpr::Sin(e) => evaluate_scalar(e, var, val).sin(),
        SymExpr::Cos(e) => evaluate_scalar(e, var, val).cos(),
        SymExpr::Exp(e) => evaluate_scalar(e, var, val).exp(),
        SymExpr::Ln(e) => evaluate_scalar(e, var, val).ln(),
    }
}

#[derive(Debug)]
pub enum MathCommandOutput {
    Points(Vec<Vec3>),
    Matrix(Vec<f64>),
    Trajectory(Vec<TrajectoryPoint>),
}

#[derive(Debug, Clone)]
pub struct TrajectoryPoint {
    pub time: f64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
}
