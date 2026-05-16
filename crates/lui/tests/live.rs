#[path = "live/signal/get_set.rs"]
mod signal_get_set;

#[path = "live/signal/from.rs"]
mod signal_from;

#[path = "live/effect/auto_tracks_deps.rs"]
mod effect_auto_tracks_deps;

#[path = "live/memo/derived_value.rs"]
mod memo_derived_value;

#[path = "live/batch/coalesces_updates.rs"]
mod batch_coalesces_updates;

#[path = "live/lifecycle/mounted_fires_once.rs"]
mod lifecycle_mounted_fires_once;

#[path = "live/lifecycle/unmounted_fires_on_drop.rs"]
mod lifecycle_unmounted_fires_on_drop;

#[path = "live/component/child_gets_own_ctx.rs"]
mod component_child_gets_own_ctx;

#[path = "live/component/keyed_survives_reorder.rs"]
mod component_keyed_survives_reorder;

#[path = "live/context/provide_use.rs"]
mod context_provide_use;

#[path = "live/runtime/render_and_process.rs"]
mod runtime_render_and_process;

#[path = "live/styles/scoped_dedup.rs"]
mod styles_scoped_dedup;

#[path = "live/batch/render_coalescing.rs"]
mod batch_render_coalescing;

#[path = "live/styles/cleanup_on_unmount.rs"]
mod styles_cleanup_on_unmount;

#[path = "live/error_boundary/catches_panic.rs"]
mod error_boundary_catches_panic;

#[path = "live/store/basic.rs"]
mod store_basic;
