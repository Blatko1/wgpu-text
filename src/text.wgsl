struct VertexInput {
    [[builtin(vertex_index)]] vertex_index: u32;
    [[location(0)]] top_left: vec3<f32>;
    [[location(1)]] bottom_right: vec2<f32>;
    [[location(2)]] tex_top_left: vec2<f32>;
    [[location(3)]] tex_bottom_right: vec2<f32>;
    [[location(4)]] color: vec4<f32>;
};

struct Matrix {
    matrix: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> ortho: Matrix;

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_pos: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    var pos: vec2<f32>;
    var left: f32 = in.top_left.x;
    var right: f32 = in.bottom_right.x;
    var top: f32 = in.top_left.y;
    var bottom: f32 = in.bottom_right.y;

    switch (in.vertex_index) {
        case 0: {
            pos = vec2<f32>(left, top); 
            break;
        }
        case 1: {
            pos = vec2<f32>(right, top); 
            break;
        }
        case 2: {
            pos = vec2<f32>(left, bottom); 
            break;
        }
        case 3: {
            pos = vec2<f32>(right, bottom); 
            break;
        }
        default: {}
    }

    out.clip_position = ortho.matrix * vec4<f32>(pos, in.top_left.z, 1.0);
    out.tex_pos = in.tex_top_left;
    out.color = in.color;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 0.5);
}