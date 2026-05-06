use bevy::prelude::*;
use wgpu_html_driver_bevy::HtmlOverlay;

pub fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  mut html: NonSendMut<HtmlOverlay>,
) {
  commands.spawn((
    Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
    MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
  ));
  commands.spawn((
    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.3))),
    Transform::from_xyz(0.0, 0.5, 0.0),
  ));
  commands.spawn((
    PointLight { shadows_enabled: true, intensity: 2_000_000.0, ..default() },
    Transform::from_xyz(4.0, 8.0, 4.0),
  ));
  commands.spawn((
    Camera3d::default(),
    Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
  ));

  let parsed = wgpu_html::parser::parse(r#"
<style>
  .hud {
    font-family: system-ui, sans-serif;
    padding: 24px;
    max-width: 400px;
  }
  .card {
    background: rgba(15, 15, 35, 0.85);
    border: 1px solid rgba(100, 120, 255, 0.4);
    border-radius: 12px;
    padding: 20px;
    color: #e0e0e0;
  }
  .card h1 {
    margin: 0 0 12px;
    font-size: 22px;
    color: #7b8cff;
  }
  .card p {
    margin: 0 0 8px;
    font-size: 14px;
    line-height: 1.5;
  }
  .card .muted {
    font-size: 13px;
    color: #999;
    margin: 0 0 16px;
  }
  .buttons {
    display: flex;
    gap: 8px;
  }
  .btn {
    padding: 8px 16px;
    border-radius: 6px;
    font-size: 13px;
    cursor: pointer;
    transition: background 0.15s, color 0.15s, border-color 0.15s, transform 0.1s;
  }
  .btn:active {
    transform: scale(0.96);
  }
  .btn-primary {
    background: #7b8cff;
    color: #111;
    font-weight: bold;
  }
  .btn-primary:hover {
    background: #95a4ff;
  }
  .btn-outline {
    background: transparent;
    border: 1px solid #7b8cff;
    color: #7b8cff;
  }
  .btn-outline:hover {
    background: rgba(123, 140, 255, 0.15);
    border-color: #95a4ff;
    color: #95a4ff;
  }
  .card {
    transition: border-color 0.2s;
  }
  .card:hover {
    border-color: rgba(100, 120, 255, 0.7);
  }
  b.highlight { color: #a0cfff; }
  b.key { color: #ccc; }
</style>
<div class="hud">
  <div class="card">
    <h1>wgpu-html in Bevy</h1>
    <p>
      This HTML panel is rendered by <b class="highlight">wgpu-html</b>
      into an offscreen texture, then displayed as a Bevy UI image.
    </p>
    <p class="muted">
      Press <b class="key">F11</b> to toggle devtools.
    </p>
    <div class="buttons">
      <div class="btn btn-primary">Button A</div>
      <div class="btn btn-outline">Button B</div>
    </div>
  </div>
</div>
"#);

  html.tree_mut().merge(parsed);
}
