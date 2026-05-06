use std::{fmt, str::FromStr};

#[derive(Debug, Clone)]
pub enum HtmlDirection {
  Ltr,
  Rtl,
  Auto,
}

#[derive(Debug, Clone)]
pub enum AriaRole {
  Button,
  Checkbox,
  Dialog,
  Link,
  Listbox,
  Menu,
  Menuitem,
  Navigation,
  Option,
  Progressbar,
  Radio,
  Search,
  Slider,
  Status,
  Tab,
  Tablist,
  Textbox,
  Tooltip,
  Tree,
  Treeitem,
}

#[derive(Debug, Clone)]
pub enum LinkAs {
  Audio,
  Document,
  Embed,
  Fetch,
  Font,
  Image,
  Object,
  Script,
  Style,
  Track,
  Video,
  Worker,
}

#[derive(Debug, Clone)]
pub enum CrossOrigin {
  Anonymous,
  UseCredentials,
}

#[derive(Debug, Clone)]
pub enum ReferrerPolicy {
  NoReferrer,
  NoReferrerWhenDowngrade,
  Origin,
  OriginWhenCrossOrigin,
  SameOrigin,
  StrictOrigin,
  StrictOriginWhenCrossOrigin,
  UnsafeUrl,
}

#[derive(Debug, Clone)]
pub enum LinkTarget {
  Blank,
  SelfTarget,
  Parent,
  Top,
  Named(String),
}

#[derive(Debug, Clone)]
pub enum OlType {
  Decimal,
  LowerAlpha,
  UpperAlpha,
  LowerRoman,
  UpperRoman,
}

#[derive(Debug, Clone)]
pub enum Loading {
  Eager,
  Lazy,
}

#[derive(Debug, Clone)]
pub enum ImageDecoding {
  Sync,
  Async,
  Auto,
}

#[derive(Debug, Clone)]
pub enum Preload {
  None,
  Metadata,
  Auto,
}

#[derive(Debug, Clone)]
pub enum TrackKind {
  Subtitles,
  Captions,
  Descriptions,
  Chapters,
  Metadata,
}

#[derive(Debug, Clone)]
pub enum SvgLength {
  Px(f32),
  Percent(f32),
  Em(f32),
  Rem(f32),
  Auto,
  Raw(String),
}

#[derive(Debug, Clone)]
pub enum TableHeaderScope {
  Row,
  Col,
  RowGroup,
  ColGroup,
  Auto,
}

#[derive(Debug, Clone)]
pub enum FormMethod {
  Get,
  Post,
  Dialog,
}

#[derive(Debug, Clone)]
pub enum FormEncoding {
  UrlEncoded,
  MultipartFormData,
  TextPlain,
}

#[derive(Debug, Clone)]
pub enum AutoComplete {
  On,
  Off,
}

#[derive(Debug, Clone)]
pub enum InputType {
  Button,
  Checkbox,
  Color,
  Date,
  DatetimeLocal,
  Email,
  File,
  Hidden,
  Image,
  Month,
  Number,
  Password,
  Radio,
  Range,
  Reset,
  Search,
  Submit,
  Tel,
  Text,
  Time,
  Url,
  Week,
}

#[derive(Debug, Clone)]
pub enum CaptureMode {
  User,
  Environment,
}

#[derive(Debug, Clone)]
pub enum TextareaWrap {
  Hard,
  Soft,
  Off,
}

#[derive(Debug, Clone)]
pub enum ButtonType {
  Button,
  Submit,
  Reset,
}

#[derive(Debug, Clone)]
pub enum ShadowRootMode {
  Open,
  Closed,
}

macro_rules! html_keyword_enum {
  ($ty:ty { $($variant:path => $html:literal),+ $(,)? }) => {
    impl $ty {
      pub fn as_html_str(&self) -> &'static str {
        match self {
          $($variant => $html),+
        }
      }
    }

    impl fmt::Display for $ty {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_html_str())
      }
    }

    impl FromStr for $ty {
      type Err = ();

      fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        $(
          if value.eq_ignore_ascii_case($html) {
            return Ok($variant);
          }
        )+
        Err(())
      }
    }
  };
}

html_keyword_enum!(HtmlDirection {
  HtmlDirection::Ltr => "ltr",
  HtmlDirection::Rtl => "rtl",
  HtmlDirection::Auto => "auto",
});

html_keyword_enum!(AriaRole {
  AriaRole::Button => "button",
  AriaRole::Checkbox => "checkbox",
  AriaRole::Dialog => "dialog",
  AriaRole::Link => "link",
  AriaRole::Listbox => "listbox",
  AriaRole::Menu => "menu",
  AriaRole::Menuitem => "menuitem",
  AriaRole::Navigation => "navigation",
  AriaRole::Option => "option",
  AriaRole::Progressbar => "progressbar",
  AriaRole::Radio => "radio",
  AriaRole::Search => "search",
  AriaRole::Slider => "slider",
  AriaRole::Status => "status",
  AriaRole::Tab => "tab",
  AriaRole::Tablist => "tablist",
  AriaRole::Textbox => "textbox",
  AriaRole::Tooltip => "tooltip",
  AriaRole::Tree => "tree",
  AriaRole::Treeitem => "treeitem",
});

html_keyword_enum!(LinkAs {
  LinkAs::Audio => "audio",
  LinkAs::Document => "document",
  LinkAs::Embed => "embed",
  LinkAs::Fetch => "fetch",
  LinkAs::Font => "font",
  LinkAs::Image => "image",
  LinkAs::Object => "object",
  LinkAs::Script => "script",
  LinkAs::Style => "style",
  LinkAs::Track => "track",
  LinkAs::Video => "video",
  LinkAs::Worker => "worker",
});

impl CrossOrigin {
  pub fn as_html_str(&self) -> &'static str {
    match self {
      CrossOrigin::Anonymous => "anonymous",
      CrossOrigin::UseCredentials => "use-credentials",
    }
  }
}

impl fmt::Display for CrossOrigin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_html_str())
  }
}

impl FromStr for CrossOrigin {
  type Err = ();

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    match value.trim().to_ascii_lowercase().as_str() {
      "" | "anonymous" => Ok(CrossOrigin::Anonymous),
      "use-credentials" => Ok(CrossOrigin::UseCredentials),
      _ => Err(()),
    }
  }
}

html_keyword_enum!(ReferrerPolicy {
  ReferrerPolicy::NoReferrer => "no-referrer",
  ReferrerPolicy::NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
  ReferrerPolicy::Origin => "origin",
  ReferrerPolicy::OriginWhenCrossOrigin => "origin-when-cross-origin",
  ReferrerPolicy::SameOrigin => "same-origin",
  ReferrerPolicy::StrictOrigin => "strict-origin",
  ReferrerPolicy::StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin",
  ReferrerPolicy::UnsafeUrl => "unsafe-url",
});

impl fmt::Display for LinkTarget {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      LinkTarget::Blank => f.write_str("_blank"),
      LinkTarget::SelfTarget => f.write_str("_self"),
      LinkTarget::Parent => f.write_str("_parent"),
      LinkTarget::Top => f.write_str("_top"),
      LinkTarget::Named(value) => f.write_str(value),
    }
  }
}

impl FromStr for LinkTarget {
  type Err = ();

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    let trimmed = value.trim();
    match trimmed.to_ascii_lowercase().as_str() {
      "_blank" => Ok(LinkTarget::Blank),
      "_self" => Ok(LinkTarget::SelfTarget),
      "_parent" => Ok(LinkTarget::Parent),
      "_top" => Ok(LinkTarget::Top),
      "" => Err(()),
      _ => Ok(LinkTarget::Named(trimmed.to_string())),
    }
  }
}

impl fmt::Display for OlType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      OlType::Decimal => f.write_str("1"),
      OlType::LowerAlpha => f.write_str("a"),
      OlType::UpperAlpha => f.write_str("A"),
      OlType::LowerRoman => f.write_str("i"),
      OlType::UpperRoman => f.write_str("I"),
    }
  }
}

impl FromStr for OlType {
  type Err = ();

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    match value.trim() {
      "1" => Ok(OlType::Decimal),
      "a" => Ok(OlType::LowerAlpha),
      "A" => Ok(OlType::UpperAlpha),
      "i" => Ok(OlType::LowerRoman),
      "I" => Ok(OlType::UpperRoman),
      _ => Err(()),
    }
  }
}

html_keyword_enum!(Loading {
  Loading::Eager => "eager",
  Loading::Lazy => "lazy",
});

html_keyword_enum!(ImageDecoding {
  ImageDecoding::Sync => "sync",
  ImageDecoding::Async => "async",
  ImageDecoding::Auto => "auto",
});

impl Preload {
  pub fn as_html_str(&self) -> &'static str {
    match self {
      Preload::None => "none",
      Preload::Metadata => "metadata",
      Preload::Auto => "auto",
    }
  }
}

impl fmt::Display for Preload {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_html_str())
  }
}

impl FromStr for Preload {
  type Err = ();

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    match value.trim().to_ascii_lowercase().as_str() {
      "none" => Ok(Preload::None),
      "metadata" => Ok(Preload::Metadata),
      "" | "auto" => Ok(Preload::Auto),
      _ => Err(()),
    }
  }
}

html_keyword_enum!(TrackKind {
  TrackKind::Subtitles => "subtitles",
  TrackKind::Captions => "captions",
  TrackKind::Descriptions => "descriptions",
  TrackKind::Chapters => "chapters",
  TrackKind::Metadata => "metadata",
});

impl fmt::Display for SvgLength {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      SvgLength::Px(value) => write!(f, "{value}px"),
      SvgLength::Percent(value) => write!(f, "{value}%"),
      SvgLength::Em(value) => write!(f, "{value}em"),
      SvgLength::Rem(value) => write!(f, "{value}rem"),
      SvgLength::Auto => f.write_str("auto"),
      SvgLength::Raw(value) => f.write_str(value),
    }
  }
}

impl FromStr for SvgLength {
  type Err = ();

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("auto") {
      return Ok(SvgLength::Auto);
    }
    if let Some(value) = trimmed.strip_suffix("px") {
      return value.trim().parse::<f32>().map(SvgLength::Px).map_err(|_| ());
    }
    if let Some(value) = trimmed.strip_suffix('%') {
      return value.trim().parse::<f32>().map(SvgLength::Percent).map_err(|_| ());
    }
    if let Some(value) = trimmed.strip_suffix("rem") {
      return value.trim().parse::<f32>().map(SvgLength::Rem).map_err(|_| ());
    }
    if let Some(value) = trimmed.strip_suffix("em") {
      return value.trim().parse::<f32>().map(SvgLength::Em).map_err(|_| ());
    }
    if let Ok(value) = trimmed.parse::<f32>() {
      return Ok(SvgLength::Px(value));
    }
    Ok(SvgLength::Raw(trimmed.to_string()))
  }
}

html_keyword_enum!(TableHeaderScope {
  TableHeaderScope::Row => "row",
  TableHeaderScope::Col => "col",
  TableHeaderScope::RowGroup => "rowgroup",
  TableHeaderScope::ColGroup => "colgroup",
  TableHeaderScope::Auto => "auto",
});

html_keyword_enum!(FormMethod {
  FormMethod::Get => "get",
  FormMethod::Post => "post",
  FormMethod::Dialog => "dialog",
});

html_keyword_enum!(FormEncoding {
  FormEncoding::UrlEncoded => "application/x-www-form-urlencoded",
  FormEncoding::MultipartFormData => "multipart/form-data",
  FormEncoding::TextPlain => "text/plain",
});

html_keyword_enum!(AutoComplete {
  AutoComplete::On => "on",
  AutoComplete::Off => "off",
});

html_keyword_enum!(InputType {
  InputType::Button => "button",
  InputType::Checkbox => "checkbox",
  InputType::Color => "color",
  InputType::Date => "date",
  InputType::DatetimeLocal => "datetime-local",
  InputType::Email => "email",
  InputType::File => "file",
  InputType::Hidden => "hidden",
  InputType::Image => "image",
  InputType::Month => "month",
  InputType::Number => "number",
  InputType::Password => "password",
  InputType::Radio => "radio",
  InputType::Range => "range",
  InputType::Reset => "reset",
  InputType::Search => "search",
  InputType::Submit => "submit",
  InputType::Tel => "tel",
  InputType::Text => "text",
  InputType::Time => "time",
  InputType::Url => "url",
  InputType::Week => "week",
});

html_keyword_enum!(CaptureMode {
  CaptureMode::User => "user",
  CaptureMode::Environment => "environment",
});

html_keyword_enum!(TextareaWrap {
  TextareaWrap::Hard => "hard",
  TextareaWrap::Soft => "soft",
  TextareaWrap::Off => "off",
});

html_keyword_enum!(ButtonType {
  ButtonType::Button => "button",
  ButtonType::Submit => "submit",
  ButtonType::Reset => "reset",
});

html_keyword_enum!(ShadowRootMode {
  ShadowRootMode::Open => "open",
  ShadowRootMode::Closed => "closed",
});

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn keyword_enums_round_trip_attribute_values() {
    assert!(matches!(
      "datetime-local".parse::<InputType>().unwrap(),
      InputType::DatetimeLocal
    ));
    assert_eq!(InputType::DatetimeLocal.to_string(), "datetime-local");

    assert!(matches!(
      "use-credentials".parse::<CrossOrigin>().unwrap(),
      CrossOrigin::UseCredentials
    ));
    assert_eq!(CrossOrigin::UseCredentials.to_string(), "use-credentials");

    assert!(matches!(
      "strict-origin-when-cross-origin".parse::<ReferrerPolicy>().unwrap(),
      ReferrerPolicy::StrictOriginWhenCrossOrigin
    ));
    assert_eq!(
      ReferrerPolicy::StrictOriginWhenCrossOrigin.to_string(),
      "strict-origin-when-cross-origin"
    );
  }

  #[test]
  fn link_target_supports_reserved_and_named_targets() {
    assert!(matches!("_blank".parse::<LinkTarget>().unwrap(), LinkTarget::Blank));
    assert_eq!(LinkTarget::Blank.to_string(), "_blank");

    let named = "preview-frame".parse::<LinkTarget>().unwrap();
    assert!(matches!(named, LinkTarget::Named(ref value) if value == "preview-frame"));
    assert_eq!(named.to_string(), "preview-frame");
    assert!("".parse::<LinkTarget>().is_err());
  }

  #[test]
  fn svg_length_round_trips_units_and_raw_values() {
    assert!(matches!("16px".parse::<SvgLength>().unwrap(), SvgLength::Px(16.0)));
    assert_eq!(SvgLength::Percent(50.0).to_string(), "50%");

    let raw = "calc(100% - 1rem)".parse::<SvgLength>().unwrap();
    assert!(matches!(raw, SvgLength::Raw(ref value) if value == "calc(100% - 1rem)"));
    assert_eq!(raw.to_string(), "calc(100% - 1rem)");
  }
}
