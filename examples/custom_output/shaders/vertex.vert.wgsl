struct GlobalMatrix {
    proj_view: mat4x4<f32>,
}

struct VertexOutput {
    @location(0) texp: vec2<f32>,
    @builtin(position) member: vec4<f32>,
}

var<private> pos_1: vec3<f32>;
var<private> tex_pos_1: vec2<f32>;
@group(0) @binding(0) 
var<uniform> global: GlobalMatrix;
var<private> texp: vec2<f32>;
var<private> gl_Position: vec4<f32>;

fn main_1() {
    let _e5 = tex_pos_1;
    texp = _e5;
    let _e7 = global.proj_view;
    let _e8 = pos_1;
    gl_Position = (_e7 * vec4<f32>(_e8.x, _e8.y, _e8.z, 1.0));
    return;
}

@vertex 
fn main(@location(0) pos: vec3<f32>, @location(1) tex_pos: vec2<f32>) -> VertexOutput {
    pos_1 = pos;
    tex_pos_1 = tex_pos;
    main_1();
    let _e13 = texp;
    let _e15 = gl_Position;
    return VertexOutput(_e13, _e15);
}
