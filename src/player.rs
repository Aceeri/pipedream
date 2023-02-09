use bevy::{prelude::*, utils::Duration};
use bevy_atmosphere::prelude::*;
use bevy_rapier3d::prelude::*;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

use crate::controller;
use crate::controller::SourceMovement;
use bevy_rapier3d::prelude::KinematicCharacterController;

pub struct PlayerPlugin;

#[derive(Debug)]
pub struct CastingState {
    pub casting: bool,
    pub last_cast: Option<Timer>,
}

#[derive(Debug)]
pub struct PlayerState {
    pub casting_state: CastingState,
    pub speed: f32,
    pub midair: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState {
            casting_state: CastingState {
                casting: false,
                last_cast: None,
            },
            speed: 50.,
            midair: false,
        }
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Serialize, Deserialize)]
pub enum PlayerAction {
    Forward,
    Back,
    Left,
    Right,
    Sprint,
    Jump,
    PrimaryAction,
}

pub struct InputBindings(InputMap<PlayerAction>);

impl Default for InputBindings {
    fn default() -> Self {
        Self(
            InputMap::new([
                (KeyCode::W, PlayerAction::Forward),
                (KeyCode::S, PlayerAction::Back),
                (KeyCode::A, PlayerAction::Left),
                (KeyCode::D, PlayerAction::Right),
                (KeyCode::LShift, PlayerAction::Sprint),
                (KeyCode::Space, PlayerAction::Jump),
            ])
            .insert(MouseButton::Left, PlayerAction::PrimaryAction)
            .build(),
        )
    }
}

#[derive(Component, Debug, Default)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub state: PlayerState,
}

#[derive(Component)]
pub struct LocalPlayer;

#[derive(Component, Default)]
pub struct NetworkPosition(pub Transform);

#[derive(Component, Debug)]
struct Health(u32);

#[derive(Component)]
pub struct PlayerCam(u64);

#[derive(Component)]
pub struct Follow(pub Option<Entity>);

#[derive(Component)]
pub struct Prediction {
    velocity: Vec3,
    grounded: bool,
}

impl Default for Prediction {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            grounded: true,
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerAction>::default())
            .add_startup_system(client_camera_spawn)
            .add_startup_system(spawn_player)
            .add_system(cursor_grab)
            .add_system(controller::controller_inputs)
            .add_fixed_timestep(controller::TICK_RATE, "fixed")
            .add_fixed_timestep_system("fixed", 0, controller::controller_movement)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                camera_follow_entity.before(bevy::transform::TransformSystem::TransformPropagate),
            );
    }
}

fn client_camera_spawn(mut commands: Commands, mut asset_server: ResMut<AssetServer>) {
    info!("booting");
    spawn_camera(&mut commands, &mut *asset_server, 1);
}

pub fn spawn_player(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    println!("Entering world...");

    let mut player_entity = commands.spawn_empty();
    player_entity
        .insert(Health(100))
        .insert(Player { id: 1, ..default() })
        .insert(Collider::capsule_y(0.5, 0.5))
        .insert(ColliderDebugColor(Color::GREEN))
        .insert(RigidBody::KinematicPositionBased)
        //.insert(LockedAxes::ROTATION_LOCKED)
        .insert(KinematicCharacterController {
            offset: CharacterLength::Absolute(1.0),
            ..default()
        })
        .insert(KinematicCharacterControllerOutput::default())
        .insert(SourceMovement::default())
        .insert(ActiveEvents::COLLISION_EVENTS);

    player_entity.insert(InputManagerBundle::<PlayerAction> {
        action_state: ActionState::default(),
        input_map: InputBindings::default().0,
    });
    player_entity.insert(LocalPlayer);

    player_entity.insert(TransformBundle::from_transform(Transform::from_xyz(
        0., 2., 0.,
    )));
}

pub fn spawn_camera(commands: &mut Commands, asset_server: &mut AssetServer, id: u64) {
    let cam = Camera3dBundle {
        projection: PerspectiveProjection {
            fov: 0.93,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };
    commands
        .spawn_empty()
        .insert(cam)
        .insert(PlayerCam(id))
        .insert(Follow(None))
        .insert(AtmosphereCamera::default())
        .insert(SceneBundle {
            scene: asset_server.load("wizard_rotated.gltf#Scene0"),
            transform: Transform::from_xyz(0., 2., 0.),
            ..default()
        });
}

fn toggle_grab_cursor(window: &mut Window) {
    if window.cursor_visible() {
        window.set_cursor_visibility(false);
        window.set_cursor_grab_mode(bevy::window::CursorGrabMode::Confined);
    } else {
        window.set_cursor_visibility(true);
        window.set_cursor_grab_mode(bevy::window::CursorGrabMode::None);
    }
}

pub fn cursor_grab(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(window);
        }
    }
}

pub fn camera_follow_entity(
    players: Query<(&Transform, &Player, &SourceMovement)>,
    mut cameras: Query<(&mut Transform, &PlayerCam), Without<Player>>,
) {
    let (mut camera, _) = cameras.single_mut();
    let (player, _, source_movement) = players.single();
    let rotate_camera_by = Quat::from_euler(
        EulerRot::ZYX,
        0.0,
        source_movement.input_state.yaw,
        source_movement.input_state.pitch,
    );
    let target_translation = player.translation + (player.forward() * 0.3) + (player.up() * 2.);
    camera.translation = camera.translation.lerp(target_translation, 0.1);
    camera.rotation = rotate_camera_by;
}
