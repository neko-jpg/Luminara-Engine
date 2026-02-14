use crate::symbolic::SymExpr;

pub struct ShaderGenerator;

impl ShaderGenerator {
    /// Generates a simple WGSL fragment shader from a symbolic expression.
    /// The expression is assumed to compute a float value which will be used for intensity/color.
    pub fn generate_wgsl(expr: &SymExpr) -> String {
        let code = Self::expr_to_wgsl(expr);
        format!(
            r#"
struct Input {{
    time: f32,
    uv: vec2<f32>,
}};

@group(0) @binding(0) var<uniform> input: Input;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {{
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    return vec4<f32>(pos[in_vertex_index], 0.0, 1.0);
}}

@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {{
    let t = input.time;
    let x = coord.x / 800.0; // Assume 800x600 for simplicity or pass resolution
    let y = coord.y / 600.0;

    let val = {};

    // Map value to color (heatmap style)
    let color = vec3<f32>(val, val * 0.5, 1.0 - val);
    return vec4<f32>(color, 1.0);
}}
"#,
            code
        )
    }

    fn expr_to_wgsl(expr: &SymExpr) -> String {
        match expr {
            SymExpr::Const(c) => format!("{:.4}", c),
            SymExpr::Var(name) => match name.as_str() {
                "t" | "time" => "t".to_string(),
                "x" => "x".to_string(),
                "y" => "y".to_string(),
                _ => "0.0".to_string(),
            },
            SymExpr::Add(a, b) => format!("({} + {})", Self::expr_to_wgsl(a), Self::expr_to_wgsl(b)),
            SymExpr::Sub(a, b) => format!("({} - {})", Self::expr_to_wgsl(a), Self::expr_to_wgsl(b)),
            SymExpr::Mul(a, b) => format!("({} * {})", Self::expr_to_wgsl(a), Self::expr_to_wgsl(b)),
            SymExpr::Div(a, b) => format!("({} / {})", Self::expr_to_wgsl(a), Self::expr_to_wgsl(b)),
            SymExpr::Sin(a) => format!("sin({})", Self::expr_to_wgsl(a)),
            SymExpr::Cos(a) => format!("cos({})", Self::expr_to_wgsl(a)),
            _ => "0.0".to_string(),
        }
    }
}
