[[block]]
struct Uniforms {
    resolution: vec2<f32>;
    min_max_range: vec2<f32>;
    draw_colour: bool;
};

struct FragmentOutput {
    [[location(0)]] f_color: vec4<f32>;
};

[[group(0), binding(0)]]
var depth_tex: texture_2d<u32>;
[[group(0), binding(1)]]
var<uniform> uniforms: Uniforms;

fn map_range(val: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    return (val - in_min) / (in_max - in_min) * (out_max - out_min) + out_min;
}

fn hsb2rgb(c: vec3<f32>) -> vec3<f32> {
    var rgb: vec3<f32> = clamp((abs((((vec3<f32>((c.x * 6.0)) + vec3<f32>(0.0, 4.0, 2.0)) % vec3<f32>(6.0)) - vec3<f32>(3.0))) - vec3<f32>(1.0)), vec3<f32>(0.0), vec3<f32>(1.0));
    rgb = ((rgb * rgb) * (vec3<f32>(3.0) - (2.0 * rgb)));
    return (c.z * mix(vec3<f32>(1.0), rgb, vec3<f32>(c.y)));
}


[[stage(fragment)]]
fn main([[location(0)]] tex_coords: vec2<f32>) -> FragmentOutput {
    let itex_coords = tex_coords * uniforms.resolution;
    let tex = textureLoad(depth_tex, vec2<i32>(itex_coords), 0);
    var v = map_range(f32(tex.x), uniforms.min_max_range.x, uniforms.min_max_range.y, 0.0, 1.0);
    // let hue = map_range(v, 0.0, 1.0, 0.6666667, 0.3);

    if(uniforms.draw_colour) {
        let col = hsb2rgb(vec3<f32>(v,1.0,1.0));
        return FragmentOutput(vec4<f32>(col, 1.0));
    } else {
        return FragmentOutput(vec4<f32>(v, v, v, 1.0));
    }
}

