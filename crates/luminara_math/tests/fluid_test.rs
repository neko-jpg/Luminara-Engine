use luminara_math::dynamics::fluid_viz::FluidVisualization;

#[test]
fn test_fluid_viz_initialization() {
    let viz = FluidVisualization::new(64, 64);
    assert_eq!(viz.width, 64);
    assert_eq!(viz.height, 64);
    // solver initialization is implicit
}
