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
    var position: vec2<f32>;
    switch in_vertex_index {
        case 0: { position = vec2<f32>(0.0, 0.0); }
        case 1: { position = vec2<f32>(1.0, 0.0); }
        case 2: { position = vec2<f32>(1.0, 1.0); }
        case 3: { position = vec2<f32>(1.0, 1.0); }
        case 4: { position = vec2<f32>(0.0, 1.0); }
        case 5: { position = vec2<f32>(0.0, 0.0); }
        default { discard; }
    }

    var result: VertexOutput;
    result.position = transform * vec4<f32>(position - vec2<f32>(0.5, 0.5), 1.0, 1.0);
    result.tex_coord = tex_transform * vec4<f32>(position, 1.0, 1.0);
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, tex_sampler, vertex.tex_coord.xy);
}

