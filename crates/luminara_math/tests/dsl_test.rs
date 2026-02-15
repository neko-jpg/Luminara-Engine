use luminara_math::dsl::{MathCommandExecutor, MathCommandOutput, MathDesignCommand};
use luminara_math::symbolic::SymExpr;
use proptest::prelude::*;
use std::rc::Rc;

#[test]
fn test_trajectory_command() {
    // position(t) = t^2
    let t = SymExpr::var("t");
    let expr = SymExpr::pow(t, SymExpr::constant(2.0));

    let cmd = MathDesignCommand::Trajectory {
        equation: expr,
        variable: "t".to_string(),
        t_start: 0.0,
        t_end: 2.0,
        steps: 3, // 0.0, 1.0, 2.0
    };

    let executor = MathCommandExecutor;
    let result = executor.execute(&cmd).unwrap();

    if let MathCommandOutput::Trajectory(points) = result {
        assert_eq!(points.len(), 3);

        // t=0: p=0, v=0, a=2
        assert_eq!(points[0].time, 0.0);
        assert!((points[0].position.x - 0.0).abs() < 1e-5);
        assert!((points[0].velocity.x - 0.0).abs() < 1e-5);
        assert!((points[0].acceleration.x - 2.0).abs() < 1e-5);

        // t=1: p=1, v=2, a=2
        assert_eq!(points[1].time, 1.0);
        assert!((points[1].position.x - 1.0).abs() < 1e-5);
        assert!((points[1].velocity.x - 2.0).abs() < 1e-5);
        assert!((points[1].acceleration.x - 2.0).abs() < 1e-5);

        // t=2: p=4, v=4, a=2
        assert_eq!(points[2].time, 2.0);
        assert!((points[2].position.x - 4.0).abs() < 1e-5);
        assert!((points[2].velocity.x - 4.0).abs() < 1e-5);
        assert!((points[2].acceleration.x - 2.0).abs() < 1e-5);
    } else {
        panic!("Wrong output type");
    }
}

// Property testing for DSL
// Generate simple expressions and check if velocity/acceleration are consistent roughly?
// v approx (p(t+dt) - p(t-dt)) / 2dt

prop_compose! {
    fn arb_quadratic()(a in -2.0f64..2.0, b in -2.0f64..2.0, c in -2.0f64..2.0) -> Rc<SymExpr> {
        // a*t^2 + b*t + c
        let t = SymExpr::var("t");
        let at2 = SymExpr::mul(SymExpr::constant(a), SymExpr::pow(t.clone(), SymExpr::constant(2.0)));
        let bt = SymExpr::mul(SymExpr::constant(b), t.clone());
        let const_c = SymExpr::constant(c);

        SymExpr::add(SymExpr::add(at2, bt), const_c)
    }
}

proptest! {
    // Property 24: MathDesignCommand Automatic Differentiation
    // Validates: Requirements 10.7
    #[test]
    fn prop_auto_diff_consistency(expr in arb_quadratic()) {
        let cmd = MathDesignCommand::Trajectory {
            equation: expr,
            variable: "t".to_string(),
            t_start: 0.0,
            t_end: 1.0,
            steps: 5,
        };

        let executor = MathCommandExecutor;
        let result = executor.execute(&cmd).unwrap();

        if let MathCommandOutput::Trajectory(points) = result {
            for i in 1..points.len()-1 {
                let p_prev = points[i-1].position.x as f64;
                let p_next = points[i+1].position.x as f64;
                let dt = points[i+1].time - points[i-1].time;
                let v_approx = (p_next - p_prev) / dt;

                let v_exact = points[i].velocity.x as f64;

                // Quadratic: derivative is linear. Central difference is exact for quadratic.
                prop_assert!((v_exact as f64 - v_approx as f64).abs() < 1e-4,
                    "Velocity mismatch at {}: exact {}, approx {}", points[i].time, v_exact, v_approx);
            }
        }
    }
}
