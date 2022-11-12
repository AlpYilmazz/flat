use bevy::{prelude::*, app::AppExit};
use flat::FlatEngineComplete;

fn exit_on_esc(key: Res<Input<KeyCode>>, mut app_exit: EventWriter<AppExit>) {
    if key.pressed(KeyCode::Escape) {
        app_exit.send_default();
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(FlatEngineComplete)
        .add_system(exit_on_esc)
        .run();
}
