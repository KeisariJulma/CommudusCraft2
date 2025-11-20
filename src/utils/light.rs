use bevy::prelude::*;

pub(crate) struct LightPluginSource;
impl Plugin for LightPluginSource {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_lights);
    }
}



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

