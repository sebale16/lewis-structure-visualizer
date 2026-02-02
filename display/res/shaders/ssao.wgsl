struct Uniforms {
    proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    kernel: array<vec4<f32>, 64>,
    radius: f32,
    bias: f32,
    padding: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var depthTexture: texture_depth_2d;
@group(0) @binding(2) var normalTexture: texture_2d<f32>;
@group(0) @binding(3) var noiseTexture: texture_2d<f32>;
@group(0) @binding(4) var ssaoSampler: sampler;
@group(0) @binding(5) var outTexture: texture_storage_2d<r32float, write>;

// gets view-space position of fragment
fn getViewPos(uv: vec2<f32>, fullDims: vec2<f32>) -> vec3<f32> {
    // what is depth value at this uv
    let pixelCoords = vec2<i32>(uv * fullDims);
    let depth = textureLoad(depthTexture, pixelCoords, 0);
    // based on depth, turn this texture point into 3d: how camera saw it
    let ndc = vec4<f32>(uv.x * 2.0 - 1.0, (1.0 - uv.y) * 2.0 - 1.0, depth, 1.0);
    // undo projection matrix so that each unit at each depth is the same length
    let viewPos = uniforms.inv_proj * ndc;
    return viewPos.xyz / viewPos.w;
}

@compute @workgroup_size(8, 8)
fn compute_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dimsFull = vec2<f32>(textureDimensions(depthTexture));
    let dimsSSAO = vec2<f32>(textureDimensions(outTexture));

    // don't process out of bounds pixels
    if (id.x >= u32(dimsSSAO.x) || id.y >= u32(dimsSSAO.y)) { return; }

    let ssaoCoords = vec2<i32>(id.xy);
    let normalizedUV = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(dimsSSAO);

    let fullResPixelCoords = vec2<i32>(normalizedUV * dimsFull);

    // do not occlude background
    let centerDepth = textureLoad(depthTexture, fullResPixelCoords, 0);
    if (centerDepth >= 1.0) {
        textureStore(outTexture, ssaoCoords, vec4<f32>(1.0));
        return;
    }

    let fragPos = getViewPos(normalizedUV, dimsFull);
    // scale normal to full res
    let normal = textureLoad(normalTexture, fullResPixelCoords, 0).xyz;
    let noiseCoords = vec2<i32>(id.xy % 16u);
    let randVec = textureLoad(noiseTexture, noiseCoords, 0).xyz;

    // build tangent, bitangent, normal matrix
    let tangent = normalize(randVec - normal * dot(randVec, normal));
    let bitangent = cross(normal, tangent);
    let TBN = mat3x3<f32>(tangent, bitangent, normal);

    var occlusion = 0.0;
    for (var i = 0; i < 64; i++) {
        // transform sampled position to align with orientation of fragment
        let samplePos = TBN * uniforms.kernel[i].xyz;
        // worldPos.z has where random sample is
        let worldPos = fragPos + samplePos * uniforms.radius;

        // what does camera see to query what is the depth at that worldPos
        let offset = uniforms.proj * vec4<f32>(worldPos, 1.0);
        let offsetUV = (offset.xy / offset.w) * vec2<f32>(0.5, -0.5) + 0.5;

        // skip if depth at current sampled position is background
        let samplePixelCoords = vec2<i32>(offsetUV * dimsFull);
        let sampleDepth = textureLoad(depthTexture, samplePixelCoords, 0);
        if (sampleDepth >= 1.0) {
            continue; 
        }

        // actualPos.z has what is z value of object that is actually rendered
        let actualPos = getViewPos(offsetUV, dimsFull);

        // range check to not render shadows as a result of occlusion from distant objects
        let dist = abs(fragPos.z - actualPos.z);
        // let rangeCheck = exp(-dist * 10.0 / uniforms.radius); 
        let rangeCheck = smoothstep(uniforms.radius, 0.0, dist);
        // if actualPos.z > worldPos.z, then sampled pos is in behind object
        if (actualPos.z >= worldPos.z + uniforms.bias || actualPos.z < worldPos.z - uniforms.bias) {
            occlusion += 1.0 * rangeCheck;
        }
    }

    // normalize
    let finalSSAO = 1.0 - (occlusion / 64.0);
    textureStore(outTexture, vec2<i32>(id.xy), vec4<f32>(finalSSAO));
}
