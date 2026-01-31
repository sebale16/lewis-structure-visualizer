struct VertexInput {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
    @location(1) normal: vec3f,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;

@vertex
fn vs_main(v: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.position = camera.view_proj * model_matrix * vec4f(v.position, 1.0);
    out.color = instance.color;
    out.normal = normalize((camera.view_proj * model_matrix * vec4f(v.normal, 0.0)).xyz);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let L = normalize(vec3f(0.0, 1.0, -1.0));

    let dotNL = dot(in.normal, L);
    
    let wrap = 1.5; 
    let wrappedDot = (dotNL + wrap) / (1.0 + wrap);
    
    let diffuse = wrappedDot * wrappedDot;

    let ambient = 0.3;
    let finalLighting = max(diffuse, ambient);

    return vec4f(in.color.rgb * finalLighting, in.color.a);
}
