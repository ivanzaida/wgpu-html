//! Public types for the layout crate: geometry primitives, the
//! [`LayoutBox`] tree node, form-control helpers, and associated enums.

use std::collections::BTreeMap;

use lui_assets::ImageData;
use lui_models::common::css_enums::{BorderStyle, Cursor, Overflow, PointerEvents, Resize, TextOverflow, UserSelect};
use lui_text::{ShapedRun, TextContext};
use lui_tree::{Node, ScrollOffset, TextCursor, Tree};

use crate::color::Color;
use crate::hit_test;
use crate::layout_profile::LayoutProfiler;
use crate::ImageCache;

/// Axis-aligned rectangle in physical pixels, top-left origin.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
  pub x: f32,
  pub y: f32,
  pub w: f32,
  pub h: f32,
}

impl Rect {
  pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
    Self { x, y, w, h }
  }

  /// Half-open hit test: a point on the top/left edge is inside,
  /// a point on the bottom/right edge is not.
  pub fn contains(&self, x: f32, y: f32) -> bool {
    x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
  }
}

/// Per-side insets in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Insets {
  pub top: f32,
  pub right: f32,
  pub bottom: f32,
  pub left: f32,
}

impl Insets {
  pub const fn zero() -> Self {
    Self {
      top: 0.0,
      right: 0.0,
      bottom: 0.0,
      left: 0.0,
    }
  }
  pub fn horizontal(self) -> f32 {
    self.left + self.right
  }
  pub fn vertical(self) -> f32 {
    self.top + self.bottom
  }
}

/// One corner's radius. `h` is the horizontal extent, `v` the vertical.
/// Equal components describe a circular corner; otherwise an ellipse.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Radius {
  pub h: f32,
  pub v: f32,
}

impl Radius {
  pub const fn zero() -> Self {
    Self { h: 0.0, v: 0.0 }
  }
  pub const fn circle(r: f32) -> Self {
    Self { h: r, v: r }
  }
  pub fn is_zero(self) -> bool {
    self.h <= 0.0 && self.v <= 0.0
  }
}

/// Per-corner elliptical radii in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CornerRadii {
  pub top_left: Radius,
  pub top_right: Radius,
  pub bottom_right: Radius,
  pub bottom_left: Radius,
}

impl CornerRadii {
  pub const fn zero() -> Self {
    Self {
      top_left: Radius::zero(),
      top_right: Radius::zero(),
      bottom_right: Radius::zero(),
      bottom_left: Radius::zero(),
    }
  }
  pub fn any_nonzero(&self) -> bool {
    !self.top_left.is_zero() || !self.top_right.is_zero() || !self.bottom_right.is_zero() || !self.bottom_left.is_zero()
  }
}

/// Per-side border colors, top / right / bottom / left.
#[derive(Debug, Clone, Copy, Default)]
pub struct BorderColors {
  pub top: Option<Color>,
  pub right: Option<Color>,
  pub bottom: Option<Color>,
  pub left: Option<Color>,
}

impl BorderColors {
  pub fn any(&self) -> bool {
    self.top.is_some() || self.right.is_some() || self.bottom.is_some() || self.left.is_some()
  }
}

/// Per-side border styles. `None` means the property was unset; paint
/// treats that as `solid` for sides that have a width and a colour
/// (matching the convention `border: <w> <c>` implies "solid" in
/// CSS source — we already inflate the shorthand to `solid` in the
/// parser for that case, but per-side longhand can leave it unset).
#[derive(Debug, Clone, Default)]
pub struct BorderStyles {
  pub top: Option<BorderStyle>,
  pub right: Option<BorderStyle>,
  pub bottom: Option<BorderStyle>,
  pub left: Option<BorderStyle>,
}

/// One laid-out box. Coordinates are absolute (already translated for
/// every parent on the path); the paint pass just reads them.
#[derive(Debug, Clone)]
pub struct LayoutBox {
  /// Includes margin. Used for sibling stacking.
  pub margin_rect: Rect,
  /// The rect a background / border would paint (margin excluded).
  pub border_rect: Rect,
  /// The rect children are laid out into (margin + border + padding excluded).
  pub content_rect: Rect,
  /// Parsed box-shadow list. Empty means no shadows.
  pub box_shadows: Vec<crate::shadow::BoxShadow>,
  /// Resolved background color, if any.
  pub background: Option<Color>,
  /// Rectangle the background fills. Driven by `background-clip`:
  /// `border-box` (default) → equal to `border_rect`; `padding-box` →
  /// border-rect inset by border thickness; `content-box` → equal to
  /// `content_rect`.
  pub background_rect: Rect,
  /// Per-corner radii to use when painting the background — already
  /// reduced from the outer radii to match `background_rect`.
  pub background_radii: CornerRadii,
  /// Per-side border thickness, in physical pixels.
  pub border: Insets,
  /// Per-side border color. `None` for a side means that edge is
  /// skipped during paint (we don't track foreground color yet, so
  /// there's no spec-default fallback).
  pub border_colors: BorderColors,
  /// Per-side border style. Currently honoured for `solid`, `dashed`,
  /// `dotted`, `none`, `hidden`. Other values render as solid.
  pub border_styles: BorderStyles,
  /// Per-corner radii. Currently parsed and laid out but **not yet
  /// rendered** (paint emits straight-edge quads).
  pub border_radius: CornerRadii,
  pub kind: BoxKind,
  /// For text leaves (`BoxKind::Text`): the shaped + atlas-packed
  /// glyph run, in run-relative pixel coordinates with `(0, 0)` at
  /// the line-box top-left. `None` when no font is registered or
  /// the text was empty.
  pub text_run: Option<ShapedRun>,
  /// For text leaves: the resolved foreground color used when paint
  /// emits glyph quads. Defaults to opaque black if unset.
  pub text_color: Option<Color>,
  /// When `true`, the `text_run` is a form control's internal
  /// content (placeholder or typed value) and should not participate
  /// in document-level drag-to-select. Mirrors browsers, where
  /// `::placeholder` and input values are excluded from the
  /// document's selectable text.
  pub text_unselectable: bool,
  /// CSS `text-decoration-line`s active on this box (parsed from
  /// the `text-decoration` shorthand). Painted as solid quads at
  /// the appropriate vertical offset for text leaves.
  pub text_decorations: Vec<TextDecorationLine>,
  /// Effective `overflow-x` / `overflow-y` values after shorthand,
  /// longhand, and cross-axis computed-value resolution. `Visible`
  /// is the no-op default; clipping values ask the paint pass and
  /// hit testing to constrain descendants on the matching axis.
  pub overflow: OverflowAxes,
  pub resize: Resize,
  pub text_overflow: Option<TextOverflow>,
  /// Computed CSS 2D transform, composed into a single affine matrix.
  /// `None` means no transform (identity). Paint applies this to all
  /// rects emitted for this box and its descendants.
  pub transform: Option<crate::transform::Transform2D>,
  /// Transform origin in pixels relative to the border-box top-left.
  pub transform_origin: (f32, f32),
  /// Computed CSS opacity for this element. Paint multiplies this
  /// into the inherited opacity for the whole subtree.
  pub opacity: f32,
  /// Resolved `pointer-events`. `None` means hit-testing passes
  /// through this element to whatever is behind it, but children
  /// with `Auto` are still hittable (CSS spec).
  pub pointer_events: PointerEvents,
  /// Resolved `user-select`. `None` suppresses text selection and
  /// highlight painting for this box (inherited to descendants).
  pub user_select: UserSelect,
  /// Resolved CSS `cursor`. Used by the host to set the OS pointer.
  pub cursor: Cursor,
  /// Resolved CSS `z-index`. `None` means `auto`. Only meaningful
  /// for positioned elements (`absolute` / `relative` / `fixed`).
  pub z_index: Option<i32>,
  /// Decoded image data for `<img>` elements. `None` for non-image
  /// boxes. The `Arc` allows cheap cloning through the display list.
  pub image: Option<ImageData>,
  /// Pre-computed CSS `background-image` paint info, if the element
  /// has one. Carries the decoded RGBA texture data plus a list of
  /// already-positioned tile rectangles (one for `no-repeat`, many
  /// for `repeat` modes). The painter just iterates the tiles.
  pub background_image: Option<BackgroundImagePaint>,
  pub first_line_color: Option<Color>,
  pub first_letter_color: Option<Color>,
  pub selection_bg: Option<Color>,
  pub selection_fg: Option<Color>,
  /// Resolved CSS `accent-color`. Used by form-control paint to tint
  /// checked checkboxes, radio buttons, and range slider thumbs/fills.
  pub accent_color: Option<Color>,
  /// Resolved `--lui-*` vendor custom properties.
  pub lui: LuiProperties,
  pub lui_popup: Option<std::sync::Arc<lui_models::LuiPopupStyle>>,
  pub lui_color_picker: Option<std::sync::Arc<lui_models::LuiColorPickerStyle>>,
  pub lui_calendar: Option<std::sync::Arc<lui_models::LuiCalendarStyle>>,
  pub file_button: Option<FileButtonStyle>,
  pub children: Vec<LayoutBox>,
  /// `true` when `position: fixed` so paint knows to counter
  /// viewport scroll translation.
  pub is_fixed: bool,
  pub form_control: Option<FormControlInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormControlKind {
  Checkbox { checked: bool },
  Radio { checked: bool },
  Range { value: f32, min: f32, max: f32 },
  Color { r: f32, g: f32, b: f32, a: f32 },
  Date { year: i32, month: u8, day: u8 },
  DatetimeLocal { year: i32, month: u8, day: u8, hour: u8, minute: u8 },
  File { file_name: Option<String>, file_count: usize, disabled: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormControlInfo {
  pub kind: FormControlKind,
}

#[derive(Debug, Clone)]
pub struct FileButtonStyle {
  pub background: Option<Color>,
  pub color: Option<Color>,
  pub border_color: Option<Color>,
  pub border_radius: f32,
  pub padding: [f32; 4],
  pub cursor: Cursor,
  pub text_run: Option<ShapedRun>,
}

impl Default for FileButtonStyle {
  fn default() -> Self {
    Self {
      background: Some([0.93, 0.93, 0.93, 1.0]),
      color: Some([0.0, 0.0, 0.0, 1.0]),
      border_color: Some([0.6, 0.6, 0.6, 1.0]),
      border_radius: 3.0,
      padding: [4.0, 6.0, 4.0, 6.0],
      cursor: Cursor::Pointer,
      text_run: None,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LuiProperties {
  pub track_color: Option<Color>,
  pub thumb_color: Option<Color>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverflowAxes {
  pub x: Overflow,
  pub y: Overflow,
  pub scrollbar_width: f32,
  pub scrollbar_thumb: Option<Color>,
  pub scrollbar_track: Option<Color>,
}

impl OverflowAxes {
  pub const fn visible() -> Self {
    Self {
      x: Overflow::Visible,
      y: Overflow::Visible,
      scrollbar_width: 10.0,
      scrollbar_thumb: None,
      scrollbar_track: None,
    }
  }

  pub const fn clips_x(self) -> bool {
    !matches!(self.x, Overflow::Visible)
  }

  pub const fn clips_y(self) -> bool {
    !matches!(self.y, Overflow::Visible)
  }

  pub const fn clips_both(self) -> bool {
    self.clips_x() && self.clips_y()
  }

  pub const fn clips_any(self) -> bool {
    self.clips_x() || self.clips_y()
  }
}

/// Pre-computed CSS `background-image` paint metadata. The texture
/// is uploaded once per `image_id`; the painter emits one image quad
/// per entry in `tiles`.
#[derive(Debug, Clone)]
pub struct BackgroundImagePaint {
  /// Same `image_id` scheme as [`ImageData`] — keys the renderer's
  /// GPU texture cache.
  pub image_id: u64,
  /// Decoded RGBA8 bytes for the source image (intrinsic size).
  pub data: std::sync::Arc<Vec<u8>>,
  /// Texture upload size (intrinsic dimensions of the decoded
  /// image). Each tile rect is drawn at its own size; UVs map the
  /// full `[0,1]²` to the rect, so the renderer stretches as
  /// needed for `cover` / `contain` / explicit lengths.
  pub width: u32,
  pub height: u32,
  /// On-screen rectangles to paint into. Already positioned in
  /// absolute physical pixels and filtered to those that overlap
  /// the box's `background_rect`.
  pub tiles: Vec<Rect>,
}

/// Single decoration line drawn over / under / through a text run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDecorationLine {
  Underline,
  LineThrough,
  Overline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxKind {
  /// A regular block element.
  Block,
  /// A text leaf. The shaped run lives in `LayoutBox::text_run`.
  Text,
}

impl LayoutBox {
  /// Whether this subtree contains any animated image (GIF, APNG,
  /// animated WebP). Used by the render loop to decide whether to
  /// keep requesting redraws for frame advancement.
  pub fn has_animated_images(&self) -> bool {
    if self.image.as_ref().is_some_and(|img| img.frames.is_some()) {
      return true;
    }
    self.children.iter().any(|c| c.has_animated_images())
  }

  /// Index path from `self` to the deepest descendant whose
  /// `border_rect` contains `point`. An empty path means `self` is the
  /// deepest match. `None` if the point is outside `self`.
  ///
  /// The layout tree mirrors the source element tree 1:1, so this
  /// path can be applied to a [`Tree`] / [`Node`] to navigate to the
  /// corresponding element.
  pub fn hit_path(&self, point: (f32, f32)) -> Option<Vec<usize>> {
    let (x, y) = point;
    hit_test::collect_hit_path(self, x, y, None)
  }

  /// Like [`hit_path`] but compensates for per-element scroll
  /// offsets. Each `overflow:scroll` / `overflow:auto` container
  /// shifts the test point by its scroll offset so children
  /// scrolled into view are correctly matched.
  ///
  /// `scroll_offsets` is the same map stored in
  /// `InteractionState::scroll_offsets`.
  pub fn hit_path_scrolled(
    &self,
    point: (f32, f32),
    scroll_offsets: &BTreeMap<Vec<usize>, ScrollOffset>,
  ) -> Option<Vec<usize>> {
    let mut path = Vec::new();
    hit_test::collect_hit_path_scrolled(self, point.0, point.1, scroll_offsets, &mut path, None)
  }

  /// Resolve the CSS cursor for a hit-tested path. Walks from the
  /// deepest element to the root and returns the first non-Auto
  /// cursor found (matching CSS inheritance behaviour).
  pub fn cursor_at_path(&self, path: &[usize]) -> Cursor {
    // Walk from deepest to root, return the first non-Auto cursor.
    for depth in (0..=path.len()).rev() {
      if let Some(b) = self.box_at_path(&path[..depth]) {
        if !matches!(b.cursor, Cursor::Auto) {
          return b.cursor.clone();
        }
      }
    }
    Cursor::Auto
  }

  /// Like [`hit_text_cursor`] but scroll-aware.
  pub fn hit_text_cursor_scrolled(
    &self,
    point: (f32, f32),
    scroll_offsets: &BTreeMap<Vec<usize>, ScrollOffset>,
  ) -> Option<TextCursor> {
    let path = self.hit_path_scrolled(point, scroll_offsets)?;
    let text_box = self.box_at_path(&path)?;
    if text_box.text_unselectable || text_box.user_select == UserSelect::None {
      return None;
    }
    let run = text_box.text_run.as_ref()?;
    Some(TextCursor {
      path,
      glyph_index: hit_test::hit_glyph_boundary(text_box, run, point),
    })
  }

  /// Hit-test the layout at `point` and return a mutable reference
  /// to the matching element node in `tree`. Use this to read or
  /// modify the source element (style, text, attributes, etc.).
  /// Returns `None` if the point is outside the hit-tested rendered
  /// area or the tree has
  /// no root.
  ///
  /// `tree` must be the same tree this layout was produced from; we
  /// rely on the layout's child structure mirroring the element
  /// tree's child structure 1:1.
  ///
  /// On overlap, children are walked last-to-first so the topmost
  /// (last-painted) hit wins.
  pub fn find_element_from_point<'a>(&self, tree: &'a mut Tree, point: (f32, f32)) -> Option<&'a mut Node> {
    let path = self.hit_path(point)?;
    tree.root.as_mut()?.at_path_mut(&path)
  }

  /// Like [`Self::find_element_from_point`] but returns the entire
  /// ancestor chain of element nodes from the deepest hit up to (and
  /// including) the tree root. Empty when the point is outside or
  /// the tree has no root.
  ///
  /// Soundness inherits from [`Node::ancestry_at_path_mut`]: the
  /// returned `&mut` references alias into nested subtrees of the
  /// same borrow, so two of them must never be dereferenced at the
  /// same time.
  pub fn find_elements_from_point<'a>(&self, tree: &'a mut Tree, point: (f32, f32)) -> Vec<&'a mut Node> {
    let Some(path) = self.hit_path(point) else {
      return Vec::new();
    };
    let Some(root) = tree.root.as_mut() else {
      return Vec::new();
    };
    root.ancestry_at_path_mut(&path)
  }

  /// Hit-test a text caret position at `point`.
  ///
  /// Returns a cursor into the deepest text run under the pointer.
  /// If the deepest hit box is not text, returns `None`.
  pub fn hit_text_cursor(&self, point: (f32, f32)) -> Option<TextCursor> {
    let path = self.hit_path(point)?;
    let text_box = self.box_at_path(&path)?;
    // Form control internal text (placeholder / value) and elements
    // with user-select: none are excluded from drag-to-select.
    if text_box.text_unselectable || text_box.user_select == UserSelect::None {
      return None;
    }
    let run = text_box.text_run.as_ref()?;
    Some(TextCursor {
      path,
      glyph_index: hit_test::hit_glyph_boundary(text_box, run, point),
    })
  }

  /// Return the cursor position relative to the element at
  /// `path`, or `None` if the cursor is not over that element.
  /// The returned coordinates are relative to the element's
  /// `border_rect` origin.
  ///
  /// `tree` provides the current cursor position and hover path.
  pub fn cursor_position_in(&self, tree: &lui_tree::Tree, path: &[usize]) -> Option<(f32, f32)> {
    let pos = tree.cursor_position()?;
    if !tree.is_hovered(path) {
      return None;
    }
    let b = self.box_at_path(path)?;
    Some((pos.0 - b.border_rect.x, pos.1 - b.border_rect.y))
  }

  /// Return the box at `path` (empty path means `self`).
  pub fn box_at_path(&self, path: &[usize]) -> Option<&LayoutBox> {
    let mut cursor = self;
    for &i in path {
      cursor = cursor.children.get(i)?;
    }
    Some(cursor)
  }
}

// ---------------------------------------------------------------------------
// Layout context types — shared across block, flex, grid, inline, and
// text-shaping paths.
// ---------------------------------------------------------------------------

pub(crate) struct Ctx<'a> {
  pub viewport_w: f32,
  pub viewport_h: f32,
  pub scale: f32,
  pub text: TextCtx<'a>,
  pub images: &'a mut ImageCache,
  pub profiler: Option<LayoutProfiler>,
  pub locale: &'a dyn lui_tree::Locale,
  pub date_display_value: Option<String>,
  pub date_focus_iso: Option<String>,
}

/// Wrapper so `Ctx` can borrow a `&mut TextContext` without forcing
/// every caller to thread a lifetime through. Passes shaping calls
/// through to the underlying context.
pub(crate) struct TextCtx<'a> {
  pub ctx: &'a mut TextContext,
}
