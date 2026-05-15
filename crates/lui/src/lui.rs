use std::{collections::BTreeMap, path::Path, time::Instant};

use lui_cascade::{
  cascade::{CascadeContext, InteractionState},
  media::MediaContext,
};
use lui_glyph::{FontFace, FontHandle, TextContext};
use lui_layout::engine::LayoutEngine;
use lui_parse::{HtmlDocument, Stylesheet};

use crate::{
  display_list::{DisplayList, FrameOutcome}, RenderBackend,
  RenderError,
};

pub struct Lui {
  pub doc: HtmlDocument,
  pub(crate) text_ctx: TextContext,
  cascade_ctx: CascadeContext,
  layout_engine: LayoutEngine,
  dpi_scale_override: Option<f32>,
  base_sheets: Vec<Stylesheet>,
  dynamic_sheets: BTreeMap<u64, Stylesheet>,
  next_sheet_id: u64,
  element_scroll: BTreeMap<Vec<usize>, (f32, f32)>,
  viewport_scroll: (f32, f32),
  cursor_pos: Option<(f32, f32)>,
  cursor_moved: bool,
  hover_path: Option<Vec<usize>>,
  active_path: Option<Vec<usize>>,
  last_click: Option<ClickState>,
  scrollbar_drag: Option<ScrollbarDrag>,
  scrollbar_hover: Option<(Vec<usize>, lui_cascade::cascade::ScrollbarPart)>,
  text_selection: Option<lui_core::TextSelection>,
  selecting_text: bool,
  selection_colors: lui_core::SelectionColors,
  needs_redraw: bool,
  current_cursor: String,
  form_state: BTreeMap<Vec<usize>, lui_core::form_state::FormControlState>,
  pub timers: crate::timer::Timers,
  event_handlers: Vec<LuiEventHandler>,
  next_handler_id: u64,
  pending_events: Vec<PendingEvent>,
}

type LuiHandlerFn = Box<dyn FnMut(&mut Lui, &lui_core::events::DocumentEvent) + Send>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StylesheetHandle(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventHandlerHandle(u64);

struct LuiEventHandler {
  id: u64,
  event_type: String,
  callback: LuiHandlerFn,
}

#[derive(Clone)]
struct PendingEvent {
  event: lui_core::events::DocumentEvent,
}

#[derive(Debug, Clone)]
struct ScrollbarDrag {
  path: Vec<usize>,
  html_path: Option<Vec<usize>>,
  axis: lui_layout::ScrollbarAxis,
  grab_offset: f32,
  track_start: f32,
  track_length: f32,
  thumb_length: f32,
  max_scroll: f32,
  is_viewport: bool,
}

impl Lui {
  pub fn new() -> Self {
    #[allow(unused_mut)]
    let mut s = Self {
      doc: HtmlDocument::default(),
      text_ctx: TextContext::new(),
      cascade_ctx: CascadeContext::new(),
      layout_engine: LayoutEngine::new(),
      dpi_scale_override: None,
      base_sheets: Vec::new(),
      dynamic_sheets: BTreeMap::new(),
      next_sheet_id: 1,
      element_scroll: BTreeMap::new(),
      viewport_scroll: (0.0, 0.0),
      cursor_pos: None,
      cursor_moved: false,
      hover_path: None,
      active_path: None,
      last_click: None,
      scrollbar_drag: None,
      scrollbar_hover: None,
      text_selection: None,
      selecting_text: false,
      selection_colors: lui_core::SelectionColors::default(),
      needs_redraw: false,
      current_cursor: String::from("auto"),
      form_state: BTreeMap::new(),
      timers: crate::timer::Timers::new(),
      event_handlers: Vec::new(),
      next_handler_id: 1,
      pending_events: Vec::new(),
    };

    #[cfg(feature = "ua_whatwg")]
    {
      use std::sync::LazyLock;
      static UA_SHEET: &str = include_str!("../ua/ua_whatwg.css");
      static PARSED_UA_SHEET: LazyLock<Stylesheet> =
        LazyLock::new(|| lui_parse::parse_stylesheet(UA_SHEET).unwrap_or_default());
      s.set_stylesheets(&[PARSED_UA_SHEET.clone()])
    }
    s
  }

  pub fn set_html(&mut self, html: &str) {
    self.doc = lui_parse::parse(html);
    self.rebuild_stylesheets();
  }

  pub fn doc(&self) -> &HtmlDocument {
    &self.doc
  }
  pub fn doc_mut(&mut self) -> &mut HtmlDocument {
    &mut self.doc
  }

  pub fn set_stylesheets(&mut self, sheets: &[Stylesheet]) {
    self.base_sheets = sheets.to_vec();
    self.rebuild_stylesheets();
  }

  pub fn add_stylesheet(&mut self, sheet: Stylesheet) -> StylesheetHandle {
    let id = self.next_sheet_id;
    self.next_sheet_id += 1;
    self.dynamic_sheets.insert(id, sheet);
    self.rebuild_stylesheets();
    StylesheetHandle(id)
  }

  pub fn add_stylesheets(&mut self, sheets: Vec<Stylesheet>) -> Vec<StylesheetHandle> {
    let mut handles = vec![];
    for sheet in sheets {
      let id = self.next_sheet_id;
      self.next_sheet_id += 1;
      self.dynamic_sheets.insert(id, sheet);
      handles.push(StylesheetHandle(id));
    }
    self.rebuild_stylesheets();
    handles
  }

  pub fn remove_stylesheet(&mut self, handle: StylesheetHandle) {
    if self.dynamic_sheets.remove(&handle.0).is_some() {
      self.rebuild_stylesheets();
    }
  }

  pub fn remove_stylesheets(&mut self, handles: Vec<StylesheetHandle>) {
    let mut removed_any = false;

    for handle in handles {
      if self.dynamic_sheets.remove(&handle.0).is_some() {
        removed_any = true;
      }
    }

    if removed_any {
      self.rebuild_stylesheets();
    }
  }

  fn rebuild_stylesheets(&mut self) {
    let mut all = self.base_sheets.clone();
    all.extend(self.dynamic_sheets.values().cloned());
    all.extend(self.doc.stylesheets.iter().cloned());
    self.cascade_ctx.set_stylesheets(&all);
  }

  pub fn register_font(&mut self, face: FontFace) -> FontHandle {
    self.text_ctx.register_font(face)
  }

  pub fn current_cursor(&self) -> &str {
    &self.current_cursor
  }

  pub fn take_needs_redraw(&mut self) -> bool {
    std::mem::take(&mut self.needs_redraw)
  }

  pub fn viewport_scroll(&self) -> (f32, f32) {
    self.viewport_scroll
  }

  pub fn set_dpi_scale(&mut self, scale: Option<f32>) {
    self.dpi_scale_override = scale;
  }

  pub fn timer_sender(&self) -> crate::timer::TimerSender {
    self.timers.sender()
  }

  pub fn set_timeout(
    &mut self,
    delay: std::time::Duration,
    callback: impl FnMut(&mut Lui) + Send + 'static,
  ) -> crate::timer::TimerHandle {
    self.timers.set_timeout(delay, callback)
  }

  pub fn set_interval(
    &mut self,
    interval: std::time::Duration,
    callback: impl FnMut(&mut Lui) + Send + 'static,
  ) -> crate::timer::TimerHandle {
    self.timers.set_interval(interval, callback)
  }

  pub fn set_immediate(&mut self, callback: impl FnMut(&mut Lui) + Send + 'static) -> crate::timer::TimerHandle {
    self.timers.set_immediate(callback)
  }

  pub fn clear_timer(&mut self, handle: crate::timer::TimerHandle) {
    self.timers.clear_timer(handle);
  }

  pub fn on(
    &mut self,
    event_type: &str,
    callback: impl FnMut(&mut Lui, &lui_core::events::DocumentEvent) + Send + 'static,
  ) -> EventHandlerHandle {
    let id = self.next_handler_id;
    self.next_handler_id += 1;
    self.event_handlers.push(LuiEventHandler {
      id,
      event_type: event_type.to_string(),
      callback: Box::new(callback),
    });
    EventHandlerHandle(id)
  }

  pub fn off(&mut self, handle: EventHandlerHandle) {
    self.event_handlers.retain(|h| h.id != handle.0);
  }

  pub fn text_selection(&self) -> Option<&lui_core::TextSelection> {
    self.text_selection.as_ref()
  }

  pub fn selected_text(&mut self, pw: u32, ph: u32, scale: f32) -> Option<String> {
    let sel = self.text_selection.clone()?;
    self.with_layout(pw, ph, scale, |tree, _, _, _, _| {
      crate::text_select::selected_text(&sel, &tree.root)
    })
  }

  pub fn select_all(&mut self, pw: u32, ph: u32, scale: f32) {
    let sel = self.with_layout(pw, ph, scale, |tree, _, _, _, _| {
      let anchor = crate::text_select::first_text_cursor(&tree.root)?;
      let focus = crate::text_select::last_text_cursor(&tree.root)?;
      Some(lui_core::TextSelection { anchor, focus })
    });
    self.text_selection = sel;
    self.selecting_text = false;
  }

  pub fn set_cursor_position(&mut self, x: f32, y: f32) {
    if self.cursor_pos != Some((x, y)) {
      self.cursor_pos = Some((x, y));
      self.cursor_moved = true;
      self.update_scrollbar_drag(x, y);
    }
  }

  pub fn clear_cursor_position(&mut self) {
    self.cursor_pos = None;
    self.cursor_moved = true;
    self.hover_path = None;
  }

  pub fn render_frame(
    &mut self,
    renderer: &mut dyn RenderBackend,
    physical_width: u32,
    physical_height: u32,
    scale: f32,
  ) -> FrameOutcome {
    if crate::timer::tick_timers(self) {
      self.needs_redraw = true;
    }
    let prev_hover = self.hover_path.clone();
    let did_move = self.cursor_moved;
    self.needs_redraw = false;
    let list = self.paint(physical_width, physical_height, scale);
    self.dispatch_hover_transitions(&prev_hover, did_move);
    flush_pending_events(self);
    self.flush_atlas(renderer);
    renderer.render(&list)
  }

  pub fn screenshot_to(
    &mut self,
    renderer: &mut dyn RenderBackend,
    physical_width: u32,
    physical_height: u32,
    scale: f32,
    path: impl AsRef<Path>,
  ) -> Result<(), RenderError> {
    let list = self.paint(physical_width, physical_height, scale);
    self.flush_atlas(renderer);
    renderer.capture_to(&list, physical_width, physical_height, path.as_ref())
  }

  fn paint(&mut self, pw: u32, ph: u32, scale: f32) -> DisplayList {
    let viewport_scroll = self.viewport_scroll;
    let selection = self.text_selection.clone();
    let sel_colors = self.selection_colors;
    let form_ctx = crate::paint::form::FormPaintCtx {
      focus_path: self.doc.focus_path.clone(),
      form_state: self.form_state.clone(),
    };
    self.with_layout(pw, ph, scale, |tree, text_ctx, effective_scale, vw, vh| {
      let mut list = crate::paint::paint_scaled_with_form(
        tree,
        text_ctx,
        effective_scale,
        selection.as_ref(),
        &sel_colors,
        &form_ctx,
      );
      translate_display_list(&mut list, -viewport_scroll.0, -viewport_scroll.1);
      crate::paint::paint_viewport_scrollbars(&mut list, tree, vw, vh, viewport_scroll.0, viewport_scroll.1);
      list.finalize();
      list.dpi_scale = effective_scale;
      list
    })
  }

  fn flush_atlas(&mut self, renderer: &mut dyn RenderBackend) {
    self.text_ctx.flush_dirty(|rect, data| {
      renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
    });
  }

  pub fn handle_wheel(
    &mut self,
    physical_width: u32,
    physical_height: u32,
    scale: f32,
    delta_x: f32,
    delta_y: f32,
  ) -> bool {
    let Some((cursor_x, cursor_y)) = self.cursor_pos else {
      return false;
    };

    let prevented = self.dispatch_wheel_event(physical_width, physical_height, scale, delta_x, delta_y);
    if prevented {
      return true;
    }

    let viewport_scroll = self.viewport_scroll;
    let outcome = self.with_layout(
      physical_width,
      physical_height,
      scale,
      |tree, _text_ctx, _effective_scale, vw, vh| {
        let doc_x = cursor_x + viewport_scroll.0;
        let doc_y = cursor_y + viewport_scroll.1;

        let (remaining_x, remaining_y, changed_elements) =
          if let Some(path) = tree.deepest_scrollable_path_at(doc_x, doc_y) {
            let result = tree.scroll_chain(&path, delta_x, delta_y);
            (result.remaining_x, result.remaining_y, result.changed)
          } else {
            (delta_x, delta_y, Vec::new())
          };

        let viewport_change = if remaining_x.abs() > 0.001 || remaining_y.abs() > 0.001 {
          let (max_x, max_y) = tree.viewport_scroll_bounds(vw, vh);
          let new_x = (viewport_scroll.0 + remaining_x).clamp(0.0, max_x);
          let new_y = (viewport_scroll.1 + remaining_y).clamp(0.0, max_y);
          let changed = (new_x - viewport_scroll.0).abs() > 0.001 || (new_y - viewport_scroll.1).abs() > 0.001;
          if changed { Some((new_x, new_y)) } else { None }
        } else {
          None
        };

        WheelOutcome {
          changed_elements,
          viewport_change,
        }
      },
    );

    let mut any_changed = false;
    for (path, info) in outcome.changed_elements {
      self.element_scroll.insert(path, (info.scroll_x, info.scroll_y));
      any_changed = true;
    }
    if let Some(new_scroll) = outcome.viewport_change {
      self.viewport_scroll = new_scroll;
      any_changed = true;
    }
    any_changed
  }

  /// Resolve the DOM path under the cursor. One `with_layout` call.
  fn resolve_cursor_target(&mut self, pw: u32, ph: u32, scale: f32) -> Option<Vec<usize>> {
    let (cx, cy) = self.cursor_pos?;
    let vp = self.viewport_scroll;
    let ptr = self.with_layout(pw, ph, scale, |tree, _, _, _, _| {
      tree.hit_test(cx + vp.0, cy + vp.1).map(|n| n as *const _)
    })?;
    crate::dispatch::find_node_path(&self.doc.root, ptr)
  }

  fn fire_pointer_at(
    &mut self,
    path: &[usize],
    event_type: &str,
    button: i16,
    bubbles: bool,
    cancelable: bool,
  ) -> bool {
    let (cx, cy) = self.cursor_pos.unwrap_or((0.0, 0.0));
    let mut event = lui_core::events::DocumentEvent::PointerEvent(lui_core::events::PointerEventInit {
      mouse: lui_core::events::MouseEventInit {
        ui: lui_core::events::UiEventInit {
          base: lui_core::events::EventInit {
            event_type: event_type.into(),
            bubbles,
            cancelable,
            ..Default::default()
          },
          ..Default::default()
        },
        client_x: cx as f64,
        client_y: cy as f64,
        button,
        ..Default::default()
      },
      pointer_id: MOUSE_POINTER_ID,
      pointer_type: "mouse".to_string(),
      is_primary: true,
      ..Default::default()
    });
    self.dispatch_and_queue(path, &mut event);
    event.is_default_prevented()
  }

  fn dispatch_and_queue(&mut self, path: &[usize], event: &mut lui_core::events::DocumentEvent) {
    crate::dispatch::dispatch_event(&mut self.doc, path, event);
    let event_type = event.base().event_type.as_ref();
    if self.event_handlers.iter().any(|h| h.event_type == event_type) {
      self.pending_events.push(PendingEvent { event: event.clone() });
    }
  }

  fn fire_mouse_at(&mut self, path: &[usize], event_type: &str, button: i16, bubbles: bool, cancelable: bool) -> bool {
    let (cx, cy) = self.cursor_pos.unwrap_or((0.0, 0.0));
    let mut event = lui_core::events::DocumentEvent::MouseEvent(lui_core::events::MouseEventInit {
      ui: lui_core::events::UiEventInit {
        base: lui_core::events::EventInit {
          event_type: event_type.into(),
          bubbles,
          cancelable,
          ..Default::default()
        },
        ..Default::default()
      },
      client_x: cx as f64,
      client_y: cy as f64,
      button,
      ..Default::default()
    });
    self.dispatch_and_queue(path, &mut event);
    event.is_default_prevented()
  }

  fn fire_mouse_event(&mut self, path: &[usize], event_type: &str, button: i16) -> bool {
    self.fire_mouse_at(path, event_type, button, true, true)
  }

  fn dispatch_wheel_event(&mut self, pw: u32, ph: u32, scale: f32, dx: f32, dy: f32) -> bool {
    let Some(path) = self.resolve_cursor_target(pw, ph, scale) else {
      return false;
    };
    let (cx, cy) = self.cursor_pos.unwrap_or((0.0, 0.0));
    let mut event = lui_core::events::DocumentEvent::WheelEvent(lui_core::events::WheelEventInit {
      mouse: lui_core::events::MouseEventInit {
        ui: lui_core::events::UiEventInit {
          base: lui_core::events::EventInit {
            event_type: "wheel".into(),
            bubbles: true,
            cancelable: true,
            ..Default::default()
          },
          ..Default::default()
        },
        client_x: cx as f64,
        client_y: cy as f64,
        ..Default::default()
      },
      delta_x: dx as f64,
      delta_y: dy as f64,
      delta_z: 0.0,
      delta_mode: 0,
    });
    self.dispatch_and_queue(&path, &mut event);
    event.is_default_prevented()
  }

  pub fn handle_key_down(&mut self, key: &str, code: &str, repeat: bool, modifiers: KeyModifiers) {
    self.fire_keyboard_event("keydown", key, code, repeat, &modifiers);
    self.handle_form_key(key, code, &modifiers);
  }

  pub fn handle_key_up(&mut self, key: &str, code: &str, modifiers: KeyModifiers) {
    self.fire_keyboard_event("keyup", key, code, false, &modifiers);
  }

  pub fn handle_text_input(&mut self, text: &str) {
    let Some(path) = self.doc.focus_path.clone() else {
      return;
    };
    if !self.is_text_editable(&path) {
      return;
    }
    self.ensure_form_state(&path);
    if self.fire_beforeinput_event(&path, Some(text), "insertText") {
      return;
    }
    if let Some(state) = self.form_state.get_mut(&path) {
      let (new_val, new_cur) = lui_core::text_edit::insert_text(&state.value, &state.edit_cursor, text);
      state.value = new_val;
      state.edit_cursor = new_cur;
      state.reset_blink();
      self.needs_redraw = true;
    }
    self.fire_input_event(&path, Some(text), "insertText");
  }

  fn handle_form_key(&mut self, key: &str, code: &str, modifiers: &KeyModifiers) {
    let Some(path) = self.doc.focus_path.clone() else {
      return;
    };
    if !self.is_text_editable(&path) {
      return;
    }
    self.ensure_form_state(&path);

    let is_textarea = self
      .doc
      .root
      .at_path(&path)
      .is_some_and(|n| matches!(n.element(), lui_core::HtmlElement::Textarea));
    let extend = modifiers.shift;

    let mutation: Option<(&str, &str)> = match (key, modifiers.ctrl) {
      ("Backspace", false) => Some(("deleteContentBackward", "Backspace")),
      ("Backspace", true) => Some(("deleteWordBackward", "Backspace")),
      ("Delete", false) => Some(("deleteContentForward", "Delete")),
      ("Delete", true) => Some(("deleteWordForward", "Delete")),
      ("Enter", _) if is_textarea => Some(("insertLineBreak", "Enter")),
      _ => None,
    };

    if let Some((input_type, _trigger)) = mutation {
      if self.fire_beforeinput_event(&path, None, input_type) {
        return;
      }
      match (key, modifiers.ctrl) {
        ("Backspace", false) => {
          self.mutate_form_value(&path, |v, c| lui_core::text_edit::delete_backward(v, c));
        }
        ("Backspace", true) => {
          self.mutate_form_value(&path, |v, c| lui_core::text_edit::delete_word_backward(v, c));
        }
        ("Delete", false) => {
          self.mutate_form_value(&path, |v, c| lui_core::text_edit::delete_forward(v, c));
        }
        ("Delete", true) => {
          self.mutate_form_value(&path, |v, c| lui_core::text_edit::delete_word_forward(v, c));
        }
        ("Enter", _) => {
          self.mutate_form_value(&path, |v, c| lui_core::text_edit::insert_line_break(v, c));
        }
        _ => {}
      }
      self.fire_input_event(&path, None, input_type);
      return;
    }

    // Cursor movement (no value mutation)
    let moved = match (code, modifiers.ctrl) {
      ("ArrowLeft", false) => {
        self.move_form_cursor(&path, |v, c| lui_core::text_edit::move_left(v, c, extend));
        true
      }
      ("ArrowRight", false) => {
        self.move_form_cursor(&path, |v, c| lui_core::text_edit::move_right(v, c, extend));
        true
      }
      ("ArrowLeft", true) => {
        self.move_form_cursor(&path, |v, c| lui_core::text_edit::move_word_left(v, c, extend));
        true
      }
      ("ArrowRight", true) => {
        self.move_form_cursor(&path, |v, c| lui_core::text_edit::move_word_right(v, c, extend));
        true
      }
      ("Home", _) => {
        self.move_form_cursor(&path, |v, c| lui_core::text_edit::move_home(v, c, extend));
        true
      }
      ("End", _) => {
        self.move_form_cursor(&path, |v, c| lui_core::text_edit::move_end(v, c, extend));
        true
      }
      ("ArrowUp", _) if is_textarea => {
        self.move_form_cursor(&path, |v, c| lui_core::text_edit::move_up(v, c, extend));
        true
      }
      ("ArrowDown", _) if is_textarea => {
        self.move_form_cursor(&path, |v, c| lui_core::text_edit::move_down(v, c, extend));
        true
      }
      _ => false,
    };

    if !moved && modifiers.ctrl && key == "a" {
      self.move_form_cursor(&path, |v, _| lui_core::text_edit::select_all(v));
      return;
    }

    if moved {
      return;
    }

    // Printable character insertion (single-char keys without ctrl)
    if !modifiers.ctrl && !modifiers.alt && !modifiers.meta && key.len() <= 4 && !key.starts_with("F") {
      let ch_count = key.chars().count();
      if ch_count == 1 && !key.chars().next().unwrap().is_control() {
        self.handle_text_input(key);
      }
    }
  }

  fn is_text_editable(&self, path: &[usize]) -> bool {
    let Some(node) = self.doc.root.at_path(path) else {
      return false;
    };
    match node.element() {
      lui_core::HtmlElement::Textarea => true,
      lui_core::HtmlElement::Input => {
        let input_type = node.attr("type").map(|v| v.as_ref()).unwrap_or("text");
        matches!(
          input_type,
          "text" | "password" | "email" | "search" | "tel" | "url" | "number"
        )
      }
      _ => false,
    }
  }

  fn ensure_form_state(&mut self, path: &[usize]) {
    if self.form_state.contains_key(path) {
      return;
    }
    let initial = self
      .doc
      .root
      .at_path(path)
      .and_then(|n| n.attr("value"))
      .map(|v| v.as_ref())
      .unwrap_or("");
    self
      .form_state
      .insert(path.to_vec(), lui_core::form_state::FormControlState::new(initial));
  }

  pub fn form_value(&self, path: &[usize]) -> Option<&str> {
    self.form_state.get(path).map(|s| s.value.as_str())
  }

  fn mutate_form_value(
    &mut self,
    path: &[usize],
    op: impl FnOnce(&str, &lui_core::text_selection::EditCursor) -> (String, lui_core::text_selection::EditCursor),
  ) {
    if let Some(state) = self.form_state.get_mut(path) {
      let (new_val, new_cur) = op(&state.value, &state.edit_cursor);
      state.value = new_val;
      state.edit_cursor = new_cur;
      state.reset_blink();
      self.needs_redraw = true;
    }
  }

  fn move_form_cursor(
    &mut self,
    path: &[usize],
    op: impl FnOnce(&str, &lui_core::text_selection::EditCursor) -> lui_core::text_selection::EditCursor,
  ) {
    if let Some(state) = self.form_state.get_mut(path) {
      state.edit_cursor = op(&state.value, &state.edit_cursor);
      state.reset_blink();
      self.needs_redraw = true;
    }
  }

  fn fire_beforeinput_event(&mut self, path: &[usize], data: Option<&str>, input_type: &str) -> bool {
    let mut event = lui_core::events::DocumentEvent::InputEvent(lui_core::events::InputEventInit {
      ui: lui_core::events::UiEventInit {
        base: lui_core::events::EventInit {
          event_type: "beforeinput".into(),
          bubbles: true,
          cancelable: true,
          ..Default::default()
        },
        ..Default::default()
      },
      data: data.map(|s| s.to_string()),
      is_composing: false,
      input_type: input_type.to_string(),
    });
    self.dispatch_and_queue(path, &mut event);
    event.is_default_prevented()
  }

  fn fire_input_event(&mut self, path: &[usize], data: Option<&str>, input_type: &str) {
    let mut event = lui_core::events::DocumentEvent::InputEvent(lui_core::events::InputEventInit {
      ui: lui_core::events::UiEventInit {
        base: lui_core::events::EventInit {
          event_type: "input".into(),
          bubbles: true,
          cancelable: false,
          ..Default::default()
        },
        ..Default::default()
      },
      data: data.map(|s| s.to_string()),
      is_composing: false,
      input_type: input_type.to_string(),
    });
    self.dispatch_and_queue(path, &mut event);
  }

  pub fn handle_copy(&mut self, pw: u32, ph: u32, scale: f32) -> Option<String> {
    let path = self.doc.focus_path.clone().unwrap_or_default();
    if self.fire_clipboard_event(&path, "copy") {
      return None;
    }
    if let Some(state) = self.form_state.get(&path) {
      return state.selected_text();
    }
    self.selected_text(pw, ph, scale)
  }

  pub fn handle_cut(&mut self, pw: u32, ph: u32, scale: f32) -> Option<String> {
    let path = self.doc.focus_path.clone().unwrap_or_default();
    if self.fire_clipboard_event(&path, "cut") {
      return None;
    }
    if self.form_state.contains_key(&path) {
      let text = self.form_state.get(&path).and_then(|s| s.selected_text());
      if text.is_some() {
        self.mutate_form_value(&path, |v, c| lui_core::text_edit::delete_selection(v, c));
        self.fire_input_event(&path, None, "deleteByCut");
      }
      return text;
    }
    self.selected_text(pw, ph, scale)
  }

  pub fn handle_paste(&mut self, text: &str) {
    let path = self.doc.focus_path.clone().unwrap_or_default();
    if self.fire_clipboard_event_with_data(&path, "paste", Some(text)) {
      return;
    }
    if !self.is_text_editable(&path) {
      return;
    }
    self.ensure_form_state(&path);
    if self.fire_beforeinput_event(&path, Some(text), "insertFromPaste") {
      return;
    }
    self.mutate_form_value(&path, |v, c| lui_core::text_edit::insert_text(v, c, text));
    self.fire_input_event(&path, Some(text), "insertFromPaste");
  }

  fn fire_clipboard_event(&mut self, path: &[usize], event_type: &str) -> bool {
    self.fire_clipboard_event_with_data(path, event_type, None)
  }

  fn fire_clipboard_event_with_data(&mut self, path: &[usize], event_type: &str, data: Option<&str>) -> bool {
    let mut event = lui_core::events::DocumentEvent::ClipboardEvent(lui_core::events::ClipboardEventInit {
      base: lui_core::events::EventInit {
        event_type: event_type.into(),
        bubbles: true,
        cancelable: true,
        ..Default::default()
      },
      clipboard_data: data.map(|s| s.to_string()),
    });
    self.dispatch_and_queue(path, &mut event);
    event.is_default_prevented()
  }

  fn fire_keyboard_event(&mut self, event_type: &str, key: &str, code: &str, repeat: bool, _modifiers: &KeyModifiers) {
    let path = self.doc.focus_path.clone().unwrap_or_default();
    let mut event = lui_core::events::DocumentEvent::KeyboardEvent(lui_core::events::KeyboardEventInit {
      ui: lui_core::events::UiEventInit {
        base: lui_core::events::EventInit {
          event_type: event_type.into(),
          bubbles: true,
          cancelable: true,
          ..Default::default()
        },
        ..Default::default()
      },
      key: key.to_string(),
      code: code.to_string(),
      location: 0,
      repeat,
      is_composing: false,
    });
    if let lui_core::events::DocumentEvent::KeyboardEvent(ref mut kb) = event {
      kb.ui.base.event_type = event_type.into();
    }
    self.dispatch_and_queue(&path, &mut event);
  }

  fn dispatch_hover_transitions(&mut self, prev: &Option<Vec<usize>>, did_move: bool) {
    let hover_changed = *prev != self.hover_path;
    if !hover_changed && !did_move {
      return;
    }

    let prev_path = prev.as_deref().unwrap_or(&[]);
    let curr_path = self.hover_path.clone();
    let curr = curr_path.as_deref().unwrap_or(&[]);
    let common = common_prefix_len(prev_path, curr);

    if hover_changed {
      if !prev_path.is_empty() {
        self.fire_pointer_at(prev_path, "pointerout", 0, true, true);
        self.fire_mouse_at(prev_path, "mouseout", 0, true, true);
      }

      for depth in (common..prev_path.len()).rev() {
        self.fire_pointer_at(&prev_path[..=depth], "pointerleave", 0, false, false);
        self.fire_mouse_at(&prev_path[..=depth], "mouseleave", 0, false, false);
      }

      if !curr.is_empty() {
        self.fire_pointer_at(curr, "pointerover", 0, true, true);
        self.fire_mouse_at(curr, "mouseover", 0, true, true);
      }

      for depth in common..curr.len() {
        self.fire_pointer_at(&curr[..=depth], "pointerenter", 0, false, false);
        self.fire_mouse_at(&curr[..=depth], "mouseenter", 0, false, false);
      }
    }

    if did_move && !curr.is_empty() {
      self.fire_pointer_at(curr, "pointermove", 0, true, true);
      self.fire_mouse_at(curr, "mousemove", 0, true, true);
    }
  }

  /// Dispatch a mouse event at the current cursor position.
  /// Performs one layout pass for hit-testing.
  pub fn dispatch_mouse_event(&mut self, pw: u32, ph: u32, scale: f32, event_type: &str, button: i16) -> bool {
    let Some(path) = self.resolve_cursor_target(pw, ph, scale) else {
      return false;
    };
    self.fire_mouse_event(&path, event_type, button)
  }

  pub fn handle_mouse_down(&mut self, pw: u32, ph: u32, scale: f32, button: i16) -> bool {
    if button == 0 {
      if let Some(drag) = self.try_start_scrollbar_drag(pw, ph, scale) {
        self.scrollbar_drag = Some(drag);
        return true;
      }
    }

    if button == 0 {
      let vp = self.viewport_scroll;
      let cursor_pos = self.cursor_pos;
      let click_count = self.click_count(button);
      let text_cursor = cursor_pos.and_then(|(cx, cy)| {
        self.with_layout(pw, ph, scale, |tree, text_ctx, _, _, _| {
          crate::text_hit::hit_text_cursor(tree, cx + vp.0, cy + vp.1, text_ctx)
        })
      });

      if let Some(tc) = text_cursor {
        if click_count >= 3 {
          let sel = self.with_layout(pw, ph, scale, |tree, text_ctx, _, _, _| {
            crate::text_select::select_line(&tc, &tree.root, text_ctx)
          });
          self.text_selection = sel;
          self.selecting_text = false;
        } else if click_count == 2 {
          let sel = self.with_layout(pw, ph, scale, |tree, text_ctx, _, _, _| {
            crate::text_select::select_word(&tc, &tree.root, text_ctx)
          });
          self.text_selection = sel;
          self.selecting_text = false;
        } else {
          self.text_selection = Some(lui_core::TextSelection {
            anchor: tc.clone(),
            focus: tc,
          });
          self.selecting_text = true;
        }
      } else {
        self.text_selection = None;
        self.selecting_text = false;
      }
    }

    let path = self.resolve_cursor_target(pw, ph, scale);
    self.active_path = path.clone();
    if button == 0 {
      self.set_focus(path.as_deref());
    }
    match path {
      Some(p) => {
        self.fire_pointer_at(&p, "pointerdown", button, true, true);
        self.fire_mouse_event(&p, "mousedown", button)
      }
      None => false,
    }
  }

  fn set_focus(&mut self, target: Option<&[usize]>) {
    let new_focus = target.and_then(|path| {
      for len in (1..=path.len()).rev() {
        let candidate = &path[..len];
        if let Some(node) = self.doc.root.at_path(candidate) {
          if is_focusable(node) {
            return Some(candidate.to_vec());
          }
        }
      }
      None
    });

    let old_focus = self.doc.focus_path.clone();
    if old_focus == new_focus {
      return;
    }

    if let Some(ref old) = old_focus {
      if self
        .form_state
        .get(old.as_slice())
        .is_some_and(|s| s.value_changed_since_focus())
      {
        self.fire_change_event(old);
      }
      self.fire_focus_event(old, "blur", false);
      self.fire_focus_event(old, "focusout", true);
    }

    self.doc.focus_path = new_focus.clone();

    if let Some(ref new) = new_focus {
      if self.is_text_editable(new) {
        self.ensure_form_state(new);
      }
      self.fire_focus_event(new, "focus", false);
      self.fire_focus_event(new, "focusin", true);
    }
  }

  fn fire_change_event(&mut self, path: &[usize]) {
    let mut event = lui_core::events::DocumentEvent::Event(lui_core::events::EventInit {
      event_type: "change".into(),
      bubbles: true,
      cancelable: false,
      ..Default::default()
    });
    self.dispatch_and_queue(path, &mut event);
  }

  fn fire_focus_event(&mut self, path: &[usize], event_type: &str, bubbles: bool) {
    let mut event = lui_core::events::DocumentEvent::FocusEvent(lui_core::events::FocusEventInit {
      ui: lui_core::events::UiEventInit {
        base: lui_core::events::EventInit {
          event_type: event_type.into(),
          bubbles,
          cancelable: false,
          ..Default::default()
        },
        ..Default::default()
      },
      related_target: None,
    });
    self.dispatch_and_queue(&path, &mut event);
  }

  pub fn handle_mouse_up(&mut self, pw: u32, ph: u32, scale: f32, button: i16) -> bool {
    if button == 0 && self.scrollbar_drag.take().is_some() {
      return true;
    }
    self.active_path = None;
    let Some(path) = self.resolve_cursor_target(pw, ph, scale) else {
      return false;
    };
    self.fire_pointer_at(&path, "pointerup", button, true, true);
    self.fire_mouse_event(&path, "mouseup", button)
  }

  pub fn handle_click(&mut self, pw: u32, ph: u32, scale: f32, button: i16) -> bool {
    self.dispatch_mouse_event(pw, ph, scale, "click", button)
  }

  /// Combined mouseup + click/dblclick/contextmenu with a single hit-test.
  pub fn handle_mouse_release(&mut self, pw: u32, ph: u32, scale: f32, button: i16) -> bool {
    if button == 0 && self.scrollbar_drag.take().is_some() {
      return true;
    }
    if button == 0 {
      self.selecting_text = false;
      if self.text_selection.as_ref().is_some_and(|s| s.is_collapsed()) {
        self.text_selection = None;
      }
    }
    self.active_path = None;
    let Some(path) = self.resolve_cursor_target(pw, ph, scale) else {
      return false;
    };
    self.fire_pointer_at(&path, "pointerup", button, true, true);
    let up = self.fire_mouse_event(&path, "mouseup", button);

    match button {
      0 => {
        let click = self.fire_mouse_event(&path, "click", button);
        let dbl = self.maybe_fire_dblclick(&path, button);
        if !dbl {
          self.record_click(button);
        }
        up || click || dbl
      }
      2 => {
        let ctx = self.fire_mouse_event(&path, "contextmenu", button);
        up || ctx
      }
      _ => up,
    }
  }

  fn maybe_fire_dblclick(&mut self, path: &[usize], button: i16) -> bool {
    let (cx, cy) = self.cursor_pos.unwrap_or((0.0, 0.0));
    let is_dbl = self.last_click.as_ref().is_some_and(|lc| {
      lc.button == button
        && lc.time.elapsed().as_millis() < DBLCLICK_THRESHOLD_MS
        && (lc.pos.0 - cx).abs() < DBLCLICK_DISTANCE_PX
        && (lc.pos.1 - cy).abs() < DBLCLICK_DISTANCE_PX
    });
    if is_dbl {
      self.last_click = None;
      self.fire_mouse_event(path, "dblclick", button);
      true
    } else {
      false
    }
  }

  fn record_click(&mut self, button: i16) {
    let (cx, cy) = self.cursor_pos.unwrap_or((0.0, 0.0));
    let count = self.click_count(button);
    self.last_click = Some(ClickState {
      time: Instant::now(),
      pos: (cx, cy),
      button,
      count,
    });
  }

  fn click_count(&self, button: i16) -> u8 {
    let (cx, cy) = self.cursor_pos.unwrap_or((0.0, 0.0));
    self
      .last_click
      .as_ref()
      .filter(|lc| {
        lc.button == button
          && lc.time.elapsed().as_millis() < DBLCLICK_THRESHOLD_MS
          && (lc.pos.0 - cx).abs() < DBLCLICK_DISTANCE_PX
          && (lc.pos.1 - cy).abs() < DBLCLICK_DISTANCE_PX
      })
      .map(|lc| lc.count + 1)
      .unwrap_or(1)
  }

  fn try_start_scrollbar_drag(&mut self, pw: u32, ph: u32, scale: f32) -> Option<ScrollbarDrag> {
    let (cx, cy) = self.cursor_pos?;
    let vp = self.viewport_scroll;

    let hit = self.with_layout(pw, ph, scale, |tree, _, _, vw, vh| {
      let doc_x = cx + vp.0;
      let doc_y = cy + vp.1;

      if let Some(hit) = tree.scrollbar_hit_test(doc_x, doc_y) {
        return Some((hit, false));
      }

      let bar_w = crate::paint::viewport_scrollbar_width(tree);
      if bar_w <= 0.0 {
        return None;
      }
      let (max_x, max_y) = tree.viewport_scroll_bounds(vw, vh);
      viewport_scrollbar_hit(cx, cy, vw, vh, bar_w, vp.0, vp.1, max_x, max_y).map(|hit| (hit, true))
    })?;

    let (hit, is_viewport) = hit;
    let html_path = if !hit.node_ptr.is_null() {
      crate::dispatch::find_node_path(&self.doc.root, hit.node_ptr)
    } else {
      None
    };
    let mut drag = ScrollbarDrag {
      path: hit.path,
      html_path,
      axis: hit.axis,
      grab_offset: hit.grab_offset,
      track_start: hit.track_start,
      track_length: hit.track_length,
      thumb_length: hit.thumb_length,
      max_scroll: hit.max_scroll,
      is_viewport,
    };

    if !hit.on_thumb {
      self.apply_scrollbar_jump(&drag, cx, cy);
      drag.grab_offset = drag.thumb_length * 0.5;
    }

    Some(drag)
  }

  fn apply_scrollbar_jump(&mut self, drag: &ScrollbarDrag, cx: f32, cy: f32) {
    let vp = self.viewport_scroll;
    let mouse_on_track = match drag.axis {
      lui_layout::ScrollbarAxis::Vertical => cy + if !drag.is_viewport { vp.1 } else { 0.0 } - drag.track_start,
      lui_layout::ScrollbarAxis::Horizontal => cx + if !drag.is_viewport { vp.0 } else { 0.0 } - drag.track_start,
    };
    let travel = (drag.track_length - drag.thumb_length).max(0.001);
    let fraction = ((mouse_on_track - drag.thumb_length * 0.5) / travel).clamp(0.0, 1.0);
    let new_scroll = fraction * drag.max_scroll;

    if drag.is_viewport {
      match drag.axis {
        lui_layout::ScrollbarAxis::Vertical => self.viewport_scroll.1 = new_scroll,
        lui_layout::ScrollbarAxis::Horizontal => self.viewport_scroll.0 = new_scroll,
      }
    } else {
      let prev = self.element_scroll.get(&drag.path).copied().unwrap_or((0.0, 0.0));
      let updated = match drag.axis {
        lui_layout::ScrollbarAxis::Vertical => (prev.0, new_scroll),
        lui_layout::ScrollbarAxis::Horizontal => (new_scroll, prev.1),
      };
      self.element_scroll.insert(drag.path.clone(), updated);
    }
  }

  fn update_scrollbar_drag(&mut self, x: f32, y: f32) {
    let Some(drag) = &self.scrollbar_drag else { return };
    let vp = self.viewport_scroll;

    let mouse_on_track = match drag.axis {
      lui_layout::ScrollbarAxis::Vertical => y + if !drag.is_viewport { vp.1 } else { 0.0 } - drag.track_start,
      lui_layout::ScrollbarAxis::Horizontal => x + if !drag.is_viewport { vp.0 } else { 0.0 } - drag.track_start,
    };

    let travel = (drag.track_length - drag.thumb_length).max(0.001);
    let fraction = ((mouse_on_track - drag.grab_offset) / travel).clamp(0.0, 1.0);
    let new_scroll = fraction * drag.max_scroll;

    let drag = drag.clone();
    if drag.is_viewport {
      match drag.axis {
        lui_layout::ScrollbarAxis::Vertical => self.viewport_scroll.1 = new_scroll,
        lui_layout::ScrollbarAxis::Horizontal => self.viewport_scroll.0 = new_scroll,
      }
    } else {
      let prev = self.element_scroll.get(&drag.path).copied().unwrap_or((0.0, 0.0));
      let updated = match drag.axis {
        lui_layout::ScrollbarAxis::Vertical => (prev.0, new_scroll),
        lui_layout::ScrollbarAxis::Horizontal => (new_scroll, prev.1),
      };
      self.element_scroll.insert(drag.path, updated);
    }
  }

  fn with_layout<T>(
    &mut self,
    pw: u32,
    ph: u32,
    scale: f32,
    f: impl for<'a> FnOnce(&mut lui_layout::LayoutTree<'a>, &mut TextContext, f32, f32, f32) -> T,
  ) -> T {
    let effective_scale = self.dpi_scale_override.unwrap_or(scale);
    let vw = pw as f32 / effective_scale;
    let vh = ph as f32 / effective_scale;

    let media = MediaContext {
      viewport_width: vw,
      viewport_height: vh,
      dpi: 96.0 * effective_scale,
      ..MediaContext::default()
    };
    let scrollbar_active = self.scrollbar_drag.as_ref().and_then(|drag| {
      let path = drag.html_path.clone()?;
      Some((path, lui_cascade::cascade::ScrollbarPart::Thumb))
    });
    let interaction = InteractionState {
      hover_path: self.hover_path.clone(),
      active_path: self.active_path.clone(),
      focus_path: self.doc.focus_path.clone(),
      scrollbar_hover: self.scrollbar_hover.clone(),
      scrollbar_active,
      ..Default::default()
    };
    let styled = self.cascade_ctx.cascade(&self.doc.root, &media, &interaction);
    let mut tree = self.layout_engine.layout(&styled, vw, vh, &mut self.text_ctx);
    apply_element_scroll_state(&mut tree, &self.element_scroll);

    let (max_x, max_y) = tree.viewport_scroll_bounds(vw, vh);
    self.viewport_scroll.0 = self.viewport_scroll.0.clamp(0.0, max_x);
    self.viewport_scroll.1 = self.viewport_scroll.1.clamp(0.0, max_y);

    if self.cursor_moved {
      self.cursor_moved = false;
      let prev_hover_path = self.hover_path.clone();
      let prev_scrollbar_hover = self.scrollbar_hover.clone();
      if let Some((cx, cy)) = self.cursor_pos {
        let doc_x = cx + self.viewport_scroll.0;
        let doc_y = cy + self.viewport_scroll.1;
        self.hover_path = tree
          .hit_test(doc_x, doc_y)
          .and_then(|n| crate::dispatch::find_node_path(&self.doc.root, n as *const _));
        self.current_cursor = tree.cursor_at(doc_x, doc_y).to_string();

        self.scrollbar_hover = tree.scrollbar_hit_test(doc_x, doc_y).and_then(|hit| {
          use lui_cascade::cascade::ScrollbarPart;
          let part = if hit.on_thumb {
            ScrollbarPart::Thumb
          } else {
            ScrollbarPart::Track
          };
          let path = crate::dispatch::find_node_path(&self.doc.root, hit.node_ptr)?;
          Some((path, part))
        });
        if self.scrollbar_hover.is_none() {
          let bar_w = crate::paint::viewport_scrollbar_width(&tree);
          if bar_w > 0.0 {
            let (max_x, max_y) = tree.viewport_scroll_bounds(vw, vh);
            self.scrollbar_hover = viewport_scrollbar_hit(
              cx,
              cy,
              vw,
              vh,
              bar_w,
              self.viewport_scroll.0,
              self.viewport_scroll.1,
              max_x,
              max_y,
            )
            .map(|hit| {
              use lui_cascade::cascade::ScrollbarPart;
              let part = if hit.on_thumb {
                ScrollbarPart::Thumb
              } else {
                ScrollbarPart::Track
              };
              (crate::paint::viewport_scrollbar_style_path(&tree), part)
            });
          }
        }
      } else {
        self.hover_path = None;
        self.scrollbar_hover = None;
        self.current_cursor = String::from("auto");
      }
      if self.hover_path != prev_hover_path || self.scrollbar_hover != prev_scrollbar_hover {
        self.needs_redraw = true;
      }

      if self.selecting_text {
        if let Some((cx, cy)) = self.cursor_pos {
          let doc_x = cx + self.viewport_scroll.0;
          let doc_y = cy + self.viewport_scroll.1;
          if let Some(tc) = crate::text_hit::hit_text_cursor(&tree, doc_x, doc_y, &mut self.text_ctx) {
            if let Some(sel) = self.text_selection.as_mut() {
              sel.focus = tc;
            }
          }
        }
      }
    }

    f(&mut tree, &mut self.text_ctx, effective_scale, vw, vh)
  }
}

fn apply_element_scroll_state(tree: &mut lui_layout::LayoutTree<'_>, state: &BTreeMap<Vec<usize>, (f32, f32)>) {
  for (path, (sx, sy)) in state {
    tree.set_scroll_at_path(path, *sx, *sy);
  }
}

fn flush_pending_events(lui: &mut Lui) {
  let events = std::mem::take(&mut lui.pending_events);
  if events.is_empty() {
    return;
  }
  let mut handlers = std::mem::take(&mut lui.event_handlers);
  for pending in &events {
    let event_type = pending.event.base().event_type.as_ref();
    for handler in handlers.iter_mut() {
      if handler.event_type == event_type {
        (handler.callback)(lui, &pending.event);
      }
    }
  }
  lui.event_handlers = handlers;
}

fn translate_display_list(list: &mut DisplayList, dx: f32, dy: f32) {
  for quad in &mut list.quads {
    quad.rect.x += dx;
    quad.rect.y += dy;
  }
  for image in &mut list.images {
    image.rect.x += dx;
    image.rect.y += dy;
  }
  for glyph in &mut list.glyphs {
    glyph.rect.x += dx;
    glyph.rect.y += dy;
  }
  for clip in &mut list.clips {
    if let Some(rect) = clip.rect.as_mut() {
      rect.x += dx;
      rect.y += dy;
    }
  }
}

struct WheelOutcome {
  changed_elements: Vec<(Vec<usize>, lui_layout::ScrollInfo)>,
  viewport_change: Option<(f32, f32)>,
}

fn common_prefix_len(a: &[usize], b: &[usize]) -> usize {
  a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
}

#[derive(Debug, Clone, Copy, Default)]
pub struct KeyModifiers {
  pub ctrl: bool,
  pub shift: bool,
  pub alt: bool,
  pub meta: bool,
}

const MOUSE_POINTER_ID: i32 = 1;

const DBLCLICK_THRESHOLD_MS: u128 = 500;
const DBLCLICK_DISTANCE_PX: f32 = 5.0;

struct ClickState {
  time: Instant,
  pos: (f32, f32),
  button: i16,
  count: u8,
}

const VIEWPORT_SCROLLBAR_MARGIN: f32 = 2.0;

fn viewport_scrollbar_hit(
  cx: f32,
  cy: f32,
  vw: f32,
  vh: f32,
  bar_w: f32,
  scroll_x: f32,
  scroll_y: f32,
  max_x: f32,
  max_y: f32,
) -> Option<lui_layout::ScrollbarHit> {
  let margin = VIEWPORT_SCROLLBAR_MARGIN;
  let show_y = max_y > 0.5;
  let show_x = max_x > 0.5;

  if show_y {
    let track_x = vw - bar_w - margin;
    let track_y = margin;
    let track_h = vh - margin * 2.0 - if show_x { bar_w } else { 0.0 };

    if cx >= track_x && cx <= track_x + bar_w && cy >= track_y && cy <= track_y + track_h {
      let doc_h = vh + max_y;
      let thumb_h = (track_h * vh / doc_h).clamp(24.0, track_h);
      let travel = (track_h - thumb_h).max(0.0);
      let thumb_y = track_y + travel * (scroll_y / max_y.max(1.0));
      let on_thumb = cy >= thumb_y && cy <= thumb_y + thumb_h;
      return Some(lui_layout::ScrollbarHit {
        path: Vec::new(),
        node_ptr: std::ptr::null(),
        axis: lui_layout::ScrollbarAxis::Vertical,
        on_thumb,
        grab_offset: if on_thumb { cy - thumb_y } else { thumb_h * 0.5 },
        track_start: track_y,
        track_length: track_h,
        thumb_length: thumb_h,
        max_scroll: max_y,
      });
    }
  }

  if show_x {
    let track_x = margin;
    let track_y = vh - bar_w - margin;
    let track_w = vw - margin * 2.0 - if show_y { bar_w } else { 0.0 };

    if cx >= track_x && cx <= track_x + track_w && cy >= track_y && cy <= track_y + bar_w {
      let doc_w = vw + max_x;
      let thumb_w = (track_w * vw / doc_w).clamp(24.0, track_w);
      let travel = (track_w - thumb_w).max(0.0);
      let thumb_x = track_x + travel * (scroll_x / max_x.max(1.0));
      let on_thumb = cx >= thumb_x && cx <= thumb_x + thumb_w;
      return Some(lui_layout::ScrollbarHit {
        path: Vec::new(),
        node_ptr: std::ptr::null(),
        axis: lui_layout::ScrollbarAxis::Horizontal,
        on_thumb,
        grab_offset: if on_thumb { cx - thumb_x } else { thumb_w * 0.5 },
        track_start: track_x,
        track_length: track_w,
        thumb_length: thumb_w,
        max_scroll: max_x,
      });
    }
  }

  None
}

fn is_focusable(node: &lui_parse::HtmlNode) -> bool {
  use lui_core::HtmlElement;
  let tabindex = node.attr("tabindex").and_then(|v| v.parse::<i32>().ok());
  if matches!(tabindex, Some(t) if t < 0) {
    return false;
  }
  match node.element() {
    HtmlElement::Input => {
      if node.attr("disabled").is_some() {
        return false;
      }
      !matches!(node.attr("type"), Some(t) if t.as_ref() == "hidden")
    }
    HtmlElement::Textarea | HtmlElement::Select | HtmlElement::Button => node.attr("disabled").is_none(),
    HtmlElement::A => node.attr("href").is_some(),
    HtmlElement::Summary => true,
    _ => tabindex.is_some_and(|t| t >= 0),
  }
}
