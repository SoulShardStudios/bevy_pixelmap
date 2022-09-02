//! Displays a single [`Sprite`], created from an image.
mod pixel_map;
use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use pixel_map::{PixelMap, PixelMaps};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(spawn_shit)
        .add_plugin(PixelMaps)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands
        .spawn()
        .insert(PixelMap::new(UVec2 { x: 100, y: 100 }, 100, None));
}

fn spawn_shit(mut query: Query<&mut PixelMap>) {
    for mut pixel_map in query.iter_mut() {
        let image_2 = Image::new(
            Extent3d {
                depth_or_array_layers: 1,
                width: pixel_map.chunk_size.x,
                height: pixel_map.chunk_size.y,
            },
            TextureDimension::D2,
            vec![255; (pixel_map.chunk_size.x * pixel_map.chunk_size.y * 4) as usize],
            TextureFormat::Rgba8Unorm,
        );
        pixel_map.add_chunk(IVec2 { x: 0, y: 0 }, None);
        pixel_map.add_chunk(IVec2 { x: 1, y: 0 }, None);
        pixel_map.add_chunk(IVec2 { x: 0, y: 1 }, None);
        pixel_map.add_chunk(IVec2 { x: 1, y: 1 }, Some(image_2));
    }
}
