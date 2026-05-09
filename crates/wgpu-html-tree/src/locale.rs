use std::sync::Arc;

/// Format a date using a pattern string.
///
/// Recognized tokens: `dd`, `mm`, `yyyy`. Everything else is literal.
///
/// Examples: `"dd/mm/yyyy"`, `"mm.dd.yyyy"`, `"yyyy-mm-dd"`.
pub fn format_date_pattern(pattern: &str, y: i32, m: u8, d: u8) -> String {
  let mut out = String::with_capacity(pattern.len() + 4);
  let bytes = pattern.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    if i + 4 <= bytes.len() && &bytes[i..i + 4] == b"yyyy" {
      out.push_str(&format!("{y:04}"));
      i += 4;
    } else if i + 2 <= bytes.len() && &bytes[i..i + 2] == b"dd" {
      out.push_str(&format!("{d:02}"));
      i += 2;
    } else if i + 2 <= bytes.len() && &bytes[i..i + 2] == b"mm" {
      out.push_str(&format!("{m:02}"));
      i += 2;
    } else {
      out.push(bytes[i] as char);
      i += 1;
    }
  }
  out
}

pub trait Locale: Send + Sync + std::fmt::Debug {
  fn key(&self) -> &str { "en-US" }
  fn date_pattern(&self) -> &str { "mm/dd/yyyy" }
  fn format_date(&self, y: i32, m: u8, d: u8) -> String {
    format_date_pattern(self.date_pattern(), y, m, d)
  }
  fn format_datetime(&self, y: i32, m: u8, d: u8, hour: u8, min: u8) -> String {
    let date = self.format_date(y, m, d);
    format!("{date} {hour:02}:{min:02}")
  }
  fn date_placeholder(&self) -> String {
    self.date_pattern().to_string()
  }
  fn datetime_placeholder(&self) -> String {
    format!("{} hh:mm", self.date_pattern())
  }
  fn month_name(&self, month: u8) -> &str;
  fn month_short(&self, month: u8) -> &str;
  fn weekday_name(&self, weekday: u8) -> &str;
  fn weekday_short(&self, weekday: u8) -> &str;
  fn first_day_of_week(&self) -> u8 { 0 }
  fn file_browse_label(&self) -> &str { "Browse\u{2026}" }
  fn file_no_file_label(&self) -> &str { "No file chosen" }
  fn color_picker_rgba_label(&self) -> &str { "RGBA" }
  fn color_picker_hex_label(&self) -> &str { "Hex" }
  fn date_picker_reset_label(&self) -> &str { "Reset" }
}

#[derive(Debug)]
pub struct DefaultLocale;

impl DefaultLocale {
  pub fn new() -> Arc<dyn Locale> {
    Arc::new(Self)
  }
}

const MONTHS: [&str; 12] = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
];

const MONTHS_SHORT: [&str; 12] = [
  "Jan", "Feb", "Mar", "Apr", "May", "Jun",
  "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const WEEKDAYS: [&str; 7] = [
  "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
];

const WEEKDAYS_SHORT: [&str; 7] = [
  "Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun",
];

impl Locale for DefaultLocale {
  fn month_name(&self, month: u8) -> &str {
    MONTHS.get(month.wrapping_sub(1) as usize).unwrap_or(&"")
  }

  fn month_short(&self, month: u8) -> &str {
    MONTHS_SHORT.get(month.wrapping_sub(1) as usize).unwrap_or(&"")
  }

  fn weekday_name(&self, weekday: u8) -> &str {
    WEEKDAYS.get(weekday as usize).unwrap_or(&"")
  }

  fn weekday_short(&self, weekday: u8) -> &str {
    WEEKDAYS_SHORT.get(weekday as usize).unwrap_or(&"")
  }
}
