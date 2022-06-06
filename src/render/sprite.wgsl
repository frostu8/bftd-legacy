struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@group(0)
@binding(0)
var<uniform> transform: mat3x3<f32>;

@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) tex_coord: vec2<f32>) -> VertexOutput {
    //let x = f32(i32(in_vertex_index) - 1);

    var result: VertexOutput;
    result.position = vec4<f32>(position, 0.0, 1.0);
    result.tex_coord = tex_coord;
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}

