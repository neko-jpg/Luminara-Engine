//! # Overlay Text Renderer
//!
//! Minimal 2D overlay for rendering text and colored rectangles on top of the
//! 3D scene. Uses an embedded 8×8 bitmap font (ASCII 32–126) and a single
//! draw call per frame.

use luminara_core::shared_types::Resource;
use luminara_diagnostic::profiler::OverlayRendererInterface;
use wgpu::util::DeviceExt;

// ============================================================================
// Public types
// ============================================================================

/// Draw command queued for the current frame.
#[derive(Debug, Clone)]
pub enum OverlayCommand {
    /// Filled rectangle in screen-pixel coordinates.
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
    },
    /// Rectangle with vertical gradient (top_color -> bottom_color).
    GradientRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        top_color: [f32; 4],
        bottom_color: [f32; 4],
    },
    /// Text string rendered with the built-in 8×8 font.
    Text {
        x: f32,
        y: f32,
        text: String,
        color: [f32; 4],
        scale: f32,
    },
}

/// GPU-backed overlay renderer.  Insert as a resource; `render_system` will
/// drain the command queue each frame.
pub struct OverlayRenderer {
    /// Draw commands to be rendered this frame (cleared after rendering).
    pub commands: Vec<OverlayCommand>,
    // -- lazy GPU state --
    initialized: bool,
    pipeline: Option<wgpu::RenderPipeline>,
    font_bind_group: Option<wgpu::BindGroup>,
}

impl Resource for OverlayRenderer {}

impl Default for OverlayRenderer {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            initialized: false,
            pipeline: None,
            font_bind_group: None,
        }
    }
}

impl OverlayRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue a filled rectangle.
    pub fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        self.commands
            .push(OverlayCommand::Rect { x, y, w, h, color });
    }

    /// Queue a rectangle with a vertical gradient (top to bottom).
    pub fn draw_gradient_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        top_color: [f32; 4],
        bottom_color: [f32; 4],
    ) {
        self.commands.push(OverlayCommand::GradientRect {
            x,
            y,
            w,
            h,
            top_color,
            bottom_color,
        });
    }

    /// Queue a text string at the given pixel position.
    /// `scale` multiplies the base 8×8 character size (1.0 = 8 px, 2.0 = 16 px, …).
    /// Includes a 1-pixel drop shadow for readability.
    pub fn draw_text(&mut self, x: f32, y: f32, text: &str, color: [f32; 4], scale: f32) {
        // Shadow offset (1px at scale 1.0)
        let shadow_offset = 1.0 * scale;
        self.commands.push(OverlayCommand::Text {
            x: x + shadow_offset,
            y: y + shadow_offset,
            text: text.to_string(),
            color: [0.0, 0.0, 0.0, color[3]], // Shadow alpha matches text alpha
            scale,
        });

        self.commands.push(OverlayCommand::Text {
            x,
            y,
            text: text.to_string(),
            color,
            scale,
        });
    }

    /// Queue text with a full outline (shadow in 4 cardinal directions + diagonals).
    /// This provides superior readability against any background.
    pub fn draw_text_outlined(
        &mut self,
        x: f32,
        y: f32,
        text: &str,
        color: [f32; 4],
        outline_color: [f32; 4],
        scale: f32,
    ) {
        let d = 1.0 * scale; // outline thickness
                             // 8-direction outline
        let offsets: [(f32, f32); 8] = [
            (-d, -d),
            (0.0, -d),
            (d, -d),
            (-d, 0.0),
            (d, 0.0),
            (-d, d),
            (0.0, d),
            (d, d),
        ];
        for (ox, oy) in &offsets {
            self.commands.push(OverlayCommand::Text {
                x: x + ox,
                y: y + oy,
                text: text.to_string(),
                color: outline_color,
                scale,
            });
        }
        // Foreground text on top
        self.commands.push(OverlayCommand::Text {
            x,
            y,
            text: text.to_string(),
            color,
            scale,
        });
    }

    /// Clear all queued commands without rendering.
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Render all queued commands into the given render target, then clear.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        surface_format: wgpu::TextureFormat,
        screen_width: u32,
        screen_height: u32,
    ) {
        if self.commands.is_empty() {
            return;
        }

        self.ensure_initialized(device, queue, surface_format);

        let vertices = self.generate_vertices(screen_width, screen_height);
        if vertices.is_empty() {
            self.commands.clear();
            return;
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Overlay VB"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Overlay Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // draw on top
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            pass.set_pipeline(self.pipeline.as_ref().unwrap());
            pass.set_bind_group(0, self.font_bind_group.as_ref().unwrap(), &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            // Disable backface culling implicitly by topology or ensure winding is correct.
            // Vertices are generated as CCW (TL->BL->TR, TR->BL->BR) which is standard.
            pass.draw(0..vertices.len() as u32, 0..1);
        }

        self.commands.clear();
    }

    // --------------------------------------------------------------------
    // Internals
    // --------------------------------------------------------------------

    fn ensure_initialized(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) {
        if self.initialized {
            return;
        }

        // ── Font texture (128 × 48  R8Unorm) ──────────────────────────
        // Use sRGB format if surface is sRGB, but for font mask R8Unorm is fine as alpha.
        // However, if we blend, we need to be careful about color space.
        // The shader uses `color` uniform which is linear or sRGB?
        // Ideally UI colors are sRGB and we should convert to Linear in shader if framebuffer is sRGB-aware.
        // Assuming standard wgpu handling where shader output is written to sRGB view.

        let font_pixels = build_font_texture_data();
        let font_texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("Overlay Font"),
                size: wgpu::Extent3d {
                    width: FONT_TEX_W as u32,
                    height: FONT_TEX_H as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &font_pixels,
        );
        let font_view = font_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let font_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Overlay Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // ── Bind group layout + bind group ─────────────────────────────
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Overlay BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Overlay BG"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&font_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&font_sampler),
                },
            ],
        });

        // ── Shader + pipeline ──────────────────────────────────────────
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Overlay Shader"),
            source: wgpu::ShaderSource::Wgsl(OVERLAY_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Overlay PL"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Overlay Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[OverlayVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        self.pipeline = Some(pipeline);
        self.font_bind_group = Some(bind_group);
        self.initialized = true;
        log::info!("Overlay renderer initialised (128×48 font atlas)");
    }

    fn generate_vertices(&self, sw: u32, sh: u32) -> Vec<OverlayVertex> {
        let mut verts = Vec::with_capacity(self.commands.len() * 24);
        let sw = sw as f32;
        let sh = sh as f32;

        for cmd in &self.commands {
            match cmd {
                OverlayCommand::Rect { x, y, w, h, color } => {
                    let (x0, y0) = px_to_ndc(*x, *y, sw, sh);
                    let (x1, y1) = px_to_ndc(*x + *w, *y + *h, sw, sh);
                    let (u0, v0, u1, v1) = solid_uv();
                    push_quad(&mut verts, x0, y0, x1, y1, u0, v0, u1, v1, *color);
                }
                OverlayCommand::GradientRect {
                    x,
                    y,
                    w,
                    h,
                    top_color,
                    bottom_color,
                } => {
                    let (x0, y0) = px_to_ndc(*x, *y, sw, sh);
                    let (x1, y1) = px_to_ndc(*x + *w, *y + *h, sw, sh);
                    let (u0, v0, u1, v1) = solid_uv();
                    push_gradient_quad(
                        &mut verts,
                        x0,
                        y0,
                        x1,
                        y1,
                        u0,
                        v0,
                        u1,
                        v1,
                        *top_color,
                        *bottom_color,
                    );
                }
                OverlayCommand::Text {
                    x,
                    y,
                    text,
                    color,
                    scale,
                } => {
                    let cw = 8.0 * scale;
                    let ch = 8.0 * scale;
                    let mut cx = *x;
                    for c in text.chars() {
                        if c == ' ' {
                            cx += cw;
                            continue;
                        }
                        let (u0, v0, u1, v1) = char_uv(c);
                        let (x0, y0) = px_to_ndc(cx, *y, sw, sh);
                        let (x1, y1) = px_to_ndc(cx + cw, *y + ch, sw, sh);
                        push_quad(&mut verts, x0, y0, x1, y1, u0, v0, u1, v1, *color);
                        cx += cw;
                    }
                }
            }
        }
        verts
    }
}

// ============================================================================
// Vertex type
// ============================================================================

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct OverlayVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

impl OverlayVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<OverlayVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Convert pixel coordinates (origin = top-left) to NDC (−1…+1, Y-up).
#[inline]
fn px_to_ndc(px: f32, py: f32, sw: f32, sh: f32) -> (f32, f32) {
    let x = (px / sw) * 2.0 - 1.0;
    let y = 1.0 - (py / sh) * 2.0;
    (x, y)
}

/// Push two triangles forming a quad.
fn push_quad(
    verts: &mut Vec<OverlayVertex>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
    color: [f32; 4],
) {
    // tri 1: TL, BL, TR (CCW)
    verts.push(OverlayVertex {
        pos: [x0, y0],
        uv: [u0, v0],
        color,
    });
    verts.push(OverlayVertex {
        pos: [x0, y1],
        uv: [u0, v1],
        color,
    });
    verts.push(OverlayVertex {
        pos: [x1, y0],
        uv: [u1, v0],
        color,
    });
    // tri 2: TR, BL, BR (CCW)
    verts.push(OverlayVertex {
        pos: [x1, y0],
        uv: [u1, v0],
        color,
    });
    verts.push(OverlayVertex {
        pos: [x0, y1],
        uv: [u0, v1],
        color,
    });
    verts.push(OverlayVertex {
        pos: [x1, y1],
        uv: [u1, v1],
        color,
    });
}

/// Push two triangles forming a quad with vertical gradient (top_color -> bottom_color).
fn push_gradient_quad(
    verts: &mut Vec<OverlayVertex>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
    top_color: [f32; 4],
    bottom_color: [f32; 4],
) {
    // y0 is top (higher NDC), y1 is bottom (lower NDC)
    // tri 1: TL, BL, TR (CCW)
    verts.push(OverlayVertex {
        pos: [x0, y0],
        uv: [u0, v0],
        color: top_color,
    });
    verts.push(OverlayVertex {
        pos: [x0, y1],
        uv: [u0, v1],
        color: bottom_color,
    });
    verts.push(OverlayVertex {
        pos: [x1, y0],
        uv: [u1, v0],
        color: top_color,
    });
    // tri 2: TR, BL, BR (CCW)
    verts.push(OverlayVertex {
        pos: [x1, y0],
        uv: [u1, v0],
        color: top_color,
    });
    verts.push(OverlayVertex {
        pos: [x0, y1],
        uv: [u0, v1],
        color: bottom_color,
    });
    verts.push(OverlayVertex {
        pos: [x1, y1],
        uv: [u1, v1],
        color: bottom_color,
    });
}

// ============================================================================
// Font atlas helpers
// ============================================================================

/// Texture dimensions (16 chars wide × 6 rows, each char 8×8).
const FONT_TEX_W: usize = 128;
const FONT_TEX_H: usize = 48;
const CHARS_PER_ROW: usize = 16;

/// UV rectangle for the solid-fill block (font index 95 = last slot).
fn solid_uv() -> (f32, f32, f32, f32) {
    char_uv_by_index(95)
}

/// UV rectangle for a printable ASCII character.
fn char_uv(c: char) -> (f32, f32, f32, f32) {
    let code = c as u32;
    let idx = if (32..=126).contains(&code) {
        (code - 32) as usize
    } else {
        0 // fallback to space slot (which is empty)
    };
    char_uv_by_index(idx)
}

fn char_uv_by_index(idx: usize) -> (f32, f32, f32, f32) {
    let col = idx % CHARS_PER_ROW;
    let row = idx / CHARS_PER_ROW;
    let u0 = (col as f32 * 8.0) / FONT_TEX_W as f32;
    let v0 = (row as f32 * 8.0) / FONT_TEX_H as f32;
    let u1 = ((col + 1) as f32 * 8.0) / FONT_TEX_W as f32;
    let v1 = ((row + 1) as f32 * 8.0) / FONT_TEX_H as f32;
    (u0, v0, u1, v1)
}

/// Expand the compact bit-per-pixel font into an R8 byte array for wgpu.
fn build_font_texture_data() -> Vec<u8> {
    let mut data = vec![0u8; FONT_TEX_W * FONT_TEX_H];
    for (idx, glyph) in FONT_DATA.iter().enumerate() {
        let col = idx % CHARS_PER_ROW;
        let row = idx / CHARS_PER_ROW;
        for y in 0..8 {
            let byte = glyph[y];
            for x in 0..8 {
                let pixel = if byte & (0x80 >> x) != 0 { 255u8 } else { 0u8 };
                let tx = col * 8 + x;
                let ty = row * 8 + y;
                data[ty * FONT_TEX_W + tx] = pixel;
            }
        }
    }
    data
}

// ============================================================================
// Overlay WGSL shader
// ============================================================================

const OVERLAY_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv:    vec2<f32>,
    @location(1) color: vec4<f32>,
};

@group(0) @binding(0) var font_tex:     texture_2d<f32>;
@group(0) @binding(1) var font_sampler: sampler;

@vertex
fn vs_main(
    @location(0) pos:   vec2<f32>,
    @location(1) uv:    vec2<f32>,
    @location(2) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv       = uv;
    out.color    = color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(font_tex, font_sampler, in.uv).r;
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
"#;

// ============================================================================
// 8×8 bitmap font — ASCII 32–126 + solid block (index 95)
// Each glyph is 8 rows; each row is a byte (MSB = leftmost pixel).
// ============================================================================

#[rustfmt::skip]
const FONT_DATA: [[u8; 8]; 96] = [
    // 32 ' '
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    // 33 '!'
    [0x18, 0x3C, 0x3C, 0x18, 0x18, 0x00, 0x18, 0x00],
    // 34 '"'
    [0x36, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    // 35 '#'
    [0x36, 0x36, 0x7F, 0x36, 0x7F, 0x36, 0x36, 0x00],
    // 36 '$'
    [0x0C, 0x3E, 0x03, 0x1E, 0x30, 0x1F, 0x0C, 0x00],
    // 37 '%'
    [0x00, 0x63, 0x33, 0x18, 0x0C, 0x66, 0x63, 0x00],
    // 38 '&'
    [0x1C, 0x36, 0x1C, 0x6E, 0x3B, 0x33, 0x6E, 0x00],
    // 39 '\''
    [0x06, 0x06, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00],
    // 40 '('
    [0x18, 0x0C, 0x06, 0x06, 0x06, 0x0C, 0x18, 0x00],
    // 41 ')'
    [0x06, 0x0C, 0x18, 0x18, 0x18, 0x0C, 0x06, 0x00],
    // 42 '*'
    [0x00, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0x00, 0x00],
    // 43 '+'
    [0x00, 0x0C, 0x0C, 0x3F, 0x0C, 0x0C, 0x00, 0x00],
    // 44 ','
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C, 0x06],
    // 45 '-'
    [0x00, 0x00, 0x00, 0x3F, 0x00, 0x00, 0x00, 0x00],
    // 46 '.'
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C, 0x00],
    // 47 '/'
    [0x60, 0x30, 0x18, 0x0C, 0x06, 0x03, 0x01, 0x00],
    // 48 '0'
    [0x3E, 0x63, 0x73, 0x7B, 0x6F, 0x67, 0x3E, 0x00],
    // 49 '1'
    [0x0C, 0x0E, 0x0C, 0x0C, 0x0C, 0x0C, 0x3F, 0x00],
    // 50 '2'
    [0x1E, 0x33, 0x30, 0x1C, 0x06, 0x33, 0x3F, 0x00],
    // 51 '3'
    [0x1E, 0x33, 0x30, 0x1C, 0x30, 0x33, 0x1E, 0x00],
    // 52 '4'
    [0x38, 0x3C, 0x36, 0x33, 0x7F, 0x30, 0x78, 0x00],
    // 53 '5'
    [0x3F, 0x03, 0x1F, 0x30, 0x30, 0x33, 0x1E, 0x00],
    // 54 '6'
    [0x1C, 0x06, 0x03, 0x1F, 0x33, 0x33, 0x1E, 0x00],
    // 55 '7'
    [0x3F, 0x33, 0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x00],
    // 56 '8'
    [0x1E, 0x33, 0x33, 0x1E, 0x33, 0x33, 0x1E, 0x00],
    // 57 '9'
    [0x1E, 0x33, 0x33, 0x3E, 0x30, 0x18, 0x0E, 0x00],
    // 58 ':'
    [0x00, 0x0C, 0x0C, 0x00, 0x00, 0x0C, 0x0C, 0x00],
    // 59 ';'
    [0x00, 0x0C, 0x0C, 0x00, 0x00, 0x0C, 0x0C, 0x06],
    // 60 '<'
    [0x18, 0x0C, 0x06, 0x03, 0x06, 0x0C, 0x18, 0x00],
    // 61 '='
    [0x00, 0x00, 0x3F, 0x00, 0x00, 0x3F, 0x00, 0x00],
    // 62 '>'
    [0x06, 0x0C, 0x18, 0x30, 0x18, 0x0C, 0x06, 0x00],
    // 63 '?'
    [0x1E, 0x33, 0x30, 0x18, 0x0C, 0x00, 0x0C, 0x00],
    // 64 '@'
    [0x3E, 0x63, 0x7B, 0x7B, 0x7B, 0x03, 0x1E, 0x00],
    // 65 'A'
    [0x0C, 0x1E, 0x33, 0x33, 0x3F, 0x33, 0x33, 0x00],
    // 66 'B'
    [0x3F, 0x66, 0x66, 0x3E, 0x66, 0x66, 0x3F, 0x00],
    // 67 'C'
    [0x3C, 0x66, 0x03, 0x03, 0x03, 0x66, 0x3C, 0x00],
    // 68 'D'
    [0x1F, 0x36, 0x66, 0x66, 0x66, 0x36, 0x1F, 0x00],
    // 69 'E'
    [0x7F, 0x46, 0x16, 0x1E, 0x16, 0x46, 0x7F, 0x00],
    // 70 'F'
    [0x7F, 0x46, 0x16, 0x1E, 0x16, 0x06, 0x0F, 0x00],
    // 71 'G'
    [0x3C, 0x66, 0x03, 0x03, 0x73, 0x66, 0x7C, 0x00],
    // 72 'H'
    [0x33, 0x33, 0x33, 0x3F, 0x33, 0x33, 0x33, 0x00],
    // 73 'I'
    [0x1E, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x1E, 0x00],
    // 74 'J'
    [0x78, 0x30, 0x30, 0x30, 0x33, 0x33, 0x1E, 0x00],
    // 75 'K'
    [0x67, 0x66, 0x36, 0x1E, 0x36, 0x66, 0x67, 0x00],
    // 76 'L'
    [0x0F, 0x06, 0x06, 0x06, 0x46, 0x66, 0x7F, 0x00],
    // 77 'M'
    [0x63, 0x77, 0x7F, 0x7F, 0x6B, 0x63, 0x63, 0x00],
    // 78 'N'
    [0x63, 0x67, 0x6F, 0x7B, 0x73, 0x63, 0x63, 0x00],
    // 79 'O'
    [0x1C, 0x36, 0x63, 0x63, 0x63, 0x36, 0x1C, 0x00],
    // 80 'P'
    [0x3F, 0x66, 0x66, 0x3E, 0x06, 0x06, 0x0F, 0x00],
    // 81 'Q'
    [0x1E, 0x33, 0x33, 0x33, 0x3B, 0x1E, 0x38, 0x00],
    // 82 'R'
    [0x3F, 0x66, 0x66, 0x3E, 0x36, 0x66, 0x67, 0x00],
    // 83 'S'
    [0x1E, 0x33, 0x07, 0x0E, 0x38, 0x33, 0x1E, 0x00],
    // 84 'T'
    [0x3F, 0x2D, 0x0C, 0x0C, 0x0C, 0x0C, 0x1E, 0x00],
    // 85 'U'
    [0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x3F, 0x00],
    // 86 'V'
    [0x33, 0x33, 0x33, 0x33, 0x33, 0x1E, 0x0C, 0x00],
    // 87 'W'
    [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00],
    // 88 'X'
    [0x63, 0x63, 0x36, 0x1C, 0x1C, 0x36, 0x63, 0x00],
    // 89 'Y'
    [0x33, 0x33, 0x33, 0x1E, 0x0C, 0x0C, 0x1E, 0x00],
    // 90 'Z'
    [0x7F, 0x63, 0x31, 0x18, 0x4C, 0x66, 0x7F, 0x00],
    // 91 '['
    [0x1E, 0x06, 0x06, 0x06, 0x06, 0x06, 0x1E, 0x00],
    // 92 '\\'
    [0x03, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x40, 0x00],
    // 93 ']'
    [0x1E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x1E, 0x00],
    // 94 '^'
    [0x08, 0x1C, 0x36, 0x63, 0x00, 0x00, 0x00, 0x00],
    // 95 '_'
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF],
    // 96 '`'
    [0x0C, 0x0C, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00],
    // 97 'a'
    [0x00, 0x00, 0x1E, 0x30, 0x3E, 0x33, 0x6E, 0x00],
    // 98 'b'
    [0x07, 0x06, 0x06, 0x3E, 0x66, 0x66, 0x3B, 0x00],
    // 99 'c'
    [0x00, 0x00, 0x1E, 0x33, 0x03, 0x33, 0x1E, 0x00],
    // 100 'd'
    [0x38, 0x30, 0x30, 0x3E, 0x33, 0x33, 0x6E, 0x00],
    // 101 'e'
    [0x00, 0x00, 0x1E, 0x33, 0x3F, 0x03, 0x1E, 0x00],
    // 102 'f'
    [0x1C, 0x36, 0x06, 0x0F, 0x06, 0x06, 0x0F, 0x00],
    // 103 'g'
    [0x00, 0x00, 0x6E, 0x33, 0x33, 0x3E, 0x30, 0x1F],
    // 104 'h'
    [0x07, 0x06, 0x36, 0x6E, 0x66, 0x66, 0x67, 0x00],
    // 105 'i'
    [0x0C, 0x00, 0x0E, 0x0C, 0x0C, 0x0C, 0x1E, 0x00],
    // 106 'j'
    [0x30, 0x00, 0x30, 0x30, 0x30, 0x33, 0x33, 0x1E],
    // 107 'k'
    [0x07, 0x06, 0x66, 0x36, 0x1E, 0x36, 0x67, 0x00],
    // 108 'l'
    [0x0E, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x1E, 0x00],
    // 109 'm'
    [0x00, 0x00, 0x33, 0x7F, 0x7F, 0x6B, 0x63, 0x00],
    // 110 'n'
    [0x00, 0x00, 0x1F, 0x33, 0x33, 0x33, 0x33, 0x00],
    // 111 'o'
    [0x00, 0x00, 0x1E, 0x33, 0x33, 0x33, 0x1E, 0x00],
    // 112 'p'
    [0x00, 0x00, 0x3B, 0x66, 0x66, 0x3E, 0x06, 0x0F],
    // 113 'q'
    [0x00, 0x00, 0x6E, 0x33, 0x33, 0x3E, 0x30, 0x78],
    // 114 'r'
    [0x00, 0x00, 0x3B, 0x6E, 0x66, 0x06, 0x0F, 0x00],
    // 115 's'
    [0x00, 0x00, 0x3E, 0x03, 0x1E, 0x30, 0x1F, 0x00],
    // 116 't'
    [0x08, 0x0C, 0x3E, 0x0C, 0x0C, 0x2C, 0x18, 0x00],
    // 117 'u'
    [0x00, 0x00, 0x33, 0x33, 0x33, 0x33, 0x6E, 0x00],
    // 118 'v'
    [0x00, 0x00, 0x33, 0x33, 0x33, 0x1E, 0x0C, 0x00],
    // 119 'w'
    [0x00, 0x00, 0x63, 0x6B, 0x7F, 0x7F, 0x36, 0x00],
    // 120 'x'
    [0x00, 0x00, 0x63, 0x36, 0x1C, 0x36, 0x63, 0x00],
    // 121 'y'
    [0x00, 0x00, 0x33, 0x33, 0x33, 0x3E, 0x30, 0x1F],
    // 122 'z'
    [0x00, 0x00, 0x3F, 0x19, 0x0C, 0x26, 0x3F, 0x00],
    // 123 '{'
    [0x38, 0x0C, 0x0C, 0x07, 0x0C, 0x0C, 0x38, 0x00],
    // 124 '|'
    [0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x18, 0x00],
    // 125 '}'
    [0x07, 0x0C, 0x0C, 0x38, 0x0C, 0x0C, 0x07, 0x00],
    // 126 '~'
    [0x6E, 0x3B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    // 127: solid block (used for filled rectangles)
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
];


// ============================================================================
// Integration with luminara_diagnostic
// ============================================================================

impl OverlayRendererInterface for OverlayRenderer {
    fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        self.draw_rect(x, y, w, h, color);
    }

    fn draw_text(&mut self, x: f32, y: f32, text: &str, color: [f32; 4], scale: f32) {
        self.draw_text(x, y, text, color, scale);
    }
}
