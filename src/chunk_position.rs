use bevy::prelude::*;

pub struct IChunkPosition {
    pub inner: UVec2,
    pub outer: IVec2,
}

impl IChunkPosition {
    pub fn from_world(position: IVec2, chunk_size: UVec2) -> Self {
        IChunkPosition {
            inner: UVec2 {
                x: position.x.rem_euclid(chunk_size.x as i32) as u32,
                y: position.y.rem_euclid(chunk_size.y as i32) as u32,
            },
            outer: IVec2 {
                x: position.x / chunk_size.x as i32,
                y: position.y / chunk_size.y as i32,
            },
        }
    }
}

pub struct ChunkPosition {
    pub inner: Vec2,
    pub outer: IVec2,
}

impl ChunkPosition {
    pub fn from_world(position: Vec2, chunk_size: UVec2) -> Self {
        ChunkPosition {
            inner: Vec2 {
                x: position.x.rem_euclid(chunk_size.x as f32),
                y: position.y.rem_euclid(chunk_size.y as f32),
            },
            outer: IVec2 {
                x: (position.x / chunk_size.x as f32).floor() as i32,
                y: (position.y / chunk_size.y as f32).floor() as i32,
            },
        }
    }
}
