@group(0) @binding(0) var ssaoTextureIn: texture_2d<f32>;
@group(0) @binding(1) var depthTexture: texture_depth_2d;
@group(0) @binding(2) var ssaoBlurredOut: texture_storage_2d<r32float, write>;

@compute @workgroup_size(8, 8)
fn blur_ssao_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let coords = vec2<i32>(id.xy);
    let centerDepth = textureLoad(depthTexture, coords, 0);
    let centerAO = textureLoad(ssaoTextureIn, coords, 0).r;

    var result = 0.0;
    var totalWeight = 0.0;

    // 4x4 kernel
    for (var x = -2; x <= 2; x++) {
        for (var y = -2; y <= 2; y++) {
            let offset = vec2<i32>(x, y);
            let depthAtOffset = textureLoad(depthTexture, coords + offset, 0);
            let aoAtOffset = textureLoad(ssaoTextureIn, coords + offset, 0).r;

            // only blend if depths are similar
            let weight = 1.0 / (0.0001 + abs(centerDepth - depthAtOffset));

            result += aoAtOffset * weight;
            totalWeight += weight;
        }
    }

    textureStore(ssaoBlurredOut, coords, vec4<f32>(result / totalWeight));
}
