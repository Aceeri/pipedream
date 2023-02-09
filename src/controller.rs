use bevy::{audio::Source, input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::player::{self, Player, PlayerAction, PlayerCam};

const SENS: f32 = 0.02736;
pub static TICK_RATE: std::time::Duration = std::time::Duration::from_millis(32);

#[derive(Component)]
pub struct SourceMovement {
    pub velocity: Vec3,
    pub grounded: bool,
    pub inputs: Vec<PlayerAction>,
    pub input_state: InputState,
    pub direction: Vec3,
    sprinting: bool,
    primary_action: bool,
}

#[derive(Debug, Default, Resource)]
pub struct InputState {
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for SourceMovement {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            grounded: true,
            inputs: Vec::new(),
            input_state: InputState::default(),
            direction: Vec3::ZERO,
            sprinting: false,
            primary_action: false,
        }
    }
}

pub fn controller_inputs(
    mut input_query: Query<(&ActionState<PlayerAction>, &mut SourceMovement)>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    windows: Res<Windows>,
) {
    if input_query.is_empty() {
        return;
    }
    let (actions, mut source_movement) = input_query.single_mut();

    //MOUSE
    if let Some(w) = windows.get_primary() {
        if !w.is_focused() || w.cursor_visible() {
            return;
        }
    }

    for event in mouse_motion_events.iter() {
        let yaw = (SENS * event.delta.x).to_radians();
        let pitch = (SENS * event.delta.y).to_radians();
        source_movement.input_state.pitch -= pitch;
        source_movement.input_state.yaw -= yaw;
    }
    source_movement.input_state.pitch = source_movement.input_state.pitch.clamp(-1.5, 1.5);

    //KEYBOARD
    let pressed = actions.get_pressed();
    let mut direction = Vec3::ZERO;
    for action in pressed {
        match action {
            PlayerAction::Forward => direction += Vec3::Z,
            PlayerAction::Back => direction -= Vec3::Z,
            PlayerAction::Left => direction -= Vec3::X,
            PlayerAction::Right => direction += Vec3::X,
            PlayerAction::Jump => direction += Vec3::Y,
            PlayerAction::Sprint => source_movement.sprinting = true,
            _ => {}
        }
    }
    let released = actions.get_just_released();
    for action in released {
        match action {
            PlayerAction::Sprint => source_movement.sprinting = false,
            _ => {}
        }
    }
    source_movement.direction = direction;
}

pub fn controller_movement(
    mut player_query: Query<
        (
            &mut Transform,
            &mut KinematicCharacterController,
            &KinematicCharacterControllerOutput,
            &SourceMovement,
        ),
        With<Player>,
    >,
    mut camera_query: Query<&mut Transform, (With<PlayerCam>, Without<Player>)>,
) {
    let mut camera = camera_query.single_mut();
    let local_z = camera.local_z();
    let forward = -Vec3::new(local_z.x, 0., local_z.z);
    let right = Vec3::new(local_z.z, 0., -local_z.x);

    for (mut player, mut controller, controller_output, source_movement) in player_query.iter_mut()
    {
        let unit_direction = (source_movement.direction.z * forward
            + source_movement.direction.x * right)
            .normalize_or_zero();
        let mut accelerate = match controller_output.grounded {
            false => {
                source_air_movement(
                    unit_direction, //+ Vec3::new(0., -5., 0.),
                    controller_output.effective_translation,
                    0.5,
                    1.5,
                    TICK_RATE.as_secs_f32(),
                ) + Vec3::new(0., -0.01 * 2., 0.)
            }
            true => {
                source_ground_movement(
                    unit_direction,
                    controller_output.effective_translation,
                    120.0,
                    20.0,
                    2.,
                    TICK_RATE.as_secs_f32(),
                ) * if source_movement.sprinting { 2.2 } else { 1. }
            }
        };

        if source_movement.direction.y == 1.0 && controller_output.grounded {
            accelerate.y = 0.95;
        }

        //Move player
        let translate_by = controller_output
            .effective_translation
            .lerp(accelerate, 1.0);
        //controller.translation = Some(translate_by);
        player.translation = player.translation + (unit_direction / 3.);

        //Rotate player
        player.rotation = Quat::from_axis_angle(Vec3::Y, source_movement.input_state.yaw);
    }
}

fn source_accelerate(
    unit_direction: Vec3,
    previous_velocity: Vec3,
    accelerate: f32,
    max_velocity: f32,
    dt: f32,
) -> Vec3 {
    //Projection of current velocity onto the acceleration direction
    let projected_velocity = Vec3::dot(previous_velocity, unit_direction);
    //Accelerated velocity in the direction of movement
    let mut acceleration_velocity = accelerate * dt;
    if projected_velocity + acceleration_velocity > max_velocity {
        acceleration_velocity = max_velocity - projected_velocity;
    }
    let accelerated = previous_velocity + unit_direction * acceleration_velocity;
    accelerated
}

fn source_ground_movement(
    unit_direction: Vec3,
    mut previous_velocity: Vec3,
    friction: f32,
    ground_accelerate: f32,
    max_ground_velocity: f32,
    dt: f32,
) -> Vec3 {
    let speed = previous_velocity.length();
    if speed != 0.0 {
        let drop = speed * friction * dt;
        previous_velocity *= f32::max(speed - drop, 0.0) / speed;
    }
    source_accelerate(
        unit_direction,
        previous_velocity,
        ground_accelerate,
        max_ground_velocity,
        dt,
    )
}

fn source_air_movement(
    unit_direction: Vec3,
    previous_velocity: Vec3,
    air_acceleration: f32,
    max_air_velocity: f32,
    dt: f32,
) -> Vec3 {
    source_accelerate(
        unit_direction,
        previous_velocity,
        air_acceleration,
        max_air_velocity,
        dt,
    )
}
