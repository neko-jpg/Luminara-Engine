use luminara_math::symbolic::{ShaderGenerator, SymExpr};

#[test]
fn test_symexpr_to_wgsl() {
    // Expr: sin(x * t) + 0.5
    let expr = SymExpr::add(
        SymExpr::sin(SymExpr::mul(SymExpr::var("x"), SymExpr::var("t"))),
        SymExpr::constant(0.5),
    );

    let wgsl = ShaderGenerator::generate_wgsl(&expr);

    // Verify structure
    assert!(wgsl.contains("let t = input.time;"));
    assert!(wgsl.contains("let val = (sin((x * t)) + 0.5000);"));
    assert!(wgsl.contains("@fragment"));
}
