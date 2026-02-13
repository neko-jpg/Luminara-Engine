use luminara_math::Mat4;
use luminara_render::camera::{Camera, Projection};
use proptest::prelude::*;

// ============================================================================
// Property Test 10: Projection Mode Support
// Validates: Requirements 6.1, 6.4
// ============================================================================

/// Strategy for generating valid FOV values (in degrees)
/// Typical range: 30-120 degrees
fn fov_strategy() -> impl Strategy<Value = f32> {
    30.0f32..120.0f32
}

/// Strategy for generating valid near plane values
/// Must be positive and less than far plane
fn near_plane_strategy() -> impl Strategy<Value = f32> {
    0.01f32..10.0f32
}

/// Strategy for generating valid far plane values
/// Must be greater than near plane
fn far_plane_strategy(near: f32) -> impl Strategy<Value = f32> {
    (near + 1.0)..(near + 10000.0)
}

/// Strategy for generating valid orthographic size values
/// Typical range: 1-100 units
fn ortho_size_strategy() -> impl Strategy<Value = f32> {
    1.0f32..100.0f32
}

/// Strategy for generating valid aspect ratios
/// Common ranges: 0.5 (portrait) to 3.0 (ultra-wide)
fn aspect_ratio_strategy() -> impl Strategy<Value = f32> {
    0.5f32..3.0f32
}

/// Strategy for generating perspective projection parameters
fn perspective_params_strategy() -> impl Strategy<Value = (f32, f32, f32, f32)> {
    (
        fov_strategy(),
        near_plane_strategy(),
        aspect_ratio_strategy(),
    )
        .prop_flat_map(|(fov, near, aspect)| {
            far_plane_strategy(near).prop_map(move |far| (fov, near, far, aspect))
        })
}

/// Strategy for generating orthographic projection parameters
fn orthographic_params_strategy() -> impl Strategy<Value = (f32, f32, f32, f32)> {
    (
        ortho_size_strategy(),
        near_plane_strategy(),
        aspect_ratio_strategy(),
    )
        .prop_flat_map(|(size, near, aspect)| {
            far_plane_strategy(near).prop_map(move |far| (size, near, far, aspect))
        })
}

/// Check if a matrix is a valid projection matrix
/// A valid projection matrix should:
/// 1. Not contain NaN or infinity values
/// 2. Not be the identity matrix (unless degenerate case)
/// 3. Have non-zero determinant (invertible) - but can be very small for extreme ranges
fn is_valid_projection_matrix(mat: &Mat4) -> bool {
    // Check for NaN or infinity
    for col in 0..4 {
        for row in 0..4 {
            let val = mat.col(col)[row];
            if val.is_nan() || val.is_infinite() {
                return false;
            }
        }
    }

    // Check that it's not the identity matrix (projection should transform)
    if *mat == Mat4::IDENTITY {
        return false;
    }

    // Check that determinant is non-zero (matrix is invertible)
    // Use a very small epsilon since orthographic projections with large depth ranges
    // can have very small determinants
    let det = mat.determinant();
    if det.abs() < 1e-20 {
        return false;
    }

    true
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 10: Projection Mode Support**
    ///
    /// For any camera component, setting it to either Perspective or Orthographic
    /// projection mode should produce a valid projection matrix with the specified
    /// parameters (FOV/size, near, far, aspect ratio).
    ///
    /// **Validates: Requirements 6.1, 6.4**
    #[test]
    fn prop_perspective_projection_produces_valid_matrix(
        (fov, near, far, aspect) in perspective_params_strategy()
    ) {
        // Create camera with perspective projection
        let camera = Camera {
            projection: Projection::Perspective { fov, near, far },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Generate projection matrix
        let proj_matrix = camera.projection_matrix(aspect);

        // Verify the matrix is valid
        prop_assert!(
            is_valid_projection_matrix(&proj_matrix),
            "Perspective projection should produce a valid matrix.\n\
             FOV: {}, Near: {}, Far: {}, Aspect: {}\n\
             Matrix: {:?}",
            fov, near, far, aspect, proj_matrix
        );

        // Verify the matrix is not identity
        prop_assert_ne!(
            proj_matrix,
            Mat4::IDENTITY,
            "Projection matrix should not be identity"
        );

        // Verify specific properties of perspective projection matrix
        // In a perspective projection matrix (left-handed):
        // - Element [2][2] relates to depth mapping (near/far)
        // - Element [2][3] should be non-zero (perspective divide)
        // - Element [3][2] should be non-zero
        let m23 = proj_matrix.col(2)[3];
        let m32 = proj_matrix.col(3)[2];

        prop_assert!(
            m23.abs() > 1e-6 || m32.abs() > 1e-6,
            "Perspective projection should have non-zero perspective divide component.\n\
             m23: {}, m32: {}",
            m23, m32
        );

        // Verify the matrix transforms points correctly
        // A point at the near plane should map to near depth
        // A point at the far plane should map to far depth
        // Note: perspective_rh looks down -Z, so near plane is at z = -near
        let near_point = luminara_math::Vec4::new(0.0, 0.0, -near, 1.0);
        let transformed_near = proj_matrix * near_point;

        // After perspective divide, z should be at near depth
        if transformed_near.w.abs() > 1e-6 {
            let ndc_z = transformed_near.z / transformed_near.w;
            prop_assert!(
                ndc_z >= 0.0 && ndc_z <= 1.0,
                "Transformed near point should be in NDC range [0, 1].\n\
                 NDC Z: {}",
                ndc_z
            );
        }
    }

    #[test]
    fn prop_orthographic_projection_produces_valid_matrix(
        (size, near, far, aspect) in orthographic_params_strategy()
    ) {
        // Create camera with orthographic projection
        let camera = Camera {
            projection: Projection::Orthographic { size, near, far },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Generate projection matrix
        let proj_matrix = camera.projection_matrix(aspect);

        // Verify the matrix is valid
        prop_assert!(
            is_valid_projection_matrix(&proj_matrix),
            "Orthographic projection should produce a valid matrix.\n\
             Size: {}, Near: {}, Far: {}, Aspect: {}\n\
             Matrix: {:?}",
            size, near, far, aspect, proj_matrix
        );

        // Verify the matrix is not identity
        prop_assert_ne!(
            proj_matrix,
            Mat4::IDENTITY,
            "Projection matrix should not be identity"
        );

        // Verify specific properties of orthographic projection matrix
        // In an orthographic projection matrix:
        // - Element [3][3] should be 1.0 (no perspective divide)
        // - Element [2][3] should be 0.0 (no perspective in the w row)
        // Note: Element [3][2] can be non-zero (depth translation component)
        let m33 = proj_matrix.col(3)[3];
        let m23 = proj_matrix.col(2)[3];

        prop_assert!(
            (m33 - 1.0).abs() < 1e-5,
            "Orthographic projection should have w component = 1.0.\n\
             m33: {}",
            m33
        );

        prop_assert!(
            m23.abs() < 1e-5,
            "Orthographic projection should have no perspective divide in w row.\n\
             m23: {}",
            m23
        );

        // Verify the matrix transforms points correctly
        // Points should maintain their relative positions without perspective distortion
        // Note: orthographic_rh looks down -Z
        let test_point = luminara_math::Vec4::new(1.0, 1.0, -(near + far) / 2.0, 1.0);
        let transformed = proj_matrix * test_point;

        // In orthographic projection, w should remain 1.0
        prop_assert!(
            (transformed.w - 1.0).abs() < 1e-5,
            "Orthographic projection should preserve w = 1.0.\n\
             Transformed w: {}",
            transformed.w
        );

        // Z should be mapped to NDC range
        prop_assert!(
            transformed.z >= 0.0 && transformed.z <= 1.0,
            "Transformed point should have z in NDC range [0, 1].\n\
             Z: {}",
            transformed.z
        );
    }

    /// Test that different projection modes produce different matrices
    #[test]
    fn prop_projection_modes_produce_different_matrices(
        (fov, near, far, aspect) in perspective_params_strategy(),
        size in ortho_size_strategy()
    ) {
        // Create perspective camera
        let perspective_camera = Camera {
            projection: Projection::Perspective { fov, near, far },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Create orthographic camera with same near/far
        let orthographic_camera = Camera {
            projection: Projection::Orthographic { size, near, far },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Generate projection matrices
        let persp_matrix = perspective_camera.projection_matrix(aspect);
        let ortho_matrix = orthographic_camera.projection_matrix(aspect);

        // Verify they produce different matrices
        prop_assert_ne!(
            persp_matrix,
            ortho_matrix,
            "Perspective and orthographic projections should produce different matrices"
        );
    }

    /// Test that changing projection parameters produces different matrices
    #[test]
    fn prop_different_parameters_produce_different_matrices(
        (fov1, near1, far1, aspect) in perspective_params_strategy(),
        fov2 in fov_strategy()
    ) {
        // Ensure fov2 is different from fov1
        prop_assume!((fov1 - fov2).abs() > 1.0);

        // Create two cameras with different FOV
        let camera1 = Camera {
            projection: Projection::Perspective { fov: fov1, near: near1, far: far1 },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        let camera2 = Camera {
            projection: Projection::Perspective { fov: fov2, near: near1, far: far1 },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Generate projection matrices
        let matrix1 = camera1.projection_matrix(aspect);
        let matrix2 = camera2.projection_matrix(aspect);

        // Verify they produce different matrices
        prop_assert_ne!(
            matrix1,
            matrix2,
            "Different FOV values should produce different projection matrices.\n\
             FOV1: {}, FOV2: {}",
            fov1, fov2
        );
    }
}

// ============================================================================
// Property Test 11: Projection Matrix Update
// Validates: Requirements 6.2
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 11: Projection Matrix Update**
    ///
    /// For any camera, changing its projection parameters (FOV, size, near, far)
    /// should result in a recomputed projection matrix that differs from the
    /// previous matrix.
    ///
    /// **Validates: Requirements 6.2**
    #[test]
    fn prop_perspective_fov_change_updates_matrix(
        (fov1, near, far, aspect) in perspective_params_strategy(),
        fov_delta in 5.0f32..30.0f32
    ) {
        // Create initial camera
        let mut camera = Camera {
            projection: Projection::Perspective { fov: fov1, near, far },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Get initial projection matrix
        let matrix1 = camera.projection_matrix(aspect);

        // Change FOV
        let fov2 = fov1 + fov_delta;
        camera.projection = Projection::Perspective { fov: fov2, near, far };

        // Get updated projection matrix
        let matrix2 = camera.projection_matrix(aspect);

        // Verify matrices are different
        prop_assert_ne!(
            matrix1,
            matrix2,
            "Changing FOV from {} to {} should produce different projection matrix",
            fov1, fov2
        );

        // Verify both matrices are valid
        prop_assert!(
            is_valid_projection_matrix(&matrix1),
            "Initial matrix should be valid"
        );
        prop_assert!(
            is_valid_projection_matrix(&matrix2),
            "Updated matrix should be valid"
        );
    }

    #[test]
    fn prop_perspective_near_change_updates_matrix(
        (fov, near1, far, aspect) in perspective_params_strategy(),
        near_multiplier in 1.5f32..3.0f32
    ) {
        // Create initial camera
        let mut camera = Camera {
            projection: Projection::Perspective { fov, near: near1, far },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Get initial projection matrix
        let matrix1 = camera.projection_matrix(aspect);

        // Change near plane (ensure it stays less than far)
        let near2 = (near1 * near_multiplier).min(far * 0.5);
        prop_assume!(near2 < far);
        prop_assume!((near1 - near2).abs() > 0.01); // Ensure meaningful change

        camera.projection = Projection::Perspective { fov, near: near2, far };

        // Get updated projection matrix
        let matrix2 = camera.projection_matrix(aspect);

        // Verify matrices are different
        prop_assert_ne!(
            matrix1,
            matrix2,
            "Changing near plane from {} to {} should produce different projection matrix",
            near1, near2
        );

        // Verify both matrices are valid
        prop_assert!(
            is_valid_projection_matrix(&matrix1),
            "Initial matrix should be valid"
        );
        prop_assert!(
            is_valid_projection_matrix(&matrix2),
            "Updated matrix should be valid"
        );
    }

    #[test]
    fn prop_perspective_far_change_updates_matrix(
        (fov, near, far1, aspect) in perspective_params_strategy(),
        far_multiplier in 1.5f32..3.0f32
    ) {
        // Create initial camera
        let mut camera = Camera {
            projection: Projection::Perspective { fov, near, far: far1 },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Get initial projection matrix
        let matrix1 = camera.projection_matrix(aspect);

        // Change far plane (ensure it stays greater than near)
        let far2 = far1 * far_multiplier;
        prop_assume!(far2 > near);
        prop_assume!((far1 - far2).abs() > 1.0); // Ensure meaningful change

        camera.projection = Projection::Perspective { fov, near, far: far2 };

        // Get updated projection matrix
        let matrix2 = camera.projection_matrix(aspect);

        // Verify matrices are different
        prop_assert_ne!(
            matrix1,
            matrix2,
            "Changing far plane from {} to {} should produce different projection matrix",
            far1, far2
        );

        // Verify both matrices are valid
        prop_assert!(
            is_valid_projection_matrix(&matrix1),
            "Initial matrix should be valid"
        );
        prop_assert!(
            is_valid_projection_matrix(&matrix2),
            "Updated matrix should be valid"
        );
    }

    #[test]
    fn prop_orthographic_size_change_updates_matrix(
        (size1, near, far, aspect) in orthographic_params_strategy(),
        size_multiplier in 1.5f32..3.0f32
    ) {
        // Create initial camera
        let mut camera = Camera {
            projection: Projection::Orthographic { size: size1, near, far },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Get initial projection matrix
        let matrix1 = camera.projection_matrix(aspect);

        // Change size
        let size2 = size1 * size_multiplier;
        prop_assume!((size1 - size2).abs() > 0.1); // Ensure meaningful change

        camera.projection = Projection::Orthographic { size: size2, near, far };

        // Get updated projection matrix
        let matrix2 = camera.projection_matrix(aspect);

        // Verify matrices are different
        prop_assert_ne!(
            matrix1,
            matrix2,
            "Changing orthographic size from {} to {} should produce different projection matrix",
            size1, size2
        );

        // Verify both matrices are valid
        prop_assert!(
            is_valid_projection_matrix(&matrix1),
            "Initial matrix should be valid"
        );
        prop_assert!(
            is_valid_projection_matrix(&matrix2),
            "Updated matrix should be valid"
        );
    }

    #[test]
    fn prop_orthographic_near_change_updates_matrix(
        (size, near1, far, aspect) in orthographic_params_strategy(),
        near_multiplier in 1.5f32..3.0f32
    ) {
        // Create initial camera
        let mut camera = Camera {
            projection: Projection::Orthographic { size, near: near1, far },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Get initial projection matrix
        let matrix1 = camera.projection_matrix(aspect);

        // Change near plane (ensure it stays less than far)
        let near2 = (near1 * near_multiplier).min(far * 0.5);
        prop_assume!(near2 < far);
        prop_assume!((near1 - near2).abs() > 0.01); // Ensure meaningful change

        camera.projection = Projection::Orthographic { size, near: near2, far };

        // Get updated projection matrix
        let matrix2 = camera.projection_matrix(aspect);

        // Verify matrices are different
        prop_assert_ne!(
            matrix1,
            matrix2,
            "Changing orthographic near plane from {} to {} should produce different projection matrix",
            near1, near2
        );

        // Verify both matrices are valid
        prop_assert!(
            is_valid_projection_matrix(&matrix1),
            "Initial matrix should be valid"
        );
        prop_assert!(
            is_valid_projection_matrix(&matrix2),
            "Updated matrix should be valid"
        );
    }

    #[test]
    fn prop_orthographic_far_change_updates_matrix(
        (size, near, far1, aspect) in orthographic_params_strategy(),
        far_multiplier in 1.5f32..3.0f32
    ) {
        // Create initial camera
        let mut camera = Camera {
            projection: Projection::Orthographic { size, near, far: far1 },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Get initial projection matrix
        let matrix1 = camera.projection_matrix(aspect);

        // Change far plane (ensure it stays greater than near)
        let far2 = far1 * far_multiplier;
        prop_assume!(far2 > near);
        prop_assume!((far1 - far2).abs() > 1.0); // Ensure meaningful change

        camera.projection = Projection::Orthographic { size, near, far: far2 };

        // Get updated projection matrix
        let matrix2 = camera.projection_matrix(aspect);

        // Verify matrices are different
        prop_assert_ne!(
            matrix1,
            matrix2,
            "Changing orthographic far plane from {} to {} should produce different projection matrix",
            far1, far2
        );

        // Verify both matrices are valid
        prop_assert!(
            is_valid_projection_matrix(&matrix1),
            "Initial matrix should be valid"
        );
        prop_assert!(
            is_valid_projection_matrix(&matrix2),
            "Updated matrix should be valid"
        );
    }
}

// ============================================================================
// Property Test 12: Camera Aspect Ratio Update
// Validates: Requirements 6.5
// ============================================================================

/// Strategy for generating valid window dimensions
/// Typical range: 320x240 to 3840x2160 (4K)
fn window_dimensions_strategy() -> impl Strategy<Value = (u32, u32)> {
    (320u32..3840u32, 240u32..2160u32)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 12: Camera Aspect Ratio Update**
    ///
    /// For any camera, when the window is resized, the camera's aspect ratio
    /// should be updated to match the new window dimensions (width / height).
    /// This is verified by checking that projection matrices computed with
    /// different aspect ratios produce different results.
    ///
    /// **Validates: Requirements 6.5**
    #[test]
    fn prop_camera_aspect_ratio_updates_on_resize_perspective(
        (fov, near, far, _) in perspective_params_strategy(),
        (width1, height1) in window_dimensions_strategy(),
        (width2, height2) in window_dimensions_strategy()
    ) {
        // Ensure the two window sizes produce different aspect ratios
        let aspect1 = width1 as f32 / height1 as f32;
        let aspect2 = width2 as f32 / height2 as f32;
        prop_assume!((aspect1 - aspect2).abs() > 0.01); // Meaningful difference

        // Create a perspective camera
        let camera = Camera {
            projection: Projection::Perspective { fov, near, far },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Compute projection matrix with first aspect ratio (initial window size)
        let matrix1 = camera.projection_matrix(aspect1);

        // Simulate window resize by computing projection matrix with new aspect ratio
        let matrix2 = camera.projection_matrix(aspect2);

        // Verify that different aspect ratios produce different projection matrices
        prop_assert_ne!(
            matrix1,
            matrix2,
            "Window resize from {}x{} (aspect {}) to {}x{} (aspect {}) should produce different projection matrix",
            width1, height1, aspect1, width2, height2, aspect2
        );

        // Verify both matrices are valid
        prop_assert!(
            is_valid_projection_matrix(&matrix1),
            "Projection matrix with aspect ratio {} should be valid", aspect1
        );
        prop_assert!(
            is_valid_projection_matrix(&matrix2),
            "Projection matrix with aspect ratio {} should be valid", aspect2
        );

        // Verify that the aspect ratio is correctly reflected in the projection matrix
        // For perspective projection, the aspect ratio affects the horizontal FOV
        // We can verify this by checking that the x-scaling differs between matrices
        let x_scale1 = matrix1.col(0)[0];
        let x_scale2 = matrix2.col(0)[0];

        // The x-scale should be inversely proportional to aspect ratio
        // (wider aspect = smaller x-scale to fit more horizontally)
        if aspect1 > aspect2 {
            prop_assert!(
                x_scale1 < x_scale2,
                "Wider aspect ratio {} should have smaller x-scale than narrower aspect ratio {}.\n\
                 x_scale1: {}, x_scale2: {}",
                aspect1, aspect2, x_scale1, x_scale2
            );
        } else {
            prop_assert!(
                x_scale1 > x_scale2,
                "Narrower aspect ratio {} should have larger x-scale than wider aspect ratio {}.\n\
                 x_scale1: {}, x_scale2: {}",
                aspect1, aspect2, x_scale1, x_scale2
            );
        }
    }

    #[test]
    fn prop_camera_aspect_ratio_updates_on_resize_orthographic(
        (size, near, far, _) in orthographic_params_strategy(),
        (width1, height1) in window_dimensions_strategy(),
        (width2, height2) in window_dimensions_strategy()
    ) {
        // Ensure the two window sizes produce different aspect ratios
        let aspect1 = width1 as f32 / height1 as f32;
        let aspect2 = width2 as f32 / height2 as f32;
        prop_assume!((aspect1 - aspect2).abs() > 0.01); // Meaningful difference

        // Create an orthographic camera
        let camera = Camera {
            projection: Projection::Orthographic { size, near, far },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Compute projection matrix with first aspect ratio (initial window size)
        let matrix1 = camera.projection_matrix(aspect1);

        // Simulate window resize by computing projection matrix with new aspect ratio
        let matrix2 = camera.projection_matrix(aspect2);

        // Verify that different aspect ratios produce different projection matrices
        prop_assert_ne!(
            matrix1,
            matrix2,
            "Window resize from {}x{} (aspect {}) to {}x{} (aspect {}) should produce different projection matrix",
            width1, height1, aspect1, width2, height2, aspect2
        );

        // Verify both matrices are valid
        prop_assert!(
            is_valid_projection_matrix(&matrix1),
            "Projection matrix with aspect ratio {} should be valid", aspect1
        );
        prop_assert!(
            is_valid_projection_matrix(&matrix2),
            "Projection matrix with aspect ratio {} should be valid", aspect2
        );

        // Verify that the aspect ratio is correctly reflected in the projection matrix
        // For orthographic projection, the aspect ratio affects the horizontal bounds
        // We can verify this by checking that the x-scaling differs between matrices
        let x_scale1 = matrix1.col(0)[0];
        let x_scale2 = matrix2.col(0)[0];

        // The x-scale should be inversely proportional to aspect ratio
        // (wider aspect = smaller x-scale to fit more horizontally)
        if aspect1 > aspect2 {
            prop_assert!(
                x_scale1 < x_scale2,
                "Wider aspect ratio {} should have smaller x-scale than narrower aspect ratio {}.\n\
                 x_scale1: {}, x_scale2: {}",
                aspect1, aspect2, x_scale1, x_scale2
            );
        } else {
            prop_assert!(
                x_scale1 > x_scale2,
                "Narrower aspect ratio {} should have larger x-scale than wider aspect ratio {}.\n\
                 x_scale1: {}, x_scale2: {}",
                aspect1, aspect2, x_scale1, x_scale2
            );
        }
    }

    /// Test that aspect ratio changes are proportional to window dimension changes
    #[test]
    fn prop_aspect_ratio_proportional_to_dimensions(
        (fov, near, far, _) in perspective_params_strategy(),
        (width, height) in window_dimensions_strategy(),
        width_multiplier in 1.2f32..2.0f32
    ) {
        let camera = Camera {
            projection: Projection::Perspective { fov, near, far },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Original aspect ratio
        let aspect1 = width as f32 / height as f32;
        let matrix1 = camera.projection_matrix(aspect1);

        // Resize width only (height stays the same)
        let new_width = (width as f32 * width_multiplier) as u32;
        let aspect2 = new_width as f32 / height as f32;
        let matrix2 = camera.projection_matrix(aspect2);

        // Verify matrices are different
        prop_assert_ne!(
            matrix1,
            matrix2,
            "Changing width from {} to {} should produce different projection matrix",
            width, new_width
        );

        // Verify the aspect ratio change is reflected in the x-scale
        let x_scale1 = matrix1.col(0)[0];
        let x_scale2 = matrix2.col(0)[0];

        // Since width increased, aspect ratio increased, so x-scale should decrease
        prop_assert!(
            x_scale2 < x_scale1,
            "Increasing width should decrease x-scale.\n\
             Original: {}x{} (aspect {}), x_scale: {}\n\
             New: {}x{} (aspect {}), x_scale: {}",
            width, height, aspect1, x_scale1,
            new_width, height, aspect2, x_scale2
        );
    }
}

// ============================================================================
// Additional unit tests for edge cases
// ============================================================================

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_extreme_aspect_ratios() {
        let camera = Camera {
            projection: Projection::Perspective {
                fov: 60.0,
                near: 0.1,
                far: 1000.0,
            },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Test very wide aspect ratio
        let wide_matrix = camera.projection_matrix(4.0);
        assert!(is_valid_projection_matrix(&wide_matrix));

        // Test very narrow aspect ratio
        let narrow_matrix = camera.projection_matrix(0.25);
        assert!(is_valid_projection_matrix(&narrow_matrix));
    }

    #[test]
    fn test_extreme_fov() {
        // Very narrow FOV
        let narrow_fov_camera = Camera {
            projection: Projection::Perspective {
                fov: 10.0,
                near: 0.1,
                far: 1000.0,
            },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };
        let matrix = narrow_fov_camera.projection_matrix(1.6);
        assert!(is_valid_projection_matrix(&matrix));

        // Very wide FOV
        let wide_fov_camera = Camera {
            projection: Projection::Perspective {
                fov: 170.0,
                near: 0.1,
                far: 1000.0,
            },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };
        let matrix = wide_fov_camera.projection_matrix(1.6);
        assert!(is_valid_projection_matrix(&matrix));
    }

    #[test]
    fn test_near_far_plane_relationship() {
        // Near and far very close together
        let camera = Camera {
            projection: Projection::Perspective {
                fov: 60.0,
                near: 1.0,
                far: 1.1,
            },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };
        let matrix = camera.projection_matrix(1.6);
        assert!(is_valid_projection_matrix(&matrix));

        // Near and far very far apart
        let camera = Camera {
            projection: Projection::Perspective {
                fov: 60.0,
                near: 0.01,
                far: 100000.0,
            },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };
        let matrix = camera.projection_matrix(1.6);
        assert!(is_valid_projection_matrix(&matrix));
    }

    #[test]
    fn test_aspect_ratio_update_on_window_resize() {
        let camera = Camera {
            projection: Projection::Perspective {
                fov: 60.0,
                near: 0.1,
                far: 1000.0,
            },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Initial window size: 800x600 (aspect 1.333)
        let aspect1 = 800.0 / 600.0;
        let matrix1 = camera.projection_matrix(aspect1);

        // Resize to 1920x1080 (aspect 1.778)
        let aspect2 = 1920.0 / 1080.0;
        let matrix2 = camera.projection_matrix(aspect2);

        // Matrices should be different
        assert_ne!(matrix1, matrix2);

        // Both should be valid
        assert!(is_valid_projection_matrix(&matrix1));
        assert!(is_valid_projection_matrix(&matrix2));
    }

    #[test]
    fn test_square_window_aspect_ratio() {
        let camera = Camera {
            projection: Projection::Perspective {
                fov: 60.0,
                near: 0.1,
                far: 1000.0,
            },
            clear_color: luminara_math::Color::BLACK,
            is_active: true,
        };

        // Square window (aspect 1.0)
        let matrix = camera.projection_matrix(1.0);
        assert!(is_valid_projection_matrix(&matrix));
    }
}
