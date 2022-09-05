//! Displays a single [`Sprite`], created from an image.
mod chunk_position;
mod pixel_map;
use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_inspector_egui::WorldInspectorPlugin;
use pixel_map::{PixelMap, PixelMaps};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(place_pixels)
        .add_plugin(PixelMaps)
        .add_plugin(WorldInspectorPlugin::new())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
    let id = commands
        .spawn()
        .insert_bundle(TransformBundle {
            ..Default::default()
        })
        .insert_bundle(VisibilityBundle {
            ..Default::default()
        })
        .id();
    commands
        .entity(id)
        .insert(PixelMap::new(UVec2 { x: 100, y: 100 }, None, id));
}

fn place_pixels(mut query: Query<&mut PixelMap>) {
    for mut pixel_map in query.iter_mut() {
        let mut color: [u8; 4] = [0, 0, 0, 255];

        for x in 0..200 {
            for y in 0..200 {
                color[0] = color[0].wrapping_add(1);
                color[1] = color[1].wrapping_add(1);
                color[2] = color[2].wrapping_add(1);
                pixel_map.set_pixel(
                    IVec2 {
                        x: x - 100,
                        y: y - 100,
                    },
                    color,
                );
            }
        }
    }
}
