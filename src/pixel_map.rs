use std::convert::TryInto;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
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
        let color = default_chunk_color.unwrap_or([0, 0, 0, 0]);
        let mut empty = empty_texture.unwrap_or_else(|| {
            Image::new_fill(
                Extent3d {
                    depth_or_array_layers: 1,
                    width: chunk_size.x,
                    height: chunk_size.y,
                },
                TextureDimension::D2,
                &color,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::all(),
            )
        });
        empty.sampler = sampler.unwrap_or_else(ImageSampler::nearest);
        PixelMap {
            chunk_size,
            img_data: Vec::new(),
            positions: HashMap::new(),
            empty_texture: empty,
            root_entity,
            default_chunk_color: color,
        }
    }

    pub fn get_pixels(
        &self,
        world_positions: &[IVec2],
        textures: &Res<Assets<Image>>,
    ) -> Vec<[u8; 4]> {
        let mut resources: HashMap<IVec2, Option<&Vec<u8>>> =
            HashMap::with_capacity(world_positions.len());
        world_positions.iter().for_each(|&x| {
            let outer = get_chunk_outer_i(x, self.chunk_size);
            resources.entry(outer).or_insert_with(|| {
                self.positions
                    .get(&outer)
                    .and_then(|&pos| textures.get(&self.img_data[pos]).map(|img| &img.data))
            });
        });
        world_positions
            .iter()
            .map(|&x| {
                let outer = get_chunk_outer_i(x, self.chunk_size);
                resources[&outer].map_or(self.default_chunk_color, |true_res| {
                    let ind = get_chunk_index_i(x, self.chunk_size);
                    true_res[ind..ind + 4]
                        .try_into()
                        .unwrap_or(self.default_chunk_color)
                })
            })
            .collect()
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
        let computed_position = (chunk_position * self.chunk_size.as_ivec2()).as_vec2();
        let tex_handle = textures.add(self.empty_texture.clone());
        let id = commands
            .spawn(SpriteBundle {
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
        pixels: (Vec<IVec2>, Vec<[u8; 4]>),
        commands: &mut Commands,
        textures: &mut ResMut<Assets<Image>>,
    ) {
        pixels
            .0
            .iter()
            .zip(pixels.1.iter())
            .for_each(|(&position, &color)| {
                let chunk_pos = get_chunk_outer_i(position, self.chunk_size);
                self.add_chunk(chunk_pos, commands, textures);
                let pos = self.positions[&chunk_pos];
                let ind = get_chunk_index_i(position, self.chunk_size) * 4;
                textures.get_mut(&self.img_data[pos]).unwrap().data[ind..ind + 4]
                    .copy_from_slice(&color);
            });
    }
}
