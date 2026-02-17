# Visual Regression Testing Guide

## Overview

Visual regression testing ensures that rendering output remains consistent across code changes. This system captures reference images (golden images) and compares them against current rendering output to detect unintended visual changes.

## Test Scenes

The visual regression test suite includes the following scenes:

1. **cornell_box**: Basic lighting and material test with colored walls
2. **pbr_materials**: PBR material properties testing (metallic and roughness gradients)
3. **shadow_maps**: Shadow rendering accuracy with shadow patterns
4. **sprite_rendering**: 2D sprite batching and transparency rendering
5. **debug_visualization**: Debug wireframes, gizmos, and grid rendering
6. **pbr_spheres**: Legacy PBR test (maintained for compatibility)

## Running Tests

### Standard Test Run

```bash
cargo test --test visual_regression_test
```

This will:
- Render each test scene
- Compare against golden images in `tests/golden/`
- Fail if differences exceed the threshold (1% by default)
- Save diff images to `tests/diffs/` when regressions are detected

### First Run (Creating Golden Images)

On the first run, if golden images don't exist, they will be automatically created:

```bash
cargo test --test visual_regression_test
```

The test will create `tests/golden/` directory and save reference images.

## Updating Reference Images

When you make intentional rendering changes (e.g., improving lighting, fixing bugs, adding features), you need to update the golden images:

### Step 1: Review Changes

First, run the tests to see what changed:

```bash
cargo test --test visual_regression_test
```

If tests fail, check the diff images in `tests/diffs/` to verify the changes are correct.

### Step 2: Update Golden Images

If the changes are intentional and correct:

```bash
UPDATE_GOLDEN_IMAGES=1 cargo test --test visual_regression_test
```

On Windows PowerShell:
```powershell
$env:UPDATE_GOLDEN_IMAGES=1; cargo test --test visual_regression_test
```

### Step 3: Commit Updated Images

```bash
git add crates/luminara_render/tests/golden/*.png
git commit -m "Update visual regression golden images for [reason]"
```

## Understanding Test Results

### Successful Test

```
test visual_regression_integration_tests::test_cornell_box_regression ... ok
```

The rendered output matches the golden image within the tolerance threshold.

### Failed Test

```
test visual_regression_integration_tests::test_cornell_box_regression ... FAILED

Visual regression detected in 'cornell_box' scene!
Difference: 0.0523 (threshold: 0.0100)
Diff image saved to: tests/diffs/cornell_box_diff.png
To update golden image, run: UPDATE_GOLDEN_IMAGES=1 cargo test
```

This indicates:
- The scene differs from the golden image by 5.23%
- The threshold is 1%
- A diff image has been saved showing the differences in red

### Inspecting Diff Images

Diff images highlight differences in red:
- **Black pixels**: No difference
- **Dark red pixels**: Minor differences
- **Bright red pixels**: Significant differences

## Tolerance Levels

The default tolerance is **1%** (0.01), which allows for minor floating-point precision differences while catching significant visual changes.

You can adjust tolerance per test if needed:

```rust
run_visual_regression_test("my_scene", 0.02); // 2% tolerance
```

### Recommended Tolerances

- **0.0**: Pixel-perfect comparison (use for deterministic rendering)
- **0.01**: Standard tolerance (recommended for most tests)
- **0.02-0.05**: Loose tolerance (for scenes with anti-aliasing or temporal effects)

## Adding New Test Scenes

### Step 1: Add Scene Rendering

Edit `visual_regression_test.rs` and add your scene to `render_test_scene()`:

```rust
fn render_test_scene(scene_name: &str) -> FrameBuffer {
    let mut framebuffer = FrameBuffer::new(800, 600);
    
    match scene_name {
        // ... existing scenes ...
        "my_new_scene" => {
            // Render your scene here
            for y in 0..600 {
                for x in 0..800 {
                    let color = compute_pixel_color(x, y);
                    framebuffer.set_pixel(x, y, color);
                }
            }
        }
        _ => {}
    }
    
    framebuffer
}
```

### Step 2: Add Test Function

```rust
#[test]
fn test_my_new_scene_regression() {
    run_visual_regression_test("my_new_scene", 0.01);
}
```

### Step 3: Generate Golden Image

```bash
cargo test --test visual_regression_test -- test_my_new_scene_regression
```

This will create `tests/golden/my_new_scene.png`.

### Step 4: Commit

```bash
git add crates/luminara_render/tests/visual_regression_test.rs
git add crates/luminara_render/tests/golden/my_new_scene.png
git commit -m "Add visual regression test for my_new_scene"
```

## Integration with Real Rendering

Currently, the tests use simulated rendering. To integrate with actual GPU rendering:

1. **Setup headless rendering context** (using wgpu with no window)
2. **Render to texture** instead of screen
3. **Read back pixels** from GPU to CPU
4. **Create FrameBuffer** from pixel data
5. **Compare** using existing comparison logic

Example integration:

```rust
fn render_test_scene_gpu(scene_name: &str) -> FrameBuffer {
    // Create headless wgpu instance
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    })).unwrap();
    
    // Create device and queue
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor::default(),
        None,
    )).unwrap();
    
    // Create render target texture
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d { width: 800, height: 600, depth_or_array_layers: 1 },
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        // ... other fields
    });
    
    // Render scene to texture
    // ... rendering code ...
    
    // Read back pixels
    let pixels = read_texture_pixels(&device, &queue, &texture);
    
    FrameBuffer {
        width: 800,
        height: 600,
        pixels,
    }
}
```

## Continuous Integration

Add to your CI pipeline:

```yaml
- name: Run visual regression tests
  run: cargo test --test visual_regression_test
  
- name: Upload diff images on failure
  if: failure()
  uses: actions/upload-artifact@v3
  with:
    name: visual-regression-diffs
    path: crates/luminara_render/tests/diffs/
```

## Best Practices

1. **Keep golden images in version control** - They document expected rendering output
2. **Review diff images carefully** - Don't blindly update golden images
3. **Use descriptive scene names** - Makes it easy to identify what's being tested
4. **Test different rendering features** - Lighting, shadows, materials, transparency, etc.
5. **Run tests before committing** - Catch regressions early
6. **Document intentional changes** - Explain why golden images were updated in commit messages

## Troubleshooting

### Tests fail on different machines

This can happen due to:
- **GPU differences**: Different GPUs may produce slightly different results
- **Driver differences**: Different driver versions can affect rendering
- **Floating-point precision**: Different CPUs may have different FP behavior

**Solution**: Increase tolerance slightly (e.g., 0.02 instead of 0.01) or use deterministic rendering paths.

### Golden images are too large

PNG images can be large for high-resolution tests.

**Solution**: 
- Use lower resolution for tests (800x600 is usually sufficient)
- Enable PNG compression
- Consider using JPEG for non-critical tests (with quality=95)

### Tests are slow

Visual regression tests can be slow if rendering is complex.

**Solution**:
- Use simpler test scenes
- Run visual tests separately from unit tests
- Use parallel test execution: `cargo test --test visual_regression_test -- --test-threads=4`

## Future Enhancements

Planned improvements:

1. **GPU-accelerated comparison** - Use compute shaders for faster comparison
2. **Perceptual diff metrics** - Use SSIM or other perceptual metrics instead of pixel-by-pixel
3. **Automatic tolerance calculation** - Suggest appropriate tolerance based on scene complexity
4. **Web-based diff viewer** - Interactive HTML report for reviewing differences
5. **Temporal stability tests** - Ensure rendering is stable across multiple frames
