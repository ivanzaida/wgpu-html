use lui_cascade::media::{MediaContext, evaluate_media};
use lui_parse::parse_media_query_list;

fn ctx(w: f32, h: f32) -> MediaContext {
  MediaContext {
    viewport_width: w,
    viewport_height: h,
    ..Default::default()
  }
}

#[test]
fn min_width_matches() {
  let mql = parse_media_query_list("(min-width: 768px)").unwrap();
  assert!(evaluate_media(&mql, &ctx(1024.0, 768.0)));
}

#[test]
fn min_width_rejects() {
  let mql = parse_media_query_list("(min-width: 768px)").unwrap();
  assert!(!evaluate_media(&mql, &ctx(500.0, 768.0)));
}

#[test]
fn max_width_matches() {
  let mql = parse_media_query_list("(max-width: 600px)").unwrap();
  assert!(evaluate_media(&mql, &ctx(400.0, 300.0)));
}

#[test]
fn max_width_rejects() {
  let mql = parse_media_query_list("(max-width: 600px)").unwrap();
  assert!(!evaluate_media(&mql, &ctx(800.0, 600.0)));
}

#[test]
fn min_and_max_width() {
  let mql = parse_media_query_list("(min-width: 600px) and (max-width: 1200px)").unwrap();
  assert!(evaluate_media(&mql, &ctx(800.0, 600.0)));
  assert!(!evaluate_media(&mql, &ctx(400.0, 300.0)));
  assert!(!evaluate_media(&mql, &ctx(1400.0, 900.0)));
}

#[test]
fn screen_type_matches() {
  let mql = parse_media_query_list("screen").unwrap();
  assert!(evaluate_media(
    &mql,
    &MediaContext {
      is_screen: true,
      ..Default::default()
    }
  ));
  assert!(!evaluate_media(
    &mql,
    &MediaContext {
      is_screen: false,
      ..Default::default()
    }
  ));
}

#[test]
fn print_type_matches() {
  let mql = parse_media_query_list("print").unwrap();
  assert!(evaluate_media(
    &mql,
    &MediaContext {
      is_screen: false,
      ..Default::default()
    }
  ));
  assert!(!evaluate_media(
    &mql,
    &MediaContext {
      is_screen: true,
      ..Default::default()
    }
  ));
}

#[test]
fn screen_and_min_width() {
  let mql = parse_media_query_list("screen and (min-width: 768px)").unwrap();
  assert!(evaluate_media(&mql, &ctx(1024.0, 768.0)));
  assert!(!evaluate_media(&mql, &ctx(500.0, 300.0)));
}

#[test]
fn not_modifier_inverts() {
  let mql = parse_media_query_list("not print").unwrap();
  assert!(evaluate_media(
    &mql,
    &MediaContext {
      is_screen: true,
      ..Default::default()
    }
  ));
  assert!(!evaluate_media(
    &mql,
    &MediaContext {
      is_screen: false,
      ..Default::default()
    }
  ));
}

#[test]
fn comma_separated_is_or() {
  let mql = parse_media_query_list("(max-width: 600px), (min-width: 1200px)").unwrap();
  assert!(evaluate_media(&mql, &ctx(400.0, 300.0)));
  assert!(evaluate_media(&mql, &ctx(1400.0, 900.0)));
  assert!(!evaluate_media(&mql, &ctx(800.0, 600.0)));
}

#[test]
fn orientation_portrait() {
  let mql = parse_media_query_list("(orientation: portrait)").unwrap();
  assert!(evaluate_media(&mql, &ctx(400.0, 800.0)));
  assert!(!evaluate_media(&mql, &ctx(800.0, 400.0)));
}

#[test]
fn orientation_landscape() {
  let mql = parse_media_query_list("(orientation: landscape)").unwrap();
  assert!(evaluate_media(&mql, &ctx(800.0, 400.0)));
  assert!(!evaluate_media(&mql, &ctx(400.0, 800.0)));
}

#[test]
fn color_boolean() {
  let mql = parse_media_query_list("(color)").unwrap();
  assert!(evaluate_media(&mql, &Default::default()));
}

#[test]
fn hover_feature() {
  let mql = parse_media_query_list("(hover: hover)").unwrap();
  assert!(evaluate_media(&mql, &Default::default()));
}

#[test]
fn min_height() {
  let mql = parse_media_query_list("(min-height: 500px)").unwrap();
  assert!(evaluate_media(&mql, &ctx(800.0, 600.0)));
  assert!(!evaluate_media(&mql, &ctx(800.0, 400.0)));
}
