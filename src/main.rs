//! Displays a single [`Sprite`], created from an image.
mod chunk_position;
mod pixel_map;
use bevy::{prelude::*, render::camera::ScalingMode, utils::HashMap};
use bevy_inspector_egui::WorldInspectorPlugin;
use pixel_map::{PixelMap, PixelMaps};

const WINDOW_SIZE: UVec2 = UVec2 { x: 426, y: 240 }; // 240p

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
    let half = WINDOW_SIZE / 2;
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::None,
            bottom: -(half.y as f32),
            top: half.y as f32,
            left: -(half.x as f32),
            right: half.x as f32,

            ..Default::default()
        },
        ..Default::default()
    });
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
        .insert(PixelMap::new(UVec2 { x: 100, y: 100 }, id, None, None));
}

fn place_pixels(mut query: Query<&mut PixelMap>) {
    for mut pixel_map in query.iter_mut() {
        let mut pixels = HashMap::new();

        let black: [u8; 4] = [0, 0, 0, 255];
        let white: [u8; 4] = [255, 255, 255, 255];

        for x in 0..200 {
            for y in 0..200 {
                let ind = (x + y) % 2 == 0;
                pixels.insert(
                    IVec2 {
                        x: x - 100,
                        y: y - 100,
                    },
                    if ind { black } else { white },
                );
            }
        }
        pixel_map.set_pixels(pixels);
    }
}
