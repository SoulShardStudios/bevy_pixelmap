//! Displays a single [`Sprite`], created from an image.
mod chunk_position;
mod pixel_map;
use bevy::{prelude::*, render::camera::ScalingMode, utils::HashMap};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use line_drawing::Bresenham;
use pixel_map::{PixelMap, PixelMaps};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

const WINDOW_SIZE: UVec2 = UVec2 { x: 426, y: 240 }; // 240p

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(place_uv_test)
        .add_system(place_line_test)
        .add_plugin(PixelMaps)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(WorldInspectorPlugin::new())
        .run();
}

fn setup(mut commands: Commands) {
    let half = WINDOW_SIZE / 2;
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::Fixed { width: WINDOW_SIZE[0] as f32, height: WINDOW_SIZE[1] as f32 },
            ..Default::default()
        },
        ..Default::default()
    });
    let id = commands
        .spawn(TransformBundle {
            ..Default::default()
        }, )
        .insert(VisibilityBundle {
            ..Default::default()
        })
        .id();
    commands
        .entity(id)
        .insert(PixelMap::new(UVec2 { x: 100, y: 100 }, id, None, None));
}

fn place_uv_test(mut query: Query<&mut PixelMap>) {
    for mut pixel_map in query.iter_mut() {
        let mut pixels = HashMap::new();

        for x in 0..255 {
            for y in 0..255 {
                let color: [u8; 4] = [x as u8, y as u8, 0, 255];
                pixels.insert(
                    IVec2 {
                        x: x - 127,
                        y: y - 127,
                    },
                    color,
                );
            }
        }
        pixel_map.set_pixels(pixels);
    }
}

fn place_line_test(mut query: Query<&mut PixelMap>) {
    let color: [u8; 4] = [255, 255, 255, 255];
    for mut pixel_map in query.iter_mut() {
        let pixels = HashMap::from_iter(
            Bresenham::new((-100, -100), (50, 75)).map(|(x, y)| (IVec2 { x: x, y: y }, color)),
        );
        pixel_map.set_pixels(pixels)
    }
}
