use bevy_app::{App, AppExit, StartupStage};
use bevy_asset::{Assets, Handle, HandleId};
use bevy_ecs::{
    prelude::{Component, EventWriter},
    query::With,
    system::{Commands, Local, Query, Res, ResMut},
};
use cgmath::{Deg, Quaternion, Rotation3, Vector3, Zero};
use flat::{
    input::{keyboard::KeyCode, Input},
    render::{
        camera::{Camera, PerspectiveCameraBundle, Visible},
        mesh::{self, Mesh},
        resource::{buffer::Vertex, pipeline::RenderPipeline},
        RenderCamera, RenderEntityBundle,
    },
    shaders::{ShaderInstance, TestWgsl},
    transform::{GlobalTransform, Transform},
    util::{store, AssetStore, Primary, PrimaryEntity, Refer, Store},
    window::{events::CreateWindow, WindowDescriptor, WindowId, Windows},
    FlatEngineComplete,
};

fn test_create_pipeline_test_wgsl(
    device: Res<wgpu::Device>,
    mut render_pipelines: ResMut<Store<RenderPipeline>>,
    mut refer_pipelines: ResMut<AssetStore<Refer<RenderPipeline>>>,
) {
    let handle_id = HandleId::from("test.wgsl");
    let refer = store(&mut render_pipelines, TestWgsl::pipeline(&device));
    refer_pipelines.insert(handle_id, refer);
}

fn test_create_primary_camera(mut commands: Commands) {
    let camera_bundle = PerspectiveCameraBundle::new(WindowId::primary()); // Primary Window
    let primary_camera = commands.spawn().insert_bundle(camera_bundle).id();

    commands.insert_resource(PrimaryEntity::<Camera>::new(primary_camera));
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct MainPlayer;

fn test_create_render_entity(
    mut commands: Commands,
    primary_camera: Primary<Camera>,
    mut meshes: ResMut<Assets<Mesh<Vertex>>>,
    pipelines: Res<AssetStore<Refer<RenderPipeline>>>,

    mut windows: ResMut<Windows>,
    mut create_window_event_writer: EventWriter<CreateWindow>,
) {
    let pipeline = pipelines.get(&HandleId::from("test.wgsl")).unwrap().clone();
    let render_camera = RenderCamera(primary_camera.get());

    let transform = Transform {
        rotation: Quaternion::from_axis_angle(Vector3::unit_y(), Deg(45.0)),
        ..Default::default()
    };
    let global_transform = GlobalTransform::from(transform);

    let cube_mesh = mesh::primitive::create_unit_cube();
    let handle_cube_mesh = meshes.set(HandleId::random::<Mesh<Vertex>>(), cube_mesh);

    commands
        .spawn_bundle(RenderEntityBundle {
            pipeline,
            render_camera,
            global_transform: global_transform.clone(),
            transform: transform.clone(),
            render_asset: handle_cube_mesh,
        })
        .insert(Visible)
        .insert(Player)
        .insert(MainPlayer);

    let second_window_id = windows.reserve_id();
    create_window_event_writer.send(CreateWindow {
        id: second_window_id,
        desc: WindowDescriptor {
            title: Some("Second Window".to_string()),
        },
    });

    let camera_bundle = PerspectiveCameraBundle::new(second_window_id); // Primary Window
    let second_window_camera = commands.spawn().insert_bundle(camera_bundle).id();

    let cube_mesh = mesh::primitive::create_unit_cube();
    let handle_cube_mesh = meshes.set(HandleId::random::<Mesh<Vertex>>(), cube_mesh);

    commands
        .spawn_bundle(RenderEntityBundle {
            pipeline,
            render_camera: RenderCamera(second_window_camera),
            global_transform,
            transform,
            render_asset: handle_cube_mesh,
        })
        .insert(Visible)
        .insert(Player);
}

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

    for mut player_transform in cube.iter_mut() {
        player_transform.translation += dif;
    }
}

fn test_asset_remove(
    mut local_asset: Local<Option<Mesh<Vertex>>>,
    key: Res<Input<KeyCode>>,
    mut assets: ResMut<Assets<Mesh<Vertex>>>,
    main_player: Query<&Handle<Mesh<Vertex>>, With<MainPlayer>>,
) {
    if key.just_pressed(KeyCode::Space) {
        let handle = main_player.single();
        match local_asset.as_ref() {
            Some(_) => {
                let asset = local_asset.take().unwrap();
                assets.set_untracked(handle, asset);
            }
            None => {
                let asset = assets.remove(handle).unwrap();
                local_asset.replace(asset);
            }
        }
    }
}

fn exit_on_esc(key: Res<Input<KeyCode>>, mut app_exit: EventWriter<AppExit>) {
    if key.pressed(KeyCode::Escape) {
        app_exit.send_default();
    }
}

fn main() {
    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        title: Some("Primary Window".to_string()),
    })
    .add_plugins(FlatEngineComplete)
    .add_startup_system_to_stage(StartupStage::PreStartup, test_create_pipeline_test_wgsl)
    .add_startup_system_to_stage(StartupStage::PreStartup, test_create_primary_camera)
    .add_startup_system(test_create_render_entity)
    .add_system(control_cube)
    .add_system(test_asset_remove)
    .add_system(exit_on_esc)
    .run();
}
