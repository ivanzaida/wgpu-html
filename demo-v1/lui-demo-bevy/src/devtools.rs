use bevy::{
  asset::RenderAssetUsages,
  input::keyboard::KeyboardInput,
  prelude::*,
  render::render_resource::{Extent3d, TextureDimension, TextureFormat},
  window::CursorMoved,
};
use lui_devtools::Devtools;
use lui_driver_bevy::{HtmlOverlay, key_code_to_dom_code, logical_key_to_dom_key};
use lui_renderer_wgpu::RenderBackend;

#[derive(Component)]
pub struct DevtoolsWindow;

#[derive(Component)]
pub struct DevtoolsUi;

pub struct DevtoolsState {
  devtools: Devtools,
  image_cache: lui::layout::ImageCache,
  pipeline_cache: lui::PipelineCache,
  window_entity: Option<Entity>,
  image_handle: Option<Handle<Image>>,
}

pub fn has_devtools(res: Option<NonSend<DevtoolsState>>) -> bool {
  res.is_some()
}

pub fn setup(world: &mut World) {
  let devtools = {
    let mut overlay = world
      .get_non_send_resource_mut::<HtmlOverlay>()
      .expect("HtmlOverlay must exist — add LuiPlugin before this system");
    Devtools::attach(&mut overlay.tree, false)
  };

  world.insert_non_send_resource(DevtoolsState {
    devtools,
    image_cache: lui::layout::ImageCache::default(),
    pipeline_cache: lui::PipelineCache::new(),
    window_entity: None,
    image_handle: None,
  });
}

pub fn input_system(
  mut dt: NonSendMut<DevtoolsState>,
  mut cursor_events: MessageReader<CursorMoved>,
  mut keyboard_events: MessageReader<KeyboardInput>,
) {
  let Some(win) = dt.window_entity else { return };
  let d = &mut *dt;

  for event in cursor_events.read() {
    if event.window != win {
      continue;
    }
    if let Some(layout) = d.pipeline_cache.layout() {
      lui::interactivity::pointer_move(d.devtools.tree_mut(), layout, (event.position.x, event.position.y));
    }
  }

  for event in keyboard_events.read() {
    if event.window != win {
      continue;
    }
    let key = logical_key_to_dom_key(&event.logical_key);
    let code = key_code_to_dom_code(&event.key_code);
    let tree = d.devtools.tree_mut();
    if event.state.is_pressed() {
      tree.key_down(key, code, event.repeat);
    } else {
      tree.key_up(key, code);
    }
  }
}

pub fn update_system(
  mut commands: Commands,
  mut dt: NonSendMut<DevtoolsState>,
  overlay: NonSend<HtmlOverlay>,
  mut images: ResMut<Assets<Image>>,
  dt_windows: Query<&Window, With<DevtoolsWindow>>,
  dt_ui_entities: Query<Entity, With<DevtoolsUi>>,
) {
  dt.devtools.poll(&overlay.tree);

  let enabled = dt.devtools.is_enabled();
  let has_window = dt.window_entity.is_some();

  if enabled && !has_window {
    let win = commands
      .spawn((
        DevtoolsWindow,
        Window {
          title: "lui · Devtools".into(),
          resolution: bevy::window::WindowResolution::new(600, 720),
          ..default()
        },
      ))
      .id();

    let image = Image::new_fill(
      Extent3d {
        width: 600,
        height: 720,
        depth_or_array_layers: 1,
      },
      TextureDimension::D2,
      &[30, 30, 36, 255],
      TextureFormat::Rgba8UnormSrgb,
      RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let handle = images.add(image);

    let cam = commands
      .spawn((
        DevtoolsUi,
        Camera2d,
        bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(win)),
      ))
      .id();
    commands.spawn((
      DevtoolsUi,
      ImageNode {
        image: handle.clone(),
        ..default()
      },
      Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
      },
      UiTargetCamera(cam),
    ));

    dt.window_entity = Some(win);
    dt.image_handle = Some(handle);
  }

  if !enabled && has_window {
    if let Some(win) = dt.window_entity.take() {
      commands.entity(win).despawn();
    }
    for e in dt_ui_entities.iter() {
      commands.entity(e).despawn();
    }
    dt.image_handle = None;
    dt.pipeline_cache.invalidate();
    return;
  }

  if !enabled || !dt.devtools.needs_redraw() {
    return;
  }

  let Ok(window) = dt_windows.single() else { return };
  let scale = window.scale_factor();
  let phys_w = (window.width() * scale).ceil() as u32;
  let phys_h = (window.height() * scale).ceil() as u32;

  let d = &mut *dt;
  render_devtools(d, &mut images, phys_w, phys_h, scale);
}

fn render_devtools(d: &mut DevtoolsState, images: &mut Assets<Image>, phys_w: u32, phys_h: u32, scale: f32) {
  thread_local! {
      static DT_RENDERER: std::cell::RefCell<Option<(lui_renderer_wgpu::Renderer, lui_text::TextContext)>> =
          std::cell::RefCell::new(None);
  }

  DT_RENDERER.with(|cell| {
    let mut borrow = cell.borrow_mut();
    if borrow.is_none() {
      let mut r = pollster::block_on(lui_renderer_wgpu::Renderer::headless());
      lui_renderer_wgpu::RenderBackend::set_clear_color(&mut r, [0.12, 0.12, 0.14, 1.0]);
      let tc = lui_text::TextContext::new(lui_renderer_wgpu::GLYPH_ATLAS_SIZE);
      *borrow = Some((r, tc));
    }
    let (renderer, text_ctx) = borrow.as_mut().unwrap();

    text_ctx.sync_fonts(&d.devtools.tree().fonts);

    let (mut list, _layout, _timings) = lui::paint_tree_cached(
      d.devtools.tree_mut(),
      text_ctx,
      &mut d.image_cache,
      phys_w as f32,
      phys_h as f32,
      scale,
      0.0,
      &mut d.pipeline_cache,
    );
    list.finalize();

    text_ctx.atlas.flush_dirty(|rect, data| {
      renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
    });

    let Ok(rgba) = renderer.render_to_rgba(&list, phys_w, phys_h) else {
      return;
    };

    if let Some(handle) = &d.image_handle {
      if let Some(image) = images.get_mut(handle) {
        if image.width() != phys_w || image.height() != phys_h {
          image.resize(Extent3d {
            width: phys_w,
            height: phys_h,
            depth_or_array_layers: 1,
          });
        }
        image.data = Some(rgba);
      }
    }

    d.devtools.frame_rendered();
  });
}
