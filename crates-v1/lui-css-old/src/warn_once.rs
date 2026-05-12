use std::collections::HashSet;
use std::fmt;
use std::sync::{Mutex, OnceLock};

static WARNED_MESSAGES: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

pub(crate) fn warn_once_impl(args: fmt::Arguments<'_>) {
  let message = args.to_string();

  let warned = WARNED_MESSAGES.get_or_init(|| Mutex::new(HashSet::new()));

  let mut warned = warned.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

  if warned.insert(message.clone()) {
    println!("[lui-css] {message}");
  }
}

#[macro_export]
macro_rules! warn_once {
    ($($arg:tt)*) => {
        $crate::warn_once::warn_once_impl(format_args!($($arg)*))
    };
}
