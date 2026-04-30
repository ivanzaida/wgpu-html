/// SVG `<path>` element.
#[derive(Debug, Clone, Default)]
pub struct SvgPath {
    pub id: Option<String>,
    /// SVG `d` attribute — the path data string.
    pub d: Option<String>,
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: Option<String>,
    pub fill_rule: Option<String>,
    pub opacity: Option<String>,
    pub transform: Option<String>,
}
