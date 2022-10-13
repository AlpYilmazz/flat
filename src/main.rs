use bevy_app::App;
use bevy_ecs::{
    query::With,
    system::{Query, Res},
};
use cgmath::{Vector3, Zero};
use flat::{
    input::{keyboard::KeyCode, Input},
    render::Player,
    transform::Transform,
    FlatEngineComplete,
};

fn control_cube(key: Res<Input<KeyCode>>, mut cube: Query<&mut Transform, With<Player>>) {
    const SPEED: f32 = 0.2;

    let dif = SPEED
        * if key.pressed(KeyCode::W) {
            -Vector3::unit_z()
        } else if key.pressed(KeyCode::A) {
            -Vector3::unit_x()
        } else if key.pressed(KeyCode::S) {
            Vector3::unit_z()
        } else if key.pressed(KeyCode::D) {
            Vector3::unit_x()
        } else {
            Vector3::zero()
        };

    let mut player_transform = cube.get_single_mut().unwrap();
    player_transform.translation += dif;
}

fn main() {
    let mut app = App::new();
    app.add_plugins(FlatEngineComplete)
        .add_system(control_cube)
        .run();
}
