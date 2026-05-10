//! Back-compat alias for the old shared-state name.

pub type Store<T> = crate::core::observable::Observable<T>;
