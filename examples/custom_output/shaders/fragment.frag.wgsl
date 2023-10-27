struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

var<private> texp_1: vec2<f32>;
var<private> color: vec4<f32>;
@group(0) @binding(1) 
var t: texture_2d<f32>;
@group(0) @binding(2) 
var s: sampler;

fn main_1() {
    let _e5 = texp_1;
    let _e6 = textureSample(t, s, _e5);
    color = _e6;
    return;
}

@fragment 
fn main(@location(0) texp: vec2<f32>) -> FragmentOutput {
    texp_1 = texp;
    main_1();
    let _e11 = color;
    return FragmentOutput(_e11);
}
