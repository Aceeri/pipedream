use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{PresentMode, WindowPosition},
};
use bevy_atmosphere::prelude::*;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use bevy_rapier3d::prelude::*;

mod controller;
mod goblin;
mod player;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            title: "mana".into(),
            width: 1280.,
            height: 1024.,
            present_mode: PresentMode::AutoNoVsync,
            position: WindowPosition::Centered,
            ..default()
        },
        ..default()
    }));
    app.add_plugin(FramepacePlugin);
    app.add_plugin(AtmospherePlugin);
    app.insert_resource(Msaa { samples: 4 })
        .add_plugin(goblin::GoblinPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_startup_system(setup)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(RapierConfiguration { ..default() });

    if cfg!(debug_assertions) {
        //app.add_plugin(WorldInspectorPlugin);
        app.add_plugin(RapierDebugRenderPlugin::default());
        app.add_plugin(LogDiagnosticsPlugin::default());
        app.add_plugin(FrameTimeDiagnosticsPlugin::default());
    }

    info!(
        "Booting with version {} (tag={}, commit={}) {}",
        env!("CARGO_PKG_VERSION"),
        option_env!("GIT_COMMIT_DESCRIBE").unwrap_or("not specified"),
        option_env!("GIT_COMMIT").unwrap_or("not specified"),
        option_env!("DIRTY").unwrap_or(""),
    );

    app.run();
}

#[derive(Component, Debug)]
pub struct Floor;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: ResMut<AssetServer>,
    //mut settings: ResMut<FramepaceSettings>,
) {
    println!("Booting...");

    //settings.limiter = Limiter::from_framerate(800.0);
    let texture_handle = asset_server.load("dirt_14.png");

    // this material renders the texture normally
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 200.0 })),
            material: material_handle,
            ..default()
        })
        .insert(GlobalTransform::default())
        .insert(Collider::cuboid(100.0, 0.01, 100.0))
        .insert(RigidBody::Fixed)
        .insert(Floor);

    //WALLS AND OBSTACLES -------------------------------------------------------------
    let positions = [
        Vec3::new(15., 7.5, -10.),
        Vec3::new(7.5, 7.5, 15.),
        Vec3::new(17., 7.5, 25.),
    ];
    for position in positions {
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(5., 15., 5.))),
                material: materials.add(Color::ORANGE.into()),
                ..default()
            })
            .insert(Transform {
                translation: position,
                ..default()
            })
            .insert(Collider::cuboid(2.5, 7.5, 2.5))
            .insert(RigidBody::Fixed);
    }
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(20., 0.2, 20.))),
            material: materials.add(Color::ORANGE.into()),
            ..default()
        })
        .insert(Transform {
            translation: Vec3::new(-30., 0.1, -5.),
            ..default()
        })
        .insert(Collider::cuboid(10., 0.1, 10.))
        .insert(RigidBody::Fixed);

    //WALLS AND OBSTACLES -------------------------------------------------------------

    //TODO:
    //mess with OrthographicProjection of shadows for better quality
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_axis_angle(Vec3::new(-0.7, 0.7, 0.), 0.8),
            ..default()
        },
        ..default()
    });
}
