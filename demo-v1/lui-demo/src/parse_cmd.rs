use std::collections::HashMap;

#[derive(Debug)]
pub enum CmdVar {
  String(String),
  Number(i32),
  #[allow(dead_code)]
  Bool(bool),
}

pub struct CmdLine {
  pub flags: HashMap<String, CmdVar>,
  pub positional: Vec<String>,
}

impl CmdLine {
  /// Extract a `--key=value` flag as an `Option<String>`.
  pub fn flag_str(&self, key: &str) -> Option<String> {
    match self.flags.get(key)? {
      CmdVar::String(s) => Some(s.clone()),
      CmdVar::Bool(_) => None,
      CmdVar::Number(n) => Some(n.to_string()),
    }
  }
}

/// Parse `--flag=value`, `--flag`, and positional arguments.
///
/// - `--key=value` → `flags["--key"] = String(value)` (or `Number`/`Bool` if parseable)
/// - `--key` (no `=`) → `flags["--key"] = Bool(true)`
/// - Anything else → added to `positional`
pub fn parse_command_line() -> CmdLine {
  let mut flags = HashMap::new();
  let mut positional = Vec::new();

  for arg in std::env::args().skip(1) {
    if let Some((key, value)) = arg.split_once('=') {
      flags.insert(key.to_string(), parse_cmd_value(value));
    } else if arg.starts_with('-') {
      flags.insert(arg, CmdVar::Bool(true));
    } else {
      positional.push(arg);
    }
  }

  CmdLine { flags, positional }
}

fn parse_cmd_value(value: &str) -> CmdVar {
  if let Ok(num) = value.parse::<i32>() {
    CmdVar::Number(num)
  } else if let Ok(b) = value.parse::<bool>() {
    CmdVar::Bool(b)
  } else {
    CmdVar::String(value.to_string())
  }
}
