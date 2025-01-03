use std::borrow::Cow;
use std::path::PathBuf;

use bevy::render::render_resource::{
    CachedComputePipelineId, CachedPipelineState, CommandEncoderDescriptor, ComputePassDescriptor,
    ComputePipelineDescriptor, IntoBinding, PipelineCache,
};
use bevy::render::renderer::RenderQueue;
use bevy::utils::hashbrown::HashMap;
use bevy::{
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::{RenderAssetUsages, RenderAssets},
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntry, BindingType,
            BufferBindingType, BufferInitDescriptor, BufferUsages, Extent3d, ShaderStages,
            StorageTextureAccess, TextureDimension, TextureFormat, TextureUsages,
            TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::{GpuImage, ImageSampler},
        Render, RenderApp, RenderSet,
    },
};
use lazy_static::lazy_static;
use std::iter::once;
use std::path::Path;

lazy_static! {
    static ref ASSETS_PATH: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("shaders");
}

fn get_chunk_inner_i(position: IVec2, chunk_size: UVec2) -> UVec2 {
    UVec2 {
        x: position.x.rem_euclid(chunk_size.x as i32) as u32,
        y: position.y.rem_euclid(chunk_size.y as i32) as u32,
    }
}

fn get_chunk_index_i(position: IVec2, chunk_size: UVec2) -> usize {
    let inner = get_chunk_inner_i(position, chunk_size);
    return (((chunk_size.y - inner.y - 1) * chunk_size.x) + inner.x) as usize;
}

fn get_chunk_outer_i(position: IVec2, chunk_size: UVec2) -> IVec2 {
    IVec2 {
        x: (position.x as f64 / chunk_size.x as f64).floor() as i32,
        y: (position.y as f64 / chunk_size.y as f64).floor() as i32,
    }
}

#[derive(Component)]
pub struct PixelChunk;

#[derive(Component, ExtractComponent, Clone)]
pub struct PixelMap {
    chunk_size: UVec2,
    image_data: Vec<Handle<Image>>,
    positions: HashMap<IVec2, usize>,
    simulation_shaders: Vec<String>,
    empty_texture: Image,
    root_entity: Entity,
    default_chunk_color: [u8; 4],
    texture_queue: Vec<PixelPositionedTexture>,
    texture_to_chunk_posses: HashMap<IVec2, Vec<PixelPositionedTexture>>,
}

#[derive(Clone, Debug)]
pub struct PixelPositionedTexture {
    pub position: IVec2,
    pub image: Handle<Image>,
    pub size: UVec2,
}

impl PixelMap {
    pub fn new(
        chunk_size: UVec2,
        root_entity: Entity,
        empty_texture: Option<Image>,
        sampler: Option<ImageSampler>,
        default_chunk_color: Option<[u8; 4]>,
        simulation_shaders: Vec<String>,
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
                TextureFormat::Rgba8Unorm,
                RenderAssetUsages::all(),
            )
        });
        empty.texture_descriptor.usage = TextureUsages::COPY_DST
            | TextureUsages::TEXTURE_BINDING
            | TextureUsages::STORAGE_BINDING;
        empty.sampler = sampler.unwrap_or_else(ImageSampler::nearest);
        PixelMap {
            chunk_size,
            image_data: Vec::new(),
            positions: HashMap::new(),
            empty_texture: empty,
            root_entity,
            default_chunk_color: color,
            texture_queue: vec![],
            texture_to_chunk_posses: HashMap::new(),
            simulation_shaders,
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
                    .and_then(|&pos| textures.get(&self.image_data[pos]).map(|img| &img.data))
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
                sprite: Sprite {
                    image: tex_handle.clone(),
                    ..default()
                },
                transform: Transform::from_xyz(computed_position.x, computed_position.y, 0.0),
                ..Default::default()
            })
            .insert(PixelChunk)
            .id();
        commands.entity(self.root_entity).add_child(id);
        self.positions.insert(chunk_position, self.positions.len());
        self.image_data.push(tex_handle);
    }

    pub fn set_pixels_gpu(
        &mut self,
        textures: Vec<PixelPositionedTexture>,
        images: &mut ResMut<Assets<Image>>,
    ) {
        textures.iter().for_each(|positioned_image| {
            let hi = positioned_image.image.id();
            images
                .get_mut(hi)
                .expect("expect loaded")
                .texture_descriptor
                .format = TextureFormat::Rgba8Unorm;
            images
                .get_mut(hi)
                .expect("expect loaded")
                .texture_descriptor
                .usage = TextureUsages::COPY_DST
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING;
        });
        self.texture_queue.extend(textures);
    }
}

pub struct PixelMapGpuComputePlugin;

#[derive(Resource)]
struct RenderData {
    bind_map: HashMap<IVec2, Vec<(BindGroup, UVec2)>>,
    shader_map: HashMap<IVec2, CachedComputePipelineId>,
    bind_map_2: HashMap<IVec2, Vec<(BindGroup, UVec2)>>,
    shader_map_2: HashMap<IVec2, Vec<CachedComputePipelineId>>,
}

impl Plugin for PixelMapGpuComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<PixelMap>::default())
            .add_systems(Update, prepare_chunks);
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(Render, prepare_binds.in_set(RenderSet::PrepareBindGroups))
            .add_systems(Render, apply_ops)
            .insert_resource(RenderData {
                bind_map: HashMap::default(),
                shader_map: HashMap::default(),
                bind_map_2: HashMap::default(),
                shader_map_2: HashMap::default(),
            });
    }
}

trait Points {
    fn points(&self) -> Vec<IVec2>;
}

impl Points for IRect {
    fn points(&self) -> Vec<IVec2> {
        let mut points = Vec::with_capacity((self.width().abs() * self.height().abs()) as usize);
        for x in self.min.x..self.max.x {
            for y in self.min.y..self.max.y {
                points.push(IVec2 { x, y });
            }
        }
        points
    }
}

fn prepare_chunks(
    mut pixel_map_query: Query<&mut PixelMap>,
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
) {
    for mut pixel_map in pixel_map_query.iter_mut() {
        let mut texture_to_chunk_posses: HashMap<IVec2, Vec<PixelPositionedTexture>> =
            HashMap::new();
        for tex in pixel_map.texture_queue.iter() {
            let c_pos_start = get_chunk_outer_i(tex.position, pixel_map.chunk_size);
            let c_pos_end =
                get_chunk_outer_i(tex.position + tex.size.as_ivec2(), pixel_map.chunk_size);
            let points = IRect {
                min: c_pos_start - IVec2 { x: 1, y: 1 },
                max: c_pos_end + IVec2 { x: 1, y: 1 }, // padding necessary for properly updating all chunks
            }
            .points();

            for position in points.iter() {
                if texture_to_chunk_posses.contains_key(position) {
                    texture_to_chunk_posses
                        .get_mut(position)
                        .expect("contains key")
                        .push(tex.clone())
                } else {
                    texture_to_chunk_posses.insert(*position, vec![tex.clone()]);
                }
            }
        }
        for (k, _v) in texture_to_chunk_posses.iter() {
            pixel_map.add_chunk(*k, &mut commands, &mut textures);
        }
        pixel_map.texture_to_chunk_posses = texture_to_chunk_posses;
        pixel_map.texture_queue.clear();
    }
}

fn prepare_binds(
    pixel_map_query: Query<&PixelMap>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    mut render_data: ResMut<RenderData>,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
) {
    for pixel_map in pixel_map_query.iter() {
        for (chunk_pos, chunk_texes) in pixel_map.texture_to_chunk_posses.iter() {
            let input_texture_pos_buffer =
                render_device.create_buffer_with_data(&BufferInitDescriptor {
                    label: Some("input_texture_pos_buffer"),
                    contents: bytemuck::cast_slice(&[
                        chunk_pos.x * pixel_map.chunk_size.x as i32,
                        chunk_pos.y * pixel_map.chunk_size.y as i32,
                    ]),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                });

            let input_texture_size_buffer =
                render_device.create_buffer_with_data(&BufferInitDescriptor {
                    label: Some("input_texture_size_buffer"),
                    contents: bytemuck::cast_slice(&[
                        pixel_map.chunk_size.x,
                        pixel_map.chunk_size.y,
                    ]),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                });

            for texes_chunk in chunk_texes.into_iter() {
                let source_texture_pos_buffer =
                    render_device.create_buffer_with_data(&BufferInitDescriptor {
                        label: Some("source_texture_pos_buffer"),
                        contents: bytemuck::cast_slice(&[
                            texes_chunk.position.x,
                            texes_chunk.position.y,
                        ]),
                        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                    });
                let source_texture_size_buffer =
                    render_device.create_buffer_with_data(&BufferInitDescriptor {
                        label: Some("source_texture_size_buffer"),
                        contents: bytemuck::cast_slice(&[texes_chunk.size.x, texes_chunk.size.y]),
                        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                    });
                let input_view = gpu_images
                    .get(&pixel_map.image_data[pixel_map.positions[chunk_pos]])
                    .unwrap();
                let source_view = &gpu_images.get(texes_chunk.image.id()).unwrap().texture_view;

                let layout = PixelMapShaderLayoutInput::new(&render_device).bind_group_layout;
                let binds = render_device.create_bind_group(
                    "pixel map bind group",
                    &layout,
                    &BindGroupEntries::sequential((
                        input_view.texture_view.into_binding(),
                        input_texture_pos_buffer.as_entire_binding(),
                        input_texture_size_buffer.as_entire_binding(),
                        source_texture_pos_buffer.as_entire_binding(),
                        source_texture_size_buffer.as_entire_binding(),
                        source_view.into_binding(),
                    )),
                );
                let layout_2 = PixelMapShaderLayoutInput::new(&render_device).bind_group_layout_2;
                let binds_2 = render_device.create_bind_group(
                    "pixel map bind group",
                    &layout_2,
                    &BindGroupEntries::sequential((
                        input_view.texture_view.into_binding(),
                        input_texture_pos_buffer.as_entire_binding(),
                        input_texture_size_buffer.as_entire_binding(),
                    )),
                );
                if !render_data.bind_map.contains_key(chunk_pos) {
                    render_data
                        .bind_map
                        .insert(*chunk_pos, vec![(binds, pixel_map.chunk_size)]);
                } else {
                    render_data
                        .bind_map
                        .get_mut(chunk_pos)
                        .unwrap()
                        .push((binds, pixel_map.chunk_size));
                }
                if !render_data.shader_map.contains_key(chunk_pos) {
                    let shader = asset_server.load(ASSETS_PATH.join("place_tex.wgsl"));
                    let pipeline =
                        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                            label: None,
                            layout: vec![layout.clone()],
                            push_constant_ranges: Vec::new(),
                            shader: shader.clone(),
                            zero_initialize_workgroup_memory: true,
                            shader_defs: vec![],
                            entry_point: Cow::from("main"),
                        });
                    assert!(render_data
                        .shader_map
                        .insert(*chunk_pos, pipeline)
                        .is_none());
                }
                if !render_data.bind_map_2.contains_key(chunk_pos) {
                    render_data
                        .bind_map_2
                        .insert(*chunk_pos, vec![(binds_2, pixel_map.chunk_size)]);
                } else {
                    render_data
                        .bind_map_2
                        .get_mut(chunk_pos)
                        .unwrap()
                        .push((binds_2, pixel_map.chunk_size));
                }
                if !render_data.shader_map_2.contains_key(chunk_pos) {
                    for shader_id in pixel_map.simulation_shaders.iter() {
                        let shader = asset_server.load(shader_id.clone());
                        let pipeline =
                            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                                label: None,
                                layout: vec![layout_2.clone()],
                                push_constant_ranges: Vec::new(),
                                zero_initialize_workgroup_memory: true,
                                shader: shader.clone(),
                                shader_defs: vec![],
                                entry_point: Cow::from("main"),
                            });
                        if let Some(vec) = render_data.shader_map_2.get_mut(chunk_pos) {
                            vec.push(pipeline);
                        } else {
                            render_data.shader_map_2.insert(*chunk_pos, vec![pipeline]);
                        }
                    }
                }
            }
        }
    }
}

fn apply_ops(
    mut render_data: ResMut<RenderData>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    render_queue: Res<RenderQueue>,
) {
    // Layer 1 operations
    for (chunk_pos, bind_groups) in render_data.bind_map.iter() {
        let pipeline_id = *render_data.shader_map.get(chunk_pos).unwrap();
        if let CachedPipelineState::Ok(_) = pipeline_cache.get_compute_pipeline_state(pipeline_id) {
            let pipeline = pipeline_cache.get_compute_pipeline(pipeline_id).unwrap();
            for binds in bind_groups.iter() {
                let mut command_encoder =
                    render_device.create_command_encoder(&CommandEncoderDescriptor::default());
                {
                    let mut pass =
                        command_encoder.begin_compute_pass(&ComputePassDescriptor::default());
                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(0, &binds.0, &[]);
                    pass.dispatch_workgroups(binds.1.x / 8, binds.1.y / 8, 1);
                }

                render_queue.submit(once(command_encoder.finish()));
            }
        }
    }

    for (chunk_pos, bind_groups) in render_data.bind_map_2.iter() {
        for pipeline_id in (*render_data.shader_map_2.get(chunk_pos).unwrap()).iter() {
            if let CachedPipelineState::Ok(_) =
                pipeline_cache.get_compute_pipeline_state(pipeline_id.clone())
            {
                let pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline_id.clone())
                    .unwrap();
                for binds in bind_groups.iter() {
                    let mut command_encoder =
                        render_device.create_command_encoder(&CommandEncoderDescriptor::default());
                    {
                        let mut pass =
                            command_encoder.begin_compute_pass(&ComputePassDescriptor::default());
                        pass.set_pipeline(pipeline);
                        pass.set_bind_group(0, &binds.0, &[]);
                        pass.dispatch_workgroups(binds.1.x / 8, binds.1.y / 8, 1);
                    }

                    render_queue.submit(once(command_encoder.finish()));
                }
            }
        }
    }

    render_data.bind_map_2.clear();
    render_data.bind_map.clear();
}

struct PixelMapShaderLayoutInput {
    pub bind_group_layout: BindGroupLayout,
    pub bind_group_layout_2: BindGroupLayout,
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
            ],
        );

        let bind_group_layout_2 = device.create_bind_group_layout(
            Some("set_pixels_cpu Bind Group Layout 2"),
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
            ],
        );

        Self {
            bind_group_layout,
            bind_group_layout_2,
        }
    }
}
