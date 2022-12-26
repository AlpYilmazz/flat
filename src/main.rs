use bevy::{
    app::AppExit,
    prelude::{
        App, AssetServer, Camera, Color, Commands, EventWriter, Input, KeyCode, Res, Transform,
        Vec3,
    },
    render::camera::CameraRenderGraph,
};
use flat::{FlatBevyPlugins, FlatEngineComplete};

fn exit_on_esc(key: Res<Input<KeyCode>>, mut app_exit: EventWriter<AppExit>) {
    if key.pressed(KeyCode::Escape) {
        app_exit.send_default();
    }
}

fn spawn_sprite(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle = asset_server.load("happy-tree.png");
    commands.spawn(bevy::prelude::Camera2dBundle::default());
    commands.spawn_batch([
        bevy::sprite::SpriteBundle {
            // sprite: todo!(),
            // transform: todo!(),
            // global_transform: todo!(),
            texture: texture_handle.clone(),
            // visibility: todo!(),
            // computed_visibility: todo!(),
            ..Default::default()
        },
        bevy::sprite::SpriteBundle {
            sprite: bevy::sprite::Sprite {
                color: Color::RED,
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(100.0, 100.0, 1.0)),
            // global_transform: todo!(),
            texture: texture_handle,
            // visibility: todo!(),
            // computed_visibility: todo!(),
            ..Default::default()
        },
        // SpriteBundle {
        //     sprite: Sprite { color: Color::RED },
        //     global_transform: Default::default(),
        //     transform: Transform::from_translation(Vec3::new(1.0, 1.0, 0.0)),
        //     texture: texture_handle,
        //     visibility: Default::default(),
        // },
    ]);
}

fn main() {
    let mut app = App::new();
    app
        // .add_plugins(FlatEngineComplete)
        .add_plugin(FlatBevyPlugins)
        // .add_plugin(bevy::core_pipeline::CorePipelinePlugin)
        // .add_plugin(bevy::sprite::SpritePlugin)
        .add_system(exit_on_esc)
        .add_startup_system(spawn_sprite)
        .run();
}
