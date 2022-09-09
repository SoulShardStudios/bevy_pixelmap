use bevy::{
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
    utils::HashMap,
};

use crate::chunk_position::{get_chunk_index_i, get_chunk_outer_i};
#[derive(Component)]
pub struct PixelChunk;

#[derive(Component)]
pub struct PixelMap {
    chunk_size: UVec2,
    img_data: Vec<Handle<Image>>,
    positions: HashMap<IVec2, usize>,
    pub empty_texture: Image,
    root_entity: Entity,
    default_chunk_color: [u8; 4],
}

impl PixelMap {
    pub fn new(
        chunk_size: UVec2,
        root_entity: Entity,
        empty_texture: Option<Image>,
        sampler: Option<ImageSampler>,
        default_chunk_color: Option<[u8; 4]>,
    ) -> Self {
        assert!(chunk_size.x > 0);
        assert!(chunk_size.y > 0);

        let color = match default_chunk_color {
            Some(x) => x,
            None => [0, 0, 0, 0],
        };

        let mut empty = match empty_texture {
            Some(x) => x,
            None => Image::new_fill(
                Extent3d {
                    depth_or_array_layers: 1,
                    width: chunk_size.x,
                    height: chunk_size.y,
                },
                TextureDimension::D2,
                &color,
                TextureFormat::Rgba8UnormSrgb,
            ),
        };

        let img_sampler = match sampler {
            Some(x) => x,
            None => ImageSampler::nearest(),
        };

        empty.sampler_descriptor = img_sampler;

        PixelMap {
            chunk_size: chunk_size,
            img_data: Vec::new(),
            positions: HashMap::new(),
            empty_texture: empty,
            root_entity: root_entity,
            default_chunk_color: color,
        }
    }

    pub fn get_pixels(
        &self,
        world_positions: &Vec<IVec2>,
        textures: &Res<Assets<Image>>,
    ) -> Vec<[u8; 4]> {
        let mut resources: HashMap<IVec2, Option<&Vec<u8>>> = HashMap::new();
        for x in world_positions {
            let outer = get_chunk_outer_i(*x, self.chunk_size);
            if resources.contains_key(&outer) {
                continue;
            }
            if !self.positions.contains_key(&outer) {
                resources.insert(outer, None);
                continue;
            }
            resources.insert(
                outer,
                Some(
                    &textures
                        .get(&self.img_data[self.positions[&outer]])
                        .unwrap()
                        .data,
                ),
            );
        }
        return world_positions
            .iter()
            .map(|x| {
                let outer = get_chunk_outer_i(*x, self.chunk_size);
                let resource = resources[&outer];
                match resource {
                    Some(true_res) => {
                        let ind = get_chunk_index_i(*x, self.chunk_size);
                        [
                            true_res[ind],
                            true_res[ind + 1],
                            true_res[ind + 2],
                            true_res[ind + 3],
                        ]
                    }
                    None => self.default_chunk_color,
                }
            })
            .collect();
    }

    pub fn add_chunk(
        &mut self,
        chunk_position: IVec2,
        commands: &mut Commands,
        textures: &mut ResMut<Assets<Image>>,
    ) {
        if self.positions.contains_key(&chunk_position) {
            return;
        }
        let computed_position: Vec2 = (chunk_position * self.chunk_size.as_ivec2()).as_vec2();
        let tex_handle = textures.add(self.empty_texture.clone());
        let id = commands
            .spawn_bundle(SpriteBundle {
                texture: tex_handle.clone(),
                transform: Transform::from_xyz(computed_position.x, computed_position.y, 0.0),
                ..Default::default()
            })
            .insert(PixelChunk)
            .id();
        commands.entity(self.root_entity).add_child(id);
        self.positions.insert(chunk_position, self.positions.len());
        self.img_data.push(tex_handle);
    }

    pub fn set_pixels(
        &mut self,
        pixels: HashMap<IVec2, [u8; 4]>,
        commands: &mut Commands,
        textures: &mut ResMut<Assets<Image>>,
    ) {
        for (position, color) in pixels.iter() {
            let chunk_pos = get_chunk_outer_i(*position, self.chunk_size);
            self.add_chunk(chunk_pos, commands, textures);
            let pos = self.positions[&chunk_pos];
            let ind = get_chunk_index_i(*position, self.chunk_size);
            let data = &mut textures.get_mut(&self.img_data[pos]).unwrap().data;

            data[ind * 4 + 0] = color[0];
            data[ind * 4 + 1] = color[1];
            data[ind * 4 + 2] = color[2];
            data[ind * 4 + 3] = color[3];
        }
    }
}
