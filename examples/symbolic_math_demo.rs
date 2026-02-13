use luminara_math::symbolic::{SymExpr, differentiate, simplify, compile_to_fn};
use std::collections::HashMap;

fn main() {
    println!("Luminara Math - Symbolic Math Demo");
    println!("==================================\n");

    // 1. Define symbolic variables
    let x = SymExpr::var("x");
    let y = SymExpr::var("y");

    // 2. Build an expression: f(x, y) = x^2 + sin(y)
    let expr = SymExpr::add(
        SymExpr::pow(x.clone(), SymExpr::constant(2.0)),
        SymExpr::sin(y.clone())
    );
    println!("Expression: f(x, y) = {}", expr);

    // 3. Differentiate wrt x
    let df_dx = differentiate(&expr, "x");
    println!("df/dx (raw): {}", df_dx);
    println!("df/dx (simplified): {}", simplify(&df_dx));

    // 4. Differentiate wrt y
    let df_dy = differentiate(&expr, "y");
    println!("df/dy (raw): {}", df_dy);
    println!("df/dy (simplified): {}", simplify(&df_dy));

    // 5. Compile and Evaluate
    let func = compile_to_fn(&expr);
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), 3.0);
    vars.insert("y".to_string(), std::f32::consts::PI as f64 / 2.0);

    let val = func.eval(&vars);
    println!("\nEvaluation at x=3, y=PI/2:");
    println!("x^2 + sin(y) = 3^2 + 1 = {}", val);
}
