mod devtools;
mod scene;

use bevy::prelude::*;
use wgpu_html_driver_bevy::WgpuHtmlPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "wgpu-html · Bevy Demo".into(),
                resolution: bevy::window::WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WgpuHtmlPlugin)
        .add_systems(Startup, (scene::setup, devtools::setup).chain())
        .add_systems(PreUpdate, devtools::input_system.run_if(devtools::has_devtools))
        .add_systems(Update, devtools::update_system.run_if(devtools::has_devtools))
        .run();
}
