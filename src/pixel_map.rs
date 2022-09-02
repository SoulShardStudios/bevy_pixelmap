use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::HashMap,
};

use crate::chunk_position::IChunkPosition;
#[derive(Component)]
pub struct PixelChunk;

#[derive(Component)]
pub struct PixelMap {
    pub chunk_size: UVec2,
    pub img_data: Vec<(Image, Handle<Image>)>,
    pub modified: Vec<bool>,
    pub positions: Vec<IVec2>,
    pub entities: Vec<Entity>,
    pub chunk_add_queue: HashMap<IVec2, Option<Image>>,
    pub empty_texture: Image,
    pub root_entity: Entity,
}

impl PixelMap {
    pub fn new(chunk_size: UVec2, empty_texture: Option<Image>, root_entity: Entity) -> Self {
        assert!(chunk_size.x > 0);
        assert!(chunk_size.y > 0);

        let empty = match empty_texture {
            Some(x) => x,
            None => Image::new(
                Extent3d {
                    depth_or_array_layers: 1,
                    width: chunk_size.x,
                    height: chunk_size.y,
                },
                TextureDimension::D2,
                vec![0; (chunk_size.x * chunk_size.y * 4) as usize],
                TextureFormat::Rgba8Unorm,
            ),
        };

        PixelMap {
            chunk_size: chunk_size,
            img_data: Vec::new(),
            modified: Vec::new(),
            positions: Vec::new(),
            entities: Vec::new(),
            empty_texture: empty,
            chunk_add_queue: HashMap::new(),
            root_entity: root_entity,
        }
    }

    pub fn add_chunk(&mut self, position: IVec2, texture: Option<Image>) {
        self.chunk_add_queue.insert(position, texture);
    }

    pub fn set_pixel(&mut self, world_position: IVec2, color: Color) {
        let chunk_pos = IChunkPosition::from_world(world_position, self.chunk_size);
        self.img_data[0].0.data[0] = 255;
    }
}

fn add_pixel_map_chunks(
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
    mut query: Query<&mut PixelMap>,
) {
    let mut ct = 0;

    for mut pixel_map in query.iter_mut() {
        let mut added_images = Vec::new();
        let mut added_entities = Vec::new();
        let mut added_positions = Vec::new();
        for (position, texture) in pixel_map.chunk_add_queue.iter() {
            ct += 1;
            if pixel_map.positions.contains(position) {
                println!("contained");
                continue;
            }

            let pos = position.clone();
            let tex = match texture {
                Some(x) => x.clone(),
                None => pixel_map.empty_texture.clone(),
            };

            let computed_position: Vec2 = (pos * pixel_map.chunk_size.as_ivec2()).as_vec2();

            let tex_handle = textures.add(tex.clone());
            let id = commands
                .spawn_bundle(SpriteBundle {
                    texture: tex_handle.clone(),
                    transform: Transform::from_xyz(computed_position.x, computed_position.y, 0.0),
                    ..Default::default()
                })
                .insert(PixelChunk)
                .id();
            commands.entity(pixel_map.root_entity).add_child(id);

            added_positions.push(*position);
            added_entities.push(id);
            added_images.push((tex, tex_handle));
        }
        println!("{}", ct);
        pixel_map.entities.append(&mut added_entities);
        pixel_map.positions.append(&mut added_positions);
        pixel_map.img_data.append(&mut added_images);
        pixel_map.chunk_add_queue.clear();
    }
}

pub struct PixelMaps;

impl Plugin for PixelMaps {
    fn build(&self, app: &mut App) {
        app.add_system(add_pixel_map_chunks);
    }
}
