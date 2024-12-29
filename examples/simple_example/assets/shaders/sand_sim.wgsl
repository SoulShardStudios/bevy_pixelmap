@group(0) @binding(0) var input_texture: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(1) var<uniform> input_texture_pos: vec2<i32>;
@group(0) @binding(2) var<uniform> input_texture_size: vec2<u32>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let coords = vec2<i32>(invocation_id.xy);

    // Read the current pixel
    let current_pixel: vec4<f32> = textureLoad(input_texture, coords);

    if coords.x == 0 || coords.y == 0 || coords.x == i32(input_texture_size.x) - 1 || coords.y == i32(input_texture_size.y) - 1 {
        return;
    }

    // Simulate sand falling logic
    if (current_pixel.r > 0.0) { // If the pixel contains sand
        let below_coords = coords + vec2<i32>(0, -1); // One pixel below
        let below_pixel: vec4<f32> = textureLoad(input_texture, below_coords);

        if (below_pixel.r == 0.0) { // If the below pixel is empty
            textureStore(input_texture, below_coords, current_pixel);
            textureStore(input_texture, coords, vec4<f32>(0.0)); // Clear current pixel
        } else { 
            // Check diagonal pixels for sliding
            let left_coords = coords + vec2<i32>(-1, -1); // Diagonally left below
            let right_coords = coords + vec2<i32>(1, -1);  // Diagonally right below
            let left_pixel: vec4<f32> = textureLoad(input_texture, left_coords);
            let right_pixel: vec4<f32> = textureLoad(input_texture, right_coords);

            if (left_pixel.r == 0.0) { // Slide left if empty
                textureStore(input_texture, left_coords, current_pixel);
                textureStore(input_texture, coords, vec4<f32>(0.0)); // Clear current pixel
            } else if (right_pixel.r == 0.0) { // Slide right if empty
                textureStore(input_texture, right_coords, current_pixel);
                textureStore(input_texture, coords, vec4<f32>(0.0)); // Clear current pixel
            }
        }
    }
}