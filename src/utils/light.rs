use bevy::prelude::*;

pub(crate) struct LightPlugin;

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add ambient light for the whole world
            .insert_resource(AmbientLight {
                color: Color::srgb(0.8, 0.8, 0.8), // light gray ambient light
                brightness: 1.0, // full brightness
                affects_lightmapped_meshes: false,
            });


        app.add_systems(Startup, spawn_sun);
        app.add_systems(Update, toggle_fullbright);
        app.insert_resource(Fullbright(false));
    }
}


#[derive(Resource)]
pub struct Fullbright(pub bool);

pub fn setup_lights(mut commands: Commands) {
    commands.spawn((
        PointLight {
            intensity: 1000.0,
            range: 100.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(0.0, 10.0, 10.0)),
        GlobalTransform::default(),
    ));
}

fn spawn_sun(mut commands: Commands) {
    // Directional light pointing downwards
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 50000.0, // strong sunlight
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -std::f32::consts::FRAC_PI_4, 0.0, 0.0)),
        GlobalTransform::default(),
    ));
}

fn toggle_fullbright(input: Res<ButtonInput<KeyCode>>, mut fullbright: ResMut<Fullbright>) {
    if input.just_pressed(KeyCode::KeyF) {
        fullbright.0 = !fullbright.0;
        info!("Fullbright: {}", fullbright.0);
    }
}
