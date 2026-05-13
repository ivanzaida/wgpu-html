/// Typed CSS dimension units.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CssUnit {
    // Absolute length
    Px, Cm, Mm, Q, In, Pt, Pc,
    // Font-relative length
    Em, Rem, Ex, Ch,
    // Viewport-relative length
    Vw, Vh, Vmin, Vmax, Vi, Vb,
    // Angle
    Deg, Rad, Grad, Turn,
    // Time
    S, Ms,
    // Frequency
    Hz, Khz,
    // Resolution
    Dpi, Dpcm, Dppx,
    // Flex
    Fr,
    // Unknown / not-yet-supported
    Other(String),
}

impl CssUnit {
    pub fn from_str(s: &str) -> CssUnit {
        match s {
            "px" => CssUnit::Px, "cm" => CssUnit::Cm, "mm" => CssUnit::Mm,
            "Q" => CssUnit::Q, "in" => CssUnit::In, "pt" => CssUnit::Pt, "pc" => CssUnit::Pc,
            "em" => CssUnit::Em, "rem" => CssUnit::Rem, "ex" => CssUnit::Ex, "ch" => CssUnit::Ch,
            "vw" => CssUnit::Vw, "vh" => CssUnit::Vh, "vmin" => CssUnit::Vmin, "vmax" => CssUnit::Vmax,
            "vi" => CssUnit::Vi, "vb" => CssUnit::Vb,
            "deg" => CssUnit::Deg, "rad" => CssUnit::Rad, "grad" => CssUnit::Grad, "turn" => CssUnit::Turn,
            "s" => CssUnit::S, "ms" => CssUnit::Ms,
            "Hz" => CssUnit::Hz, "kHz" => CssUnit::Khz,
            "dpi" => CssUnit::Dpi, "dpcm" => CssUnit::Dpcm, "dppx" => CssUnit::Dppx,
            "fr" => CssUnit::Fr,
            other => CssUnit::Other(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            CssUnit::Px => "px", CssUnit::Cm => "cm", CssUnit::Mm => "mm", CssUnit::Q => "Q",
            CssUnit::In => "in", CssUnit::Pt => "pt", CssUnit::Pc => "pc",
            CssUnit::Em => "em", CssUnit::Rem => "rem", CssUnit::Ex => "ex", CssUnit::Ch => "ch",
            CssUnit::Vw => "vw", CssUnit::Vh => "vh", CssUnit::Vmin => "vmin", CssUnit::Vmax => "vmax",
            CssUnit::Vi => "vi", CssUnit::Vb => "vb",
            CssUnit::Deg => "deg", CssUnit::Rad => "rad", CssUnit::Grad => "grad", CssUnit::Turn => "turn",
            CssUnit::S => "s", CssUnit::Ms => "ms",
            CssUnit::Hz => "Hz", CssUnit::Khz => "kHz",
            CssUnit::Dpi => "dpi", CssUnit::Dpcm => "dpcm", CssUnit::Dppx => "dppx",
            CssUnit::Fr => "fr",
            CssUnit::Other(s) => s.as_str(),
        }
    }
}
