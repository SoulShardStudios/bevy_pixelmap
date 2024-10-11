@group(0) @binding(0) var input_texture: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(1) var<uniform> input_texture_pos: vec2<i32>;
@group(0) @binding(2) var<uniform> input_texture_size: vec2<u32>;
@group(0) @binding(3) var<uniform> source_texture_pos: vec2<i32>;
@group(0) @binding(4) var<uniform> source_texture_size: vec2<u32>;
@group(0) @binding(5) var source_texture: texture_storage_2d<rgba8unorm, read>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let coords: vec2<i32> = vec2<i32>(invocation_id.xy);
    var source_texture_id = input_texture_pos - source_texture_pos + vec2<i32>(coords.x, coords.y * -1 + i32(input_texture_size.y));
    source_texture_id = vec2<i32>(source_texture_id.x,i32(source_texture_size.y) -  source_texture_id.y);
    var source_pixel: vec4<f32> = textureLoad(source_texture, source_texture_id);
    if (source_pixel.a > 0.0) {
        textureStore(input_texture, coords, source_pixel);
    }
}
