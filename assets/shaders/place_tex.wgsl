@group(0) @binding(0) var input_texture: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(1) var<uniform> input_texture_pos: vec2<i32>;
@group(0) @binding(2) var<uniform> input_texture_size: vec2<u32>;
@group(0) @binding(3) var<storage> source_texture_pos: array<vec2<i32>, 32>;
@group(0) @binding(4) var<storage> source_texture_size: array<vec2<u32>, 32>;

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

    textureStore(input_texture, coords, vec4<f32>(1.0));

    /*

    for (var i: u32 = 0; i < 32; i = i + 1) {
        let source_pos = source_texture_pos[i];
        let source_size = source_texture_size[i];
        let source_coords_texture = coords - source_pos;

        if (source_coords_texture.x < 0 || source_coords_texture.x >= i32(source_size.x) ||
            source_coords_texture.y < 0 || source_coords_texture.y >= i32(source_size.y)) {
            continue;
        }

        let source_coords_world = source_pos + source_coords_texture;

        if (source_coords_world.x < input_texture_pos.x || source_coords_world.x >= input_texture_pos.x + i32(input_texture_size.x) ||
            source_coords_world.y < input_texture_pos.y || source_coords_world.y >= input_texture_pos.y + i32(input_texture_size.y)) {
            continue;
        }

        var source_pixel: vec4<f32> = vec4<f32>(0.0);
        if (i == 0u) {
            source_pixel = textureLoad(source_textures_0, source_coords_texture);
        } else if (i == 1u) {
            source_pixel = textureLoad(source_textures_1, source_coords_texture);
        } else if (i == 2u) {
            source_pixel = textureLoad(source_textures_2, source_coords_texture);
        } else if (i == 3u) {
            source_pixel = textureLoad(source_textures_3, source_coords_texture);
        } else if (i == 4u) {
            source_pixel = textureLoad(source_textures_4, source_coords_texture);
        } else if (i == 5u) {
            source_pixel = textureLoad(source_textures_5, source_coords_texture);
        } else if (i == 6u) {
            source_pixel = textureLoad(source_textures_6, source_coords_texture);
        } else if (i == 7u) {
            source_pixel = textureLoad(source_textures_7, source_coords_texture);
        }

        if (source_pixel.a > 0.0) {
            textureStore(input_texture, coords, source_pixel);
        }
        textureStore(input_texture, coords, vec4<f32>(1.0));
    }

    */

}
