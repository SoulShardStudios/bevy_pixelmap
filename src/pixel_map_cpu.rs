use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
    utils::HashMap,
};

#[derive(Component)]
pub struct PixelChunk;

#[derive(Component)]
pub struct PixelMap {
    chunk_size: UVec2,
    img_data: Vec<Handle<Image>>,
    positions: HashMap<IVec2, usize>,
    set_pixel_queue_positions: Vec<IVec2>,
    set_pixel_queue_colors: Vec<[u8; 4]>,
    pub empty_texture: Image,
    root_entity: Entity,
}

pub fn get_chunk_inner_i(position: IVec2, chunk_size: UVec2) -> UVec2 {
    UVec2 {
        x: position.x.rem_euclid(chunk_size.x as i32) as u32,
        y: position.y.rem_euclid(chunk_size.y as i32) as u32,
    }
}

pub fn get_chunk_outer_i(position: IVec2, chunk_size: UVec2) -> IVec2 {
    IVec2 {
        x: (position.x as f64 / chunk_size.x as f64).floor() as i32,
        y: (position.y as f64 / chunk_size.y as f64).floor() as i32,
    }
}

pub fn get_chunk_index_i(position: IVec2, chunk_size: UVec2) -> usize {
    let inner = get_chunk_inner_i(position, chunk_size);
    return (((chunk_size.y - inner.y - 1) * chunk_size.x) + inner.x) as usize;
}

impl PixelMap {
    pub fn new(
        chunk_size: UVec2,
        root_entity: Entity,
        empty_texture: Option<Image>,
        sampler: Option<ImageSampler>,
    ) -> Self {
        assert!(chunk_size.x > 0 && chunk_size.y > 0);

        let mut empty = empty_texture.unwrap_or_else(|| {
            Image::new_fill(
                Extent3d {
                    depth_or_array_layers: 1,
                    width: chunk_size.x,
                    height: chunk_size.y,
                },
                TextureDimension::D2,
                &[0, 0, 0, 0],
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::all(),
            )
        });

        empty.sampler = sampler.unwrap_or(ImageSampler::nearest());

        PixelMap {
            chunk_size,
            img_data: Vec::new(),
            positions: HashMap::new(),
            empty_texture: empty,
            set_pixel_queue_positions: vec![],
            set_pixel_queue_colors: vec![],
            root_entity,
        }
    }

    pub fn set_pixels(
        &mut self,
        pixel_positions: Vec<IVec2>,
        pixel_colors: Vec<[u8; 4]>,
    ) -> Result<(), &str> {
        if pixel_positions.len() != pixel_colors.len() {
            return Err("Mismatched lengths for pixel positions and colors.");
        }
        self.set_pixel_queue_positions.extend(pixel_positions);
        self.set_pixel_queue_colors.extend(pixel_colors);
        Ok(())
    }
}

fn add_pixel_map_chunks(
    mut commands: Commands,
    mut textures: ResMut<Assets<Image>>,
    mut pixel_map_query: Query<&mut PixelMap>,
) {
    for mut pixel_map in pixel_map_query.iter_mut() {
        let chunk_size = pixel_map.chunk_size;
        let empty_texture = pixel_map.empty_texture.clone();
        let root_entity = pixel_map.root_entity;

        let mut added_positions = HashMap::new();
        let mut new_img_data = Vec::new(); // Temporary storage to avoid mutable borrow issues

        for &position in &pixel_map.set_pixel_queue_positions {
            let c_pos = get_chunk_outer_i(position, chunk_size);

            if pixel_map.positions.contains_key(&c_pos) || added_positions.contains_key(&c_pos) {
                continue;
            }

            let computed_position: Vec2 = (c_pos * chunk_size.as_ivec2()).as_vec2();
            let tex_handle = textures.add(empty_texture.clone());
            let id = commands
                .spawn(SpriteBundle {
                    texture: tex_handle.clone(),
                    transform: Transform::from_xyz(computed_position.x, computed_position.y, 0.0),
                    ..Default::default()
                })
                .insert(PixelChunk)
                .id();

            commands.entity(root_entity).add_child(id);
            added_positions.insert(c_pos, pixel_map.img_data.len() + new_img_data.len());
            new_img_data.push(tex_handle);
        }

        // After the loop, extend img_data and positions
        pixel_map.positions.extend(added_positions.drain());
        pixel_map.img_data.append(&mut new_img_data);

        for (&position, &color) in pixel_map
            .set_pixel_queue_positions
            .iter()
            .zip(&pixel_map.set_pixel_queue_colors)
        {
            if let Some(&pos) = pixel_map
                .positions
                .get(&get_chunk_outer_i(position, pixel_map.chunk_size))
            {
                let ind = get_chunk_index_i(position, pixel_map.chunk_size) * 4;
                let data = &mut textures
                    .get_mut(&pixel_map.img_data[pos])
                    .expect("Mutable access to texture data")
                    .data;
                data[ind..ind + 4].copy_from_slice(&color);
            }
        }

        pixel_map.set_pixel_queue_positions.clear();
        pixel_map.set_pixel_queue_colors.clear();
    }
}

pub struct PixelMaps;

impl Plugin for PixelMaps {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, add_pixel_map_chunks);
    }
}
