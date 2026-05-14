use crate::ArcStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CssColor {
  Rgb(u8, u8, u8),
  Rgba(u8, u8, u8, u8),
  Hsl(u16, u8, u8),
  Hsla(u16, u8, u8, u8),
  Hwb(u16, u8, u8),
  Hwba(u16, u8, u8, u8),
  Hex(ArcStr),
  Named(ArcStr),
}
