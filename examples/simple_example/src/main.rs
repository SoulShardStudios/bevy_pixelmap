use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_pixelmap::*;
extern crate bevy;
extern crate rand;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use line_drawing::Bresenham;

use rand::random;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (place_line_test_cpu, get_pixel_test_cpu, place_tex_test_gpu),
        )
        .add_plugins(PixelMapGpuComputePlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .run();
}

#[derive(Resource)]
struct Imgs(Vec<Handle<Image>>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d::default(),
        Transform::from_translation(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 10.0,
        }),
    ));
    let id = commands
        .spawn(Transform::default())
        .insert(Visibility::Visible)
        .id();
    commands.entity(id).insert(PixelMap::new(
        UVec2 { x: 1000, y: 1000 },
        id,
        None,
        None,
        None,
    ));

    commands.insert_resource(Imgs(vec![
        asset_server.load("images/1.png"),
        asset_server.load("images/2.png"),
    ]));
    commands.insert_resource(Lines(Handle::default()));
}

fn get_pixel_test_cpu(query: Query<&PixelMap>, textures: Res<Assets<Image>>) {
    for pixel_map in query.iter() {
        let _pixel = pixel_map.get_pixels_cpu(&vec![IVec2 { x: 0, y: 0 }], &textures);
    }
}

#[derive(Resource)]
struct Lines(Handle<Image>);

fn place_line_test_cpu(
    mut query: Query<&mut PixelMap>,
    mut textures: ResMut<Assets<Image>>,
    mut lines: ResMut<Lines>,
) {
    let mut count = 0;
    for mut pixel_map in query.iter_mut() {
        let mut pixels = vec![0; 255 * 255 * 4];
        for _ in 0..100 {
            let color: [u8; 4] = [random::<u8>(), random::<u8>(), random::<u8>(), 255];
            Bresenham::new(
                (random::<i8>() as i32, random::<i8>() as i32),
                (random::<i8>() as i32, random::<i8>() as i32),
            )
            .map(|(x, y)| IVec2 { x, y })
            .for_each(|pos| {
                let adjusted_pos = IVec2 {
                    x: pos.x + 128,
                    y: pos.y + 128,
                };
                let hopefully = (adjusted_pos.x as usize + adjusted_pos.y as usize * 255) * 4;
                if hopefully + 4 <= pixels.len() {
                    pixels[hopefully..hopefully + 4].copy_from_slice(&color);
                    count += 1;
                }
            });
        }
        let img = Image::new(
            Extent3d {
                width: 255,
                height: 255,
                ..Default::default()
            },
            TextureDimension::D2,
            pixels,
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::all(),
        );
        textures.remove(lines.0.id());
        let handle = textures.add(img);
        lines.0 = handle.clone();
        pixel_map.set_pixels_gpu(
            vec![PixelPositionedTexture {
                position: IVec2::new(random::<i8>() as i32 * 20, random::<i8>() as i32 * 20),
                image: handle,
                size: UVec2 { x: 255, y: 255 },
            }],
            &mut textures,
        );
    }
}

fn place_tex_test_gpu(
    mut query: Query<&mut PixelMap>,
    mut textures: ResMut<Assets<Image>>,
    imgs: Res<Imgs>,
) {
    let ok = textures.get(imgs.0[0].id()).is_some() && textures.get(imgs.0[1].id()).is_some();
    for mut pixel_map in query.iter_mut() {
        if ok {
            pixel_map.set_pixels_gpu(
                vec![
                    PixelPositionedTexture {
                        image: imgs.0[0].clone(),
                        position: IVec2::new(
                            random::<i8>() as i32 * 20,
                            random::<i8>() as i32 * 20,
                        ),
                        size: UVec2::new(860, 888),
                    },
                    PixelPositionedTexture {
                        image: imgs.0[1].clone(),
                        position: IVec2::new(
                            random::<i8>() as i32 * 20,
                            random::<i8>() as i32 * 20,
                        ),
                        size: UVec2::new(860, 219),
                    },
                ]
                .iter()
                .cloned()
                .cycle()
                .take(8)
                .collect::<Vec<_>>(),
                &mut textures,
            );
        }
    }
}
