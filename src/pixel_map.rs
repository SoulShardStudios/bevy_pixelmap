use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::HashMap,
};

#[derive(Component)]
pub struct PixelMap {
    pub chunk_size: UVec2,
    pub chunks: HashMap<IVec2, Entity>,
    pub chunk_add_queue: HashMap<IVec2, Option<Image>>,
    pub chunk_remove_queue: Vec<IVec2>,
    pub empty_texture: Image,
    pub pixels_per_unit: u32,
}

impl PixelMap {
    pub fn new(chunk_size: UVec2, pixels_per_unit: u32, empty_texture: Option<Image>) -> Self {
        assert!(chunk_size.x > 0);
        assert!(chunk_size.y > 0);

        let raw_tex_data: Vec<u8> = (0..(chunk_size.x * chunk_size.y))
            .map(|x| vec![255, 0, 0, 128])
            .flat_map(|x| x.into_iter())
            .collect();

        let empty = match empty_texture {
            Some(x) => x,
            None => Image::new(
                Extent3d {
                    depth_or_array_layers: 1,
                    width: chunk_size.x,
                    height: chunk_size.y,
                },
                TextureDimension::D2,
                raw_tex_data,
                TextureFormat::Rgba8Unorm,
            ),
        };

        PixelMap {
            chunk_size: chunk_size,
            chunks: HashMap::new(),
            empty_texture: empty,
            pixels_per_unit: pixels_per_unit,
            chunk_add_queue: HashMap::new(),
            chunk_remove_queue: Vec::new(),
        }
    }

    pub fn add_chunk(&mut self, position: IVec2, texture: Option<Image>) {
        self.chunk_add_queue.insert(position, texture);
    }
    pub fn remove_chunk(&mut self, position: IVec2) {
        self.chunk_remove_queue.push(position);
    }
}

fn add_pixel_map_chunks(
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
    mut query: Query<&mut PixelMap>,
) {
    for mut pixel_map in query.iter_mut() {
        let mut to_add = HashMap::new();
        for (position, texture) in pixel_map.chunk_add_queue.iter() {
            if pixel_map.chunks.contains_key(position) {
                continue;
            }

            let pos = position.clone();
            let tex = match texture {
                Some(x) => x.clone(),
                None => pixel_map.empty_texture.clone(),
            };

            let computed_position: Vec2 = (pos
                * pixel_map.chunk_size.as_ivec2()
                * IVec2 {
                    x: pixel_map.pixels_per_unit as i32,
                    y: pixel_map.pixels_per_unit as i32,
                })
            .as_vec2();

            let tex_handle = textures.add(tex);

            let id = commands
                .spawn_bundle(SpriteBundle {
                    texture: tex_handle,
                    transform: Transform::from_xyz(computed_position.x, computed_position.y, 0.0),
                    ..Default::default()
                })
                .id();

            to_add.insert(pos, id);
        }
        pixel_map.chunks.extend(to_add.iter());
        pixel_map.chunk_add_queue.clear();
    }
}

pub struct PixelMaps;

impl Plugin for PixelMaps {
    fn build(&self, app: &mut App) {
        app.add_system(add_pixel_map_chunks);
    }
}
