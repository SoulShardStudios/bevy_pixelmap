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
    pub img_data: Vec<Handle<Image>>,
    pub modified: Vec<bool>,
    pub positions: HashMap<IVec2, usize>,
    pub entities: Vec<Entity>,
    pub set_pixel_queue: HashMap<IVec2, [u8; 4]>,
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
            positions: HashMap::new(),
            entities: Vec::new(),
            empty_texture: empty,
            set_pixel_queue: HashMap::new(),
            root_entity: root_entity,
        }
    }

    pub fn set_pixel(&mut self, world_position: IVec2, color: [u8; 4]) {
        self.set_pixel_queue.insert(world_position, color);
    }
}

fn add_pixel_map_chunks(
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
    mut pixel_map_query: Query<&mut PixelMap>,
) {
    for mut pixel_map in pixel_map_query.iter_mut() {
        let mut added_images = Vec::new();
        let mut added_entities = Vec::new();
        let mut added_positions = HashMap::new();
        for (position, color) in pixel_map.set_pixel_queue.iter() {
            let c_pos = IChunkPosition::from_world(*position, pixel_map.chunk_size);

            if !pixel_map.positions.contains_key(&c_pos.outer)
                && !added_positions.contains_key(&c_pos.outer)
            {
                let computed_position: Vec2 =
                    (c_pos.outer * pixel_map.chunk_size.as_ivec2()).as_vec2();
                let tex_handle = textures.add(pixel_map.empty_texture.clone());
                let id = commands
                    .spawn_bundle(SpriteBundle {
                        texture: tex_handle.clone(),
                        transform: Transform::from_xyz(
                            computed_position.x,
                            computed_position.y,
                            0.0,
                        ),
                        ..Default::default()
                    })
                    .insert(PixelChunk)
                    .id();
                commands.entity(pixel_map.root_entity).add_child(id);

                added_positions.insert(c_pos.outer, pixel_map.img_data.len());
                added_entities.push(id);
                added_images.push(tex_handle);
                continue;
            }
        }
        pixel_map.entities.append(&mut added_entities);
        pixel_map.positions.extend(added_positions.iter());
        pixel_map.img_data.append(&mut added_images);
        for (position, color) in pixel_map.set_pixel_queue.iter() {
            let c_pos = IChunkPosition::from_world(*position, pixel_map.chunk_size);
            let pos = pixel_map.positions[&c_pos.outer];
            let ind = c_pos.get_index(pixel_map.chunk_size) * 4;
            let data = &mut textures.get_mut(&pixel_map.img_data[pos]).unwrap().data;
            data[ind + 0] = color[0];
            data[ind + 1] = color[1];
            data[ind + 2] = color[2];
            data[ind + 3] = color[3];
        }
        pixel_map.set_pixel_queue.clear();
    }
}

pub struct PixelMaps;

impl Plugin for PixelMaps {
    fn build(&self, app: &mut App) {
        app.add_system(add_pixel_map_chunks);
    }
}
