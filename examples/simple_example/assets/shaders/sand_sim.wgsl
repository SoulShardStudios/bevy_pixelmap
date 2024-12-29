@group(0) @binding(0) var input_texture: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(1) var<uniform> input_texture_pos: vec2<i32>;
@group(0) @binding(2) var<uniform> input_texture_size: vec2<u32>;
@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let coords = vec2<i32>(invocation_id.xy);
    let current_pixel: vec4<f32> = textureLoad(input_texture, coords);
    if coords.x == 0 || coords.y == 0 || coords.x == i32(input_texture_size.x) - 1 || coords.y == i32(input_texture_size.y) - 1 {
        return;
    }
    if ( current_pixel.r == 1.0 && current_pixel.a == 1.0) {
        return;
    }
    if (current_pixel.a > 0.0) {
        let below_coords = coords + vec2<i32>(0, -1);
        let below_pixel: vec4<f32> = textureLoad(input_texture, below_coords);
        if (below_pixel.r == 1.0 && below_pixel.a == 1.0) { 
            return;
        }
        if (below_pixel.a == 0.0) { 
            textureStore(input_texture, below_coords, current_pixel);
            textureStore(input_texture, coords, vec4<f32>(0.0)); 
        } else { 
            let left_coords = coords + vec2<i32>(-1, -1);
            let right_coords = coords + vec2<i32>(1, -1); 
            let left_pixel: vec4<f32> = textureLoad(input_texture, left_coords);
            let right_pixel: vec4<f32> = textureLoad(input_texture, right_coords);
            if (left_pixel.r == 1.0 && left_pixel.a == 1.0) { 
                return;
            }
            else if (right_pixel.r == 1.0 && right_pixel.a == 1.0) { 
                return;
            }
            if (left_pixel.a == 0.0) {
                textureStore(input_texture, left_coords, current_pixel);
                textureStore(input_texture, coords, vec4<f32>(0.0)); 
            } else if (right_pixel.a == 0.0) { 
                textureStore(input_texture, right_coords, current_pixel);
                textureStore(input_texture, coords, vec4<f32>(0.0));
            }
        }
    }
}