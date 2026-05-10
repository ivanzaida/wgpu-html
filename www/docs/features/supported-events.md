---
sidebar_position: 4
---

# Supported Events

All events use the typed event system in `lui-events`. Each event carries `target_path`, `current_target_path`, `phase`, `bubbles`, `cancelable`, `default_prevented`, and `time_origin`.

## Mouse Events

| Event | Bubbles | Notes |
|---|---|---|
| `mousedown` | Yes | Includes button, position, modifier info |
| `mouseup` | Yes | Synthesises click via common ancestor |
| `click` | Yes | Synthesised from mousedown + mouseup |
| `dblclick` | Yes | Synthesised from two rapid clicks |
| `auxclick` | Yes | Middle/other button click |
| `mousemove` | No (per-element) | Dispatched to hovered element |
| `mouseenter` | No | Synthesised on hover path change |
| `mouseleave` | No | Synthesised on hover path change |
| `contextmenu` | Yes | Right-click |
| `dragstart`, `dragend`, `drag` | Yes | `draggable` attribute on node |
| `dragenter`, `dragleave`, `dragover`, `drop` | Yes | |

## Keyboard Events

| Event | Bubbles | Notes |
|---|---|---|
| `keydown` | Yes | Dispatched along focused path; Tab/Shift+Tab handled internally |
| `keyup` | Yes | |

Accompanying data: `key` (DOM key string), `code` (physical key), `repeat`, modifier state.

## Focus Events

| Event | Bubbles | Notes |
|---|---|---|
| `focus` | No | Dispatched when element gains focus |
| `blur` | No | Dispatched when element loses focus |
| `focusin` | Yes | Bubbling version of focus |
| `focusout` | Yes | Bubbling version of blur |

## Input Events

| Event | Bubbles | Notes |
|---|---|---|
| `input` | Yes | Content change in editable controls |
| `beforeinput` | Yes | Before content is modified |
| `change` | Yes | Value committed (checkbox toggle, range change, etc.) |

## Form Events

| Event | Bubbles | Notes |
|---|---|---|
| `submit` | Yes | Form submission (Enter in input, submit button) |
| `reset` | Yes | Reset button clicked |
| `select` | Yes | Text selection change |
| `selectionchange` | Yes | Text selection range changed |

## Scroll & Wheel Events

| Event | Bubbles | Notes |
|---|---|---|
| `scroll` | No (target only) | Scrollable element scroll |
| `wheel` | Yes | Mouse wheel with delta mode |
| `scrollend` | No | Scroll animation ended |

## Clipboard Events

| Event | Bubbles | Notes |
|---|---|---|
| `copy` | Yes | Ctrl+C; `cut_selection` + clipboard |
| `cut` | Yes | Ctrl+X; removes text + clipboard |
| `paste` | Yes | Ctrl+V; clipboard → focused input |

## Media Events

| Event | Bubbles | Notes |
|---|---|---|
| `load` | No | Image loaded |
| `error` | No | Image/load error |
| `abort` | No | Load aborted |
| `canplay` | No | Media can play (not triggered) |

## Touch Events (parsed, minimal dispatch)

| Event | Notes |
|---|---|
| `touchstart`, `touchmove`, `touchend`, `touchcancel` | Parser recognizes but no multi-touch dispatch |

## Callback Types

```rust
pub type MouseCallback = Arc<dyn Fn(&MouseEvent) + Send + Sync>;
pub type EventCallback = Arc<dyn Fn(&HtmlEvent) + Send + Sync>;
```
