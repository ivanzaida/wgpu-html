mod devtools;
mod scene;

use bevy::prelude::*;
use lui_driver_bevy::LuiPlugin;

fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
      primary_window: Some(Window {
        title: "lui · Bevy Demo".into(),
        resolution: bevy::window::WindowResolution::new(1280, 720),
        ..default()
      }),
      ..default()
    }))
    .add_plugins(LuiPlugin)
    .add_systems(Startup, (scene::setup, devtools::setup).chain())
    .add_systems(PreUpdate, devtools::input_system.run_if(devtools::has_devtools))
    .add_systems(Update, devtools::update_system.run_if(devtools::has_devtools))
    .run();
}
