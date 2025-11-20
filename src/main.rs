mod world;
mod utils;
use bevy::prelude::*;
use bevy::window::{ExitCondition, PresentMode, WindowMode};
use crate::world::ChunkPlugin::ChunkPlugin;
use crate::utils::light::LightPluginSource;
use crate::utils::camera::SimpleCameraPlugin;
use crate::world::seed::WorldSeed;

fn main() {
    let mut app = App::new();
        app.add_plugins(
        DefaultPlugins
            .set(bevy::log::LogPlugin {
                filter: "info,wgpu_core=warn,wgpu_hal=off,rechannel=warn".into(),
                level: bevy::log::Level::DEBUG,
                custom_layer: |_| None,
                fmt_layer:|_| None,
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Immediate,
                    title: "CC".to_string(),
                    name: Some("CC".to_string()),
                    resizable: true,
                    mode: WindowMode::Windowed,
                    resolution: (1280, 720).into(),
                    ..Default::default()
                }),
                primary_cursor_options: None,
                exit_condition: ExitCondition::OnPrimaryClosed,
                close_when_requested: true,
            }),
    );
    app.insert_resource(WorldSeed::default());
    app.add_plugins(ChunkPlugin);
    app.add_plugins(LightPluginSource);
    app.add_plugins(SimpleCameraPlugin);

    app.run();
}
