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

    // Parse HTML into nodes and append to the overlay tree.
    let parsed = wgpu_html::parser::parse(r#"
<div id="hud" style="font-family: system-ui, sans-serif; padding: 24px; max-width: 400px;">
    <div style="
        background: rgba(15, 15, 35, 0.85);
        border: 1px solid rgba(100, 120, 255, 0.4);
        border-radius: 12px; padding: 20px; color: #e0e0e0;
    ">
        <h1 style="margin:0 0 12px;font-size:22px;color:#7b8cff;">wgpu-html in Bevy</h1>
        <p style="margin:0 0 8px;font-size:14px;line-height:1.5;">
            This HTML panel is rendered by <b style="color:#a0cfff;">wgpu-html</b>
            into an offscreen texture, then displayed as a Bevy UI image.
        </p>
        <p style="margin:0 0 16px;font-size:13px;color:#999;">
            Press <b style="color:#ccc;">F11</b> to toggle devtools.
        </p>
        <div style="display:flex;gap:8px;">
            <div style="background:#7b8cff;color:#111;padding:8px 16px;border-radius:6px;font-size:13px;font-weight:bold;">Button A</div>
            <div style="background:transparent;border:1px solid #7b8cff;color:#7b8cff;padding:8px 16px;border-radius:6px;font-size:13px;">Button B</div>
        </div>
    </div>
</div>
"#);

    if let Some(root) = parsed.root {
        for child in root.children {
            html.append(child);
        }
    }
}
