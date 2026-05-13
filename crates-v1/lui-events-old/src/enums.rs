//! Enum types used across the DOM event hierarchy.

/// Which phase of event dispatch the event is currently in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EventPhase {
  /// The event is not being dispatched.
  #[default]
  None,
  /// The event is propagating through the target's ancestors in capture order.
  CapturingPhase,
  /// The event has arrived at the event's target.
  AtTarget,
  /// The event is propagating through the target's ancestors in bubbling order.
  BubblingPhase,
}

/// The kind of pointing device that produced a `PointerEvent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerType {
  Mouse,
  Pen,
  Touch,
}

/// Unit in which `WheelEvent` delta values are expressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WheelDeltaMode {
  /// Delta is in CSS pixels.
  Pixel,
  /// Delta is in lines.
  Line,
  /// Delta is in pages.
  Page,
}

/// Key position on the keyboard for `KeyboardEvent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyboardLocation {
  Standard,
  Left,
  Right,
  Numpad,
}

/// State of a `<details>` or similar togglable element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToggleState {
  Open,
  Closed,
}

/// The type of text editing action that produced an `InputEvent`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputType {
  InsertText,
  InsertReplacementText,
  InsertLineBreak,
  InsertParagraph,
  InsertOrderedList,
  InsertUnorderedList,
  InsertHorizontalRule,
  InsertFromYank,
  InsertFromDrop,
  InsertFromPaste,
  InsertFromPasteAsQuotation,
  InsertTranspose,
  InsertCompositionText,
  InsertLink,
  DeleteWordBackward,
  DeleteWordForward,
  DeleteSoftLineBackward,
  DeleteSoftLineForward,
  DeleteEntireSoftLine,
  DeleteHardLineBackward,
  DeleteHardLineForward,
  DeleteByDrag,
  DeleteByCut,
  DeleteContent,
  DeleteContentBackward,
  DeleteContentForward,
  HistoryUndo,
  HistoryRedo,
  FormatBold,
  FormatItalic,
  FormatUnderline,
  FormatStrikeThrough,
  FormatSuperscript,
  FormatSubscript,
  FormatJustifyFull,
  FormatJustifyCenter,
  FormatJustifyRight,
  FormatJustifyLeft,
  FormatIndent,
  FormatOutdent,
  FormatRemove,
  FormatSetBlockTextDirection,
  FormatSetInlineTextDirection,
  FormatBackColor,
  FormatFontColor,
  FormatFontName,
}
