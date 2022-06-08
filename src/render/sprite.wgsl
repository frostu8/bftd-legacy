struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec4<f32>,
};

@group(0)
@binding(0)
var tex_sampler: sampler;

@group(0)
@binding(1)
var tex: texture_2d<f32>;

@group(0)
@binding(2)
var<uniform> transform: mat4x4<f32>;

@group(0)
@binding(3)
var<uniform> tex_transform: mat4x4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    let x = f32((i32(in_vertex_index) + 2) / 3 % 2);
    let y = f32((i32(in_vertex_index) + 1) / 3 % 2);
    let v = 1.0 - y;
    let position = vec2<f32>(x, y) - vec2<f32>(0.5, 0.5);
    let tex_coord = vec2<f32>(x, v);

    var result: VertexOutput;
    result.position = transform * vec4<f32>(position, 1.0, 1.0);
    result.tex_coord = tex_transform * vec4<f32>(tex_coord, 1.0, 1.0);
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, tex_sampler, vertex.tex_coord.xy);
}

