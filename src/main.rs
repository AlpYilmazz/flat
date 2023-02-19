use bevy::{
    app::AppExit,
    prelude::{App, AssetServer, Assets, Commands, EventWriter, Input, KeyCode, Res, Transform, Vec3, Component, Query, With}, time::Time,
};
use flat::{
    render::{
        camera::component::{CameraBundle, PerspectiveProjection},
        mesh::Mesh,
        resource::buffer::Vertex,
    },
    sprite::{bundle::SpriteBundle, UNIT_SQUARE_HANDLE},
    FlatEngineComplete,
};

fn exit_on_esc(key: Res<Input<KeyCode>>, mut app_exit: EventWriter<AppExit>) {
    if key.pressed(KeyCode::Escape) {
        app_exit.send_default();
    }
}

#[derive(Component)]
struct Player;

fn spawn_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    meshes: Res<Assets<Mesh<Vertex>>>,
) {
    let unit_square_mesh = meshes.get_handle(UNIT_SQUARE_HANDLE);
    let texture_handle = asset_server.load("happy-tree.png");
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_scale(Vec3::new(10.0, 10.0, 10.0)),
            mesh: unit_square_mesh,
            texture: texture_handle,
            ..Default::default()
        },
        Player,
    ));

    // TODO: Sprite does not show
    commands.spawn(CameraBundle::<PerspectiveProjection> {
        transform: Transform::from_xyz(0.0, 0.0, 20.0),
        ..Default::default()
    });
}

fn control_player(key: Res<Input<KeyCode>>, mut player: Query<&mut Transform, With<Player>>) {
    const SPEED: f32 = 0.4;

    let dif = SPEED
        * if key.pressed(KeyCode::W) {
            Vec3::NEG_Z
        } else if key.pressed(KeyCode::A) {
            Vec3::NEG_X
        } else if key.pressed(KeyCode::S) {
            Vec3::Z
        } else if key.pressed(KeyCode::D) {
            Vec3::X
        } else {
            Vec3::ZERO
        };

    for mut player_transform in player.iter_mut() {
        player_transform.translation += dif;
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(FlatEngineComplete)
        // .add_plugin(FlatBevyPlugins)
        // .add_plugin(bevy::core_pipeline::CorePipelinePlugin)
        // .add_plugin(bevy::sprite::SpritePlugin)
        .add_system(exit_on_esc)
        .add_startup_system(spawn_sprite)
        .add_system(control_player)
        .run();
}
