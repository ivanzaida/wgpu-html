// Auto-generated from types.json. DO NOT EDIT.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CssType {
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#tokendef-open-paren
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    OpenParenToken,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#tokendef-close-paren
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    CloseParenToken,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#tokendef-open-square
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    OpenSquareToken,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#tokendef-close-square
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    CloseSquareToken,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#tokendef-open-curly
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    OpenCurlyToken,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#tokendef-close-curly
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    CloseCurlyToken,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#typedef-absolute-size
    /// syntax: [ xx-small | x-small | small | medium | large | x-large | xx-large | xxx-large ]
    AbsoluteSize,
    ///
    /// href: https://drafts.csswg.org/css-speech-1/#typedef-voice-family-age
    /// prose: Possible values are child, young and old, indicating the preferred age category to match during voice selection.
    /// for_parents: [voice-family]
    Age,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#typedef-color-alpha-value
    /// syntax: <number> | <percentage>
    /// for_parents: [<color>]
    AlphaValue,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#anb-production
    /// syntax: odd | even | <integer> | <n-dimension> | '+'? n | -n | <ndashdigit-dimension> | '+'? <ndashdigit-ident> | <dashndashdigit-ident> | <n-dimension> <signed-integer> | '+'? n <signed-integer> | -n <signed-integer> | <ndash-dimension> <signless-integer> | '+'? n- <signless-integer> | -n- <signless-integer> | <n-dimension> ['+' | '-'] <signless-integer> | '+'? n ['+' | '-'] <signless-integer> | -n ['+' | '-'] <signless-integer>
    Anb,
    ///
    /// href: https://drafts.csswg.org/css-anchor-position-1/#typedef-anchor-name
    /// syntax: <dashed-ident>
    AnchorName,
    ///
    /// href: https://drafts.csswg.org/css-anchor-position-1/#typedef-anchor-side
    /// syntax: inside | outside | top | left | right | bottom | start | end | self-start | self-end | <percentage> | center
    AnchorSide,
    ///
    /// href: https://drafts.csswg.org/css-anchor-position-1/#typedef-anchor-size
    /// syntax: width | height | block | inline | self-block | self-inline
    AnchorSize,
    ///
    /// href: https://drafts.csswg.org/css-anchor-position-2/#typedef-anchored-in-parens
    /// syntax: ( <anchored-query> ) | ( <anchored-feature> ) | <general-enclosed>
    AnchoredInParens,
    ///
    /// href: https://drafts.csswg.org/css-anchor-position-2/#typedef-anchored-query
    /// syntax: not <anchored-in-parens> | <anchored-in-parens> [ [ and <anchored-in-parens> ]* | [ or <anchored-in-parens> ]* ] | <anchored-feature>
    AnchoredQuery,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#angle-value
    /// prose: Angle values are <dimension>s denoted by <angle>. The angle unit identifiers are:
    Angle,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#typedef-angle-percentage
    /// syntax: [ <angle> | <percentage> ]
    AnglePercentage,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-angular-color-hint
    /// syntax: <angle-percentage> | <zero>
    AngularColorHint,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-angular-color-stop
    /// syntax: <color> <color-stop-angle>?
    AngularColorStop,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-angular-color-stop-list
    /// syntax: <angular-color-stop> , [ <angular-color-hint>? , <angular-color-stop> ]#?
    AngularColorStopList,
    ///
    /// href: https://drafts.csswg.org/css-will-change-1/#typedef-animateable-feature
    /// syntax: scroll-position | contents | <custom-ident>
    AnimateableFeature,
    ///
    /// href: https://drafts.csswg.org/animation-triggers-1/#typedef-animation-action
    /// prose: The possible <animation-action> values, and what effect they have in each animation state:
    AnimationAction,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-any-value
    /// prose: The <any-value> production is identical to <declaration-value>, but also allows top-level <semicolon-token> tokens and <delim-token> tokens with a value of "!". It represents the entirety of what valid CSS can be in any context.
    AnyValue,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-shape-arc-command
    /// syntax: arc <command-end-point> [ [ of <length-percentage>{1,2} ] && <arc-sweep>? && <arc-size>? && [rotate <angle>]? ]
    /// for_parents: [shape()]
    ArcCommand,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-shape-arc-size
    /// syntax: large | small
    /// prose: <arc-size> can be large or small, indicating that the larger or smaller, respectively, of the two possible arcs must be chosen. If omitted, this defaults to small.
    /// for_parents: [shape()]
    ArcSize,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-shape-arc-sweep
    /// syntax: cw | ccw
    /// prose: <arc-sweep> can be cw or ccw, indicating that the arc that is traced around the ellipse clockwise or counter-clockwise from the center, respectively, must be chosen. If omitted, this defaults to ccw.
    /// for_parents: [shape()]
    ArcSweep,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-at-keyword-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    AtKeywordToken,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-at-rule-list
    /// prose: <at-rule-list>: only at-rules are allowed; declarations and qualified rules are automatically invalid.
    AtRuleList,
    ///
    /// href: https://drafts.csswg.org/css-backgrounds-4/#typedef-attachment
    /// syntax: scroll | fixed | local
    Attachment,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-attr-args
    /// syntax: attr( <declaration-value>, <declaration-value>? )
    AttrArgs,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-attr-matcher
    /// syntax: [ '~' | '|' | '^' | '$' | '*' ]? '='
    AttrMatcher,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-attr-modifier
    /// syntax: i | s
    AttrModifier,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-attr-name
    /// syntax: [ <ident-token>? '|' ]? <ident-token>
    AttrName,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-attr-type
    /// syntax: type( <syntax> ) | raw-string | number | <attr-unit>
    AttrType,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-attr-unit
    /// syntax: <custom-ident>
    AttrUnit,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-attribute-selector
    /// syntax: '[' <wq-name> ']' | '[' <wq-name> <attr-matcher> [ <string-token> | <ident-token> ] <attr-modifier>? ']'
    AttributeSelector,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-auto-line-color-list
    /// syntax: <line-color-or-repeat>#? , <auto-repeat-line-color> , <line-color-or-repeat>#?
    AutoLineColorList,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-auto-line-style-list
    /// syntax: <line-style-or-repeat>#? , <auto-repeat-line-style> , <line-style-or-repeat>#?
    AutoLineStyleList,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-auto-line-width-list
    /// syntax: <line-width-or-repeat>#? , <auto-repeat-line-width> , <line-width-or-repeat>#?
    AutoLineWidthList,
    ///
    /// href: https://drafts.csswg.org/css-grid-3/#typedef-auto-repeat
    /// syntax: repeat( [ auto-fill | auto-fit ] , [ <line-names>? <track-size> ]+ <line-names>? )
    AutoRepeat,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-auto-repeat-line-color
    /// syntax: repeat( auto , [ <color> ]# )
    AutoRepeatLineColor,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-auto-repeat-line-style
    /// syntax: repeat( auto , [ <line-style> ]# )
    AutoRepeatLineStyle,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-auto-repeat-line-width
    /// syntax: repeat( auto , [ <line-width> ]# )
    AutoRepeatLineWidth,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#typedef-auto-track-list
    /// syntax: [ <line-names>? [ <fixed-size> | <fixed-repeat> ] ]* <line-names>? <auto-repeat> [ <line-names>? [ <fixed-size> | <fixed-repeat> ] ]* <line-names>?
    AutoTrackList,
    ///
    /// href: https://drafts.csswg.org/css-text-4/#typedef-autospace
    /// syntax: no-autospace | [ ideograph-alpha || ideograph-numeric || punctuation ] || [ insert | replace ]
    Autospace,
    ///
    /// href: https://drafts.csswg.org/scroll-animations-1/#typedef-axis
    /// syntax: block | inline | x | y
    Axis,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-bad-string-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    BadStringToken,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-bad-url-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    BadUrlToken,
    ///
    /// href: https://drafts.csswg.org/css-inline-3/#typedef-baseline-metric
    /// syntax: text-bottom | alphabetic | ideographic | middle | central | mathematical | hanging | text-top
    /// prose: The <baseline-metric> value, which identifies specific baseline metrics, expands to
    BaselineMetric,
    ///
    /// href: https://drafts.csswg.org/css-align-3/#typedef-baseline-position
    /// syntax: [ first | last ]? && baseline
    BaselinePosition,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-basic-shape
    /// syntax: <basic-shape-rect> | <circle()> | <ellipse()> | <polygon()> | <path()> | <shape()>
    /// prose: The <basic-shape> type can be specified using basic shape functions. When using this syntax to define shapes, the reference box is defined by each property that uses <basic-shape> values. The coordinate system for the shape has its origin on the top-left corner of the reference box with the x-axis running to the right and the y-axis running downwards. All the lengths expressed in percentages are resolved from the used dimensions of the reference box.
    BasicShape,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-basic-shape-rect
    /// syntax: <inset()> | <rect()> | <xywh()>
    BasicShapeRect,
    ///
    /// href: https://drafts.csswg.org/css-backgrounds-4/#typedef-bg-clip
    /// syntax: <visual-box> | [ border-area || text ]
    BgClip,
    ///
    /// href: https://drafts.csswg.org/css-backgrounds-4/#typedef-bg-image
    /// syntax: <image> | none
    BgImage,
    ///
    /// href: https://drafts.csswg.org/css-backgrounds-4/#typedef-bg-layer
    /// syntax: <bg-image> || <bg-position> [ / <bg-size> ]? || <repeat-style> || <attachment> || <bg-clip> || <visual-box>
    BgLayer,
    ///
    /// href: https://drafts.csswg.org/css-backgrounds-4/#typedef-bg-position
    /// syntax: <position> | <position-three>
    /// prose: Its value is given as a comma-separated list of <bg-position> values, which are interpreted as <position> values with the resized background image as the alignment subject and the background positioning area as the alignment container.
    BgPosition,
    ///
    /// href: https://drafts.csswg.org/css-backgrounds-4/#typedef-bg-size
    /// syntax: [ <length-percentage [0,∞]> | auto ]{1,2} | cover | contain
    BgSize,
    ///
    /// href: https://drafts.csswg.org/compositing-2/#ltblendmodegt
    /// syntax: normal | darken | multiply | color-burn | lighten | screen | color-dodge | overlay | soft-light | hard-light | difference | exclusion | hue | saturation | color | luminosity
    BlendMode,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-block-contents
    /// prose: When writing a rule grammar, <block-contents> represents this agnostic parsing. It must only be used as the sole value in a block, and represents that no restrictions are implicitly placed on what the block can contain.
    BlockContents,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-boolean-condition
    /// prose: Where <boolean-condition> is a boolean algebra a la Media Queries 4 § 3 Syntax, but with media() and supports() functions as leaves.
    BooleanCondition,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-boolean-expr
    /// syntax: not <boolean-expr-group> | <boolean-expr-group> [ [ and <boolean-expr-group> ]* | [ or <boolean-expr-group> ]* ]
    /// prose: Several contexts (such as @media, @supports, if(), ...) specify conditions, and allow combining those conditions with boolean logic (and/or/not/grouping). Because they use the same non-trivial recursive syntax structure, the special <boolean-expr> production represents this pattern generically.
    BooleanExpr,
    ///
    /// href: https://drafts.csswg.org/css-borders-4/#typedef-border-radius
    /// syntax: <slash-separated-border-radius-syntax> | <legacy-border-radius-syntax>
    BorderRadius,
    ///
    /// href: https://drafts.csswg.org/css2/#value-def-border-style
    /// prose: The border style properties specify the line style of a box’s border (solid, double, dashed, etc.). The properties defined in this section refer to the <border-style> value type, which may take one of the following values:
    BorderStyle,
    ///
    /// href: https://drafts.csswg.org/css2/#value-def-border-width
    /// prose: The border width properties specify the width of the border area. The properties defined in this section refer to the <border-width> value type, which may take one of the following values:
    BorderWidth,
    ///
    /// href: https://drafts.csswg.org/css2/#value-def-bottom
    /// prose: <top>, <right>, <bottom>, and <left> may either have a <length> value or auto. Negative lengths are permitted. The value auto means that a given edge of the clipping region will be the same as the edge of the element’s generated border box (i.e., auto means the same as 0 for <top> and <left>, the same as the used value of the height plus the sum of vertical padding and border widths for <bottom>, and the same as the used value of the width plus the sum of the horizontal padding and border widths for <right>, such that four auto values result in the clipping region being the same as the element’s border box).
    Bottom,
    ///
    /// href: https://drafts.csswg.org/css-box-4/#typedef-box
    /// prose: The following <box> CSS keywords are defined for use in properties (such as transform-box and background-clip) that need to refer to various box edges:
    Box,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#typedef-calc-keyword
    /// syntax: e | pi | infinity | -infinity | NaN
    CalcKeyword,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#typedef-calc-product
    /// syntax: <calc-value> [ [ '*' | / ] <calc-value> ]*
    CalcProduct,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-calc-size-basis
    /// syntax: [ <size-keyword> | <calc-size()> | any | <calc-sum> ]
    CalcSizeBasis,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#typedef-calc-sum
    /// syntax: <calc-product> [ [ '+' | '-' ] <calc-product> ]*
    CalcSum,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#typedef-calc-value
    /// syntax: <number> | <dimension> | <percentage> | <calc-keyword> | ( <calc-sum> )
    CalcValue,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-cdc-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    CdcToken,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-cdo-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    CdoToken,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-cf-image
    /// syntax: [ <image> | <color> ] && <percentage [0,100]>?
    CfImage,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-class-selector
    /// syntax: '.' <ident-token>
    ClassSelector,
    ///
    /// href: https://drafts.csswg.org/css-masking-1/#typedef-clip-source
    /// syntax: <url>
    ClipSource,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-cmyk-component
    /// syntax: <number> | <percentage> | none
    CmykComponent,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-colon-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    ColonToken,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-color
    /// syntax: <color-base> | currentColor | <system-color> | <contrast-color()> | <device-cmyk()> | <light-dark-color>
    /// prose: Colors in CSS are represented by the <color> type:
    Color,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-color-base
    /// syntax: <hex-color> | <color-function> | <named-color> | <color-mix()> | transparent
    ColorBase,
    ///
    /// href: https://drafts.csswg.org/css-fonts-5/#color-font-tech-values
    /// syntax: [color-COLRv0 | color-COLRv1 | color-SVG | color-sbix | color-CBDT ]
    ColorFontTech,
    ///
    /// href: https://drafts.csswg.org/css-color-hdr-1/#typedef-color-function
    /// syntax: <rgb()> | <rgba()> | <hsl()> | <hsla()> | <hwb()> | <lab()> | <lch()> | <oklab()> | <oklch()> | <ictcp()> | <jzazbz()> | <jzczhz()> | <alpha()> | <color()> | <hdr-color()>
    ColorFunction,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#color-interpolation-method
    /// syntax: in [ <rectangular-color-space> | <polar-color-space> <hue-interpolation-method>? | <custom-color-space> ]
    ColorInterpolationMethod,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-color-space
    /// syntax: <rectangular-color-space> | <polar-color-space> | <custom-color-space>
    ColorSpace,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-color-stop
    /// syntax: <color-stop-length> | <color-stop-angle>
    ColorStop,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-color-stop-angle
    /// syntax: [ <angle-percentage> | <zero> ]{1,2}
    ColorStopAngle,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-color-stop-length
    /// syntax: <length-percentage>{1,2}
    ColorStopLength,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-color-stop-list
    /// syntax: <linear-color-stop> , [ <linear-color-hint>? , <linear-color-stop> ]#?
    ColorStopList,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-color-stripe
    /// syntax: <color> && [ <length-percentage> | <flex> ]?
    /// prose: Each <color-stripe> entry defines a solid-color stripe with the specified <color> and thickness. If the thickness is omitted, it defaults to 1fr. Thickness values are interpreted as follows:
    ColorStripe,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-colorspace-params
    /// syntax: [<custom-params> | <predefined-rgb-params> | <xyz-params>]
    ColorspaceParams,
    ///
    /// href: https://drafts.csswg.org/selectors-5/#typedef-combinator
    /// syntax: '>' | '+' | '~' | [ '|' '|' ] | [ / <wq-name> / ]
    Combinator,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-comma-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    CommaToken,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-shape-command-end-point
    /// syntax: [ to <position> | by <coordinate-pair> ]
    /// for_parents: [shape()]
    CommandEndPoint,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#common-lig-values
    /// syntax: [ common-ligatures | no-common-ligatures ]
    CommonLigValues,
    ///
    /// href: https://drafts.csswg.org/css-ui-4/#typedef-appearance-compat-auto
    /// syntax: searchfield | textarea | checkbox | radio | menulist | listbox | meter | progress-bar | button
    /// for_parents: [appearance]
    CompatAuto,
    ///
    /// href: https://drafts.csswg.org/css-ui-4/#typedef-appearance-compat-special
    /// syntax: textfield | menulist-button
    /// for_parents: [appearance]
    CompatSpecial,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-complex-real-selector
    /// syntax: <compound-selector> [ <combinator>? <compound-selector> ]*
    ComplexRealSelector,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-complex-real-selector-list
    /// syntax: <complex-real-selector>#
    ComplexRealSelectorList,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-complex-selector
    /// syntax: <complex-selector-unit> [ <combinator>? <complex-selector-unit> ]*
    ComplexSelector,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-complex-selector-list
    /// syntax: <complex-selector>#
    ComplexSelectorList,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-complex-selector-unit
    /// syntax: [ <compound-selector>? <pseudo-compound-selector>* ]!
    ComplexSelectorUnit,
    ///
    /// href: https://drafts.csswg.org/compositing-2/#compositemode
    /// syntax: clear | copy | source-over | destination-over | source-in | destination-in | source-out | destination-out | source-atop | destination-atop | xor | lighter | plus-darker | plus-lighter
    CompositeMode,
    ///
    /// href: https://drafts.csswg.org/css-masking-1/#typedef-compositing-operator
    /// syntax: add | subtract | intersect | exclude
    CompositingOperator,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-compound-selector
    /// syntax: [ <type-selector>? <subclass-selector>* ]!
    CompoundSelector,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-compound-selector-list
    /// syntax: <compound-selector>#
    CompoundSelectorList,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-conic-gradient-syntax
    /// syntax: [ [ [ from [ <angle> | <zero> ] ]? [ at <position> ]? ] || <color-interpolation-method> ]? , <angular-color-stop-list>
    ConicGradientSyntax,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-container-condition
    /// syntax: [ <container-name>? <container-query>? ]!
    ContainerCondition,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-container-name
    /// syntax: <custom-ident>
    ContainerName,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-container-query
    /// syntax: not <query-in-parens> | <query-in-parens> [ [ and <query-in-parens> ]* | [ or <query-in-parens> ]* ]
    ContainerQuery,
    ///
    /// href: https://drafts.csswg.org/css-align-3/#typedef-content-distribution
    /// syntax: space-between | space-around | space-evenly | stretch
    ContentDistribution,
    ///
    /// href: https://drafts.csswg.org/css-gcpm-4/#typedef-content-level
    /// syntax: element | content | text | <attr()> | <counter>
    ContentLevel,
    ///
    /// href: https://drafts.csswg.org/css-gcpm-3/#content-list
    /// syntax: [ <string> | <counter()> | <counters()> | <content()> | <attr()> ]+
    ContentListContentList,
    ///
    /// href: https://drafts.csswg.org/css-content-3/#typedef-content-content-list
    /// syntax: [ <string> | <image> | <attr()> | contents | <quote> | <leader()> | <target> | <string()> | <content()> | <counter> ]+
    /// prose: Replaces the element’s contents with one or more anonymous inline boxes corresponding to the specified values, in the order specified. Its normal contents are suppressed and do not generate boxes, as if they were display: none. Each value contributes an inline box to the element’s contents. For <image>, this is an inline anonymous replaced element; for the others, it’s an anonymous inline run of text. If an <image> represents an invalid image, the user agent must do one of the following: "Skip" the <image>, generating nothing for it. Display some indication that the image can’t be displayed in place of the <image>, such as a "broken image" icon. This specification intentionally does not define which behavior a user agent must use, but it must use one or the other consistently.
    /// for_parents: [content]
    ContentListTypedefContentContentList,
    ///
    /// href: https://drafts.csswg.org/css-align-3/#typedef-content-position
    /// syntax: center | start | end | flex-start | flex-end
    ContentPosition,
    ///
    /// href: https://drafts.csswg.org/css-content-3/#typedef-content-content-replacement
    /// syntax: <image>
    /// for_parents: [content]
    ContentReplacement,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#contextual-alt-values
    /// syntax: [ contextual | no-contextual ]
    ContextualAltValues,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-shape-control-point
    /// syntax: [ <position> | <relative-control-point> ]
    /// for_parents: [shape()]
    ControlPoint,
    ///
    /// href: https://drafts.csswg.org/css-box-4/#typedef-coord-box
    /// syntax: <paint-box> | view-box
    CoordBox,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-shape-coordinate-pair
    /// syntax: <length-percentage>{2}
    /// for_parents: [shape()]
    CoordinatePair,
    ///
    /// href: https://drafts.csswg.org/css-borders-4/#typedef-corner-shape-value
    /// syntax: round | scoop | bevel | notch | square | squircle | <superellipse()>
    CornerShapeValue,
    ///
    /// href: https://drafts.csswg.org/css-lists-3/#typedef-counter
    /// syntax: <counter()> | <counters()>
    Counter,
    ///
    /// href: https://drafts.csswg.org/css-lists-3/#typedef-counter-name
    /// syntax: <custom-ident>
    /// prose: Counters are referred to in CSS syntax using the <counter-name> type, which represents their name as a <custom-ident>. A <counter-name> name cannot match the keyword none; such an identifier is invalid as a <counter-name>.
    CounterName,
    ///
    /// href: https://drafts.csswg.org/css-counter-styles-3/#typedef-counter-style
    /// syntax: <counter-style-name> | <symbols()>
    CounterStyle,
    ///
    /// href: https://drafts.csswg.org/css-counter-styles-3/#typedef-counter-style-name
    /// syntax: <custom-ident>
    /// prose: <counter-style-name> is a <custom-ident> that is not an ASCII case-insensitive match for none. When used here, to define a counter style, it also cannot be any of the non-overridable counter-style names (in other uses that merely reference a counter style, such as the extend system, these are allowed). The <counter-style-name> is a tree-scoped name.
    CounterStyleName,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-request-url-modifier-cross-origin-modifier
    /// syntax: cross-origin(anonymous | use-credentials)
    /// for_parents: [<request-url-modifier>]
    CrossOriginModifier,
    ///
    /// href: https://drafts.csswg.org/css-mixins-1/#typedef-css-type
    /// syntax: <syntax-component> | <type()>
    CssType,
    ///
    /// href: https://drafts.csswg.org/css-easing-2/#typedef-cubic-bezier-easing-function
    /// syntax: ease | ease-in | ease-out | ease-in-out | <cubic-bezier()>
    CubicBezierEasingFunction,
    ///
    /// href: https://drafts.csswg.org/css-ui-4/#typedef-cursor-cursor-image
    /// syntax: [ <url> | <url-set> ] <number>{2}?
    /// for_parents: [cursor]
    CursorImage,
    ///
    /// href: https://drafts.csswg.org/css-ui-4/#typedef-cursor-predefined
    /// syntax: auto | default | none | context-menu | help | pointer | progress | wait | cell | crosshair | text | vertical-text | alias | copy | move | no-drop | not-allowed | grab | grabbing | e-resize | n-resize | ne-resize | nw-resize | s-resize | se-resize | sw-resize | w-resize | ew-resize | ns-resize | nesw-resize | nwse-resize | col-resize | row-resize | all-scroll | zoom-in | zoom-out
    CursorPredefined,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-shape-curve-command
    /// syntax: curve [ [ to <position> with <control-point> [ / <control-point> ]? ] | [ by <coordinate-pair> with <relative-control-point> [ / <relative-control-point> ]? ] ]
    /// for_parents: [shape()]
    CurveCommand,
    ///
    /// href: https://drafts.csswg.org/css-extensions-1/#typedef-custom-arg
    /// syntax: '$' <ident-token>
    CustomArg,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-custom-color-space
    /// syntax: <dashed-ident>
    CustomColorSpace,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#identifier-value
    /// prose: Some properties accept arbitrary author-defined identifiers as a component value. This generic data type is denoted by <custom-ident>, and represents any valid CSS identifier that would not be misinterpreted as a pre-defined keyword in that property’s value definition. Such identifiers are fully case-sensitive (meaning they’re compared using the "identical to" operation), even in the ASCII range (e.g. example and EXAMPLE are two different, unrelated user-defined identifiers).
    CustomIdent,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-custom-params
    /// syntax: <dashed-ident> [ <number> | <percentage> | none ]+
    CustomParams,
    ///
    /// href: https://drafts.csswg.org/css-variables-2/#typedef-custom-property-name
    /// prose: A custom property is any property whose name starts with two dashes (U+002D HYPHEN-MINUS), like --foo. The <custom-property-name> production corresponds to this: it’s defined as any <dashed-ident> (a valid identifier that starts with two dashes), except -- itself, which is reserved for future use by CSS. Custom properties are solely for use by authors and users; CSS will never give them a meaning beyond what is presented here.
    CustomPropertyName,
    ///
    /// href: https://drafts.csswg.org/css-extensions-1/#typedef-custom-selector
    /// syntax: <custom-arg>? : <extension-name> [ ( <custom-arg>+#? ) ]?
    CustomSelector,
    ///
    /// href: https://w3c.github.io/svgwg/svg2-draft/painting.html#DataTypeDasharray
    /// syntax: [ [ <length-percentage> | <number> ]+ ]#
    Dasharray,
    ///
    /// href: https://drafts.csswg.org/css-mixins-1/#typedef-dashed-function
    /// prose: A <dashed-function> is a functional notation whose function name starts with two dashes (U+002D HYPHEN-MINUS). Its argument grammar is:
    DashedFunction,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#typedef-dashed-ident
    /// prose: The <dashed-ident> production is a <custom-ident>, with all the case-sensitivity that implies, with the additional restriction that it must start with two dashes (U+002D HYPHEN-MINUS).
    DashedIdent,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-dashndashdigit-ident
    /// syntax: <ident-token>
    DashndashdigitIdent,
    ///
    /// href: https://drafts.csswg.org/css-speech-1/#typedef-voice-volume-decibel
    /// prose: The <decibel> type denotes a dimension with a "dB" (decibel unit) unit identifier. Decibels represent the ratio of the squares of the new signal amplitude a1 and the current amplitude a0, as per the following logarithmic equation: volume(dB) = 20 × log10(a1 / a0).
    /// for_parents: [voice-volume]
    Decibel,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-declaration
    /// prose: <declaration> here matches anything that would be successfully parsed by consume a declaration, ignoring the context-validation check at the end of that algorithm. Notably, this includes a trailing !important, which is valid but ignored for the purpose of @supports.
    Declaration,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-declaration-list
    /// prose: <declaration-list>: only declarations are allowed; at-rules and qualified rules are automatically invalid.
    DeclarationList,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-declaration-rule-list
    /// prose: <declaration-rule-list>: declarations and at-rules are allowed; qualified rules are automatically invalid.
    DeclarationRuleList,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value
    /// prose: The <declaration-value> production matches any sequence of one or more tokens, so long as the sequence does not contain <bad-string-token>, <bad-url-token>, unmatched <)-token>, <]-token>, or <}-token>, or top-level <semicolon-token> tokens or <delim-token> tokens with a value of "!". It represents the entirety of what a valid declaration can have as its value.
    DeclarationValue,
    ///
    /// href: https://drafts.csswg.org/css-mixins-1/#typedef-default-value
    /// syntax: <declaration-value>
    DefaultValue,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-delim-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    DelimToken,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#typedef-deprecated-color
    /// syntax: ActiveBorder | ActiveCaption | AppWorkspace | Background | ButtonHighlight | ButtonShadow | CaptionText | InactiveBorder | InactiveCaption | InactiveCaptionText | InfoBackground | InfoText | Menu | MenuText | Scrollbar | ThreeDDarkShadow | ThreeDFace | ThreeDHighlight | ThreeDLightShadow | ThreeDShadow | Window | WindowFrame | WindowText
    /// prose: The deprecated system colors are represented as the <deprecated-color> sub-type, and are defined as:
    DeprecatedColor,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#typedef-dimension
    /// prose: The general term dimension refers to a number with a unit attached to it; and is denoted by <dimension>.
    Dimension,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-dimension-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    DimensionToken,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#discretionary-lig-values
    /// syntax: [ discretionary-ligatures | no-discretionary-ligatures ]
    DiscretionaryLigValues,
    ///
    /// href: https://drafts.csswg.org/css-display-4/#typedef-display-box
    /// syntax: contents | none
    DisplayBox,
    ///
    /// href: https://drafts.csswg.org/css-display-4/#typedef-display-inside
    /// syntax: flow | flow-root | table | flex | grid | ruby
    DisplayInside,
    ///
    /// href: https://drafts.csswg.org/css-display-4/#typedef-display-internal
    /// syntax: table-row-group | table-header-group | table-footer-group | table-row | table-cell | table-column-group | table-column | table-caption | ruby-base | ruby-text | ruby-base-container | ruby-text-container
    DisplayInternal,
    ///
    /// href: https://drafts.csswg.org/css-display-4/#typedef-display-legacy
    /// syntax: inline-block | inline-table | inline-flex | inline-grid
    DisplayLegacy,
    ///
    /// href: https://drafts.csswg.org/css-display-4/#typedef-display-listitem
    /// syntax: <display-outside>? && [ flow | flow-root ]? && list-item
    DisplayListitem,
    ///
    /// href: https://drafts.csswg.org/css-display-4/#typedef-display-outside
    /// syntax: block | inline | run-in
    DisplayOutside,
    ///
    /// href: https://drafts.csswg.org/css-easing-2/#typedef-easing-function
    /// syntax: <linear-easing-function> | <cubic-bezier-easing-function> | <step-easing-function>
    EasingFunction,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#east-asian-variant-values
    /// syntax: [ jis78 | jis83 | jis90 | jis04 | simplified | traditional ]
    EastAsianVariantValues,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#east-asian-width-values
    /// syntax: [ full-width | proportional-width ]
    EastAsianWidthValues,
    ///
    /// href: https://drafts.csswg.org/css-env-1/#typedef-env-args
    /// syntax: env( <declaration-value>, <declaration-value>? )
    EnvArgs,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-eof-token
    /// prose: An <eof-token> is a conceptual token, not actually produced by the tokenizer, used to indicate that the token stream has been exhausted.
    EofToken,
    ///
    /// href: https://drafts.csswg.org/animation-triggers-1/#typedef-event-trigger-event
    /// syntax: activate | interest | click | touch | dblclick | keypress(<string>) | ...
    EventTriggerEvent,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#typedef-explicit-track-list
    /// syntax: [ <line-names>? <track-size> ]+ <line-names>?
    ExplicitTrackList,
    ///
    /// href: https://drafts.csswg.org/css-extensions-1/#typedef-extension-name
    /// prose: All extensions defined in this specification use a common syntax for defining their ”names”: the <extension-name> production. An <extension-name> is any identifier that starts with two dashes (U+002D HYPHEN-MINUS), like --foo, or even exotic names like -- or ------. The CSS language will never use identifiers of this form for any language-defined purpose, so it’s safe to use them for author-defined purposes without ever having to worry about colliding with CSS-defined names.
    ExtensionName,
    ///
    /// href: https://drafts.csswg.org/css2/#value-def-family-name
    /// prose: The name of a font family of choice. In the last example, "Gill" and "Helvetica" are font families.
    FamilyName,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#feature-tag-value
    /// syntax: <opentype-tag> [ <integer [0,∞]> | on | off ]?
    /// for_parents: [font-feature-settings]
    FeatureTagValue,
    ///
    /// href: https://drafts.csswg.org/filter-effects-1/#typedef-filter-function
    /// syntax: <blur()> | <brightness()> | <contrast()> | <drop-shadow()> | <grayscale()> | <hue-rotate()> | <invert()> | <opacity()> | <sepia()> | <saturate()>
    FilterFunction,
    ///
    /// href: https://drafts.csswg.org/filter-effects-1/#typedef-filter-value-list
    /// syntax: [ <filter-function> | <url> ]+
    FilterValueList,
    ///
    /// href: https://drafts.csswg.org/css-backgrounds-4/#typedef-final-bg-layer
    /// syntax: <bg-image> || <bg-position> [ / <bg-size> ]? || <repeat-style> || <attachment> || <bg-clip> || <visual-box> || <'background-color'>
    FinalBgLayer,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#typedef-fixed-breadth
    /// syntax: <length-percentage [0,∞]>
    FixedBreadth,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#typedef-fixed-repeat
    /// syntax: repeat( [ <integer [1,∞]> ] , [ <line-names>? <fixed-size> ]+ <line-names>? )
    FixedRepeat,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#typedef-fixed-size
    /// syntax: <fixed-breadth> | minmax( <fixed-breadth> , <track-breadth> ) | minmax( <inflexible-breadth> , <fixed-breadth> )
    FixedSize,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#typedef-flex
    /// prose: A flexible length or <flex> is a dimension with the fr unit, which represents a fraction of the leftover space in the grid container. Tracks sized with fr units are called flexible tracks as they flex in response to leftover space similar to how flex items with a zero base size fill space in a flex container.
    Flex,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#font-family-name-value
    /// syntax: <string> | <custom-ident>+
    /// prose: The name of a font family, such as Helvetica or Verdana in the previous example. This might be a locally-instaled font, or might be a web font.
    FontFamilyName,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#font-feature-index-value
    /// syntax: <integer>
    FontFeatureIndex,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#font-feature-value-name-value
    /// syntax: <ident>
    FontFeatureValueName,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#typedef-font-feature-values-font-feature-value-type
    /// for_parents: [@font-feature-values]
    FontFeatureValueType,
    ///
    /// href: https://drafts.csswg.org/css-fonts-5/#font-features-tech-values
    /// syntax: [features-opentype | features-aat | features-graphite]
    FontFeaturesTech,
    ///
    /// href: https://drafts.csswg.org/css-fonts-5/#font-format-values
    /// syntax: [<string> | collection | embedded-opentype | opentype | svg | truetype | woff | woff2 ]
    FontFormat,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#typedef-font-src
    /// syntax: <url> [ format( <font-format> ) ]? [ tech( <font-tech># ) ]? | local( <font-family-name> )
    FontSrc,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#typedef-font-src-list
    /// prose: To parse a <font-src-list> production, parse a list of <font-src>s.
    FontSrcList,
    ///
    /// href: https://drafts.csswg.org/css-fonts-5/#font-tech-values
    /// syntax: [<font-features-tech> | <color-font-tech> | variations | palettes | incremental ]
    FontTech,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#font-variant-css21-values
    /// syntax: normal | small-caps
    FontVariantCss2,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#font-weight-absolute-values
    /// syntax: [ normal | bold | <number [1,1000]> ]
    FontWeightAbsolute,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#font-width-css3-values
    /// syntax: normal | ultra-condensed | extra-condensed | condensed | semi-condensed | semi-expanded | expanded | extra-expanded | ultra-expanded
    FontWidthCss3,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-forgiving-selector-list
    /// prose: The <forgiving-selector-list> production instead parses each selector in the list individually, simply ignoring ones that fail to parse, so the remaining selectors can still be used.
    ForgivingSelectorList,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#typedef-picker-form-control-identifier
    /// syntax: select
    /// for_parents: [::picker()]
    FormControlIdentifier,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#frequency-value
    /// prose: Frequency values are dimensions denoted by <frequency>. The frequency unit identifiers are:
    Frequency,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#typedef-frequency-percentage
    /// syntax: [ <frequency> | <percentage> ]
    FrequencyPercentage,
    ///
    /// href: https://drafts.csswg.org/css-mixins-1/#typedef-function-parameter
    /// syntax: <custom-property-name> <css-type>? [ : <default-value> ]?
    FunctionParameter,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-function-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    FunctionToken,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-gap-auto-repeat-rule
    /// syntax: repeat( auto , <gap-rule># )
    GapAutoRepeatRule,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-gap-auto-rule-list
    /// syntax: <gap-rule-or-repeat>#? , <gap-auto-repeat-rule> , <gap-rule-or-repeat>#?
    GapAutoRuleList,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-gap-repeat-rule
    /// syntax: repeat( <integer [1,∞]> , <gap-rule># )
    GapRepeatRule,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-gap-rule
    /// syntax: <line-width> || <line-style> || <color>
    GapRule,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-gap-rule-list
    /// syntax: <gap-rule-or-repeat>#
    GapRuleList,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-gap-rule-or-repeat
    /// syntax: <gap-rule> | <gap-repeat-rule>
    GapRuleOrRepeat,
    ///
    /// href: https://drafts.csswg.org/css-speech-1/#typedef-voice-family-gender
    /// prose: One of the keywords male, female, or neutral, specifying a male, female, or neutral voice, respectively.
    /// for_parents: [voice-family]
    Gender,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-general-enclosed
    /// syntax: [ <function-token> <any-value>? ) ] | [ ( <any-value>? ) ]
    GeneralEnclosed,
    ///
    /// href: https://drafts.csswg.org/css2/#value-def-generic-family
    /// prose: In the example above, the last value is a generic family name. The following generic families are defined: serif (e.g., Times) sans-serif (e.g., Helvetica) cursive (e.g., Zapf-Chancery) fantasy (e.g., Western) monospace (e.g., Courier) Style sheet designers are encouraged to offer a generic font family as a last alternative. Generic font family names are keywords and must NOT be quoted.
    GenericFamily,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#typedef-generic-font-complete
    /// syntax: serif | sans-serif | system-ui | cursive | fantasy | math | monospace
    GenericFontComplete,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#typedef-generic-font-family
    /// syntax: <generic-font-script-specific>| <generic-font-complete> | <generic-font-incomplete>
    GenericFontFamily,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#typedef-generic-font-incomplete
    /// syntax: ui-serif | ui-sans-serif | ui-monospace | ui-rounded
    GenericFontIncomplete,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#typedef-generic-font-script-specific
    /// syntax: generic(fangsong) | generic(kai) | generic(khmer-mul) | generic(nastaliq)
    GenericFontScriptSpecific,
    ///
    /// href: https://drafts.csswg.org/css-speech-1/#typedef-generic-voice
    /// syntax: <age>? <gender> <integer>?
    GenericVoice,
    ///
    /// href: https://drafts.csswg.org/css-masking-1/#typedef-geometry-box
    /// syntax: <shape-box> | fill-box | stroke-box | view-box
    GeometryBox,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-gradient
    /// syntax: [ <linear-gradient()> | <repeating-linear-gradient()> | <radial-gradient()> | <repeating-radial-gradient()> | <conic-gradient()> | <repeating-conic-gradient()> ]
    Gradient,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#typedef-grid-row-start-grid-line
    /// syntax: auto | <custom-ident> | [ [ <integer [-∞,-1]> | <integer [1,∞]> ] && <custom-ident>? ] | [ span && [ <integer [1,∞]> || <custom-ident> ] ]
    /// for_parents: [grid-column-end, grid-column-start, grid-row-end, grid-row-start]
    GridLine,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-hash-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    HashToken,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#typedef-hex-color
    /// prose: The syntax of a <hex-color> is a <hash-token> token whose value consists of 3, 4, 6, or 8 hexadecimal digits. In other words, a hex color is written as a hash character, "#", followed by some number of digits 0-9 or letters a-f (the case of the letters doesn’t matter - #00ff00 is identical to #00FF00).
    HexColor,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#historical-lig-values
    /// syntax: [ historical-ligatures | no-historical-ligatures ]
    HistoricalLigValues,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-shape-horizontal-line-command
    /// syntax: hline [ to [ <length-percentage> | left | center | right | x-start | x-end ] | by <length-percentage> ]
    /// for_parents: [shape()]
    HorizontalLineCommand,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#typedef-hue
    /// syntax: <number> | <angle>
    Hue,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-hue-interpolation-method
    /// syntax: [ shorter | longer | increasing | decreasing ] hue
    HueInterpolationMethod,
    ///
    /// href: https://drafts.csswg.org/css-ui-4/#typedef-id
    /// prose: The <id> value is an ID selector [SELECT]. In response to directional navigation input corresponding to the property, the focus is navigated to the first element in tree order matching the selector. If this refers to the currently focused element, the directional navigation input respective to the nav- property is ignored — there is no need to refocus the same element. If no element matches the selector, the user agent automatically determines which element to navigate the focus to. If the focus is navigated to an element that was not otherwise focusable, it becomes focusable only as the result of this directional navigation, and the :focus pseudo-class matches the element while it is focused as such.
    Id,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-id-selector
    /// syntax: <hash-token>
    IdSelector,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#typedef-ident
    /// prose: CSS identifiers, generically denoted by <ident>, consist of a sequence of characters conforming to the <ident-token> grammar. [CSS-SYNTAX-3] Identifiers cannot be quoted; otherwise they would be interpreted as strings. CSS properties accept two classes of identifiers: pre-defined keywords and author-defined identifiers.
    Ident,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-ident-arg
    /// syntax: <string> | <integer> | <ident>
    IdentArg,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-ident-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    IdentToken,
    ///
    /// href: https://drafts.csswg.org/css2/#value-def-identifier
    Identifier,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-if-args
    /// syntax: if( [ <if-args-branch> ; ]* <if-args-branch> ;? )
    IfArgs,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-if-args-branch
    /// syntax: <declaration-value> : <declaration-value>?
    IfArgsBranch,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-if-branch
    /// syntax: <if-condition> : <declaration-value>?
    IfBranch,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-if-condition
    /// syntax: <boolean-expr[ <if-test> ]> | else
    IfCondition,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-if-test
    /// syntax: supports( [ <ident> : <declaration-value> ] | <supports-condition> ) | media( <media-feature> | <media-condition> ) | style( <style-query> )
    IfTest,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-image
    /// syntax: <url> | <image()> | <image-set()> | <cross-fade()> | <element()> | <gradient>
    Image,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-image-1d
    /// syntax: <stripes()>
    /// prose: While <image> values represent a 2-dimensional (2D) image, and <color> can be thought of as a 0-dimensional (0D) image (unvarying in either axis), some contexts require a 1-dimensional (1D) image, which specifies colors along an abstract, directionless, single-axis paint line. The <image-1D> type represents such 1D images, including the stripes() functional notation:
    Image1d,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-image-set-option
    /// syntax: [ <image> | <string> ] [ <resolution> || type(<string>) ]?
    ImageSetOption,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-image-src
    /// syntax: [ <url> | <string> ]
    ImageSrc,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-image-tags
    /// syntax: [ ltr | rtl ]
    ImageTags,
    ///
    /// href: https://drafts.csswg.org/css-cascade-5/#typedef-import-conditions
    /// syntax: [ supports( [ <supports-condition> | <declaration> ] ) ]? <media-query-list>?
    ImportConditions,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#typedef-inflexible-breadth
    /// syntax: <length-percentage [0,∞]> | min-content | max-content | auto
    InflexibleBreadth,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-inherit-args
    /// syntax: inherit( <declaration-value>, <declaration-value>? )
    InheritArgs,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-init-descriptor
    /// syntax: <init-descriptor-name> : <string>
    InitDescriptor,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-init-descriptor-name
    /// syntax: protocol | hostname | port | pathname | search | hash | base-url
    InitDescriptorName,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-init-descriptors
    /// syntax: ;* <init-descriptor> [ ;+ <init-descriptor> ]* ;*
    InitDescriptors,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-calc-interpolate-input-position
    /// syntax: <percentage> | <number> | <dimension>
    /// for_parents: [calc-interpolate(), calc-interpolate(), color-interpolate(), color-interpolate(), interpolate(), interpolate(), transform-interpolate(), transform-interpolate()]
    InputPosition,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#integer-value
    /// syntax: <number-token>
    /// prose: Integer values are denoted by <integer>.
    Integer,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-request-url-modifier-integrity-modifier
    /// syntax: integrity(<string>)
    /// for_parents: [<request-url-modifier>]
    IntegrityModifier,
    ///
    /// href: https://drafts.csswg.org/compositing-2/#isolated-propid
    /// syntax: auto | isolate
    IsolationMode,
    ///
    /// href: https://drafts.csswg.org/css-animations-1/#typedef-keyframe-block
    /// syntax: <keyframe-selector># { <declaration-list> }
    KeyframeBlock,
    ///
    /// href: https://drafts.csswg.org/css-animations-1/#typedef-keyframe-selector
    /// syntax: from | to | <percentage [0,100]>
    KeyframeSelector,
    ///
    /// href: https://drafts.csswg.org/css-animations-1/#typedef-keyframes-name
    /// syntax: <custom-ident> | <string>
    KeyframesName,
    ///
    /// href: https://drafts.csswg.org/css-cascade-5/#typedef-layer-name
    /// syntax: <ident> [ '.' <ident> ]*
    LayerName,
    ///
    /// href: https://drafts.csswg.org/css-box-4/#typedef-layout-box
    /// syntax: <visual-box> | margin-box
    LayoutBox,
    ///
    /// href: https://drafts.csswg.org/css-content-3/#typedef-leader-type
    /// syntax: dotted | solid | space | <string>
    LeaderType,
    ///
    /// href: https://drafts.csswg.org/css2/#value-def-left
    /// prose: <top>, <right>, <bottom>, and <left> may either have a <length> value or auto. Negative lengths are permitted. The value auto means that a given edge of the clipping region will be the same as the edge of the element’s generated border box (i.e., auto means the same as 0 for <top> and <left>, the same as the used value of the height plus the sum of vertical padding and border widths for <bottom>, and the same as the used value of the width plus the sum of the horizontal padding and border widths for <right>, such that four auto values result in the clipping region being the same as the element’s border box).
    Left,
    ///
    /// href: https://drafts.csswg.org/css-borders-4/#typedef-legacy-border-radius-syntax
    /// syntax: <length-percentage [0,∞]>{1,2}
    LegacyBorderRadiusSyntax,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-legacy-device-cmyk-syntax
    /// syntax: device-cmyk( <number>#{4} )
    LegacyDeviceCmykSyntax,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#typedef-legacy-hsl-syntax
    /// syntax: hsl( <hue>, <percentage>, <percentage>, <alpha-value>? )
    LegacyHslSyntax,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#typedef-legacy-hsla-syntax
    /// syntax: hsla( <hue>, <percentage>, <percentage>, <alpha-value>? )
    LegacyHslaSyntax,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-legacy-pseudo-element-selector
    /// syntax: : [before | after | first-line | first-letter]
    LegacyPseudoElementSelector,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#typedef-legacy-rgb-syntax
    /// syntax: rgb( <percentage>#{3} , <alpha-value>? ) | rgb( <number>#{3} , <alpha-value>? )
    LegacyRgbSyntax,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#typedef-legacy-rgba-syntax
    /// syntax: rgba( <percentage>#{3} , <alpha-value>? ) | rgba( <number>#{3} , <alpha-value>? )
    LegacyRgbaSyntax,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#length-value
    /// prose: Lengths refer to distance measurements and are denoted by <length> in the property definitions. A length is a dimension.
    Length,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#typedef-length-percentage
    /// syntax: [ <length> | <percentage> ]
    LengthPercentage,
    ///
    /// href: https://drafts.csswg.org/selectors-5/#typedef-level
    /// prose: where <level> is a <number-token> with its type flag set to "integer".
    Level,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-light-dark-color
    /// syntax: light-dark(<color>, <color>)
    LightDarkColor,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-light-dark-image
    /// syntax: light-dark( [ <image> | none ] , [ <image> | none ] )
    LightDarkImage,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-line-color-list
    /// syntax: <line-color-or-repeat>#
    LineColorList,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-line-color-or-repeat
    /// syntax: [ <color> | <repeat-line-color> ]
    LineColorOrRepeat,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-shape-line-command
    /// syntax: line <command-end-point>
    /// for_parents: [shape()]
    LineCommand,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#typedef-line-name-list
    /// syntax: [ <line-names> | <name-repeat> ]+
    LineNameList,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#typedef-line-names
    /// syntax: '[' <custom-ident>* ']'
    LineNames,
    ///
    /// href: https://drafts.csswg.org/css-borders-4/#typedef-line-style
    /// syntax: none | hidden | dotted | dashed | solid | double | groove | ridge | inset | outset
    LineStyle,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-line-style-list
    /// syntax: <line-style-or-repeat>#
    LineStyleList,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-line-style-or-repeat
    /// syntax: [ <line-style> | <repeat-line-style> ]
    LineStyleOrRepeat,
    ///
    /// href: https://drafts.csswg.org/css-borders-4/#typedef-line-width
    /// syntax: <length [0,∞]> | hairline | thin | medium | thick
    LineWidth,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-line-width-list
    /// syntax: <line-width-or-repeat>#
    LineWidthList,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-line-width-or-repeat
    /// syntax: [ <line-width> | <repeat-line-width> ]
    LineWidthOrRepeat,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-linear-color-hint
    /// syntax: <length-percentage>
    LinearColorHint,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-linear-color-stop
    /// syntax: <color> <color-stop-length>?
    LinearColorStop,
    ///
    /// href: https://drafts.csswg.org/css-easing-2/#typedef-linear-easing-function
    /// syntax: linear | <linear()>
    LinearEasingFunction,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-linear-gradient-syntax
    /// syntax: [ [ <angle> | <zero> | to <side-or-corner> ] || <color-interpolation-method> ]? , <color-stop-list>
    LinearGradientSyntax,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-link-condition
    /// syntax: [ <navigation-relation> ]? <link-condition-base>
    LinkCondition,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-link-condition-base
    /// syntax: <navigation-location>
    LinkConditionBase,
    ///
    /// href: https://drafts.csswg.org/css2/#value-def-margin-width
    /// prose: The properties defined in this section refer to the <margin-width> value type, which may take one of the following values:
    MarginWidth,
    ///
    /// href: https://w3c.github.io/svgwg/svg2-draft/painting.html#DataTypeMarkerRef
    /// syntax: <url>
    MarkerRef,
    ///
    /// href: https://drafts.csswg.org/css-masking-1/#typedef-mask-layer
    /// syntax: <mask-reference> || <position> [ / <bg-size> ]? || <repeat-style> || <geometry-box> || [ <geometry-box> | no-clip ] || <compositing-operator> || <masking-mode>
    MaskLayer,
    ///
    /// href: https://drafts.csswg.org/css-masking-1/#typedef-mask-reference
    /// syntax: none | <image> | <mask-source>
    MaskReference,
    ///
    /// href: https://drafts.csswg.org/css-masking-1/#typedef-mask-source
    /// syntax: <url>
    MaskSource,
    ///
    /// href: https://drafts.csswg.org/css-masking-1/#typedef-masking-mode
    /// syntax: alpha | luminance | match-source
    MaskingMode,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-media-and
    /// syntax: and <media-in-parens>
    MediaAnd,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-media-condition
    /// syntax: <media-not> | <media-in-parens> [ <media-and>* | <media-or>* ]
    MediaCondition,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-media-condition-without-or
    /// syntax: <media-not> | <media-in-parens> <media-and>*
    MediaConditionWithoutOr,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-media-feature
    /// syntax: [ <mf-plain> | <mf-boolean> | <mf-range> ]
    MediaFeature,
    ///
    /// href: https://drafts.csswg.org/css-cascade-6/#typedef-media-import-condition
    /// syntax: <media-query-list>
    MediaImportCondition,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-media-in-parens
    /// syntax: ( <media-condition> ) | ( <media-feature> ) | <general-enclosed>
    MediaInParens,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-media-not
    /// syntax: not <media-in-parens>
    MediaNot,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-media-or
    /// syntax: or <media-in-parens>
    MediaOr,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-media-query
    /// syntax: <media-condition> | [ not | only ]? <media-type> [ and <media-condition-without-or> ]?
    MediaQuery,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-media-query-list
    /// prose: To parse a <media-query-list> production, parse a comma-separated list of component values, then parse each entry in the returned list as a <media-query>. Its value is the list of <media-query>s so produced.
    MediaQueryList,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-media-type
    /// syntax: <ident>
    MediaType,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-mf-boolean
    /// syntax: <mf-name>
    MfBoolean,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-mf-comparison
    /// syntax: <mf-lt> | <mf-gt> | <mf-eq>
    MfComparison,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-mf-eq
    /// syntax: '='
    MfEq,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-mf-gt
    /// syntax: '>' '='?
    MfGt,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-mf-lt
    /// syntax: '<' '='?
    MfLt,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-mf-name
    /// syntax: <ident>
    MfName,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-mf-plain
    /// syntax: <mf-name> : <mf-value>
    MfPlain,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-mf-range
    /// syntax: <mf-name> <mf-comparison> <mf-value> | <mf-value> <mf-comparison> <mf-name> | <mf-value> <mf-lt> <mf-name> <mf-lt> <mf-value> | <mf-value> <mf-gt> <mf-name> <mf-gt> <mf-value>
    MfRange,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-5/#typedef-mf-value
    /// syntax: <number> | <dimension> | <ident> | <ratio>
    MfValue,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-modern-device-cmyk-syntax
    /// syntax: device-cmyk( <cmyk-component>{4} [ / [ <alpha-value> | none ] ]? )
    ModernDeviceCmykSyntax,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-modern-hsl-syntax
    /// syntax: hsl([from <color>]? [<hue> | none] [<percentage> | <number> | none] [<percentage> | <number> | none] [ / [<alpha-value> | none] ]? )
    ModernHslSyntax,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-modern-hsla-syntax
    /// syntax: hsla([from <color>]? [<hue> | none] [<percentage> | <number> | none] [<percentage> | <number> | none] [ / [<alpha-value> | none] ]? )
    ModernHslaSyntax,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-modern-rgb-syntax
    /// syntax: rgb( [ from <color> ]? [ <number> | <percentage> | none]{3} [ / [<alpha-value> | none] ]? )
    ModernRgbSyntax,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-modern-rgba-syntax
    /// syntax: rgba( [ from <color> ]? [ <number> | <percentage> | none]{3} [ / [<alpha-value> | none] ]? )
    ModernRgbaSyntax,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-shape-move-command
    /// syntax: move <command-end-point>
    /// for_parents: [shape()]
    MoveCommand,
    ///
    /// href: https://drafts.csswg.org/mediaqueries-4/#typedef-mq-boolean
    /// syntax: <integer [0,1]>
    MqBoolean,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-n-dimension
    /// syntax: <dimension-token>
    NDimension,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#typedef-name-repeat
    /// syntax: repeat( [ <integer [1,∞]> | auto-fill ], <line-names>+)
    NameRepeat,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#typedef-named-color
    /// syntax: aliceblue | antiquewhite | aqua | aquamarine | azure | beige | bisque | black | blanchedalmond | blue | blueviolet | brown | burlywood | cadetblue | chartreuse | chocolate | coral | cornflowerblue | cornsilk | crimson | cyan | darkblue | darkcyan | darkgoldenrod | darkgray | darkgreen | darkgrey | darkkhaki | darkmagenta | darkolivegreen | darkorange | darkorchid | darkred | darksalmon | darkseagreen | darkslateblue | darkslategray | darkslategrey | darkturquoise | darkviolet | deeppink | deepskyblue | dimgray | dimgrey | dodgerblue | firebrick | floralwhite | forestgreen | fuchsia | gainsboro | ghostwhite | gold | goldenrod | gray | green | greenyellow | grey | honeydew | hotpink | indianred | indigo | ivory | khaki | lavender | lavenderblush | lawngreen | lemonchiffon | lightblue | lightcoral | lightcyan | lightgoldenrodyellow | lightgray | lightgreen | lightgrey | lightpink | lightsalmon | lightseagreen | lightskyblue | lightslategray | lightslategrey | lightsteelblue | lightyellow | lime | limegreen | linen | magenta | maroon | mediumaquamarine | mediumblue | mediumorchid | mediumpurple | mediumseagreen | mediumslateblue | mediumspringgreen | mediumturquoise | mediumvioletred | midnightblue | mintcream | mistyrose | moccasin | navajowhite | navy | oldlace | olive | olivedrab | orange | orangered | orchid | palegoldenrod | palegreen | paleturquoise | palevioletred | papayawhip | peachpuff | peru | pink | plum | powderblue | purple | rebeccapurple | red | rosybrown | royalblue | saddlebrown | salmon | sandybrown | seagreen | seashell | sienna | silver | skyblue | slateblue | slategray | slategrey | snow | springgreen | steelblue | tan | teal | thistle | tomato | turquoise | violet | wheat | white | whitesmoke | yellow | yellowgreen | transparent
    /// prose: CSS defines a large set of named colors, so that common colors can be written and read more easily. A <named-color> is written as an <ident>, accepted anywhere a <color> is. As usual for CSS-defined <ident>s, all of these keywords are ASCII case-insensitive.
    NamedColor,
    ///
    /// href: https://drafts.csswg.org/css-namespaces-3/#typedef-namespace-prefix
    /// syntax: <ident>
    NamespacePrefix,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-navigation-condition
    /// syntax: not <navigation-in-parens> | <navigation-in-parens> [ and <navigation-in-parens> ]* | <navigation-in-parens> [ or <navigation-in-parens> ]*
    NavigationCondition,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-navigation-in-parens
    /// syntax: ( <navigation-condition> ) | ( <navigation-test> ) | <general-enclosed>
    NavigationInParens,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-navigation-location
    /// syntax: <route-name> | <url-pattern()>
    NavigationLocation,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-navigation-location-between-test
    /// syntax: between : <navigation-location> and <navigation-location>
    NavigationLocationBetweenTest,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-navigation-location-keyword
    /// syntax: at | with | from | to
    NavigationLocationKeyword,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-navigation-location-test
    /// syntax: <navigation-location-keyword> : <navigation-location>
    NavigationLocationTest,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-navigation-relation
    /// syntax: at | with | from | to
    NavigationRelation,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-navigation-test
    /// syntax: <navigation-location-test> | <navigation-location-between-test> | <navigation-type-test>
    NavigationTest,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-navigation-type-keyword
    /// syntax: traverse | back | forward | reload
    NavigationTypeKeyword,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-navigation-type-test
    /// syntax: history : <navigation-type-keyword>
    NavigationTypeTest,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-ndash-dimension
    /// syntax: <dimension-token>
    NdashDimension,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-ndashdigit-dimension
    /// syntax: <dimension-token>
    NdashdigitDimension,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-ndashdigit-ident
    /// syntax: <ident-token>
    NdashdigitIdent,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-ns-prefix
    /// syntax: [ <ident-token> | '*' ]? '|'
    NsPrefix,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#number-value
    /// prose: Number values are denoted by <number>, and represent real numbers, possibly with a fractional component.
    Number,
    ///
    /// href: https://drafts.csswg.org/filter-effects-1/#typedef-number-optional-number
    /// syntax: <number> <number>?
    NumberOptionalNumber,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-number-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    NumberToken,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#numeric-figure-values
    /// syntax: [ lining-nums | oldstyle-nums ]
    NumericFigureValues,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#numeric-fraction-values
    /// syntax: [ diagonal-fractions | stacked-fractions ]
    NumericFractionValues,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#numeric-spacing-values
    /// syntax: [ proportional-nums | tabular-nums ]
    NumericSpacingValues,
    ///
    /// href: https://drafts.csswg.org/motion-1/#typedef-offset-path
    /// syntax: <ray()> | <url> | <basic-shape>
    OffsetPath,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#typedef-opacity-opacity-value
    /// syntax: <number> | <percentage>
    /// prose: The opacity to be applied to the element. The resulting opacity is applied to the entire element, rather than a particular color. Opacity values outside the range [0,1] are not invalid, and are preserved in specified values, but are clamped to the range [0, 1] in computed values.
    /// for_parents: [opacity]
    OpacityValue,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#typedef-opentype-tag
    /// syntax: <string>
    OpentypeTag,
    ///
    /// href: https://drafts.csswg.org/css-ui-4/#typedef-outline-line-style
    /// syntax: none | auto | dotted | dashed | solid | double | groove | ridge | inset | outset
    /// prose: <outline-line-style> accepts the same values as <line-style> (CSS Backgrounds 3 § 3.2 Line Patterns: the border-style properties) with the same meaning, except that hidden is not a legal outline style. In addition, the outline-style property accepts the value auto.
    OutlineLineStyle,
    ///
    /// href: https://drafts.csswg.org/css-align-3/#typedef-overflow-position
    /// syntax: unsafe | safe
    OverflowPosition,
    ///
    /// href: https://drafts.csswg.org/css2/#value-def-padding-width
    /// prose: The properties defined in this section refer to the <padding-width> value type, which may take one of the following values:
    PaddingWidth,
    ///
    /// href: https://drafts.csswg.org/css-page-3/#typedef-page-selector
    /// syntax: [ <ident-token>? <pseudo-page>* ]!
    PageSelector,
    ///
    /// href: https://drafts.csswg.org/css-page-3/#typedef-page-selector-list
    /// syntax: <page-selector>#
    PageSelectorList,
    ///
    /// href: https://drafts.csswg.org/css-page-3/#typedef-page-size-page-size
    /// syntax: A5 | A4 | A3 | B5 | B4 | JIS-B5 | JIS-B4 | letter | legal | ledger
    /// prose: A page size can be specified using one of the following media names. This is the equivalent of specifying size using length values. The definition of the media names comes from Media Standardized Names [PWGMSN].
    /// for_parents: [size]
    PageSize,
    ///
    /// href: https://drafts.csswg.org/fill-stroke-3/#typedef-paint
    /// syntax: none | <image> | <svg-paint>
    Paint,
    ///
    /// href: https://drafts.csswg.org/css-box-4/#typedef-paint-box
    /// syntax: <visual-box> | fill-box | stroke-box
    PaintBox,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#typedef-font-palette-palette-identifier
    /// syntax: <dashed-ident>
    /// prose: This value identifies an CSS-defined palette to use. Users can define a palette by using the @font-palette-values rule. If no applicable @font-palette-values rule is present, this value behaves as normal. <palette-identifier> is parsed as a <dashed-ident>.
    /// for_parents: [font-palette]
    PaletteIdentifier,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-pattern-descriptor
    /// syntax: pattern : <url-pattern()>
    PatternDescriptor,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-pattern-descriptors
    /// syntax: ;* <pattern-descriptor> ;*
    PatternDescriptors,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#percentage-value
    /// prose: Percentage values are denoted by <percentage>, and indicates a value that is some fraction of another reference value.
    Percentage,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-percentage-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    PercentageToken,
    ///
    /// href: https://drafts.csswg.org/pointer-animations-1/#typedef-axis
    /// syntax: block | inline | x | y
    PointerAxis,
    ///
    /// href: https://drafts.csswg.org/pointer-animations-1/#typedef-source
    /// syntax: root | nearest | self
    PointerSource,
    ///
    /// href: https://w3c.github.io/svgwg/svg2-draft/shapes.html#DataTypePoints
    /// syntax: [ <number>+ ]#
    Points,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-polar-color-space
    /// syntax: hsl | hwb | lch | oklch
    PolarColorSpace,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-position
    /// syntax: <position-one> | <position-two> | <position-four>
    /// prose: The <position> value specifies the position of an alignment subject (e.g. a background image) inside an alignment container (e.g. its background positioning area) as a pair of offsets between the specified edges (defaulting to the left and top). Its syntax is:
    Position,
    ///
    /// href: https://drafts.csswg.org/css-anchor-position-1/#typedef-position-area
    /// syntax: [ [ left | center | right | span-left | span-right | x-start | x-end | span-x-start | span-x-end | self-x-start | self-x-end | span-self-x-start | span-self-x-end | span-all ] || [ top | center | bottom | span-top | span-bottom | y-start | y-end | span-y-start | span-y-end | self-y-start | self-y-end | span-self-y-start | span-self-y-end | span-all ] | [ block-start | center | block-end | span-block-start | span-block-end | span-all ] || [ inline-start | center | inline-end | span-inline-start | span-inline-end | span-all ] | [ self-block-start | center | self-block-end | span-self-block-start | span-self-block-end | span-all ] || [ self-inline-start | center | self-inline-end | span-self-inline-start | span-self-inline-end | span-all ] | [ start | center | end | span-start | span-end | span-all ]{1,2} | [ self-start | center | self-end | span-self-start | span-self-end | span-all ]{1,2} ]
    PositionArea,
    ///
    /// href: https://drafts.csswg.org/css-anchor-position-2/#typedef-position-area-query
    /// syntax: [ [ left | center | right | span-left | span-right | x-start | x-end | span-x-start | span-x-end | self-x-start | self-x-end | span-self-x-start | span-self-x-end | span-all | any ] || [ top | center | bottom | span-top | span-bottom | y-start | y-end | span-y-start | span-y-end | self-y-start | self-y-end | span-self-y-start | span-self-y-end | span-all | any ] | [ block-start | center | block-end | span-block-start | span-block-end | span-all | any ] || [ inline-start | center | inline-end | span-inline-start | span-inline-end | span-all | any ] | [ self-block-start | center | self-block-end | span-self-block-start | span-self-block-end | span-all | any ] || [ self-inline-start | center | self-inline-end | span-self-inline-start | span-self-inline-end | span-all | any ] | [ start | center | end | span-start | span-end | span-all | any ]{1,2} | [ self-start | center | self-end | span-self-start | span-self-end | span-all | any ]{1,2} ]
    PositionAreaQuery,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-position-four
    /// syntax: [ [ [ left | right | x-start | x-end ] <length-percentage> ] && [ [ top | bottom | y-start | y-end ] <length-percentage> ] | [ [ block-start | block-end ] <length-percentage> ] && [ [ inline-start | inline-end ] <length-percentage> ] | [ [ start | end ] <length-percentage> ]{2} ]
    PositionFour,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-position-one
    /// syntax: [ left | center | right | top | bottom | x-start | x-end | y-start | y-end | block-start | block-end | inline-start | inline-end | <length-percentage> ]
    PositionOne,
    ///
    /// href: https://drafts.csswg.org/css-backgrounds-4/#typedef-position-three
    /// syntax: [ [ left | center | right ] && [ [ top | bottom ] <length-percentage> ] | [ [ left | right ] <length-percentage> ] && [ top | center | bottom ] ]
    PositionThree,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-position-two
    /// syntax: [ [ left | center | right | x-start | x-end ] && [ top | center | bottom | y-start | y-end ] | [ left | center | right | x-start | x-end | <length-percentage> ] [ top | center | bottom | y-start | y-end | <length-percentage> ] | [ block-start | center | block-end ] && [ inline-start | center | inline-end ] | [ start | center | end ]{2} ]
    PositionTwo,
    ///
    /// href: https://drafts.csswg.org/css-color-hdr-1/#typedef-predefined-rgb
    /// syntax: srgb | srgb-linear | display-p3 | display-p3-linear | a98-rgb | prophoto-rgb | rec2020 | rec2100-pq | rec2100-hlg | rec2100-linear
    PredefinedRgb,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-predefined-rgb-params
    /// syntax: <predefined-rgb> [ <number> | <percentage> | none ]{3}
    PredefinedRgbParams,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-progress-source
    /// syntax: <percentage> | <number> | <dimension> | <'animation-timeline'>
    /// prose: The <progress-source> value type represents the interpolation progress in an interpolation notation. Its syntax is:
    ProgressSource,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-pseudo-class-selector
    /// syntax: : <ident-token> | : <function-token> <any-value> )
    PseudoClassSelector,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-pseudo-compound-selector
    /// syntax: <pseudo-element-selector> <pseudo-class-selector>*
    PseudoCompoundSelector,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-pseudo-element-selector
    /// syntax: : <pseudo-class-selector> | <legacy-pseudo-element-selector>
    PseudoElementSelector,
    ///
    /// href: https://drafts.csswg.org/css-page-3/#typedef-pseudo-page
    /// syntax: : [ left | right | first | blank ]
    PseudoPage,
    ///
    /// href: https://drafts.csswg.org/css-view-transitions-2/#typedef-pt-class-selector
    /// syntax: ['.' <custom-ident>]+
    PtClassSelector,
    ///
    /// href: https://drafts.csswg.org/css-view-transitions-2/#typedef-pt-name-and-class-selector
    /// syntax: <pt-name-selector> <pt-class-selector>? | <pt-class-selector>
    PtNameAndClassSelector,
    ///
    /// href: https://drafts.csswg.org/css-view-transitions-2/#typedef-pt-name-selector
    /// syntax: '*' | <custom-ident>
    PtNameSelector,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-qualified-rule-list
    /// prose: <qualified-rule-list>: only qualified rules are allowed; declarations and at-rules are automatically invalid.
    QualifiedRuleList,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-query-in-parens
    /// syntax: ( <container-query> ) | ( <size-feature> ) | style( <style-query> ) | scroll-state( <scroll-state-query> ) | <general-enclosed>
    QueryInParens,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#typedef-quirky-color
    /// prose: When CSS is being parsed in quirks mode, <quirky-color> is a type of <color> that is only valid in certain properties:
    QuirkyColor,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#typedef-quirky-length
    /// prose: When CSS is being parsed in quirks mode, <quirky-length> is a type of <length> that is only valid in certain properties:
    QuirkyLength,
    ///
    /// href: https://drafts.csswg.org/css-content-3/#typedef-quote
    /// syntax: open-quote | close-quote | no-open-quote | no-close-quote
    Quote,
    ///
    /// href: https://drafts.csswg.org/css-images-3/#typedef-radial-extent
    /// syntax: closest-corner | closest-side | farthest-corner | farthest-side
    RadialExtent,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-radial-gradient-syntax
    /// syntax: [ [ [ <radial-shape> || <radial-size> ]? [ at <position> ]? ] || <color-interpolation-method>]? , <color-stop-list>
    RadialGradientSyntax,
    ///
    /// href: https://drafts.csswg.org/css-images-3/#typedef-radial-shape
    /// syntax: circle | ellipse
    RadialShape,
    ///
    /// href: https://drafts.csswg.org/css-images-3/#typedef-radial-size
    /// syntax: <radial-extent> | <length [0,∞]> | <length-percentage [0,∞]>{2}
    RadialSize,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-random-cache-key
    /// syntax: <dashed-ident> || element-scoped || [ property-scoped | property-index-scoped | <random-ua-ident> ]
    RandomCacheKey,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-random-item-args
    /// syntax: random-item( <declaration-value>, [ <declaration-value>? ]# )
    RandomItemArgs,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-random-key
    /// syntax: auto | <random-cache-key> | fixed <number [0,1]>
    RandomKey,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-random-ua-ident
    /// syntax: <custom-ident>
    RandomUaIdent,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#ratio-value
    /// syntax: <number [0,∞]> [ / <number [0,∞]> ]?
    /// prose: Ratio values are denoted by <ratio>, and represent the ratio of two numeric values. It most often represents an aspect ratio, relating a width (first) to a height (second).
    Ratio,
    ///
    /// href: https://drafts.csswg.org/motion-1/#typedef-ray-size
    /// syntax: <radial-extent> | sides
    RaySize,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-rectangular-color-space
    /// syntax: srgb | srgb-linear | display-p3 | display-p3-linear | a98-rgb | prophoto-rgb | rec2020 | lab | oklab | <xyz-space>
    RectangularColorSpace,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-request-url-modifier-referrer-policy-modifier
    /// syntax: referrer-policy(no-referrer | no-referrer-when-downgrade | same-origin | origin | strict-origin | origin-when-cross-origin | strict-origin-when-cross-origin | unsafe-url)
    /// for_parents: [<request-url-modifier>]
    ReferrerPolicyModifier,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-shape-relative-control-point
    /// syntax: <coordinate-pair> [ from [ start | end | origin ] ]?
    /// for_parents: [shape()]
    RelativeControlPoint,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-relative-real-selector
    /// syntax: <combinator>? <complex-real-selector>
    RelativeRealSelector,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-relative-real-selector-list
    /// syntax: <relative-real-selector>#
    RelativeRealSelectorList,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-relative-selector
    /// syntax: <combinator>? <complex-selector>
    RelativeSelector,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-relative-selector-list
    /// syntax: <relative-selector>#
    RelativeSelectorList,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#typedef-relative-size
    /// syntax: [ larger | smaller ]
    RelativeSize,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-repeat-line-color
    /// syntax: repeat( [ <integer [1,∞]> ] , [ <color> ]# )
    RepeatLineColor,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-repeat-line-style
    /// syntax: repeat( [ <integer [1,∞]> ] , [ <line-style> ]# )
    RepeatLineStyle,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#typedef-repeat-line-width
    /// syntax: repeat( [ <integer [1,∞]> ] , [ <line-width> ]# )
    RepeatLineWidth,
    ///
    /// href: https://drafts.csswg.org/css-backgrounds-4/#typedef-repeat-style
    /// syntax: repeat-x | repeat-y | repeat-block | repeat-inline | <repetition>{1,2}
    RepeatStyle,
    ///
    /// href: https://drafts.csswg.org/css-backgrounds-4/#typedef-repetition
    /// syntax: repeat | space | round | no-repeat
    Repetition,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-request-url-modifier
    /// syntax: <cross-origin-modifier> | <integrity-modifier> | <referrer-policy-modifier>
    /// prose: <request-url-modifier>s are <url-modifier>s that affect the <url>’s resource request by applying associated URL request modifier steps. See CSS Values 4 § 4.5.4 URL Processing Model.
    RequestUrlModifier,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#resolution-value
    /// prose: Resolution units are dimensions denoted by <resolution>. The resolution unit identifiers are:
    Resolution,
    ///
    /// href: https://drafts.csswg.org/css-lists-3/#typedef-reversed-counter-name
    /// syntax: reversed( <counter-name> )
    ReversedCounterName,
    ///
    /// href: https://drafts.csswg.org/css2/#value-def-right
    /// prose: <top>, <right>, <bottom>, and <left> may either have a <length> value or auto. Negative lengths are permitted. The value auto means that a given edge of the clipping region will be the same as the edge of the element’s generated border box (i.e., auto means the same as 0 for <top> and <left>, the same as the used value of the height plus the sum of vertical padding and border widths for <bottom>, and the same as the used value of the width plus the sum of the horizontal padding and border widths for <right>, such that four auto values result in the clipping region being the same as the element’s border box).
    Right,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#typedef-rounding-strategy
    /// syntax: nearest | up | down | to-zero | line-width
    /// prose: If A is exactly equal to an integer multiple of B, round() resolves to A exactly (preserving whether A is 0⁻ or 0⁺, if relevant). Otherwise, there are two integer multiples of B that are potentially "closest" to A, lower B which is closer to −∞ and upper B which is closer to +∞. The following <rounding-strategy>s dictate how to choose between them:
    RoundingStrategy,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#typedef-route-name
    /// syntax: <dashed-ident>
    RouteName,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-rule-list
    /// prose: <rule-list>: qualified rules and at-rules are allowed; declarations are automatically invalid.
    RuleList,
    ///
    /// href: https://drafts.csswg.org/css-page-3/#typedef-safe-printable-inset
    /// prose: Let this value be <safe-printable-inset>, which is a <length>.
    SafePrintableInset,
    ///
    /// href: https://drafts.csswg.org/css-cascade-6/#typedef-scope-boundaries
    /// syntax: [ [ ( <scope-start> ) ]? [ to ( <scope-end> ) ]? ]!
    ScopeBoundaries,
    ///
    /// href: https://drafts.csswg.org/css-cascade-6/#typedef-scope-end
    /// syntax: <selector-list>
    /// prose: <scope-end> is a <selector-list> selector used to identify any scoping limits.
    ScopeEnd,
    ///
    /// href: https://drafts.csswg.org/css-cascade-6/#typedef-scope-start
    /// syntax: <selector-list>
    /// prose: <scope-start> is a <selector-list> selector used to identify the scoping root(s).
    ScopeStart,
    ///
    /// href: https://drafts.csswg.org/css-overflow-5/#typedef-scroll-button-direction
    /// syntax: up | down | left | right | block-start | block-end | inline-start | inline-end | prev | next
    /// prose: The four ::scroll-button() pseudo-elements are individually selected by the selector’s argument. A * arguments selects all four ::scroll-button()s; otherwise the selected pseudo-element is determined by the <scroll-button-direction> value:
    ScrollButtonDirection,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-scroll-state-feature
    /// prose: A container scroll-state query allows querying a container for state that depends on scroll position. It is a boolean combination of individual scroll-state features (<scroll-state-feature>) that each query a single feature of the query container. The syntax of a <scroll-state-feature> is the same as for a media feature: a feature name, a comparator, and a value.
    ScrollStateFeature,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-scroll-state-in-parens
    /// syntax: ( <scroll-state-query> ) | ( <scroll-state-feature> ) | <general-enclosed>
    ScrollStateInParens,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-scroll-state-query
    /// syntax: not <scroll-state-in-parens> | <scroll-state-in-parens> [ [ and <scroll-state-in-parens> ]* | [ or <scroll-state-in-parens> ]* ] | <scroll-state-feature>
    ScrollStateQuery,
    ///
    /// href: https://drafts.csswg.org/scroll-animations-1/#typedef-scroller
    /// syntax: root | nearest | self
    Scroller,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-calc-interpolate-segment-options
    /// prose: When appearing in the first argument, provides any type-specific interpolation options that apply to every segment in the interpolation map. (For example, color-interpolate() allows <color-interpolation-method>.) When appearing between stops, provides any type-specific interpolation options that apply to the interpolation segment between the stops on either side of this argument, overriding any default provided by a corresponding global <segment-options> argument.
    /// for_parents: [calc-interpolate(), color-interpolate(), interpolate(), transform-interpolate()]
    SegmentOptions,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-selector-list
    /// syntax: <complex-selector-list>
    SelectorList,
    ///
    /// href: https://drafts.csswg.org/css-align-3/#typedef-self-position
    /// syntax: center | start | end | self-start | self-end | flex-start | flex-end
    SelfPosition,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-semicolon-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    SemicolonToken,
    ///
    /// href: https://drafts.csswg.org/css-speech-1/#typedef-voice-pitch-semitones
    /// for_parents: [voice-pitch]
    Semitones,
    ///
    /// href: https://drafts.csswg.org/css-backgrounds-3/#typedef-shadow
    /// syntax: <color>? && [ <length>{2} [ <length [0,∞]> <length>? ]? ] && inset?
    Shadow,
    ///
    /// href: https://drafts.csswg.org/css2/#value-def-shape
    /// syntax: rect(<top>, <right>, <bottom>, <left>)
    /// prose: In CSS 2, the only valid <shape> value is: rect(<top>, <right>, <bottom>, <left>) where <top> and <bottom> specify offsets from the top border edge of the box, and <right>, and <left> specify offsets from the left border edge of the box. Authors should separate offset values with commas. User agents must support separation with commas, but may also support separation without commas (but not a combination), because a previous revision of this specification was ambiguous in this respect. <top>, <right>, <bottom>, and <left> may either have a <length> value or auto. Negative lengths are permitted. The value auto means that a given edge of the clipping region will be the same as the edge of the element’s generated border box (i.e., auto means the same as 0 for <top> and <left>, the same as the used value of the height plus the sum of vertical padding and border widths for <bottom>, and the same as the used value of the width plus the sum of the horizontal padding and border widths for <right>, such that four auto values result in the clipping region being the same as the element’s border box). When coordinates are rounded to pixel coordinates, care should be taken that no pixels remain visible when <left> and <right> have the same value (or <top> and <bottom> have the same value), and conversely that no pixels within the element’s border box remain hidden when these values are auto.
    Shape,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-shape-box
    /// syntax: <visual-box> | margin-box | half-border-box
    ShapeBox,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-shape-command
    /// syntax: <move-command> | <line-command> | close | <horizontal-line-command> | <vertical-line-command> | <curve-command> | <smooth-command> | <arc-command>
    /// prose: The sequence of <shape-command>s represent further path data commands. Each command’s starting point is the previous command’s ending point.
    ShapeCommand,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#typedef-side-or-corner
    /// syntax: [left | right] || [top | bottom]
    SideOrCorner,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-signed-integer
    /// syntax: <number-token>
    SignedInteger,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-signless-integer
    /// syntax: <number-token>
    SignlessInteger,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-simple-selector
    /// syntax: <type-selector> | <subclass-selector>
    SimpleSelector,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-simple-selector-list
    /// syntax: <simple-selector>#
    SimpleSelectorList,
    ///
    /// href: https://drafts.csswg.org/css-animations-2/#typedef-single-animation
    /// syntax: <'animation-duration'> || <easing-function> || <'animation-delay'> || <single-animation-iteration-count> || <single-animation-direction> || <single-animation-fill-mode> || <single-animation-play-state> || [ none | <keyframes-name> ] || <single-animation-timeline>
    SingleAnimation,
    ///
    /// href: https://drafts.csswg.org/css-animations-2/#typedef-single-animation-composition
    /// syntax: replace | add | accumulate
    SingleAnimationComposition,
    ///
    /// href: https://drafts.csswg.org/css-animations-1/#typedef-single-animation-direction
    /// syntax: normal | reverse | alternate | alternate-reverse
    SingleAnimationDirection,
    ///
    /// href: https://drafts.csswg.org/css-animations-1/#typedef-single-animation-fill-mode
    /// syntax: none | forwards | backwards | both
    SingleAnimationFillMode,
    ///
    /// href: https://drafts.csswg.org/css-animations-1/#typedef-single-animation-iteration-count
    /// syntax: infinite | <number [0,∞]>
    SingleAnimationIterationCount,
    ///
    /// href: https://drafts.csswg.org/css-animations-1/#typedef-single-animation-play-state
    /// syntax: running | paused
    SingleAnimationPlayState,
    ///
    /// href: https://drafts.csswg.org/css-animations-2/#typedef-single-animation-timeline
    /// syntax: auto | none | <dashed-ident> | <scroll()> | <view()>
    SingleAnimationTimeline,
    ///
    /// href: https://drafts.csswg.org/css-transitions-2/#single-transition
    /// syntax: [ none | <single-transition-property> ] || <time> || <easing-function> || <time> || <transition-behavior-value>
    SingleTransition,
    ///
    /// href: https://drafts.csswg.org/css-transitions-1/#single-transition-property
    /// syntax: all | <custom-ident>
    SingleTransitionProperty,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-size-feature
    /// prose: A container size query allows querying the size of the query container’s principal box. It is a boolean combination of individual size features (<size-feature>) that each query a single, specific dimensional feature of the query container. The syntax of a <size-feature> is the same as for a media feature: a feature name, a comparator, and a value. [mediaqueries-5] The boolean syntax and logic combining size features into a size query is the same as for CSS feature queries. (See @supports. [css-conditional-3])
    SizeFeature,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-size-keyword
    /// prose: The <size-keyword> production matches any sizing keywords allowed in the context. For example, in width, it matches auto, min-content, stretch, etc.
    SizeKeyword,
    ///
    /// href: https://drafts.csswg.org/css-borders-4/#typedef-slash-separated-border-radius-syntax
    /// syntax: <length-percentage [0,∞]> [ / <length-percentage [0,∞]> ]?
    SlashSeparatedBorderRadiusSyntax,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-shape-smooth-command
    /// syntax: smooth [ [ to <position> [ with <control-point> ]? ] | [ by <coordinate-pair> [ with <relative-control-point> ]? ] ]
    /// for_parents: [shape()]
    SmoothCommand,
    ///
    /// href: https://html.spec.whatwg.org/multipage/images.html#source-size
    /// syntax: <media-condition> <source-size-value> | auto
    SourceSize,
    ///
    /// href: https://html.spec.whatwg.org/multipage/images.html#source-size-list
    /// syntax: <source-size>#? , <source-size-value>
    SourceSizeList,
    ///
    /// href: https://html.spec.whatwg.org/multipage/images.html#source-size-value
    /// syntax: <length> | auto
    SourceSizeValue,
    ///
    /// href: https://drafts.csswg.org/css-text-4/#typedef-spacing-trim
    /// syntax: space-all | normal | space-first | trim-start | trim-both | trim-all
    SpacingTrim,
    ///
    /// href: https://drafts.csswg.org/css-borders-4/#typedef-spread-shadow
    /// syntax: <'box-shadow-color'>? && [ [ none | <length>{2} ] [ <'box-shadow-blur'> <'box-shadow-spread'>? ]? ] && <'box-shadow-position'>?
    SpreadShadow,
    ///
    /// href: https://drafts.csswg.org/css-easing-2/#typedef-step-easing-function
    /// syntax: step-start | step-end | <steps()>
    StepEasingFunction,
    ///
    /// href: https://drafts.csswg.org/css-easing-2/#typedef-step-position
    /// syntax: jump-start | jump-end | jump-none | jump-both | start | end
    StepPosition,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#string-value
    /// prose: Strings are denoted by <string>. When written literally, they consist of a sequence of characters delimited by double quotes or single quotes, corresponding to the <string-token> production in the CSS Syntax Module [CSS-SYNTAX-3].
    String,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-string-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    StringToken,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-style-feature
    /// syntax: <style-feature-plain> | <style-feature-boolean> | <style-range>
    StyleFeature,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-style-feature-boolean
    /// syntax: <style-feature-name>
    StyleFeatureBoolean,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-style-feature-name
    /// prose: A container style query allows querying the computed values of the query container. It is a boolean combination of individual style features (<style-feature>) that each query a single, specific property of the query container. The syntax of a <style-feature> is either the same as for a valid declaration[CSS-SYNTAX-3], a <style-feature-name> or a valid style range (<style-range>). The <style-feature-name> can be either a supported CSS property or a valid <custom-property-name>. The <style-feature-value> production matches any valid <declaration-value> as long as it doesn’t contain <mf-lt>, <mf-gt> and <mf-eq> tokens.
    StyleFeatureName,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-style-feature-plain
    /// syntax: <style-feature-name> : <style-feature-value>
    StyleFeaturePlain,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-style-feature-value
    /// prose: A container style query allows querying the computed values of the query container. It is a boolean combination of individual style features (<style-feature>) that each query a single, specific property of the query container. The syntax of a <style-feature> is either the same as for a valid declaration[CSS-SYNTAX-3], a <style-feature-name> or a valid style range (<style-range>). The <style-feature-name> can be either a supported CSS property or a valid <custom-property-name>. The <style-feature-value> production matches any valid <declaration-value> as long as it doesn’t contain <mf-lt>, <mf-gt> and <mf-eq> tokens.
    StyleFeatureValue,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-style-in-parens
    /// syntax: ( <style-query> ) | ( <style-feature> ) | <general-enclosed>
    StyleInParens,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-style-query
    /// syntax: not <style-in-parens> | <style-in-parens> [ [ and <style-in-parens> ]* | [ or <style-in-parens> ]* ] | <style-feature>
    StyleQuery,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-style-range
    /// syntax: <style-range-value> <mf-comparison> <style-range-value> | <style-range-value> <mf-lt> <style-range-value> <mf-lt> <style-range-value> | <style-range-value> <mf-gt> <style-range-value> <mf-gt> <style-range-value>
    StyleRange,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-style-range-value
    /// syntax: <custom-property-name> | <style-feature-value>
    StyleRangeValue,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-subclass-selector
    /// syntax: <id-selector> | <class-selector> | <attribute-selector> | <pseudo-class-selector>
    SubclassSelector,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-supports-at-rule-fn
    /// syntax: at-rule( <at-keyword-token> )
    SupportsAtRuleFn,
    ///
    /// href: https://drafts.csswg.org/css-conditional-3/#typedef-supports-condition
    /// syntax: not <supports-in-parens> | <supports-in-parens> [ and <supports-in-parens> ]* | <supports-in-parens> [ or <supports-in-parens> ]*
    SupportsCondition,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-supports-condition-name
    /// prose: Where <supports-condition-name> is an <extension-name> that defines the name of the supports query.
    SupportsConditionName,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-supports-decl
    /// syntax: ( [ <declaration> | <supports-condition-name> ] )
    SupportsDecl,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-supports-env-fn
    /// syntax: env( <ident> )
    SupportsEnvFn,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-supports-feature
    /// syntax: <supports-selector-fn> | <supports-font-tech-fn> | <supports-font-format-fn> | <supports-at-rule-fn> | <supports-named-feature-fn> | <supports-env-fn> | <supports-decl>
    SupportsFeature,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-supports-font-format-fn
    /// syntax: font-format( <font-format> )
    SupportsFontFormatFn,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-supports-font-tech-fn
    /// syntax: font-tech( <font-tech> )
    SupportsFontTechFn,
    ///
    /// href: https://drafts.csswg.org/css-cascade-6/#typedef-supports-import-condition
    /// syntax: supports( [ <supports-condition> | <declaration> ] )
    SupportsImportCondition,
    ///
    /// href: https://drafts.csswg.org/css-conditional-3/#typedef-supports-in-parens
    /// syntax: ( <supports-condition> ) | <supports-feature> | <general-enclosed>
    SupportsInParens,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#typedef-supports-named-feature-fn
    /// syntax: named-feature( <ident> )
    SupportsNamedFeatureFn,
    ///
    /// href: https://drafts.csswg.org/css-conditional-4/#typedef-supports-selector-fn
    /// syntax: selector( <complex-selector> )
    SupportsSelectorFn,
    ///
    /// href: https://drafts.csswg.org/fill-stroke-3/#typedef-svg-paint
    /// syntax: child | child( <integer> )
    SvgPaint,
    ///
    /// href: https://drafts.csswg.org/css-counter-styles-3/#typedef-symbol
    /// syntax: <string> | <image> | <custom-ident>
    Symbol,
    ///
    /// href: https://drafts.csswg.org/css-counter-styles-3/#typedef-symbols-type
    /// syntax: cyclic | numeric | alphabetic | symbolic | fixed
    SymbolsType,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-syntax
    /// syntax: '*' | <syntax-component> [ <syntax-combinator> <syntax-component> ]* | <syntax-string>
    Syntax,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-syntax-combinator
    /// syntax: '|'
    SyntaxCombinator,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-syntax-component
    /// syntax: <syntax-single-component> <syntax-multiplier>? | '<' transform-list '>'
    SyntaxComponent,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-syntax-multiplier
    /// syntax: [ '#' | '+' ]
    SyntaxMultiplier,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-syntax-single-component
    /// syntax: '<' <syntax-type-name> '>' | <ident>
    SyntaxSingleComponent,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-syntax-string
    /// syntax: <string>
    SyntaxString,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#typedef-syntax-type-name
    /// syntax: angle | color | custom-ident | image | integer | length | length-percentage | number | percentage | resolution | string | time | url | transform-function
    SyntaxTypeName,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#typedef-system-color
    /// syntax: AccentColor | AccentColorText | ActiveText | ButtonBorder | ButtonFace | ButtonText | Canvas | CanvasText | Field | FieldText | GrayText | Highlight | HighlightText | LinkText | Mark | MarkText | SelectedItem | SelectedItemText | VisitedText | <deprecated-color>
    /// prose: However, in forced colors mode, most colors on the page are forced into a restricted, user-chosen palette, see CSS Color Adjustment 1 § 5.2 Forced Colors Mode Color Palettes. The <system-color> keywords expose these user-chosen colors so that the rest of the page can integrate with this restricted palette.
    SystemColor,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#system-font-family-name-value
    /// syntax: caption | icon | menu | message-box | small-caption | status-bar
    /// prose: A locally installed system font, whose use is subject to certain constraints. In particular, it may not be used with the font-family property, but can be used with the font shorthand. The following values refer to system fonts:
    SystemFontFamilyName,
    ///
    /// href: https://drafts.csswg.org/css-content-3/#typedef-target
    /// syntax: <target-counter()> | <target-counters()> | <target-text()>
    Target,
    ///
    /// href: https://drafts.csswg.org/css-color-6/#typedef-target-contrast
    /// syntax: <wcag2>
    /// prose: The <target-contrast> argument specifies the contrast algorithm(s) to use. If no color candidates have been provided, <target-contrast> may be omitted, in which case a UA-chosen algorithm is used.
    TargetContrast,
    ///
    /// href: https://drafts.csswg.org/css-ui-4/#typedef-target-name
    /// prose: The <target-name> parameter indicates the target frame for the focus navigation. It is a <string> and it MUST NOT start with the underscore "_" character. Error handling: if it does start with an underscore, "_parent" navigates to the parent frame, "_root" is treated as root, and other values navigate to a frame by that name if it exists. If the specified target frame does not exist, the parameter will be treated as the keyword current, which means to simply use the frame that the element is in. The keyword root indicates that the user agent should target the full window.
    TargetName,
    ///
    /// href: https://drafts.csswg.org/css-inline-3/#typedef-text-edge
    /// syntax: [ text | ideographic | ideographic-ink ] | [ text | ideographic | ideographic-ink | cap | ex ] [ text | ideographic | ideographic-ink | alphabetic ]
    /// prose: The <text-edge> value, which identifies specific font metrics, expands to
    TextEdge,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#time-value
    /// prose: Time values are dimensions denoted by <time>. The time unit identifiers are:
    Time,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#typedef-time-percentage
    /// syntax: [ <time> | <percentage> ]
    TimePercentage,
    ///
    /// href: https://drafts.csswg.org/pointer-animations-1/#typedef-timeline-range-center-subject
    /// prose: The animation range center subject is the element whose principal box is the range relative to which the center of the active range is calculated, and is represented by the <timeline-range-center-subject> value type, which indicates a CSS identifier representing one of the following:
    TimelineRangeCenterSubject,
    ///
    /// href: https://drafts.csswg.org/scroll-animations-1/#typedef-timeline-range-name
    /// prose: A named timeline range is a named segment of an animation timeline. The start of the segment is represented as 0% progress through the range; the end of the segment is represented as 100% progress through the range. Multiple named timeline ranges can be associated with a given timeline, and multiple such ranges can overlap. For example, the contain range of a view progress timeline overlaps with its cover range. Named timeline ranges are represented by the <timeline-range-name> value type, which indicates a CSS identifier representing one of the predefined named timeline ranges.
    TimelineRangeName,
    ///
    /// href: https://drafts.csswg.org/css2/#value-def-top
    /// prose: <top>, <right>, <bottom>, and <left> may either have a <length> value or auto. Negative lengths are permitted. The value auto means that a given edge of the clipping region will be the same as the edge of the element’s generated border box (i.e., auto means the same as 0 for <top> and <left>, the same as the used value of the height plus the sum of vertical padding and border widths for <bottom>, and the same as the used value of the width plus the sum of the horizontal padding and border widths for <right>, such that four auto values result in the clipping region being the same as the element’s border box).
    Top,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#typedef-track-breadth
    /// syntax: <length-percentage [0,∞]> | <flex [0,∞]> | min-content | max-content | auto
    TrackBreadth,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#typedef-track-list
    /// syntax: [ <line-names>? [ <track-size> | <track-repeat> ] ]+ <line-names>?
    TrackList,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#typedef-track-repeat
    /// syntax: repeat( [ <integer [1,∞]> ] , [ <line-names>? <track-size> ]+ <line-names>? )
    TrackRepeat,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#typedef-track-size
    /// syntax: <track-breadth> | minmax( <inflexible-breadth> , <track-breadth> ) | fit-content( <length-percentage [0,∞]> )
    TrackSize,
    ///
    /// href: https://drafts.csswg.org/css-transforms-2/#typedef-transform-function
    /// syntax: <scale3d()> | <scale()> | <scaleX()> | <scaleY()> | <scaleZ()> | <translate3d()> | <translate()> | <translateX()> | <translateY()> | <translateZ()> | <rotate3d()> | <rotate()> | <rotateX()> | <rotateY()> | <rotateZ()> | <skew()> | <skewX()> | <skewY()> | <matrix3d()> | <matrix()> | <perspective()>
    /// prose: The value of the transform property is a list of <transform-function>. The set of allowed transform functions is given below. Wherever <angle> is used in this specification, a <number> that is equal to zero is also allowed, which is treated the same as an angle of zero degrees. A percentage for horizontal translations is relative to the width of the reference box. A percentage for vertical translations is relative to the height of the reference box. A percentage in a scale function is equivalent to a number, and serializes as a number in specified values. For example, scale3d(50%, 100%, 150%) serializes as scale3d(0.5, 1, 1.5).
    TransformFunction,
    ///
    /// href: https://drafts.csswg.org/css-transforms-1/#typedef-transform-list
    /// syntax: <transform-function>+
    TransformList,
    ///
    /// href: https://drafts.csswg.org/css-transitions-2/#typedef-transition-behavior-value
    /// syntax: normal | allow-discrete
    TransitionBehaviorValue,
    ///
    /// href: https://drafts.csswg.org/css-anchor-position-1/#typedef-try-size
    /// syntax: most-width | most-height | most-block-size | most-inline-size
    TrySize,
    ///
    /// href: https://drafts.csswg.org/css-anchor-position-1/#typedef-position-try-fallbacks-try-tactic
    /// syntax: flip-block || flip-inline || flip-start || flip-x || flip-y
    /// for_parents: [position-try-fallbacks]
    TryTactic,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#typedef-type
    /// syntax: '<' [ number | string ] '>'
    Type,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-type-selector
    /// syntax: <wq-name> | <ns-prefix>? '*'
    TypeSelector,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-unicode-range-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    UnicodeRangeToken,
    ///
    /// href: https://drafts.csswg.org/css2/#value-def-uri
    /// prose: URI values (Uniform Resource Identifiers, see [RFC3986], which includes URLs, URNs, etc) in this specification are denoted by <uri>. The functional notation used to designate URIs in property values is "url()", as in:
    Uri,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#url-value
    /// syntax: <url()> | <src()>
    Url,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#typedef-url-modifier
    /// prose: <url>s support specifying additional <url-modifier>s, which change the meaning or the interpretation of the URL somehow. A <url-modifier> is either an <ident> or a functional notation.
    UrlModifier,
    ///
    /// href: https://drafts.csswg.org/css-ui-4/#typedef-cursor-url-set
    /// prose: <url-set> is a limited version of image-set(), where the <image> sub-production is restricted to <url> only.
    /// for_parents: [cursor]
    UrlSet,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-url-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    UrlToken,
    ///
    /// href: https://drafts.csswg.org/css-variables-2/#typedef-var-args
    /// syntax: var( <declaration-value> , <declaration-value>? )
    VarArgs,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#typedef-shape-vertical-line-command
    /// syntax: vline [ to [ <length-percentage> | top | center | bottom | y-start | y-end ] | by <length-percentage> ]
    /// for_parents: [shape()]
    VerticalLineCommand,
    ///
    /// href: https://drafts.csswg.org/css-box-4/#typedef-visual-box
    /// syntax: content-box | padding-box | border-box
    VisualBox,
    ///
    /// href: https://drafts.csswg.org/css-speech-1/#typedef-voice-family-voice-family-name
    /// prose: Values are specific voice instances (e.g., Mike, comedian, mary, carlos2, "valley girl"). Like font-family names, voice names must either be given quoted as strings, or unquoted as a sequence of one or more CSS identifiers. If a sequence of identifiers is given as a voice name, the computed value is the name converted to a string by joining all the identifiers in the sequence by single spaces. Voice names that happen to be the same as the gender keywords (male, female and neutral) or that happen to match the CSS-wide keywords or preserve must be quoted to disambiguate with these keywords. The keyword default is reserved for future use and must also be quoted when used as voice names. It is recommended to quote voice names that contain white space, digits, or punctuation characters other than hyphens—​even if these voice names are valid in unquoted form—​in order to improve code clarity. For example: voice-family: "john doe", "Henry the-8th";
    /// for_parents: [voice-family]
    VoiceFamilyName,
    ///
    /// href: https://drafts.csswg.org/css-color-6/#typedef-wcag2
    /// syntax: wcag2 | wcag2([<number> | [ aa | aaa ] && large? ])
    Wcag2,
    ///
    /// href: https://drafts.csswg.org/css-syntax-3/#typedef-whitespace-token
    /// prose: The output of tokenization step is a stream of zero or more of the following tokens: <ident-token>, <function-token>, <at-keyword-token>, <hash-token>, <string-token>, <bad-string-token>, <url-token>, <bad-url-token>, <delim-token>, <number-token>, <percentage-token>, <dimension-token>, <unicode-range-token>, <whitespace-token>, <CDO-token>, <CDC-token>, <colon-token>, <semicolon-token>, <comma-token>, <[-token>, <]-token>, <(-token>, <)-token>, <{-token>, and <}-token>.
    WhitespaceToken,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#whole-value
    WholeValue,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#typedef-wq-name
    /// syntax: <ns-prefix>? <ident-token>
    WqName,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#typedef-xyz-params
    /// syntax: <xyz-space> [ <number> | <percentage> | none ]{3}
    XyzParams,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#typedef-xyz-space
    /// syntax: xyz | xyz-d50 | xyz-d65
    XyzSpace,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#zero-value
    /// prose: The value <zero> represents a literal number with the value 0. Expressions that merely evaluate to a <number> with the value 0 (for example, calc(0)) do not match <zero>; only literal <number-token>s do.
    Zero
}

impl CssType {
    pub fn name(self) -> &'static str {
        match self {
            CssType::OpenParenToken => "(-token",
            CssType::CloseParenToken => ")-token",
            CssType::OpenSquareToken => "[-token",
            CssType::CloseSquareToken => "]-token",
            CssType::OpenCurlyToken => "{-token",
            CssType::CloseCurlyToken => "}-token",
            CssType::AbsoluteSize => "absolute-size",
            CssType::Age => "age",
            CssType::AlphaValue => "alpha-value",
            CssType::Anb => "an+b",
            CssType::AnchorName => "anchor-name",
            CssType::AnchorSide => "anchor-side",
            CssType::AnchorSize => "anchor-size",
            CssType::AnchoredInParens => "anchored-in-parens",
            CssType::AnchoredQuery => "anchored-query",
            CssType::Angle => "angle",
            CssType::AnglePercentage => "angle-percentage",
            CssType::AngularColorHint => "angular-color-hint",
            CssType::AngularColorStop => "angular-color-stop",
            CssType::AngularColorStopList => "angular-color-stop-list",
            CssType::AnimateableFeature => "animateable-feature",
            CssType::AnimationAction => "animation-action",
            CssType::AnyValue => "any-value",
            CssType::ArcCommand => "arc-command",
            CssType::ArcSize => "arc-size",
            CssType::ArcSweep => "arc-sweep",
            CssType::AtKeywordToken => "at-keyword-token",
            CssType::AtRuleList => "at-rule-list",
            CssType::Attachment => "attachment",
            CssType::AttrArgs => "attr-args",
            CssType::AttrMatcher => "attr-matcher",
            CssType::AttrModifier => "attr-modifier",
            CssType::AttrName => "attr-name",
            CssType::AttrType => "attr-type",
            CssType::AttrUnit => "attr-unit",
            CssType::AttributeSelector => "attribute-selector",
            CssType::AutoLineColorList => "auto-line-color-list",
            CssType::AutoLineStyleList => "auto-line-style-list",
            CssType::AutoLineWidthList => "auto-line-width-list",
            CssType::AutoRepeat => "auto-repeat",
            CssType::AutoRepeatLineColor => "auto-repeat-line-color",
            CssType::AutoRepeatLineStyle => "auto-repeat-line-style",
            CssType::AutoRepeatLineWidth => "auto-repeat-line-width",
            CssType::AutoTrackList => "auto-track-list",
            CssType::Autospace => "autospace",
            CssType::Axis => "axis",
            CssType::BadStringToken => "bad-string-token",
            CssType::BadUrlToken => "bad-url-token",
            CssType::BaselineMetric => "baseline-metric",
            CssType::BaselinePosition => "baseline-position",
            CssType::BasicShape => "basic-shape",
            CssType::BasicShapeRect => "basic-shape-rect",
            CssType::BgClip => "bg-clip",
            CssType::BgImage => "bg-image",
            CssType::BgLayer => "bg-layer",
            CssType::BgPosition => "bg-position",
            CssType::BgSize => "bg-size",
            CssType::BlendMode => "blend-mode",
            CssType::BlockContents => "block-contents",
            CssType::BooleanCondition => "boolean-condition",
            CssType::BooleanExpr => "boolean-expr",
            CssType::BorderRadius => "border-radius",
            CssType::BorderStyle => "border-style",
            CssType::BorderWidth => "border-width",
            CssType::Bottom => "bottom",
            CssType::Box => "box",
            CssType::CalcKeyword => "calc-keyword",
            CssType::CalcProduct => "calc-product",
            CssType::CalcSizeBasis => "calc-size-basis",
            CssType::CalcSum => "calc-sum",
            CssType::CalcValue => "calc-value",
            CssType::CdcToken => "CDC-token",
            CssType::CdoToken => "CDO-token",
            CssType::CfImage => "cf-image",
            CssType::ClassSelector => "class-selector",
            CssType::ClipSource => "clip-source",
            CssType::CmykComponent => "cmyk-component",
            CssType::ColonToken => "colon-token",
            CssType::Color => "color",
            CssType::ColorBase => "color-base",
            CssType::ColorFontTech => "color-font-tech",
            CssType::ColorFunction => "color-function",
            CssType::ColorInterpolationMethod => "color-interpolation-method",
            CssType::ColorSpace => "color-space",
            CssType::ColorStop => "color-stop",
            CssType::ColorStopAngle => "color-stop-angle",
            CssType::ColorStopLength => "color-stop-length",
            CssType::ColorStopList => "color-stop-list",
            CssType::ColorStripe => "color-stripe",
            CssType::ColorspaceParams => "colorspace-params",
            CssType::Combinator => "combinator",
            CssType::CommaToken => "comma-token",
            CssType::CommandEndPoint => "command-end-point",
            CssType::CommonLigValues => "common-lig-values",
            CssType::CompatAuto => "compat-auto",
            CssType::CompatSpecial => "compat-special",
            CssType::ComplexRealSelector => "complex-real-selector",
            CssType::ComplexRealSelectorList => "complex-real-selector-list",
            CssType::ComplexSelector => "complex-selector",
            CssType::ComplexSelectorList => "complex-selector-list",
            CssType::ComplexSelectorUnit => "complex-selector-unit",
            CssType::CompositeMode => "composite-mode",
            CssType::CompositingOperator => "compositing-operator",
            CssType::CompoundSelector => "compound-selector",
            CssType::CompoundSelectorList => "compound-selector-list",
            CssType::ConicGradientSyntax => "conic-gradient-syntax",
            CssType::ContainerCondition => "container-condition",
            CssType::ContainerName => "container-name",
            CssType::ContainerQuery => "container-query",
            CssType::ContentDistribution => "content-distribution",
            CssType::ContentLevel => "content-level",
            CssType::ContentListContentList => "content-list",
            CssType::ContentListTypedefContentContentList => "content-list",
            CssType::ContentPosition => "content-position",
            CssType::ContentReplacement => "content-replacement",
            CssType::ContextualAltValues => "contextual-alt-values",
            CssType::ControlPoint => "control-point",
            CssType::CoordBox => "coord-box",
            CssType::CoordinatePair => "coordinate-pair",
            CssType::CornerShapeValue => "corner-shape-value",
            CssType::Counter => "counter",
            CssType::CounterName => "counter-name",
            CssType::CounterStyle => "counter-style",
            CssType::CounterStyleName => "counter-style-name",
            CssType::CrossOriginModifier => "cross-origin-modifier",
            CssType::CssType => "css-type",
            CssType::CubicBezierEasingFunction => "cubic-bezier-easing-function",
            CssType::CursorImage => "cursor-image",
            CssType::CursorPredefined => "cursor-predefined",
            CssType::CurveCommand => "curve-command",
            CssType::CustomArg => "custom-arg",
            CssType::CustomColorSpace => "custom-color-space",
            CssType::CustomIdent => "custom-ident",
            CssType::CustomParams => "custom-params",
            CssType::CustomPropertyName => "custom-property-name",
            CssType::CustomSelector => "custom-selector",
            CssType::Dasharray => "dasharray",
            CssType::DashedFunction => "dashed-function",
            CssType::DashedIdent => "dashed-ident",
            CssType::DashndashdigitIdent => "dashndashdigit-ident",
            CssType::Decibel => "decibel",
            CssType::Declaration => "declaration",
            CssType::DeclarationList => "declaration-list",
            CssType::DeclarationRuleList => "declaration-rule-list",
            CssType::DeclarationValue => "declaration-value",
            CssType::DefaultValue => "default-value",
            CssType::DelimToken => "delim-token",
            CssType::DeprecatedColor => "deprecated-color",
            CssType::Dimension => "dimension",
            CssType::DimensionToken => "dimension-token",
            CssType::DiscretionaryLigValues => "discretionary-lig-values",
            CssType::DisplayBox => "display-box",
            CssType::DisplayInside => "display-inside",
            CssType::DisplayInternal => "display-internal",
            CssType::DisplayLegacy => "display-legacy",
            CssType::DisplayListitem => "display-listitem",
            CssType::DisplayOutside => "display-outside",
            CssType::EasingFunction => "easing-function",
            CssType::EastAsianVariantValues => "east-asian-variant-values",
            CssType::EastAsianWidthValues => "east-asian-width-values",
            CssType::EnvArgs => "env-args",
            CssType::EofToken => "eof-token",
            CssType::EventTriggerEvent => "event-trigger-event",
            CssType::ExplicitTrackList => "explicit-track-list",
            CssType::ExtensionName => "extension-name",
            CssType::FamilyName => "family-name",
            CssType::FeatureTagValue => "feature-tag-value",
            CssType::FilterFunction => "filter-function",
            CssType::FilterValueList => "filter-value-list",
            CssType::FinalBgLayer => "final-bg-layer",
            CssType::FixedBreadth => "fixed-breadth",
            CssType::FixedRepeat => "fixed-repeat",
            CssType::FixedSize => "fixed-size",
            CssType::Flex => "flex",
            CssType::FontFamilyName => "font-family-name",
            CssType::FontFeatureIndex => "font-feature-index",
            CssType::FontFeatureValueName => "font-feature-value-name",
            CssType::FontFeatureValueType => "font-feature-value-type",
            CssType::FontFeaturesTech => "font-features-tech",
            CssType::FontFormat => "font-format",
            CssType::FontSrc => "font-src",
            CssType::FontSrcList => "font-src-list",
            CssType::FontTech => "font-tech",
            CssType::FontVariantCss2 => "font-variant-css2",
            CssType::FontWeightAbsolute => "font-weight-absolute",
            CssType::FontWidthCss3 => "font-width-css3",
            CssType::ForgivingSelectorList => "forgiving-selector-list",
            CssType::FormControlIdentifier => "form-control-identifier",
            CssType::Frequency => "frequency",
            CssType::FrequencyPercentage => "frequency-percentage",
            CssType::FunctionParameter => "function-parameter",
            CssType::FunctionToken => "function-token",
            CssType::GapAutoRepeatRule => "gap-auto-repeat-rule",
            CssType::GapAutoRuleList => "gap-auto-rule-list",
            CssType::GapRepeatRule => "gap-repeat-rule",
            CssType::GapRule => "gap-rule",
            CssType::GapRuleList => "gap-rule-list",
            CssType::GapRuleOrRepeat => "gap-rule-or-repeat",
            CssType::Gender => "gender",
            CssType::GeneralEnclosed => "general-enclosed",
            CssType::GenericFamily => "generic-family",
            CssType::GenericFontComplete => "generic-font-complete",
            CssType::GenericFontFamily => "generic-font-family",
            CssType::GenericFontIncomplete => "generic-font-incomplete",
            CssType::GenericFontScriptSpecific => "generic-font-script-specific",
            CssType::GenericVoice => "generic-voice",
            CssType::GeometryBox => "geometry-box",
            CssType::Gradient => "gradient",
            CssType::GridLine => "grid-line",
            CssType::HashToken => "hash-token",
            CssType::HexColor => "hex-color",
            CssType::HistoricalLigValues => "historical-lig-values",
            CssType::HorizontalLineCommand => "horizontal-line-command",
            CssType::Hue => "hue",
            CssType::HueInterpolationMethod => "hue-interpolation-method",
            CssType::Id => "id",
            CssType::IdSelector => "id-selector",
            CssType::Ident => "ident",
            CssType::IdentArg => "ident-arg",
            CssType::IdentToken => "ident-token",
            CssType::Identifier => "identifier",
            CssType::IfArgs => "if-args",
            CssType::IfArgsBranch => "if-args-branch",
            CssType::IfBranch => "if-branch",
            CssType::IfCondition => "if-condition",
            CssType::IfTest => "if-test",
            CssType::Image => "image",
            CssType::Image1d => "image-1D",
            CssType::ImageSetOption => "image-set-option",
            CssType::ImageSrc => "image-src",
            CssType::ImageTags => "image-tags",
            CssType::ImportConditions => "import-conditions",
            CssType::InflexibleBreadth => "inflexible-breadth",
            CssType::InheritArgs => "inherit-args",
            CssType::InitDescriptor => "init-descriptor",
            CssType::InitDescriptorName => "init-descriptor-name",
            CssType::InitDescriptors => "init-descriptors",
            CssType::InputPosition => "input-position",
            CssType::Integer => "integer",
            CssType::IntegrityModifier => "integrity-modifier",
            CssType::IsolationMode => "isolation-mode",
            CssType::KeyframeBlock => "keyframe-block",
            CssType::KeyframeSelector => "keyframe-selector",
            CssType::KeyframesName => "keyframes-name",
            CssType::LayerName => "layer-name",
            CssType::LayoutBox => "layout-box",
            CssType::LeaderType => "leader-type",
            CssType::Left => "left",
            CssType::LegacyBorderRadiusSyntax => "legacy-border-radius-syntax",
            CssType::LegacyDeviceCmykSyntax => "legacy-device-cmyk-syntax",
            CssType::LegacyHslSyntax => "legacy-hsl-syntax",
            CssType::LegacyHslaSyntax => "legacy-hsla-syntax",
            CssType::LegacyPseudoElementSelector => "legacy-pseudo-element-selector",
            CssType::LegacyRgbSyntax => "legacy-rgb-syntax",
            CssType::LegacyRgbaSyntax => "legacy-rgba-syntax",
            CssType::Length => "length",
            CssType::LengthPercentage => "length-percentage",
            CssType::Level => "level",
            CssType::LightDarkColor => "light-dark-color",
            CssType::LightDarkImage => "light-dark-image",
            CssType::LineColorList => "line-color-list",
            CssType::LineColorOrRepeat => "line-color-or-repeat",
            CssType::LineCommand => "line-command",
            CssType::LineNameList => "line-name-list",
            CssType::LineNames => "line-names",
            CssType::LineStyle => "line-style",
            CssType::LineStyleList => "line-style-list",
            CssType::LineStyleOrRepeat => "line-style-or-repeat",
            CssType::LineWidth => "line-width",
            CssType::LineWidthList => "line-width-list",
            CssType::LineWidthOrRepeat => "line-width-or-repeat",
            CssType::LinearColorHint => "linear-color-hint",
            CssType::LinearColorStop => "linear-color-stop",
            CssType::LinearEasingFunction => "linear-easing-function",
            CssType::LinearGradientSyntax => "linear-gradient-syntax",
            CssType::LinkCondition => "link-condition",
            CssType::LinkConditionBase => "link-condition-base",
            CssType::MarginWidth => "margin-width",
            CssType::MarkerRef => "marker-ref",
            CssType::MaskLayer => "mask-layer",
            CssType::MaskReference => "mask-reference",
            CssType::MaskSource => "mask-source",
            CssType::MaskingMode => "masking-mode",
            CssType::MediaAnd => "media-and",
            CssType::MediaCondition => "media-condition",
            CssType::MediaConditionWithoutOr => "media-condition-without-or",
            CssType::MediaFeature => "media-feature",
            CssType::MediaImportCondition => "media-import-condition",
            CssType::MediaInParens => "media-in-parens",
            CssType::MediaNot => "media-not",
            CssType::MediaOr => "media-or",
            CssType::MediaQuery => "media-query",
            CssType::MediaQueryList => "media-query-list",
            CssType::MediaType => "media-type",
            CssType::MfBoolean => "mf-boolean",
            CssType::MfComparison => "mf-comparison",
            CssType::MfEq => "mf-eq",
            CssType::MfGt => "mf-gt",
            CssType::MfLt => "mf-lt",
            CssType::MfName => "mf-name",
            CssType::MfPlain => "mf-plain",
            CssType::MfRange => "mf-range",
            CssType::MfValue => "mf-value",
            CssType::ModernDeviceCmykSyntax => "modern-device-cmyk-syntax",
            CssType::ModernHslSyntax => "modern-hsl-syntax",
            CssType::ModernHslaSyntax => "modern-hsla-syntax",
            CssType::ModernRgbSyntax => "modern-rgb-syntax",
            CssType::ModernRgbaSyntax => "modern-rgba-syntax",
            CssType::MoveCommand => "move-command",
            CssType::MqBoolean => "mq-boolean",
            CssType::NDimension => "n-dimension",
            CssType::NameRepeat => "name-repeat",
            CssType::NamedColor => "named-color",
            CssType::NamespacePrefix => "namespace-prefix",
            CssType::NavigationCondition => "navigation-condition",
            CssType::NavigationInParens => "navigation-in-parens",
            CssType::NavigationLocation => "navigation-location",
            CssType::NavigationLocationBetweenTest => "navigation-location-between-test",
            CssType::NavigationLocationKeyword => "navigation-location-keyword",
            CssType::NavigationLocationTest => "navigation-location-test",
            CssType::NavigationRelation => "navigation-relation",
            CssType::NavigationTest => "navigation-test",
            CssType::NavigationTypeKeyword => "navigation-type-keyword",
            CssType::NavigationTypeTest => "navigation-type-test",
            CssType::NdashDimension => "ndash-dimension",
            CssType::NdashdigitDimension => "ndashdigit-dimension",
            CssType::NdashdigitIdent => "ndashdigit-ident",
            CssType::NsPrefix => "ns-prefix",
            CssType::Number => "number",
            CssType::NumberOptionalNumber => "number-optional-number",
            CssType::NumberToken => "number-token",
            CssType::NumericFigureValues => "numeric-figure-values",
            CssType::NumericFractionValues => "numeric-fraction-values",
            CssType::NumericSpacingValues => "numeric-spacing-values",
            CssType::OffsetPath => "offset-path",
            CssType::OpacityValue => "opacity-value",
            CssType::OpentypeTag => "opentype-tag",
            CssType::OutlineLineStyle => "outline-line-style",
            CssType::OverflowPosition => "overflow-position",
            CssType::PaddingWidth => "padding-width",
            CssType::PageSelector => "page-selector",
            CssType::PageSelectorList => "page-selector-list",
            CssType::PageSize => "page-size",
            CssType::Paint => "paint",
            CssType::PaintBox => "paint-box",
            CssType::PaletteIdentifier => "palette-identifier",
            CssType::PatternDescriptor => "pattern-descriptor",
            CssType::PatternDescriptors => "pattern-descriptors",
            CssType::Percentage => "percentage",
            CssType::PercentageToken => "percentage-token",
            CssType::PointerAxis => "pointer-axis",
            CssType::PointerSource => "pointer-source",
            CssType::Points => "points",
            CssType::PolarColorSpace => "polar-color-space",
            CssType::Position => "position",
            CssType::PositionArea => "position-area",
            CssType::PositionAreaQuery => "position-area-query",
            CssType::PositionFour => "position-four",
            CssType::PositionOne => "position-one",
            CssType::PositionThree => "position-three",
            CssType::PositionTwo => "position-two",
            CssType::PredefinedRgb => "predefined-rgb",
            CssType::PredefinedRgbParams => "predefined-rgb-params",
            CssType::ProgressSource => "progress-source",
            CssType::PseudoClassSelector => "pseudo-class-selector",
            CssType::PseudoCompoundSelector => "pseudo-compound-selector",
            CssType::PseudoElementSelector => "pseudo-element-selector",
            CssType::PseudoPage => "pseudo-page",
            CssType::PtClassSelector => "pt-class-selector",
            CssType::PtNameAndClassSelector => "pt-name-and-class-selector",
            CssType::PtNameSelector => "pt-name-selector",
            CssType::QualifiedRuleList => "qualified-rule-list",
            CssType::QueryInParens => "query-in-parens",
            CssType::QuirkyColor => "quirky-color",
            CssType::QuirkyLength => "quirky-length",
            CssType::Quote => "quote",
            CssType::RadialExtent => "radial-extent",
            CssType::RadialGradientSyntax => "radial-gradient-syntax",
            CssType::RadialShape => "radial-shape",
            CssType::RadialSize => "radial-size",
            CssType::RandomCacheKey => "random-cache-key",
            CssType::RandomItemArgs => "random-item-args",
            CssType::RandomKey => "random-key",
            CssType::RandomUaIdent => "random-ua-ident",
            CssType::Ratio => "ratio",
            CssType::RaySize => "ray-size",
            CssType::RectangularColorSpace => "rectangular-color-space",
            CssType::ReferrerPolicyModifier => "referrer-policy-modifier",
            CssType::RelativeControlPoint => "relative-control-point",
            CssType::RelativeRealSelector => "relative-real-selector",
            CssType::RelativeRealSelectorList => "relative-real-selector-list",
            CssType::RelativeSelector => "relative-selector",
            CssType::RelativeSelectorList => "relative-selector-list",
            CssType::RelativeSize => "relative-size",
            CssType::RepeatLineColor => "repeat-line-color",
            CssType::RepeatLineStyle => "repeat-line-style",
            CssType::RepeatLineWidth => "repeat-line-width",
            CssType::RepeatStyle => "repeat-style",
            CssType::Repetition => "repetition",
            CssType::RequestUrlModifier => "request-url-modifier",
            CssType::Resolution => "resolution",
            CssType::ReversedCounterName => "reversed-counter-name",
            CssType::Right => "right",
            CssType::RoundingStrategy => "rounding-strategy",
            CssType::RouteName => "route-name",
            CssType::RuleList => "rule-list",
            CssType::SafePrintableInset => "safe-printable-inset",
            CssType::ScopeBoundaries => "scope-boundaries",
            CssType::ScopeEnd => "scope-end",
            CssType::ScopeStart => "scope-start",
            CssType::ScrollButtonDirection => "scroll-button-direction",
            CssType::ScrollStateFeature => "scroll-state-feature",
            CssType::ScrollStateInParens => "scroll-state-in-parens",
            CssType::ScrollStateQuery => "scroll-state-query",
            CssType::Scroller => "scroller",
            CssType::SegmentOptions => "segment-options",
            CssType::SelectorList => "selector-list",
            CssType::SelfPosition => "self-position",
            CssType::SemicolonToken => "semicolon-token",
            CssType::Semitones => "semitones",
            CssType::Shadow => "shadow",
            CssType::Shape => "shape",
            CssType::ShapeBox => "shape-box",
            CssType::ShapeCommand => "shape-command",
            CssType::SideOrCorner => "side-or-corner",
            CssType::SignedInteger => "signed-integer",
            CssType::SignlessInteger => "signless-integer",
            CssType::SimpleSelector => "simple-selector",
            CssType::SimpleSelectorList => "simple-selector-list",
            CssType::SingleAnimation => "single-animation",
            CssType::SingleAnimationComposition => "single-animation-composition",
            CssType::SingleAnimationDirection => "single-animation-direction",
            CssType::SingleAnimationFillMode => "single-animation-fill-mode",
            CssType::SingleAnimationIterationCount => "single-animation-iteration-count",
            CssType::SingleAnimationPlayState => "single-animation-play-state",
            CssType::SingleAnimationTimeline => "single-animation-timeline",
            CssType::SingleTransition => "single-transition",
            CssType::SingleTransitionProperty => "single-transition-property",
            CssType::SizeFeature => "size-feature",
            CssType::SizeKeyword => "size-keyword",
            CssType::SlashSeparatedBorderRadiusSyntax => "slash-separated-border-radius-syntax",
            CssType::SmoothCommand => "smooth-command",
            CssType::SourceSize => "source-size",
            CssType::SourceSizeList => "source-size-list",
            CssType::SourceSizeValue => "source-size-value",
            CssType::SpacingTrim => "spacing-trim",
            CssType::SpreadShadow => "spread-shadow",
            CssType::StepEasingFunction => "step-easing-function",
            CssType::StepPosition => "step-position",
            CssType::String => "string",
            CssType::StringToken => "string-token",
            CssType::StyleFeature => "style-feature",
            CssType::StyleFeatureBoolean => "style-feature-boolean",
            CssType::StyleFeatureName => "style-feature-name",
            CssType::StyleFeaturePlain => "style-feature-plain",
            CssType::StyleFeatureValue => "style-feature-value",
            CssType::StyleInParens => "style-in-parens",
            CssType::StyleQuery => "style-query",
            CssType::StyleRange => "style-range",
            CssType::StyleRangeValue => "style-range-value",
            CssType::SubclassSelector => "subclass-selector",
            CssType::SupportsAtRuleFn => "supports-at-rule-fn",
            CssType::SupportsCondition => "supports-condition",
            CssType::SupportsConditionName => "supports-condition-name",
            CssType::SupportsDecl => "supports-decl",
            CssType::SupportsEnvFn => "supports-env-fn",
            CssType::SupportsFeature => "supports-feature",
            CssType::SupportsFontFormatFn => "supports-font-format-fn",
            CssType::SupportsFontTechFn => "supports-font-tech-fn",
            CssType::SupportsImportCondition => "supports-import-condition",
            CssType::SupportsInParens => "supports-in-parens",
            CssType::SupportsNamedFeatureFn => "supports-named-feature-fn",
            CssType::SupportsSelectorFn => "supports-selector-fn",
            CssType::SvgPaint => "svg-paint",
            CssType::Symbol => "symbol",
            CssType::SymbolsType => "symbols-type",
            CssType::Syntax => "syntax",
            CssType::SyntaxCombinator => "syntax-combinator",
            CssType::SyntaxComponent => "syntax-component",
            CssType::SyntaxMultiplier => "syntax-multiplier",
            CssType::SyntaxSingleComponent => "syntax-single-component",
            CssType::SyntaxString => "syntax-string",
            CssType::SyntaxTypeName => "syntax-type-name",
            CssType::SystemColor => "system-color",
            CssType::SystemFontFamilyName => "system-font-family-name",
            CssType::Target => "target",
            CssType::TargetContrast => "target-contrast",
            CssType::TargetName => "target-name",
            CssType::TextEdge => "text-edge",
            CssType::Time => "time",
            CssType::TimePercentage => "time-percentage",
            CssType::TimelineRangeCenterSubject => "timeline-range-center-subject",
            CssType::TimelineRangeName => "timeline-range-name",
            CssType::Top => "top",
            CssType::TrackBreadth => "track-breadth",
            CssType::TrackList => "track-list",
            CssType::TrackRepeat => "track-repeat",
            CssType::TrackSize => "track-size",
            CssType::TransformFunction => "transform-function",
            CssType::TransformList => "transform-list",
            CssType::TransitionBehaviorValue => "transition-behavior-value",
            CssType::TrySize => "try-size",
            CssType::TryTactic => "try-tactic",
            CssType::Type => "type",
            CssType::TypeSelector => "type-selector",
            CssType::UnicodeRangeToken => "unicode-range-token",
            CssType::Uri => "uri",
            CssType::Url => "url",
            CssType::UrlModifier => "url-modifier",
            CssType::UrlSet => "url-set",
            CssType::UrlToken => "url-token",
            CssType::VarArgs => "var-args",
            CssType::VerticalLineCommand => "vertical-line-command",
            CssType::VisualBox => "visual-box",
            CssType::VoiceFamilyName => "voice-family-name",
            CssType::Wcag2 => "wcag2",
            CssType::WhitespaceToken => "whitespace-token",
            CssType::WholeValue => "whole-value",
            CssType::WqName => "wq-name",
            CssType::XyzParams => "xyz-params",
            CssType::XyzSpace => "xyz-space",
            CssType::Zero => "zero",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        const ENTRIES: &[(&str, CssType)] = &[
            ("(-token", CssType::OpenParenToken),
            (")-token", CssType::CloseParenToken),
            ("[-token", CssType::OpenSquareToken),
            ("]-token", CssType::CloseSquareToken),
            ("{-token", CssType::OpenCurlyToken),
            ("}-token", CssType::CloseCurlyToken),
            ("absolute-size", CssType::AbsoluteSize),
            ("age", CssType::Age),
            ("alpha-value", CssType::AlphaValue),
            ("an+b", CssType::Anb),
            ("anchor-name", CssType::AnchorName),
            ("anchor-side", CssType::AnchorSide),
            ("anchor-size", CssType::AnchorSize),
            ("anchored-in-parens", CssType::AnchoredInParens),
            ("anchored-query", CssType::AnchoredQuery),
            ("angle", CssType::Angle),
            ("angle-percentage", CssType::AnglePercentage),
            ("angular-color-hint", CssType::AngularColorHint),
            ("angular-color-stop", CssType::AngularColorStop),
            ("angular-color-stop-list", CssType::AngularColorStopList),
            ("animateable-feature", CssType::AnimateableFeature),
            ("animation-action", CssType::AnimationAction),
            ("any-value", CssType::AnyValue),
            ("arc-command", CssType::ArcCommand),
            ("arc-size", CssType::ArcSize),
            ("arc-sweep", CssType::ArcSweep),
            ("at-keyword-token", CssType::AtKeywordToken),
            ("at-rule-list", CssType::AtRuleList),
            ("attachment", CssType::Attachment),
            ("attr-args", CssType::AttrArgs),
            ("attr-matcher", CssType::AttrMatcher),
            ("attr-modifier", CssType::AttrModifier),
            ("attr-name", CssType::AttrName),
            ("attr-type", CssType::AttrType),
            ("attr-unit", CssType::AttrUnit),
            ("attribute-selector", CssType::AttributeSelector),
            ("auto-line-color-list", CssType::AutoLineColorList),
            ("auto-line-style-list", CssType::AutoLineStyleList),
            ("auto-line-width-list", CssType::AutoLineWidthList),
            ("auto-repeat", CssType::AutoRepeat),
            ("auto-repeat-line-color", CssType::AutoRepeatLineColor),
            ("auto-repeat-line-style", CssType::AutoRepeatLineStyle),
            ("auto-repeat-line-width", CssType::AutoRepeatLineWidth),
            ("auto-track-list", CssType::AutoTrackList),
            ("autospace", CssType::Autospace),
            ("axis", CssType::Axis),
            ("bad-string-token", CssType::BadStringToken),
            ("bad-url-token", CssType::BadUrlToken),
            ("baseline-metric", CssType::BaselineMetric),
            ("baseline-position", CssType::BaselinePosition),
            ("basic-shape", CssType::BasicShape),
            ("basic-shape-rect", CssType::BasicShapeRect),
            ("bg-clip", CssType::BgClip),
            ("bg-image", CssType::BgImage),
            ("bg-layer", CssType::BgLayer),
            ("bg-position", CssType::BgPosition),
            ("bg-size", CssType::BgSize),
            ("blend-mode", CssType::BlendMode),
            ("block-contents", CssType::BlockContents),
            ("boolean-condition", CssType::BooleanCondition),
            ("boolean-expr", CssType::BooleanExpr),
            ("border-radius", CssType::BorderRadius),
            ("border-style", CssType::BorderStyle),
            ("border-width", CssType::BorderWidth),
            ("bottom", CssType::Bottom),
            ("box", CssType::Box),
            ("calc-keyword", CssType::CalcKeyword),
            ("calc-product", CssType::CalcProduct),
            ("calc-size-basis", CssType::CalcSizeBasis),
            ("calc-sum", CssType::CalcSum),
            ("calc-value", CssType::CalcValue),
            ("CDC-token", CssType::CdcToken),
            ("CDO-token", CssType::CdoToken),
            ("cf-image", CssType::CfImage),
            ("class-selector", CssType::ClassSelector),
            ("clip-source", CssType::ClipSource),
            ("cmyk-component", CssType::CmykComponent),
            ("colon-token", CssType::ColonToken),
            ("color", CssType::Color),
            ("color-base", CssType::ColorBase),
            ("color-font-tech", CssType::ColorFontTech),
            ("color-function", CssType::ColorFunction),
            ("color-interpolation-method", CssType::ColorInterpolationMethod),
            ("color-space", CssType::ColorSpace),
            ("color-stop", CssType::ColorStop),
            ("color-stop-angle", CssType::ColorStopAngle),
            ("color-stop-length", CssType::ColorStopLength),
            ("color-stop-list", CssType::ColorStopList),
            ("color-stripe", CssType::ColorStripe),
            ("colorspace-params", CssType::ColorspaceParams),
            ("combinator", CssType::Combinator),
            ("comma-token", CssType::CommaToken),
            ("command-end-point", CssType::CommandEndPoint),
            ("common-lig-values", CssType::CommonLigValues),
            ("compat-auto", CssType::CompatAuto),
            ("compat-special", CssType::CompatSpecial),
            ("complex-real-selector", CssType::ComplexRealSelector),
            ("complex-real-selector-list", CssType::ComplexRealSelectorList),
            ("complex-selector", CssType::ComplexSelector),
            ("complex-selector-list", CssType::ComplexSelectorList),
            ("complex-selector-unit", CssType::ComplexSelectorUnit),
            ("composite-mode", CssType::CompositeMode),
            ("compositing-operator", CssType::CompositingOperator),
            ("compound-selector", CssType::CompoundSelector),
            ("compound-selector-list", CssType::CompoundSelectorList),
            ("conic-gradient-syntax", CssType::ConicGradientSyntax),
            ("container-condition", CssType::ContainerCondition),
            ("container-name", CssType::ContainerName),
            ("container-query", CssType::ContainerQuery),
            ("content-distribution", CssType::ContentDistribution),
            ("content-level", CssType::ContentLevel),
            ("content-list", CssType::ContentListTypedefContentContentList),
            ("content-list", CssType::ContentListContentList),
            ("content-position", CssType::ContentPosition),
            ("content-replacement", CssType::ContentReplacement),
            ("contextual-alt-values", CssType::ContextualAltValues),
            ("control-point", CssType::ControlPoint),
            ("coord-box", CssType::CoordBox),
            ("coordinate-pair", CssType::CoordinatePair),
            ("corner-shape-value", CssType::CornerShapeValue),
            ("counter", CssType::Counter),
            ("counter-name", CssType::CounterName),
            ("counter-style", CssType::CounterStyle),
            ("counter-style-name", CssType::CounterStyleName),
            ("cross-origin-modifier", CssType::CrossOriginModifier),
            ("css-type", CssType::CssType),
            ("cubic-bezier-easing-function", CssType::CubicBezierEasingFunction),
            ("cursor-image", CssType::CursorImage),
            ("cursor-predefined", CssType::CursorPredefined),
            ("curve-command", CssType::CurveCommand),
            ("custom-arg", CssType::CustomArg),
            ("custom-color-space", CssType::CustomColorSpace),
            ("custom-ident", CssType::CustomIdent),
            ("custom-params", CssType::CustomParams),
            ("custom-property-name", CssType::CustomPropertyName),
            ("custom-selector", CssType::CustomSelector),
            ("dasharray", CssType::Dasharray),
            ("dashed-function", CssType::DashedFunction),
            ("dashed-ident", CssType::DashedIdent),
            ("dashndashdigit-ident", CssType::DashndashdigitIdent),
            ("decibel", CssType::Decibel),
            ("declaration", CssType::Declaration),
            ("declaration-list", CssType::DeclarationList),
            ("declaration-rule-list", CssType::DeclarationRuleList),
            ("declaration-value", CssType::DeclarationValue),
            ("default-value", CssType::DefaultValue),
            ("delim-token", CssType::DelimToken),
            ("deprecated-color", CssType::DeprecatedColor),
            ("dimension", CssType::Dimension),
            ("dimension-token", CssType::DimensionToken),
            ("discretionary-lig-values", CssType::DiscretionaryLigValues),
            ("display-box", CssType::DisplayBox),
            ("display-inside", CssType::DisplayInside),
            ("display-internal", CssType::DisplayInternal),
            ("display-legacy", CssType::DisplayLegacy),
            ("display-listitem", CssType::DisplayListitem),
            ("display-outside", CssType::DisplayOutside),
            ("easing-function", CssType::EasingFunction),
            ("east-asian-variant-values", CssType::EastAsianVariantValues),
            ("east-asian-width-values", CssType::EastAsianWidthValues),
            ("env-args", CssType::EnvArgs),
            ("eof-token", CssType::EofToken),
            ("event-trigger-event", CssType::EventTriggerEvent),
            ("explicit-track-list", CssType::ExplicitTrackList),
            ("extension-name", CssType::ExtensionName),
            ("family-name", CssType::FamilyName),
            ("feature-tag-value", CssType::FeatureTagValue),
            ("filter-function", CssType::FilterFunction),
            ("filter-value-list", CssType::FilterValueList),
            ("final-bg-layer", CssType::FinalBgLayer),
            ("fixed-breadth", CssType::FixedBreadth),
            ("fixed-repeat", CssType::FixedRepeat),
            ("fixed-size", CssType::FixedSize),
            ("flex", CssType::Flex),
            ("font-family-name", CssType::FontFamilyName),
            ("font-feature-index", CssType::FontFeatureIndex),
            ("font-feature-value-name", CssType::FontFeatureValueName),
            ("font-feature-value-type", CssType::FontFeatureValueType),
            ("font-features-tech", CssType::FontFeaturesTech),
            ("font-format", CssType::FontFormat),
            ("font-src", CssType::FontSrc),
            ("font-src-list", CssType::FontSrcList),
            ("font-tech", CssType::FontTech),
            ("font-variant-css2", CssType::FontVariantCss2),
            ("font-weight-absolute", CssType::FontWeightAbsolute),
            ("font-width-css3", CssType::FontWidthCss3),
            ("forgiving-selector-list", CssType::ForgivingSelectorList),
            ("form-control-identifier", CssType::FormControlIdentifier),
            ("frequency", CssType::Frequency),
            ("frequency-percentage", CssType::FrequencyPercentage),
            ("function-parameter", CssType::FunctionParameter),
            ("function-token", CssType::FunctionToken),
            ("gap-auto-repeat-rule", CssType::GapAutoRepeatRule),
            ("gap-auto-rule-list", CssType::GapAutoRuleList),
            ("gap-repeat-rule", CssType::GapRepeatRule),
            ("gap-rule", CssType::GapRule),
            ("gap-rule-list", CssType::GapRuleList),
            ("gap-rule-or-repeat", CssType::GapRuleOrRepeat),
            ("gender", CssType::Gender),
            ("general-enclosed", CssType::GeneralEnclosed),
            ("generic-family", CssType::GenericFamily),
            ("generic-font-complete", CssType::GenericFontComplete),
            ("generic-font-family", CssType::GenericFontFamily),
            ("generic-font-incomplete", CssType::GenericFontIncomplete),
            ("generic-font-script-specific", CssType::GenericFontScriptSpecific),
            ("generic-voice", CssType::GenericVoice),
            ("geometry-box", CssType::GeometryBox),
            ("gradient", CssType::Gradient),
            ("grid-line", CssType::GridLine),
            ("hash-token", CssType::HashToken),
            ("hex-color", CssType::HexColor),
            ("historical-lig-values", CssType::HistoricalLigValues),
            ("horizontal-line-command", CssType::HorizontalLineCommand),
            ("hue", CssType::Hue),
            ("hue-interpolation-method", CssType::HueInterpolationMethod),
            ("id", CssType::Id),
            ("id-selector", CssType::IdSelector),
            ("ident", CssType::Ident),
            ("ident-arg", CssType::IdentArg),
            ("ident-token", CssType::IdentToken),
            ("identifier", CssType::Identifier),
            ("if-args", CssType::IfArgs),
            ("if-args-branch", CssType::IfArgsBranch),
            ("if-branch", CssType::IfBranch),
            ("if-condition", CssType::IfCondition),
            ("if-test", CssType::IfTest),
            ("image", CssType::Image),
            ("image-1D", CssType::Image1d),
            ("image-set-option", CssType::ImageSetOption),
            ("image-src", CssType::ImageSrc),
            ("image-tags", CssType::ImageTags),
            ("import-conditions", CssType::ImportConditions),
            ("inflexible-breadth", CssType::InflexibleBreadth),
            ("inherit-args", CssType::InheritArgs),
            ("init-descriptor", CssType::InitDescriptor),
            ("init-descriptor-name", CssType::InitDescriptorName),
            ("init-descriptors", CssType::InitDescriptors),
            ("input-position", CssType::InputPosition),
            ("integer", CssType::Integer),
            ("integrity-modifier", CssType::IntegrityModifier),
            ("isolation-mode", CssType::IsolationMode),
            ("keyframe-block", CssType::KeyframeBlock),
            ("keyframe-selector", CssType::KeyframeSelector),
            ("keyframes-name", CssType::KeyframesName),
            ("layer-name", CssType::LayerName),
            ("layout-box", CssType::LayoutBox),
            ("leader-type", CssType::LeaderType),
            ("left", CssType::Left),
            ("legacy-border-radius-syntax", CssType::LegacyBorderRadiusSyntax),
            ("legacy-device-cmyk-syntax", CssType::LegacyDeviceCmykSyntax),
            ("legacy-hsl-syntax", CssType::LegacyHslSyntax),
            ("legacy-hsla-syntax", CssType::LegacyHslaSyntax),
            ("legacy-pseudo-element-selector", CssType::LegacyPseudoElementSelector),
            ("legacy-rgb-syntax", CssType::LegacyRgbSyntax),
            ("legacy-rgba-syntax", CssType::LegacyRgbaSyntax),
            ("length", CssType::Length),
            ("length-percentage", CssType::LengthPercentage),
            ("level", CssType::Level),
            ("light-dark-color", CssType::LightDarkColor),
            ("light-dark-image", CssType::LightDarkImage),
            ("line-color-list", CssType::LineColorList),
            ("line-color-or-repeat", CssType::LineColorOrRepeat),
            ("line-command", CssType::LineCommand),
            ("line-name-list", CssType::LineNameList),
            ("line-names", CssType::LineNames),
            ("line-style", CssType::LineStyle),
            ("line-style-list", CssType::LineStyleList),
            ("line-style-or-repeat", CssType::LineStyleOrRepeat),
            ("line-width", CssType::LineWidth),
            ("line-width-list", CssType::LineWidthList),
            ("line-width-or-repeat", CssType::LineWidthOrRepeat),
            ("linear-color-hint", CssType::LinearColorHint),
            ("linear-color-stop", CssType::LinearColorStop),
            ("linear-easing-function", CssType::LinearEasingFunction),
            ("linear-gradient-syntax", CssType::LinearGradientSyntax),
            ("link-condition", CssType::LinkCondition),
            ("link-condition-base", CssType::LinkConditionBase),
            ("margin-width", CssType::MarginWidth),
            ("marker-ref", CssType::MarkerRef),
            ("mask-layer", CssType::MaskLayer),
            ("mask-reference", CssType::MaskReference),
            ("mask-source", CssType::MaskSource),
            ("masking-mode", CssType::MaskingMode),
            ("media-and", CssType::MediaAnd),
            ("media-condition", CssType::MediaCondition),
            ("media-condition-without-or", CssType::MediaConditionWithoutOr),
            ("media-feature", CssType::MediaFeature),
            ("media-import-condition", CssType::MediaImportCondition),
            ("media-in-parens", CssType::MediaInParens),
            ("media-not", CssType::MediaNot),
            ("media-or", CssType::MediaOr),
            ("media-query", CssType::MediaQuery),
            ("media-query-list", CssType::MediaQueryList),
            ("media-type", CssType::MediaType),
            ("mf-boolean", CssType::MfBoolean),
            ("mf-comparison", CssType::MfComparison),
            ("mf-eq", CssType::MfEq),
            ("mf-gt", CssType::MfGt),
            ("mf-lt", CssType::MfLt),
            ("mf-name", CssType::MfName),
            ("mf-plain", CssType::MfPlain),
            ("mf-range", CssType::MfRange),
            ("mf-value", CssType::MfValue),
            ("modern-device-cmyk-syntax", CssType::ModernDeviceCmykSyntax),
            ("modern-hsl-syntax", CssType::ModernHslSyntax),
            ("modern-hsla-syntax", CssType::ModernHslaSyntax),
            ("modern-rgb-syntax", CssType::ModernRgbSyntax),
            ("modern-rgba-syntax", CssType::ModernRgbaSyntax),
            ("move-command", CssType::MoveCommand),
            ("mq-boolean", CssType::MqBoolean),
            ("n-dimension", CssType::NDimension),
            ("name-repeat", CssType::NameRepeat),
            ("named-color", CssType::NamedColor),
            ("namespace-prefix", CssType::NamespacePrefix),
            ("navigation-condition", CssType::NavigationCondition),
            ("navigation-in-parens", CssType::NavigationInParens),
            ("navigation-location", CssType::NavigationLocation),
            ("navigation-location-between-test", CssType::NavigationLocationBetweenTest),
            ("navigation-location-keyword", CssType::NavigationLocationKeyword),
            ("navigation-location-test", CssType::NavigationLocationTest),
            ("navigation-relation", CssType::NavigationRelation),
            ("navigation-test", CssType::NavigationTest),
            ("navigation-type-keyword", CssType::NavigationTypeKeyword),
            ("navigation-type-test", CssType::NavigationTypeTest),
            ("ndash-dimension", CssType::NdashDimension),
            ("ndashdigit-dimension", CssType::NdashdigitDimension),
            ("ndashdigit-ident", CssType::NdashdigitIdent),
            ("ns-prefix", CssType::NsPrefix),
            ("number", CssType::Number),
            ("number-optional-number", CssType::NumberOptionalNumber),
            ("number-token", CssType::NumberToken),
            ("numeric-figure-values", CssType::NumericFigureValues),
            ("numeric-fraction-values", CssType::NumericFractionValues),
            ("numeric-spacing-values", CssType::NumericSpacingValues),
            ("offset-path", CssType::OffsetPath),
            ("opacity-value", CssType::OpacityValue),
            ("opentype-tag", CssType::OpentypeTag),
            ("outline-line-style", CssType::OutlineLineStyle),
            ("overflow-position", CssType::OverflowPosition),
            ("padding-width", CssType::PaddingWidth),
            ("page-selector", CssType::PageSelector),
            ("page-selector-list", CssType::PageSelectorList),
            ("page-size", CssType::PageSize),
            ("paint", CssType::Paint),
            ("paint-box", CssType::PaintBox),
            ("palette-identifier", CssType::PaletteIdentifier),
            ("pattern-descriptor", CssType::PatternDescriptor),
            ("pattern-descriptors", CssType::PatternDescriptors),
            ("percentage", CssType::Percentage),
            ("percentage-token", CssType::PercentageToken),
            ("pointer-axis", CssType::PointerAxis),
            ("pointer-source", CssType::PointerSource),
            ("points", CssType::Points),
            ("polar-color-space", CssType::PolarColorSpace),
            ("position", CssType::Position),
            ("position-area", CssType::PositionArea),
            ("position-area-query", CssType::PositionAreaQuery),
            ("position-four", CssType::PositionFour),
            ("position-one", CssType::PositionOne),
            ("position-three", CssType::PositionThree),
            ("position-two", CssType::PositionTwo),
            ("predefined-rgb", CssType::PredefinedRgb),
            ("predefined-rgb-params", CssType::PredefinedRgbParams),
            ("progress-source", CssType::ProgressSource),
            ("pseudo-class-selector", CssType::PseudoClassSelector),
            ("pseudo-compound-selector", CssType::PseudoCompoundSelector),
            ("pseudo-element-selector", CssType::PseudoElementSelector),
            ("pseudo-page", CssType::PseudoPage),
            ("pt-class-selector", CssType::PtClassSelector),
            ("pt-name-and-class-selector", CssType::PtNameAndClassSelector),
            ("pt-name-selector", CssType::PtNameSelector),
            ("qualified-rule-list", CssType::QualifiedRuleList),
            ("query-in-parens", CssType::QueryInParens),
            ("quirky-color", CssType::QuirkyColor),
            ("quirky-length", CssType::QuirkyLength),
            ("quote", CssType::Quote),
            ("radial-extent", CssType::RadialExtent),
            ("radial-gradient-syntax", CssType::RadialGradientSyntax),
            ("radial-shape", CssType::RadialShape),
            ("radial-size", CssType::RadialSize),
            ("random-cache-key", CssType::RandomCacheKey),
            ("random-item-args", CssType::RandomItemArgs),
            ("random-key", CssType::RandomKey),
            ("random-ua-ident", CssType::RandomUaIdent),
            ("ratio", CssType::Ratio),
            ("ray-size", CssType::RaySize),
            ("rectangular-color-space", CssType::RectangularColorSpace),
            ("referrer-policy-modifier", CssType::ReferrerPolicyModifier),
            ("relative-control-point", CssType::RelativeControlPoint),
            ("relative-real-selector", CssType::RelativeRealSelector),
            ("relative-real-selector-list", CssType::RelativeRealSelectorList),
            ("relative-selector", CssType::RelativeSelector),
            ("relative-selector-list", CssType::RelativeSelectorList),
            ("relative-size", CssType::RelativeSize),
            ("repeat-line-color", CssType::RepeatLineColor),
            ("repeat-line-style", CssType::RepeatLineStyle),
            ("repeat-line-width", CssType::RepeatLineWidth),
            ("repeat-style", CssType::RepeatStyle),
            ("repetition", CssType::Repetition),
            ("request-url-modifier", CssType::RequestUrlModifier),
            ("resolution", CssType::Resolution),
            ("reversed-counter-name", CssType::ReversedCounterName),
            ("right", CssType::Right),
            ("rounding-strategy", CssType::RoundingStrategy),
            ("route-name", CssType::RouteName),
            ("rule-list", CssType::RuleList),
            ("safe-printable-inset", CssType::SafePrintableInset),
            ("scope-boundaries", CssType::ScopeBoundaries),
            ("scope-end", CssType::ScopeEnd),
            ("scope-start", CssType::ScopeStart),
            ("scroll-button-direction", CssType::ScrollButtonDirection),
            ("scroll-state-feature", CssType::ScrollStateFeature),
            ("scroll-state-in-parens", CssType::ScrollStateInParens),
            ("scroll-state-query", CssType::ScrollStateQuery),
            ("scroller", CssType::Scroller),
            ("segment-options", CssType::SegmentOptions),
            ("selector-list", CssType::SelectorList),
            ("self-position", CssType::SelfPosition),
            ("semicolon-token", CssType::SemicolonToken),
            ("semitones", CssType::Semitones),
            ("shadow", CssType::Shadow),
            ("shape", CssType::Shape),
            ("shape-box", CssType::ShapeBox),
            ("shape-command", CssType::ShapeCommand),
            ("side-or-corner", CssType::SideOrCorner),
            ("signed-integer", CssType::SignedInteger),
            ("signless-integer", CssType::SignlessInteger),
            ("simple-selector", CssType::SimpleSelector),
            ("simple-selector-list", CssType::SimpleSelectorList),
            ("single-animation", CssType::SingleAnimation),
            ("single-animation-composition", CssType::SingleAnimationComposition),
            ("single-animation-direction", CssType::SingleAnimationDirection),
            ("single-animation-fill-mode", CssType::SingleAnimationFillMode),
            ("single-animation-iteration-count", CssType::SingleAnimationIterationCount),
            ("single-animation-play-state", CssType::SingleAnimationPlayState),
            ("single-animation-timeline", CssType::SingleAnimationTimeline),
            ("single-transition", CssType::SingleTransition),
            ("single-transition-property", CssType::SingleTransitionProperty),
            ("size-feature", CssType::SizeFeature),
            ("size-keyword", CssType::SizeKeyword),
            ("slash-separated-border-radius-syntax", CssType::SlashSeparatedBorderRadiusSyntax),
            ("smooth-command", CssType::SmoothCommand),
            ("source-size", CssType::SourceSize),
            ("source-size-list", CssType::SourceSizeList),
            ("source-size-value", CssType::SourceSizeValue),
            ("spacing-trim", CssType::SpacingTrim),
            ("spread-shadow", CssType::SpreadShadow),
            ("step-easing-function", CssType::StepEasingFunction),
            ("step-position", CssType::StepPosition),
            ("string", CssType::String),
            ("string-token", CssType::StringToken),
            ("style-feature", CssType::StyleFeature),
            ("style-feature-boolean", CssType::StyleFeatureBoolean),
            ("style-feature-name", CssType::StyleFeatureName),
            ("style-feature-plain", CssType::StyleFeaturePlain),
            ("style-feature-value", CssType::StyleFeatureValue),
            ("style-in-parens", CssType::StyleInParens),
            ("style-query", CssType::StyleQuery),
            ("style-range", CssType::StyleRange),
            ("style-range-value", CssType::StyleRangeValue),
            ("subclass-selector", CssType::SubclassSelector),
            ("supports-at-rule-fn", CssType::SupportsAtRuleFn),
            ("supports-condition", CssType::SupportsCondition),
            ("supports-condition-name", CssType::SupportsConditionName),
            ("supports-decl", CssType::SupportsDecl),
            ("supports-env-fn", CssType::SupportsEnvFn),
            ("supports-feature", CssType::SupportsFeature),
            ("supports-font-format-fn", CssType::SupportsFontFormatFn),
            ("supports-font-tech-fn", CssType::SupportsFontTechFn),
            ("supports-import-condition", CssType::SupportsImportCondition),
            ("supports-in-parens", CssType::SupportsInParens),
            ("supports-named-feature-fn", CssType::SupportsNamedFeatureFn),
            ("supports-selector-fn", CssType::SupportsSelectorFn),
            ("svg-paint", CssType::SvgPaint),
            ("symbol", CssType::Symbol),
            ("symbols-type", CssType::SymbolsType),
            ("syntax", CssType::Syntax),
            ("syntax-combinator", CssType::SyntaxCombinator),
            ("syntax-component", CssType::SyntaxComponent),
            ("syntax-multiplier", CssType::SyntaxMultiplier),
            ("syntax-single-component", CssType::SyntaxSingleComponent),
            ("syntax-string", CssType::SyntaxString),
            ("syntax-type-name", CssType::SyntaxTypeName),
            ("system-color", CssType::SystemColor),
            ("system-font-family-name", CssType::SystemFontFamilyName),
            ("target", CssType::Target),
            ("target-contrast", CssType::TargetContrast),
            ("target-name", CssType::TargetName),
            ("text-edge", CssType::TextEdge),
            ("time", CssType::Time),
            ("time-percentage", CssType::TimePercentage),
            ("timeline-range-center-subject", CssType::TimelineRangeCenterSubject),
            ("timeline-range-name", CssType::TimelineRangeName),
            ("top", CssType::Top),
            ("track-breadth", CssType::TrackBreadth),
            ("track-list", CssType::TrackList),
            ("track-repeat", CssType::TrackRepeat),
            ("track-size", CssType::TrackSize),
            ("transform-function", CssType::TransformFunction),
            ("transform-list", CssType::TransformList),
            ("transition-behavior-value", CssType::TransitionBehaviorValue),
            ("try-size", CssType::TrySize),
            ("try-tactic", CssType::TryTactic),
            ("type", CssType::Type),
            ("type-selector", CssType::TypeSelector),
            ("unicode-range-token", CssType::UnicodeRangeToken),
            ("uri", CssType::Uri),
            ("url", CssType::Url),
            ("url-modifier", CssType::UrlModifier),
            ("url-set", CssType::UrlSet),
            ("url-token", CssType::UrlToken),
            ("var-args", CssType::VarArgs),
            ("vertical-line-command", CssType::VerticalLineCommand),
            ("visual-box", CssType::VisualBox),
            ("voice-family-name", CssType::VoiceFamilyName),
            ("wcag2", CssType::Wcag2),
            ("whitespace-token", CssType::WhitespaceToken),
            ("whole-value", CssType::WholeValue),
            ("wq-name", CssType::WqName),
            ("xyz-params", CssType::XyzParams),
            ("xyz-space", CssType::XyzSpace),
            ("zero", CssType::Zero),
        ];
        match ENTRIES.binary_search_by_key(&name, |(n, _)| n) {
            Ok(i) => Some(ENTRIES[i].1),
            Err(_) => None,
        }
    }
}
