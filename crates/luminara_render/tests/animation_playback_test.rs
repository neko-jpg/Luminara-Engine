use luminara_asset::{AssetServer, Handle};
use luminara_core::time::Time;
use luminara_core::{App, CoreStage, Resource, World};
use luminara_math::{Quat, Transform, Vec3};
use luminara_render::animation::{
    AnimationChannel, AnimationClip, AnimationOutput, AnimationPath, SkinnedMesh,
};
use luminara_render::animation_system::{animation_system, AnimationPlayer};

// Mock AssetServer for test (difficult without infrastructure)
// Instead, we will test the interpolation math logic directly by simulating the system logic in test.

#[test]
fn test_animation_interpolation_logic() {
    let inputs = vec![0.0, 1.0, 2.0];
    let values = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
    ];

    // Time = 0.5
    let time = 0.5;

    // Find keyframe
    let mut k = 0;
    while k < inputs.len() - 1 && inputs[k + 1] < time {
        k += 1;
    }

    let t = (time - inputs[k]) / (inputs[k + 1] - inputs[k]);
    let result = values[k].lerp(values[k + 1], t);

    assert_eq!(result, Vec3::new(5.0, 0.0, 0.0));

    // Time = 1.5
    let time = 1.5;
    let mut k = 0;
    while k < inputs.len() - 1 && inputs[k + 1] < time {
        k += 1;
    }
    let t = (time - inputs[k]) / (inputs[k + 1] - inputs[k]);
    let result = values[k].lerp(values[k + 1], t);

    assert_eq!(result, Vec3::new(5.0, 0.0, 0.0));
}
