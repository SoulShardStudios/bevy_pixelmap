//! Displays a single [`Sprite`], created from an image.

use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, mut textures: ResMut<Assets<Image>>) {
    let height = 100;
    let width = 100;
    let new_texture = Image::new(
        Extent3d {
            depth_or_array_layers: 1,
            height,
            width,
        },
        TextureDimension::D2,
        vec![255; (width * height * 4) as usize],
        TextureFormat::Rgba8Unorm,
    );
    let handle = textures.add(new_texture);
    commands.spawn_bundle(Camera2dBundle::default());
    commands.spawn_bundle(SpriteBundle {
        texture: handle,
        ..default()
    });
}
