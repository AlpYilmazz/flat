use bevy::{
    app::AppExit,
    prelude::{
        App, AssetServer, Assets, Commands, Component, EventWriter, GlobalTransform, Input,
        KeyCode, Query, Res, ResMut, Transform, Vec3, With, Without, BuildChildren,
    },
};
use flat::{
    mesh3d::{bind::MeshPipelineKey, bundle::MeshBundle},
    render::{
        camera::component::{Camera, CameraBundle, PerspectiveProjection, Visibility},
        mesh::Mesh,
        resource::buffer::{Vertex, VertexC, VertexTex3, VertexBase},
        system::RenderFunctionId,
        texture::texture_arr::ImageArrayHandle,
        uniform::{Color, Radius},
    },
    shapes::skybox,
    sprite::{
        bundle::{SpriteBundle, SimpleCircleBundle, SimpleTriangleBundle}, BASE_QUAD_COLORED_HANDLE, BASE_QUAD_HANDLE, CIRCLE_RENDER_FUNCTION, BASE_TRIANGLE_HANDLE,
    },
    util::{Primary, PrimaryEntity},
    FlatEngineComplete,
};

fn exit_on_esc(key: Res<Input<KeyCode>>, mut app_exit: EventWriter<AppExit>) {
    if key.pressed(KeyCode::Escape) {
        app_exit.send_default();
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Skybox;

#[derive(Component)]
struct Player2;

fn spawn_objects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    base_meshes: Res<Assets<Mesh<VertexBase>>>,
    colored_meshes: Res<Assets<Mesh<VertexC>>>,
    mut meshes_tex3: ResMut<Assets<Mesh<VertexTex3>>>,
) {
    let base_quad_colored = colored_meshes.get_handle(BASE_QUAD_COLORED_HANDLE);
    let texture_handle = asset_server.load("happy-tree.png");
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_scale(Vec3::new(10.0, 10.0, 1.0)),
            mesh: base_quad_colored,
            texture: texture_handle,
            ..Default::default()
        },
        Player,
    ));

    let radius = 0.2;
    let radius_scale = 10.0;
    let real_radius = radius * radius_scale;
    let player2_circle = commands.spawn((
        SimpleCircleBundle {
            transform: Transform::from_scale(Vec3::new(radius_scale, radius_scale, 1.0)),
            radius: Radius(radius),
            color: Color(1.0, 0.0, 0.0, 0.6),
            ..Default::default()
        },
        Player2,
    )).id();

    let unit_triangle = base_meshes.get_handle(BASE_TRIANGLE_HANDLE);
    let triangle1 = commands.spawn((
        SimpleTriangleBundle {
            transform: Transform::from_scale(Vec3::new(1.0, 1.0, 1.0))
                * Transform::from_translation(Vec3::new(0.0, radius, 0.0)),
            mesh: unit_triangle.clone(),
            color: Color(0.0, 0.0, 1.0, 0.5),
            ..Default::default()
        },
    )).id();

    let triangle2 = commands.spawn((
        SimpleTriangleBundle {
            transform: Transform::from_scale(Vec3::new(0.3, 0.3, 1.0))
                * Transform::from_translation(Vec3::new(0.0, radius, 0.0)),
            mesh: unit_triangle,
            color: Color(0.0, 1.0, 0.0, 0.5),
            ..Default::default()
        },
    )).id();

    commands.entity(player2_circle).add_child(triangle1).add_child(triangle2);

    let skybox_mesh = meshes_tex3.add(skybox::create_skybox());
    let skybox_images = skybox::SIDES
        .iter()
        .map(|side| asset_server.load(format!("skybox/{side}.just.jpg")))
        .collect();
    commands.spawn((
        MeshBundle {
            transform: Transform::from_scale(Vec3::new(1000.0, 1000.0, 1000.0)),
            mesh: skybox_mesh,
            textures: ImageArrayHandle::with_images(skybox_images),
            render_key: MeshPipelineKey { texture_count: 6 },
            ..Default::default()
        },
        Skybox,
    ));

    let primary_camera = commands
        .spawn(CameraBundle::<PerspectiveProjection> {
            transform: Transform::from_xyz(0.0, 0.0, 20.0),
            ..Default::default()
        })
        .id();

    commands.insert_resource(PrimaryEntity::<Camera>::new(primary_camera));
}

fn control_player(
    key: Res<Input<KeyCode>>,
    mut player: Query<&mut Transform, With<Player>>,
    mut player2: Query<&mut Transform, (With<Player2>, Without<Player>)>,
) {
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
    for mut player2_transform in player2.iter_mut() {
        player2_transform.translation -= dif;
    }
}

fn skybox_follows_primary_camera(
    primary_camera: Primary<Camera>,
    mut skybox: Query<&mut Transform, With<Skybox>>,
    cameras: Query<&GlobalTransform, With<Camera>>,
) {
    let camera_transform = cameras.get(primary_camera.get()).unwrap();
    let mut skybox_transform = skybox.single_mut();
    skybox_transform.translation = camera_transform.translation();
}

fn main() {
    let mut app = App::new();
    app.add_plugins(FlatEngineComplete)
        // .add_plugin(FlatBevyPlugins)
        // .add_plugin(bevy::core_pipeline::CorePipelinePlugin)
        // .add_plugin(bevy::sprite::SpritePlugin)
        .add_system(exit_on_esc)
        .add_startup_system(spawn_objects)
        .add_system(skybox_follows_primary_camera)
        .add_system(control_player)
        .run();
}
