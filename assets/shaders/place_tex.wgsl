@group(0) @binding(0) var input_texture: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(1) var<uniform> input_texture_pos: vec2<i32>;
@group(0) @binding(2) var<uniform> input_texture_size: vec2<u32>;
@group(0) @binding(3) var<storage> source_texture_pos: array<vec2<i32>, 8>;
@group(0) @binding(4) var<storage> source_texture_size: array<vec2<u32>, 8>;

@group(0) @binding(5) var source_textures_0: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(6) var source_textures_1: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(7) var source_textures_2: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(8) var source_textures_3: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(9) var source_textures_4: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(10) var source_textures_5: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(11) var source_textures_6: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(12) var source_textures_7: texture_storage_2d<rgba8unorm, read>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let coords: vec2<i32> = vec2<i32>(invocation_id.xy);

    for (var i: u32 = 0; i < 8; i = i + 1) {
        let source_pos = source_texture_pos[i] - coords;
        let source_size = source_texture_size[i];


        var possey = input_texture_pos - source_texture_pos[i] + vec2<i32>(coords.x, coords.y * -1);
        var possey2 = vec2<i32>(possey.x, possey.y * -1 + i32(source_texture_size[i].y) - i32(input_texture_size.y));

        var source_pixel: vec4<f32> = vec4<f32>(0.0);
        if (i == 0u) {
            source_pixel = textureLoad(source_textures_0, possey2);
        } else if (i == 1u) {
            source_pixel = textureLoad(source_textures_1, possey2);
        } else if (i == 2u) {
            source_pixel = textureLoad(source_textures_2, possey2);
        } else if (i == 3u) {
            source_pixel = textureLoad(source_textures_3, possey2);
        } else if (i == 4u) {
            source_pixel = textureLoad(source_textures_4, possey2);
        } else if (i == 5u) {
            source_pixel = textureLoad(source_textures_5, possey2);
        } else if (i == 6u) {
            source_pixel = textureLoad(source_textures_6, possey2);
        } else if (i == 7u) {
            source_pixel = textureLoad(source_textures_7, possey2);
        }

        if (source_pixel.a > 0.0) {
            textureStore(input_texture, coords, source_pixel);
        }
        else{
        textureStore(input_texture, coords, vec4<f32>(1.0));
        }
    }
}
