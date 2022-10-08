use bevy_app::App;
use flat::FlatEngineComplete;

fn main() {
    let mut app = App::new();
    app.add_plugins(FlatEngineComplete).run();
}
