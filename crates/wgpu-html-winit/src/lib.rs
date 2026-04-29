//! winit ↔ wgpu-html glue.
//!
//! Two layers in this crate:
//!
//! 1. **Type translators / forwarders** (the bulk of this file).
//!    Free functions that turn winit's `KeyEvent` / `MouseButton` /
//!    `KeyCode` into the engine's [`MouseButton`], [`Modifier`],
//!    and DOM-style key-name strings, and feed them into the
//!    dispatch API on [`Tree`]. Apps that already have their own
//!    event loop wire these in by hand.
//!
//! 2. **A batteries-included harness** ([`WgpuHtmlWindow`]) that
//!    owns the window, renderer, text context, and the full
//!    `winit::ApplicationHandler` impl. A new app reduces to:
//!
//!    ```ignore
//!    let mut tree = build_tree();
//!    wgpu_html_winit::create_window(&mut tree)
//!        .with_title("My App")
//!        .with_size(1280, 720)
//!        .run()
//!        .unwrap();
//!    ```
//!
//!    The harness handles cursor tracking, mouse / keyboard
//!    forwarding, focus + Tab navigation (via the tree
//!    dispatchers), resize, and the cascade → layout → paint →
//!    render loop on every redraw. It's intentionally minimal —
//!    no profiling, no scrollbar drag, no clipboard. The demo
//!    crate is the richer example; copy from it when you need
//!    more.
//!
//! Mouse position dispatch (hit-testing against a `LayoutBox`)
//! lives in `wgpu-html`; the harness uses those wrappers
//! internally.

mod fonts;
mod window;

pub use fonts::{
    SystemFontVariant, LUCIDE_FONT_DATA, register_lucide_icons, register_system_fonts,
    system_font_variants,
};
pub use window::{
    AppHook, EventResponse, FrameTimings, HookContext, WgpuHtmlWindow, create_window,
};

use wgpu_html_tree::{Modifier, MouseButton, Tree};
use winit::event::{ElementState, KeyEvent, MouseButton as WinitMouseButton};
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};

// ── Type translators ────────────────────────────────────────────────────────

/// Map a winit `MouseButton` to the engine's [`MouseButton`].
pub fn mouse_button(button: WinitMouseButton) -> MouseButton {
    match button {
        WinitMouseButton::Left => MouseButton::Primary,
        WinitMouseButton::Right => MouseButton::Secondary,
        WinitMouseButton::Middle => MouseButton::Middle,
        WinitMouseButton::Back => MouseButton::Other(3),
        WinitMouseButton::Forward => MouseButton::Other(4),
        WinitMouseButton::Other(n) => MouseButton::Other(n.min(255) as u8),
    }
}

/// Map a winit `KeyCode` to a [`Modifier`] if it is one of the
/// four modifier keys; `None` otherwise.
pub fn keycode_to_modifier(key: KeyCode) -> Option<Modifier> {
    Some(match key {
        KeyCode::ControlLeft | KeyCode::ControlRight => Modifier::Ctrl,
        KeyCode::ShiftLeft | KeyCode::ShiftRight => Modifier::Shift,
        KeyCode::AltLeft | KeyCode::AltRight => Modifier::Alt,
        KeyCode::SuperLeft | KeyCode::SuperRight => Modifier::Meta,
        _ => return None,
    })
}

/// Map a winit `NamedKey` to its DOM `KeyboardEvent.key` string.
/// Printable characters don't come through here — they arrive as
/// `Key::Character(ch)` and are used directly.
fn named_key_to_dom(key: &NamedKey) -> &'static str {
    match key {
        NamedKey::Alt => "Alt",
        NamedKey::ArrowDown => "ArrowDown",
        NamedKey::ArrowLeft => "ArrowLeft",
        NamedKey::ArrowRight => "ArrowRight",
        NamedKey::ArrowUp => "ArrowUp",
        NamedKey::Backspace => "Backspace",
        NamedKey::CapsLock => "CapsLock",
        NamedKey::Control => "Control",
        NamedKey::Delete => "Delete",
        NamedKey::End => "End",
        NamedKey::Enter => "Enter",
        NamedKey::Escape => "Escape",
        NamedKey::F1 => "F1",
        NamedKey::F2 => "F2",
        NamedKey::F3 => "F3",
        NamedKey::F4 => "F4",
        NamedKey::F5 => "F5",
        NamedKey::F6 => "F6",
        NamedKey::F7 => "F7",
        NamedKey::F8 => "F8",
        NamedKey::F9 => "F9",
        NamedKey::F10 => "F10",
        NamedKey::F11 => "F11",
        NamedKey::F12 => "F12",
        NamedKey::Home => "Home",
        NamedKey::Insert => "Insert",
        NamedKey::Meta => "Meta",
        NamedKey::NumLock => "NumLock",
        NamedKey::PageDown => "PageDown",
        NamedKey::PageUp => "PageUp",
        NamedKey::Pause => "Pause",
        NamedKey::PrintScreen => "PrintScreen",
        NamedKey::ScrollLock => "ScrollLock",
        NamedKey::Shift => "Shift",
        NamedKey::Space => " ",
        NamedKey::Tab => "Tab",
        _ => "Unidentified",
    }
}

/// Map a winit physical `KeyCode` to a DOM `KeyboardEvent.key`
/// string, choosing between the unshifted and shifted character
/// where applicable.
///
/// **Deprecated**: prefer `event.logical_key` (via `named_key_to_dom`
/// + `Key::Character`) which respects the user's actual keyboard
/// layout. This function assumes US-QWERTY and is kept only for
/// callers that lack a `KeyEvent`.
pub fn key_to_dom_key(key: KeyCode, shift: bool) -> &'static str {
    use KeyCode::*;
    match key {
        // Letters: shift alters the case of the character.
        KeyA => {
            if shift {
                "A"
            } else {
                "a"
            }
        }
        KeyB => {
            if shift {
                "B"
            } else {
                "b"
            }
        }
        KeyC => {
            if shift {
                "C"
            } else {
                "c"
            }
        }
        KeyD => {
            if shift {
                "D"
            } else {
                "d"
            }
        }
        KeyE => {
            if shift {
                "E"
            } else {
                "e"
            }
        }
        KeyF => {
            if shift {
                "F"
            } else {
                "f"
            }
        }
        KeyG => {
            if shift {
                "G"
            } else {
                "g"
            }
        }
        KeyH => {
            if shift {
                "H"
            } else {
                "h"
            }
        }
        KeyI => {
            if shift {
                "I"
            } else {
                "i"
            }
        }
        KeyJ => {
            if shift {
                "J"
            } else {
                "j"
            }
        }
        KeyK => {
            if shift {
                "K"
            } else {
                "k"
            }
        }
        KeyL => {
            if shift {
                "L"
            } else {
                "l"
            }
        }
        KeyM => {
            if shift {
                "M"
            } else {
                "m"
            }
        }
        KeyN => {
            if shift {
                "N"
            } else {
                "n"
            }
        }
        KeyO => {
            if shift {
                "O"
            } else {
                "o"
            }
        }
        KeyP => {
            if shift {
                "P"
            } else {
                "p"
            }
        }
        KeyQ => {
            if shift {
                "Q"
            } else {
                "q"
            }
        }
        KeyR => {
            if shift {
                "R"
            } else {
                "r"
            }
        }
        KeyS => {
            if shift {
                "S"
            } else {
                "s"
            }
        }
        KeyT => {
            if shift {
                "T"
            } else {
                "t"
            }
        }
        KeyU => {
            if shift {
                "U"
            } else {
                "u"
            }
        }
        KeyV => {
            if shift {
                "V"
            } else {
                "v"
            }
        }
        KeyW => {
            if shift {
                "W"
            } else {
                "w"
            }
        }
        KeyX => {
            if shift {
                "X"
            } else {
                "x"
            }
        }
        KeyY => {
            if shift {
                "Y"
            } else {
                "y"
            }
        }
        KeyZ => {
            if shift {
                "Z"
            } else {
                "z"
            }
        }
        // Top-row digits (US layout).
        Digit0 => {
            if shift {
                ")"
            } else {
                "0"
            }
        }
        Digit1 => {
            if shift {
                "!"
            } else {
                "1"
            }
        }
        Digit2 => {
            if shift {
                "@"
            } else {
                "2"
            }
        }
        Digit3 => {
            if shift {
                "#"
            } else {
                "3"
            }
        }
        Digit4 => {
            if shift {
                "$"
            } else {
                "4"
            }
        }
        Digit5 => {
            if shift {
                "%"
            } else {
                "5"
            }
        }
        Digit6 => {
            if shift {
                "^"
            } else {
                "6"
            }
        }
        Digit7 => {
            if shift {
                "&"
            } else {
                "7"
            }
        }
        Digit8 => {
            if shift {
                "*"
            } else {
                "8"
            }
        }
        Digit9 => {
            if shift {
                "("
            } else {
                "9"
            }
        }
        // Punctuation (US layout).
        Space => " ",
        Minus => {
            if shift {
                "_"
            } else {
                "-"
            }
        }
        Equal => {
            if shift {
                "+"
            } else {
                "="
            }
        }
        BracketLeft => {
            if shift {
                "{"
            } else {
                "["
            }
        }
        BracketRight => {
            if shift {
                "}"
            } else {
                "]"
            }
        }
        Backslash => {
            if shift {
                "|"
            } else {
                "\\"
            }
        }
        Semicolon => {
            if shift {
                ":"
            } else {
                ";"
            }
        }
        Quote => {
            if shift {
                "\""
            } else {
                "'"
            }
        }
        Comma => {
            if shift {
                "<"
            } else {
                ","
            }
        }
        Period => {
            if shift {
                ">"
            } else {
                "."
            }
        }
        Slash => {
            if shift {
                "?"
            } else {
                "/"
            }
        }
        Backquote => {
            if shift {
                "~"
            } else {
                "`"
            }
        }
        // Editing / navigation.
        Enter | NumpadEnter => "Enter",
        Tab => "Tab",
        Backspace => "Backspace",
        Delete => "Delete",
        Escape => "Escape",
        Home => "Home",
        End => "End",
        PageUp => "PageUp",
        PageDown => "PageDown",
        ArrowUp => "ArrowUp",
        ArrowDown => "ArrowDown",
        ArrowLeft => "ArrowLeft",
        ArrowRight => "ArrowRight",
        Insert => "Insert",
        // Modifier keys produce a dedicated key string.
        ShiftLeft | ShiftRight => "Shift",
        ControlLeft | ControlRight => "Control",
        AltLeft | AltRight => "Alt",
        SuperLeft | SuperRight => "Meta",
        CapsLock => "CapsLock",
        // Function keys.
        F1 => "F1",
        F2 => "F2",
        F3 => "F3",
        F4 => "F4",
        F5 => "F5",
        F6 => "F6",
        F7 => "F7",
        F8 => "F8",
        F9 => "F9",
        F10 => "F10",
        F11 => "F11",
        F12 => "F12",
        _ => "Unidentified",
    }
}

/// Map a winit physical `KeyCode` to a DOM `KeyboardEvent.code`
/// string. The `code` value is layout-independent (always reflects
/// the physical key), so this is a one-to-one mapping for the
/// keys the engine cares about.
pub fn keycode_to_dom_code(key: KeyCode) -> &'static str {
    use KeyCode::*;
    match key {
        KeyA => "KeyA",
        KeyB => "KeyB",
        KeyC => "KeyC",
        KeyD => "KeyD",
        KeyE => "KeyE",
        KeyF => "KeyF",
        KeyG => "KeyG",
        KeyH => "KeyH",
        KeyI => "KeyI",
        KeyJ => "KeyJ",
        KeyK => "KeyK",
        KeyL => "KeyL",
        KeyM => "KeyM",
        KeyN => "KeyN",
        KeyO => "KeyO",
        KeyP => "KeyP",
        KeyQ => "KeyQ",
        KeyR => "KeyR",
        KeyS => "KeyS",
        KeyT => "KeyT",
        KeyU => "KeyU",
        KeyV => "KeyV",
        KeyW => "KeyW",
        KeyX => "KeyX",
        KeyY => "KeyY",
        KeyZ => "KeyZ",
        Digit0 => "Digit0",
        Digit1 => "Digit1",
        Digit2 => "Digit2",
        Digit3 => "Digit3",
        Digit4 => "Digit4",
        Digit5 => "Digit5",
        Digit6 => "Digit6",
        Digit7 => "Digit7",
        Digit8 => "Digit8",
        Digit9 => "Digit9",
        Space => "Space",
        Minus => "Minus",
        Equal => "Equal",
        BracketLeft => "BracketLeft",
        BracketRight => "BracketRight",
        Backslash => "Backslash",
        Semicolon => "Semicolon",
        Quote => "Quote",
        Comma => "Comma",
        Period => "Period",
        Slash => "Slash",
        Backquote => "Backquote",
        Enter => "Enter",
        NumpadEnter => "NumpadEnter",
        Tab => "Tab",
        Backspace => "Backspace",
        Delete => "Delete",
        Escape => "Escape",
        Home => "Home",
        End => "End",
        PageUp => "PageUp",
        PageDown => "PageDown",
        ArrowUp => "ArrowUp",
        ArrowDown => "ArrowDown",
        ArrowLeft => "ArrowLeft",
        ArrowRight => "ArrowRight",
        Insert => "Insert",
        ShiftLeft => "ShiftLeft",
        ShiftRight => "ShiftRight",
        ControlLeft => "ControlLeft",
        ControlRight => "ControlRight",
        AltLeft => "AltLeft",
        AltRight => "AltRight",
        SuperLeft => "MetaLeft",
        SuperRight => "MetaRight",
        CapsLock => "CapsLock",
        F1 => "F1",
        F2 => "F2",
        F3 => "F3",
        F4 => "F4",
        F5 => "F5",
        F6 => "F6",
        F7 => "F7",
        F8 => "F8",
        F9 => "F9",
        F10 => "F10",
        F11 => "F11",
        F12 => "F12",
        _ => "Unidentified",
    }
}

// ── Forwarders ──────────────────────────────────────────────────────────────

/// If `key` is a modifier key (Ctrl/Shift/Alt/Meta), update the
/// corresponding bit of `tree.interaction.modifiers` based on
/// `state`. Returns `true` if a modifier was changed; `false` if
/// `key` is not a modifier.
pub fn update_modifiers(tree: &mut Tree, key: KeyCode, state: ElementState) -> bool {
    let Some(modifier) = keycode_to_modifier(key) else {
        return false;
    };
    let down = state == ElementState::Pressed;
    tree.set_modifier(modifier, down);
    true
}

/// Forward a winit `KeyEvent` into the tree's keyboard dispatcher.
///
/// Translates `event.physical_key` into the DOM `key` and `code`
/// strings (using the tree's stored shift state for case folding)
/// and calls [`Tree::key_down`] / [`Tree::key_up`] depending on
/// `event.state`.
///
/// Returns `false` if `event.physical_key` is not a recognised
/// `KeyCode` (`PhysicalKey::Unidentified(_)`); `true` otherwise.
///
/// Does NOT update modifier state — call [`update_modifiers`]
/// (or [`handle_keyboard`]) for that.
pub fn forward_keyboard(tree: &mut Tree, event: &KeyEvent) -> bool {
    let PhysicalKey::Code(key) = event.physical_key else {
        return false;
    };
    let code_str = keycode_to_dom_code(key);
    // Derive the DOM `key` string from winit's `logical_key`, which
    // respects the user's actual keyboard layout. This replaces the
    // old US-QWERTY `key_to_dom_key` physical-key map.
    let logical_key_string: String;
    let key_str: &str = match &event.logical_key {
        Key::Named(named) => named_key_to_dom(named),
        Key::Character(ch) => {
            logical_key_string = ch.to_string();
            &logical_key_string
        }
        Key::Unidentified(_) | Key::Dead(_) => "Unidentified",
    };
    match event.state {
        ElementState::Pressed => {
            tree.key_down(key_str, code_str, event.repeat);
            // Feed typed text into the focused form control.
            // `KeyEvent.text` gives the correctly composed character
            // for the user's keyboard layout (winit 0.30).
            // Skip when Ctrl or Meta is held — those are shortcuts
            // (Ctrl+C, Ctrl+V, etc.), not text insertion.
            if !tree.modifiers().ctrl && !tree.modifiers().meta {
                if let Some(ref text) = event.text {
                    let s = text.as_str();
                    // Filter out control characters — Backspace, Delete,
                    // Enter, etc. are handled by `handle_edit_key` inside
                    // `key_down`. Only printable text passes through.
                    if !s.is_empty() && s.chars().all(|c| !c.is_control()) {
                        wgpu_html_tree::text_input(tree, s);
                    }
                }
            }
        }
        ElementState::Released => {
            tree.key_up(key_str, code_str);
        }
    }
    true
}

/// One-call keyboard handler: updates modifier state if the key
/// is a modifier, then forwards the keyboard event in all cases
/// (modifier keys still fire `keydown` / `keyup`, matching browser
/// behaviour).
///
/// Returns `false` if `event.physical_key` is unrecognised;
/// `true` otherwise.
pub fn handle_keyboard(tree: &mut Tree, event: &KeyEvent) -> bool {
    if let PhysicalKey::Code(key) = event.physical_key {
        update_modifiers(tree, key, event.state);
    }
    forward_keyboard(tree, event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu_html_tree::Node;

    #[test]
    fn mouse_button_maps_known_buttons() {
        assert!(matches!(
            mouse_button(WinitMouseButton::Left),
            MouseButton::Primary
        ));
        assert!(matches!(
            mouse_button(WinitMouseButton::Right),
            MouseButton::Secondary
        ));
        assert!(matches!(
            mouse_button(WinitMouseButton::Middle),
            MouseButton::Middle
        ));
        assert!(matches!(
            mouse_button(WinitMouseButton::Back),
            MouseButton::Other(3)
        ));
        assert!(matches!(
            mouse_button(WinitMouseButton::Forward),
            MouseButton::Other(4)
        ));
        assert!(matches!(
            mouse_button(WinitMouseButton::Other(7)),
            MouseButton::Other(7)
        ));
    }

    #[test]
    fn keycode_to_modifier_recognises_modifier_keys() {
        assert_eq!(
            keycode_to_modifier(KeyCode::ControlLeft),
            Some(Modifier::Ctrl)
        );
        assert_eq!(
            keycode_to_modifier(KeyCode::ShiftRight),
            Some(Modifier::Shift)
        );
        assert_eq!(keycode_to_modifier(KeyCode::AltLeft), Some(Modifier::Alt));
        assert_eq!(
            keycode_to_modifier(KeyCode::SuperRight),
            Some(Modifier::Meta)
        );
        assert_eq!(keycode_to_modifier(KeyCode::KeyA), None);
        assert_eq!(keycode_to_modifier(KeyCode::Tab), None);
    }

    #[test]
    fn key_to_dom_key_handles_shift() {
        assert_eq!(key_to_dom_key(KeyCode::KeyA, false), "a");
        assert_eq!(key_to_dom_key(KeyCode::KeyA, true), "A");
        assert_eq!(key_to_dom_key(KeyCode::Digit1, false), "1");
        assert_eq!(key_to_dom_key(KeyCode::Digit1, true), "!");
        assert_eq!(key_to_dom_key(KeyCode::Tab, false), "Tab");
        assert_eq!(key_to_dom_key(KeyCode::Tab, true), "Tab");
    }

    #[test]
    fn keycode_to_dom_code_is_layout_independent() {
        // Same code regardless of whether shift is held.
        assert_eq!(keycode_to_dom_code(KeyCode::KeyA), "KeyA");
        assert_eq!(keycode_to_dom_code(KeyCode::Digit1), "Digit1");
        assert_eq!(keycode_to_dom_code(KeyCode::ShiftLeft), "ShiftLeft");
        assert_eq!(keycode_to_dom_code(KeyCode::SuperLeft), "MetaLeft");
    }

    #[test]
    fn update_modifiers_flips_only_modifier_keys() {
        let mut tree = Tree::new(Node::new("text"));
        assert!(!tree.modifiers().shift);
        assert!(update_modifiers(
            &mut tree,
            KeyCode::ShiftLeft,
            ElementState::Pressed
        ));
        assert!(tree.modifiers().shift);
        assert!(update_modifiers(
            &mut tree,
            KeyCode::ShiftLeft,
            ElementState::Released
        ));
        assert!(!tree.modifiers().shift);
        // Non-modifier keys leave the bitmask alone.
        assert!(!update_modifiers(
            &mut tree,
            KeyCode::KeyA,
            ElementState::Pressed
        ));
        assert_eq!(tree.modifiers(), Default::default());
    }
}
