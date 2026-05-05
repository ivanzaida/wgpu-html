use std::{io::Read, path::PathBuf, sync::OnceLock, time::Duration};

use crate::{
  fetcher::{FetchResponse, Fetcher},
  FetchConfig,
};

const MAX_REDIRECTS: usize = 5;
const MAX_FETCH_ATTEMPTS: u8 = 3;
const RETRY_INITIAL_BACKOFF: Duration = Duration::from_millis(200);
const REMOTE_BODY_CAP: u64 = 32 * 1024 * 1024;

pub struct BlockingFetcher {
  asset_root: Option<PathBuf>,
}

impl BlockingFetcher {
  pub fn new() -> Self {
    Self { asset_root: None }
  }

  pub fn with_asset_root(mut self, path: impl Into<PathBuf>) -> Self {
    self.asset_root = Some(path.into());
    self
  }

  pub fn set_asset_root(&mut self, path: impl Into<PathBuf>) {
    self.asset_root = Some(path.into());
  }

  fn resolve_path(&self, url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("data:") {
      return url.to_owned();
    }
    match &self.asset_root {
      Some(root) => root.join(url).to_string_lossy().into_owned(),
      None => url.to_owned(),
    }
  }
}

impl Default for BlockingFetcher {
  fn default() -> Self {
    Self::new()
  }
}

impl Fetcher for BlockingFetcher {
  fn fetch(&self, url: &str, _config: &FetchConfig) -> Option<FetchResponse> {
    let resolved = self.resolve_path(url);
    fetch_with_retry(&resolved)
  }
}

fn fetch_with_retry(src: &str) -> Option<FetchResponse> {
  let mut backoff = RETRY_INITIAL_BACKOFF;
  for attempt in 0..MAX_FETCH_ATTEMPTS {
    if let Some(resp) = fetch_bytes(src) {
      return Some(resp);
    }
    if attempt + 1 < MAX_FETCH_ATTEMPTS {
      std::thread::sleep(backoff);
      backoff = backoff.saturating_mul(2);
    }
  }
  None
}

fn fetch_bytes(src: &str) -> Option<FetchResponse> {
  if src.starts_with("http://") || src.starts_with("https://") {
    return fetch_remote(src);
  }
  if src.starts_with("data:") {
    return fetch_data_uri(src).map(|bytes| FetchResponse { bytes, max_age: None });
  }
  std::fs::read(src)
    .ok()
    .map(|bytes| FetchResponse { bytes, max_age: None })
}

fn http_agent() -> &'static ureq::Agent {
  static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
  AGENT.get_or_init(|| ureq::AgentBuilder::new().redirects(0).build())
}

fn fetch_remote(src: &str) -> Option<FetchResponse> {
  let agent = http_agent();
  let mut current = src.to_string();
  for _ in 0..=MAX_REDIRECTS {
    let resp = agent.get(&current).call().ok()?;
    let status = resp.status();
    if (300..400).contains(&status) && status != 304 {
      let location = resp.header("Location")?;
      current = resolve_redirect_target(&current, location)?;
      continue;
    }
    if !(200..300).contains(&status) {
      return None;
    }
    let max_age = parse_cache_control_max_age(resp.header("Cache-Control"))
      .or_else(|| parse_expires_relative(resp.header("Date"), resp.header("Expires")));
    let mut buf = Vec::new();
    resp.into_reader().take(REMOTE_BODY_CAP).read_to_end(&mut buf).ok()?;
    return Some(FetchResponse { bytes: buf, max_age });
  }
  None
}

fn resolve_redirect_target(current: &str, location: &str) -> Option<String> {
  if location.starts_with("http://") || location.starts_with("https://") {
    return Some(location.to_string());
  }
  let scheme_end = current.find("://")?;
  let after_scheme = &current[scheme_end + 3..];
  let path_start = after_scheme.find('/').unwrap_or(after_scheme.len());
  let origin = &current[..scheme_end + 3 + path_start];
  if let Some(stripped) = location.strip_prefix('/') {
    Some(format!("{origin}/{stripped}"))
  } else {
    let last_slash = current.rfind('/').unwrap_or(scheme_end + 2);
    Some(format!("{}/{location}", &current[..last_slash]))
  }
}

fn parse_cache_control_max_age(header: Option<&str>) -> Option<Duration> {
  let header = header?;
  let mut zero = false;
  let mut max_age: Option<Duration> = None;
  for token in header.split(',') {
    let t = token.trim().to_ascii_lowercase();
    if t == "no-store" || t == "no-cache" {
      zero = true;
    } else if let Some(rest) = t.strip_prefix("max-age=") {
      if let Ok(secs) = rest.trim().parse::<u64>() {
        max_age = Some(Duration::from_secs(secs));
      }
    } else if let Some(rest) = t.strip_prefix("s-maxage=") {
      if let Ok(secs) = rest.trim().parse::<u64>() {
        max_age = Some(Duration::from_secs(secs));
      }
    }
  }
  if zero {
    Some(Duration::ZERO)
  } else {
    max_age
  }
}

fn parse_expires_relative(date: Option<&str>, expires: Option<&str>) -> Option<Duration> {
  let date = parse_rfc1123(date?)?;
  let expires = parse_rfc1123(expires?)?;
  expires.checked_sub(date).map(Duration::from_secs)
}

fn parse_rfc1123(s: &str) -> Option<u64> {
  let s = s.trim();
  let parts: Vec<&str> = s.split_whitespace().collect();
  if parts.len() < 5 {
    return None;
  }
  let (day, mon, year, hms) = if parts[0].ends_with(',') && parts.len() >= 6 {
    (parts[1], parts[2], parts[3], parts[4])
  } else {
    return None;
  };
  let day: u32 = day.parse().ok()?;
  let year: i64 = year.parse().ok()?;
  let mon = match mon.to_ascii_lowercase().as_str() {
    "jan" => 1u32,
    "feb" => 2,
    "mar" => 3,
    "apr" => 4,
    "may" => 5,
    "jun" => 6,
    "jul" => 7,
    "aug" => 8,
    "sep" => 9,
    "oct" => 10,
    "nov" => 11,
    "dec" => 12,
    _ => return None,
  };
  let mut hms_parts = hms.split(':');
  let hh: u64 = hms_parts.next()?.parse().ok()?;
  let mm: u64 = hms_parts.next()?.parse().ok()?;
  let ss: u64 = hms_parts.next()?.parse().ok()?;
  let y = if mon <= 2 { year - 1 } else { year };
  let era = y.div_euclid(400);
  let yoe = y - era * 400;
  let m = if mon > 2 { mon - 3 } else { mon + 9 };
  let doy = ((153 * m as i64 + 2) / 5) + day as i64 - 1;
  let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
  let days_since_epoch = era * 146097 + doe - 719468;
  if days_since_epoch < 0 {
    return None;
  }
  Some((days_since_epoch as u64) * 86400 + hh * 3600 + mm * 60 + ss)
}

fn fetch_data_uri(src: &str) -> Option<Vec<u8>> {
  let rest = src.strip_prefix("data:")?;
  let comma = rest.find(',')?;
  let meta = &rest[..comma];
  let payload = &rest[comma + 1..];
  let is_base64 = meta.split(';').any(|p| p.eq_ignore_ascii_case("base64"));
  if is_base64 {
    decode_base64(payload)
  } else {
    decode_percent_encoded(payload)
  }
}

fn decode_base64(input: &str) -> Option<Vec<u8>> {
  fn val(c: u8) -> Option<u32> {
    match c {
      b'A'..=b'Z' => Some((c - b'A') as u32),
      b'a'..=b'z' => Some((c - b'a' + 26) as u32),
      b'0'..=b'9' => Some((c - b'0' + 52) as u32),
      b'+' | b'-' => Some(62),
      b'/' | b'_' => Some(63),
      _ => None,
    }
  }
  let mut quartet: u32 = 0;
  let mut filled = 0u32;
  let mut out = Vec::with_capacity(input.len() * 3 / 4);
  for &b in input.as_bytes() {
    if matches!(b, b' ' | b'\n' | b'\r' | b'\t' | b'=') {
      continue;
    }
    let v = val(b)?;
    quartet = (quartet << 6) | v;
    filled += 6;
    if filled >= 8 {
      filled -= 8;
      out.push(((quartet >> filled) & 0xff) as u8);
    }
  }
  Some(out)
}

fn decode_percent_encoded(input: &str) -> Option<Vec<u8>> {
  let bytes = input.as_bytes();
  let mut out = Vec::with_capacity(bytes.len());
  let mut i = 0;
  while i < bytes.len() {
    let b = bytes[i];
    if b == b'%' && i + 2 < bytes.len() {
      let hi = (bytes[i + 1] as char).to_digit(16)?;
      let lo = (bytes[i + 2] as char).to_digit(16)?;
      out.push(((hi << 4) | lo) as u8);
      i += 3;
    } else {
      out.push(b);
      i += 1;
    }
  }
  Some(out)
}
