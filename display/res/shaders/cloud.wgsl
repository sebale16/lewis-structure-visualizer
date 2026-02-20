struct VertexInput {
    @location(0) cornerPos: vec2f,
    @location(1) centerPos: vec3f,
    @location(2) scale: vec2f, // (w, h)
    @location(3) color: vec3f,
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) cornerPos: vec2f,
    @location(1) color: vec3f,
}

struct CameraUniform {
    view: mat4x4<f32>,
    view_proj: mat4x4<f32>,
}

@group(1) @binding(0) var<uniform> camera: CameraUniform;
@group(2) @binding(0) var depthTexture: texture_depth_2d;
@group(2) @binding(1) var depthSampler: sampler;

@vertex
fn vs_cloud(in: VertexInput) -> VertexOutput {
    // ensure that cloud faces the screen
    let camera_right = vec3f(camera.view[0][0], camera.view[1][0], camera.view[2][0]);
    let camera_up = vec3f(camera.view[0][1], camera.view[1][1], camera.view[2][1]);

    let world_pos = in.centerPos
                    + (camera_right * in.cornerPos.x * in.scale.x)
                    + (camera_up * in.cornerPos.y * in.scale.y);

    var out: VertexOutput;
    out.position = camera.view_proj * vec4f(world_pos, 1.0);
    out.cornerPos = in.cornerPos;
    out.color = in.color;
    return out;
}

@fragment
fn fs_cloud(in: VertexOutput) -> @location(0) vec4f {
    let p = in.cornerPos * 2;
    let d = sqrt((p.x * p.y) / (0.6 * 0.6) + (p.y * p.y));

    // calculate density
    var density = exp(-d * d * 4.0) * smoothstep(1.0, 0.7, d);

    // soft edges
    let screen_uv = in.position.xy / vec2f(textureDimensions(depthTexture));
    let depth = textureSample(depthTexture, depthSampler, screen_uv);
    let depth_fade = saturate((depth - in.position.z) / 0.005);

    return vec4f(in.color, density * depth_fade * 0.8);
}
