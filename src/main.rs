mod pixel_map_cpu;
mod pixel_map_gpu;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use line_drawing::Bresenham;
use pixel_map_cpu::{PixelMap, PixelMaps};
use rand::random;
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
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: WINDOW_SIZE[0] as f32,
                height: WINDOW_SIZE[1] as f32,
            },
            ..Default::default()
        },
        ..Default::default()
    });
    let id = commands
        .spawn(TransformBundle {
            ..Default::default()
        })
        .insert(VisibilityBundle {
            ..Default::default()
        })
        .id();
    commands
        .entity(id)
        .insert(PixelMap::new(UVec2 { x: 100, y: 100 }, id, None, None));
}

fn place_uv_test(mut query: Query<&mut PixelMap>) {
    // for mut pixel_map in query.iter_mut() {
    //     let mut pixels = vec![];
    //     let mut positions = vec![];
    //
    //     for x in 0..255 {
    //         for y in 0..255 {
    //             let color: [u8; 4] = [x as u8, y as u8, 0, 255];
    //             pixels.push(color);
    //             positions.push(IVec2 {
    //                 x: x - 127,
    //                 y: y - 127,
    //             });
    //         }
    //     }
    //     pixel_map.set_pixels(positions, pixels).unwrap();
    // }
}

fn place_line_test(mut query: Query<&mut PixelMap>) {
    for mut pixel_map in query.iter_mut() {
        for _ in 0..100 {
            let color: [u8; 4] = [
                random::<u8>(),
                random::<u8>(),
                random::<u8>(),
                random::<u8>(),
            ];
            let line: Vec<IVec2> = Bresenham::new(
                (random::<i8>() as i32, random::<i8>() as i32),
                (random::<i8>() as i32, random::<i8>() as i32),
            )
            .map(|(x, y)| IVec2 { x: x, y: y })
            .collect();
            let line_len = line.len();
            pixel_map
                .set_pixels(line, std::iter::repeat(color).take(line_len).collect())
                .unwrap();
        }
    }
}
