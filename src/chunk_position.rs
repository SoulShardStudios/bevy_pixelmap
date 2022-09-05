use bevy::prelude::*;

pub fn get_chunk_inner_i(position: IVec2, chunk_size: UVec2) -> UVec2 {
    UVec2 {
        x: position.x.rem_euclid(chunk_size.x as i32) as u32,
        y: position.y.rem_euclid(chunk_size.y as i32) as u32,
    }
}
pub fn get_chunk_outer_i(position: IVec2, chunk_size: UVec2) -> IVec2 {
    IVec2 {
        x: position.x / chunk_size.x as i32,
        y: position.y / chunk_size.y as i32,
    }
}

pub fn get_chunk_index_i(position: IVec2, chunk_size: UVec2) -> usize {
    let inner = get_chunk_inner_i(position, chunk_size);
    return (inner.y * chunk_size.x + inner.x) as usize;
}
