pub fn is_leap_year(y: i32) -> bool {
  (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

pub fn days_in_month(y: i32, m: u8) -> u8 {
  match m {
    1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
    4 | 6 | 9 | 11 => 30,
    2 => if is_leap_year(y) { 29 } else { 28 },
    _ => 30,
  }
}

/// Day of week for a given date (0=Monday .. 6=Sunday).
/// Uses Tomohiko Sakamoto's algorithm.
pub fn day_of_week(y: i32, m: u8, d: u8) -> u8 {
  let t = [0i32, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
  let y = if m < 3 { y - 1 } else { y };
  let dow = (y + y / 4 - y / 100 + y / 400 + t[(m - 1) as usize] + d as i32) % 7;
  // Result: 0=Sunday, 1=Monday, ..., 6=Saturday
  // Convert to 0=Monday, ..., 6=Sunday
  ((dow + 6) % 7) as u8
}

/// Parse "YYYY-MM-DD" into (year, month, day). Returns None on invalid input.
pub fn parse_date(s: &str) -> Option<(i32, u8, u8)> {
  let s = s.trim();
  let mut parts = s.splitn(3, '-');
  let y: i32 = parts.next()?.parse().ok()?;
  let m: u8 = parts.next()?.parse().ok()?;
  let d: u8 = parts.next()?.parse().ok()?;
  if m < 1 || m > 12 || d < 1 || d > days_in_month(y, m) {
    return None;
  }
  Some((y, m, d))
}

/// Parse "YYYY-MM-DDThh:mm" or "YYYY-MM-DDThh:mm:ss" into (year, month, day, hour, minute).
pub fn parse_datetime_local(s: &str) -> Option<(i32, u8, u8, u8, u8)> {
  let s = s.trim();
  let (date_part, time_part) = s.split_once('T')?;
  let (y, m, d) = parse_date(date_part)?;
  let mut time_parts = time_part.splitn(3, ':');
  let hour: u8 = time_parts.next()?.parse().ok()?;
  let minute: u8 = time_parts.next()?.parse().ok()?;
  if hour > 23 || minute > 59 { return None; }
  Some((y, m, d, hour, minute))
}

pub fn format_date(y: i32, m: u8, d: u8) -> String {
  format!("{y:04}-{m:02}-{d:02}")
}

pub fn format_datetime_local(y: i32, m: u8, d: u8, hour: u8, min: u8) -> String {
  format!("{y:04}-{m:02}-{d:02}T{hour:02}:{min:02}")
}

pub fn prev_month(y: i32, m: u8) -> (i32, u8) {
  if m <= 1 { (y - 1, 12) } else { (y, m - 1) }
}

pub fn next_month(y: i32, m: u8) -> (i32, u8) {
  if m >= 12 { (y + 1, 1) } else { (y, m + 1) }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn leap_years() {
    assert!(is_leap_year(2000));
    assert!(is_leap_year(2024));
    assert!(!is_leap_year(1900));
    assert!(!is_leap_year(2023));
  }

  #[test]
  fn days() {
    assert_eq!(days_in_month(2024, 2), 29);
    assert_eq!(days_in_month(2023, 2), 28);
    assert_eq!(days_in_month(2024, 1), 31);
    assert_eq!(days_in_month(2024, 4), 30);
  }

  #[test]
  fn dow() {
    // 2024-01-01 is Monday
    assert_eq!(day_of_week(2024, 1, 1), 0);
    // 2024-01-07 is Sunday
    assert_eq!(day_of_week(2024, 1, 7), 6);
  }

  #[test]
  fn parse() {
    assert_eq!(parse_date("2025-05-09"), Some((2025, 5, 9)));
    assert_eq!(parse_date("bad"), None);
    assert_eq!(parse_datetime_local("2025-05-09T14:30"), Some((2025, 5, 9, 14, 30)));
  }
}
