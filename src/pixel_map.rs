use std::convert::TryInto;

use bevy::{
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssetUsages,
        render_resource::{
            BindGroupLayout, BindGroupLayoutEntry, BindingType, BufferBindingType, Extent3d,
            ShaderStages, StorageTextureAccess, TextureDimension, TextureFormat, TextureUsages,
            TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::ImageSampler,
        Render, RenderApp, RenderSet,
    },
    utils::HashMap,
};

use crate::chunk_position::{get_chunk_index_i, get_chunk_outer_i};

#[derive(Component)]
pub struct PixelChunk;

#[derive(Component, ExtractComponent, Clone)]
pub struct PixelMap {
    chunk_size: UVec2,
    img_data: Vec<Handle<Image>>,
    positions: HashMap<IVec2, usize>,
    empty_texture: Image,
    root_entity: Entity,
    default_chunk_color: [u8; 4],
    texture_queue: Vec<Handle<Image>>,
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
        empty.texture_descriptor.usage = TextureUsages::COPY_DST
            | TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING;
        empty.sampler = sampler.unwrap_or_else(ImageSampler::nearest);
        PixelMap {
            chunk_size,
            img_data: Vec::new(),
            positions: HashMap::new(),
            empty_texture: empty,
            root_entity,
            default_chunk_color: color,
            texture_queue: vec![],
        }
    }

    pub fn get_pixels_cpu(
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

    pub fn set_pixels_cpu(
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

    pub fn set_pixels_gpu(textures: &Vec<Handle<Image>>) {}
}

struct PixelMapGpuComputePlugin;

impl Plugin for PixelMapGpuComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<PixelMap>::default());
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(Render, prepare_binds.in_set(RenderSet::PrepareBindGroups))
            .add_systems(Render, apply_ops)
            .insert_resource(RenderDataResource(HashMap::new()));
    }
}

pub struct PixelMapShaderLayoutInput {
    pub bind_group_layout: BindGroupLayout,
}

impl PixelMapShaderLayoutInput {
    pub fn new(device: &RenderDevice) -> Self {
        let bind_group_layout = device.create_bind_group_layout(
            Some("set_pixels_cpu Bind Group Layout"),
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 7,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 8,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 9,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 10,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 11,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 12,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        );

        Self { bind_group_layout }
    }
}
