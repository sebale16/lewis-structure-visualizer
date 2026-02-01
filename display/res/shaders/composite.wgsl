struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// create triangle that fills entire screen
@vertex
fn vs_main(@builtin(vertex_index) in: u32) -> VertexOutput {
    var out: VertexOutput;
    // map indices 0, 1, 2, to (-1, -1), (3, -1), (-1, 3)
    let x = f32(u32(in == 1u) * 4u) - 1.0;
    let y = f32(u32(in == 2u) * 4u) - 1.0;
    
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    // transform to uv and flip y-axis
    out.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    return out;
}

@group(0) @binding(0) var colorTexture: texture_2d<f32>;
@group(0) @binding(1) var ssaoTexture: texture_2d<f32>;
@group(0) @binding(2) var compositeSampler: sampler;

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(ssaoTexture));
    let coords = vec2<i32>(uv * dims);

    let color = textureSample(colorTexture, compositeSampler, uv).rgb;
    let ao = textureLoad(ssaoTexture, coords, 0).r;

    return vec4<f32>(color * ao, 1.0);
}
