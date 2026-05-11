use std::{env, fs, process};

use lui_css_old::{CssParser, CssRule, Style};
use lui_css_old::css_parser::apply_css_property;

fn main() {
  let path = match env::args().nth(1) {
    Some(p) => p,
    None => {
      eprintln!("usage: verify_css <path-to-css-file>");
      process::exit(1);
    }
  };

  let css = match fs::read_to_string(&path) {
    Ok(s) => s,
    Err(e) => {
      eprintln!("error reading `{path}`: {e}");
      process::exit(1);
    }
  };

  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(&css);

  let mut style_count = 0;
  let mut media_count = 0;
  let mut import_count = 0;
  let mut keyframes_count = 0;
  let mut font_face_count = 0;
  let mut supports_count = 0;
  let mut unknown_count = 0;
  let mut total_declarations = 0;

  count_rules(&sheet.rules, &mut |rule| match rule {
    CssRule::Style(r) => {
      style_count += 1;
      total_declarations += r.declarations.len();
      let mut style = Style::default();
      for decl in &r.declarations.declarations {
        apply_css_property(&mut style, &decl.property, &decl.value);
      }
    }
    CssRule::Media(_) => media_count += 1,
    CssRule::Import(_) => import_count += 1,
    CssRule::Keyframes(_) => keyframes_count += 1,
    CssRule::FontFace(_) => font_face_count += 1,
    CssRule::Supports(_) => supports_count += 1,
    CssRule::Unknown(_) => {
      unknown_count += 1;
    }
  });

  println!();
  println!("=== {} ===", path);
  println!("  style rules:   {style_count}");
  println!("  declarations:  {total_declarations}");
  println!("  @media:        {media_count}");
  println!("  @import:       {import_count}");
  println!("  @keyframes:    {keyframes_count}");
  println!("  @font-face:    {font_face_count}");
  println!("  @supports:     {supports_count}");
  if unknown_count > 0 {
    println!("  unknown rules: {unknown_count}");
  }

  let total = style_count + media_count + import_count + keyframes_count + font_face_count + supports_count + unknown_count;
  if total == 0 {
    eprintln!("[warn] no rules found — file may be empty or invalid");
    process::exit(1);
  }
  if unknown_count > 0 {
    process::exit(1);
  }

  println!("  OK");
}

fn count_rules(rules: &[CssRule], visitor: &mut dyn FnMut(&CssRule)) {
  for rule in rules {
    visitor(rule);
    match rule {
      CssRule::Media(r) => count_rules(&r.rules, visitor),
      CssRule::Supports(r) => count_rules(&r.rules, visitor),
      _ => {}
    }
  }
}
