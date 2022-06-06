struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@group(0)
@binding(0)
var<uniform> transform: mat4x4<f32>;

@group(0)
@binding(1)
var tex_sampler: sampler;

@group(0)
@binding(2)
var tex: texture_2d<f32>;

@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) tex_coord: vec2<f32>) -> VertexOutput {
    let position = transform * vec4<f32>(position, 1.0, 1.0);

    var result: VertexOutput;
    result.position = position;
    result.tex_coord = tex_coord;
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, tex_sampler, vertex.tex_coord);
}

