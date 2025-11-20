use bevy::ecs::query::QuerySingleError;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

use crate::world::constants::CHUNK_SIZE;

pub struct SimpleCameraPlugin;

impl Plugin for SimpleCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, camera_movement);
        app.add_systems(Update, grab_mouse);
        app.add_systems(Update, mouse_look);
        app.insert_resource(CameraSettings::default());

    }
}

#[derive(Component)]
pub(crate) struct FCamera;

#[derive(Resource)]
struct CameraSettings {
    pub speed: f32,
}

#[derive(Component)]
struct CameraRotation {
    yaw: f32,
    pitch: f32,
}



impl Default for CameraSettings {
    fn default() -> Self {
        Self { speed: 3.0 }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        FCamera,
        CameraRotation { yaw: 0.0, pitch: 0.0 },
    ));
}


fn camera_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<FCamera>>,
    settings: Res<CameraSettings>,
) {
    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let forward = transform.rotation * Vec3::Z;
        let right = transform.rotation * Vec3::X;

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction -= forward;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction += forward;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction -= right;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction += right;
        }
        if keyboard_input.pressed(KeyCode::Space) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            direction.y -= 1.0;
        }

        if direction != Vec3::ZERO {
            transform.translation += direction.normalize() * settings.speed;
        }
    }
}


fn grab_mouse(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut cursor_opts: Query<&mut CursorOptions>,
) {
    if let Ok(mut opts) = cursor_opts.single_mut() {
        if keyboard_input.just_pressed(KeyCode::KeyG) {
            opts.visible = false;
            opts.grab_mode = CursorGrabMode::Locked;
        }

        if keyboard_input.just_pressed(KeyCode::KeyU) {
            opts.visible = true;
            opts.grab_mode = CursorGrabMode::None;
        }
    }
}


fn mouse_look(
    mut motion_evr: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut CameraRotation), With<FCamera>>,
) {
    let (mut transform, mut rotation) = query.single_mut().expect("Camera not found");


    let sensitivity = 0.002; // adjust as needed

    for ev in motion_evr.read() {
        rotation.yaw -= ev.delta.x * sensitivity;
        rotation.pitch -= ev.delta.y * sensitivity;
    }

    // Clamp pitch so camera won't flip
    rotation.pitch = rotation.pitch.clamp(-1.54, 1.54);

    // Apply yaw (Y-axis) and pitch (local X-axis)
    transform.rotation =
        Quat::from_axis_angle(Vec3::Y, rotation.yaw) *
        Quat::from_axis_angle(Vec3::X, rotation.pitch);
}


pub fn player_chunk_pos(transform: Result<&Transform, QuerySingleError>) -> (i32, i32) {
    if let Ok(transform) = transform {
        let x = (transform.translation.x / CHUNK_SIZE as f32).floor() as i32;
        let z = (transform.translation.z / CHUNK_SIZE as f32).floor() as i32;
        (x, z)
    } else {
        (0, 0) // fallback if camera not found
    }
}
