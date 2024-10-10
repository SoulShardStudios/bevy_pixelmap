mod chunk_position;
mod pixel_map;
extern crate bevy;
extern crate bevy_inspector_egui;
extern crate line_drawing;
extern crate rand;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use line_drawing::Bresenham;
use pixel_map::PixelMap;

use rand::random;
const WINDOW_SIZE: UVec2 = UVec2 { x: 426, y: 240 }; // 240p

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, place_uv_test)
        .add_systems(Update, place_line_test)
        .add_systems(Update, get_pixel_test)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: WINDOW_SIZE[0] as f32 * 3.0,
                height: WINDOW_SIZE[1] as f32 * 3.0,
            },
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 10.0,
        }),
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
    commands.entity(id).insert(PixelMap::new(
        UVec2 { x: 100, y: 100 },
        id,
        None,
        None,
        None,
    ));
}

fn place_uv_test(
    mut query: Query<&mut PixelMap>,
    mut textures: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    for mut pixel_map in query.iter_mut() {
        let mut pixels = vec![];
        let mut positions = vec![];

        for x in 0..255 {
            for y in 0..255 {
                let color: [u8; 4] = [x as u8, y as u8, 0, 255];
                pixels.push(color);
                positions.push(IVec2 {
                    x: x - 127,
                    y: y - 127,
                });
            }
        }
        pixel_map.set_pixels((positions, pixels), &mut commands, &mut textures);
    }
}

fn get_pixel_test(query: Query<&PixelMap>, textures: Res<Assets<Image>>) {
    for pixel_map in query.iter() {
        let _pixel = pixel_map.get_pixels(&vec![IVec2 { x: 0, y: 0 }], &textures);
        println!("{:#?}", _pixel);
    }
}

fn place_line_test(
    mut query: Query<&mut PixelMap>,
    mut textures: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    let mut count = 0;
    for mut pixel_map in query.iter_mut() {
        for _ in 0..100 {
            let color: [u8; 4] = [
                random::<u8>(),
                random::<u8>(),
                random::<u8>(),
                random::<u8>(),
            ];
            let line: Vec<IVec2> = Bresenham::new(
                (random::<i8>() as i32 - 1048, random::<i8>() as i32 - 1048),
                (random::<i8>() as i32 + 1048, random::<i8>() as i32 + 1048),
            )
            .map(|(x, y)| IVec2 { x, y })
            .collect();
            let line_len = line.len();
            count += line_len;
            pixel_map.set_pixels(
                (line, std::iter::repeat(color).take(line_len).collect()),
                &mut commands,
                &mut textures,
            );
        }
    }
    println!("{}", count);
}
