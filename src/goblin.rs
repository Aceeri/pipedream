use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, ColliderMassProperties, RigidBody};

pub struct GoblinPlugin;

#[derive(Component, Debug)]
struct Health(u32);

#[derive(Component, Debug)]
struct Goblin {
    name: String,
}

impl Plugin for GoblinPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(startup);
    }
}

fn startup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    for i in 1..6 {
        commands
            .spawn(SceneBundle {
                scene: asset_server.load("orc_new.gltf#Scene0"),
                transform: Transform::from_xyz((-i * 3) as f32, 0.5, 0.0),
                ..default()
            })
            .insert(Health(100))
            .insert(Goblin {
                name: "Orc".to_string(),
            })
            .insert(Collider::cuboid(0.5, 0.5, 0.5))
            .insert(ColliderMassProperties::Density(75.))
            .insert(RigidBody::Dynamic);
    }
}
