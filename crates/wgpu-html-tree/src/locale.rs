use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateFormat {
  /// day-month-year (31/12/2025)
  DMY,
  /// month-day-year (12/31/2025)
  MDY,
  /// year-month-day (2025-12-31)
  YMD,
}

impl DateFormat {
  pub fn format(&self, y: i32, m: u8, d: u8) -> String {
    match self {
      Self::DMY => format!("{d:02}/{m:02}/{y:04}"),
      Self::MDY => format!("{m:02}/{d:02}/{y:04}"),
      Self::YMD => format!("{y:04}-{m:02}-{d:02}"),
    }
  }

  pub fn format_datetime(&self, y: i32, m: u8, d: u8, hour: u8, min: u8) -> String {
    let date = self.format(y, m, d);
    format!("{date} {hour:02}:{min:02}")
  }

  pub fn placeholder(&self) -> &'static str {
    match self {
      Self::DMY => "dd/mm/yyyy",
      Self::MDY => "mm/dd/yyyy",
      Self::YMD => "yyyy-mm-dd",
    }
  }

  pub fn placeholder_datetime(&self) -> &'static str {
    match self {
      Self::DMY => "dd/mm/yyyy hh:mm",
      Self::MDY => "mm/dd/yyyy hh:mm",
      Self::YMD => "yyyy-mm-dd hh:mm",
    }
  }
}

pub trait Locale: Send + Sync + std::fmt::Debug {
  fn key(&self) -> &str { "en-US" }
  fn date_format(&self) -> DateFormat { DateFormat::MDY }
  fn month_name(&self, month: u8) -> &str;
  fn month_short(&self, month: u8) -> &str;
  fn weekday_name(&self, weekday: u8) -> &str;
  fn weekday_short(&self, weekday: u8) -> &str;
  fn first_day_of_week(&self) -> u8 { 0 }
  fn file_browse_label(&self) -> &str { "Browse\u{2026}" }
  fn file_no_file_label(&self) -> &str { "No file chosen" }
  fn color_picker_rgba_label(&self) -> &str { "RGBA" }
  fn color_picker_hex_label(&self) -> &str { "Hex" }
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
