use bevy::{
    app::AppExit,
    prelude::{App, AssetServer, Assets, Commands, EventWriter, Input, KeyCode, Res},
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

fn spawn_sprite(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    meshes: Res<Assets<Mesh<Vertex>>>,
) {
    let unit_square_mesh = meshes.get_handle(UNIT_SQUARE_HANDLE);
    let texture_handle = asset_server.load("happy-tree.png");
    commands.spawn(SpriteBundle {
        mesh: unit_square_mesh,
        texture: texture_handle,
        ..Default::default()
    });

    commands.spawn(CameraBundle::<PerspectiveProjection>::default());
}

fn main() {
    let mut app = App::new();
    app.add_plugins(FlatEngineComplete)
        // .add_plugin(FlatBevyPlugins)
        // .add_plugin(bevy::core_pipeline::CorePipelinePlugin)
        // .add_plugin(bevy::sprite::SpritePlugin)
        .add_system(exit_on_esc)
        .add_startup_system(spawn_sprite)
        .run();
}
