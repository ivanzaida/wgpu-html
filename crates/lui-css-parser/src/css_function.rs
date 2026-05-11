// Auto-generated from functions.json. DO NOT EDIT.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CssFunction {
    ///
    /// href: https://drafts.csswg.org/css-images-4/#funcdef--webkit-image-set
    /// prose: Implementations must accept -webkit-image-set() as a parse-time alias of image-set(). (It’s a valid value, with identical arguments to image-set(), and is turned into image-set() during parsing.)
    WebkitImageSet,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-abs
    /// syntax: abs( <calc-sum> )
    Abs,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-acos
    /// syntax: acos( <calc-sum> )
    Acos,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#funcdef-alpha
    /// syntax: alpha([from <color>] [ / [<alpha-value> | none] ]? )
    Alpha,
    ///
    /// href: https://drafts.csswg.org/css-anchor-position-1/#funcdef-anchor-size
    /// syntax: anchor-size( [ <anchor-name> || <anchor-size> ]? , <length-percentage>? )
    /// prose: An absolutely positioned box can use the anchor-size() function in its sizing properties to refer to the size of one or more anchor boxes. The anchor-size() function resolves to a <length>. It is only allowed in the accepted @position-try properties (and is otherwise invalid).
    AnchorSize,
    ///
    /// href: https://drafts.csswg.org/css-anchor-position-1/#funcdef-anchor
    /// syntax: anchor( <anchor-name>? && <anchor-side>, <length-percentage>? )
    /// prose: An absolutely positioned box can use the anchor() function as a value in its inset properties to refer to the position of one or more anchor boxes. The anchor() function resolves to a <length>. It is only allowed in the inset properties (and is otherwise invalid).
    Anchor,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-asin
    /// syntax: asin( <calc-sum> )
    Asin,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-atan
    /// syntax: atan( <calc-sum> )
    Atan,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-atan2
    /// syntax: atan2( <calc-sum>, <calc-sum> )
    Atan2,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-attr
    /// syntax: attr( <attr-name> <attr-type>? , <declaration-value>?)
    /// prose: The attr() function substitutes the value of an attribute on an element into a property, similar to how the var() function substitutes a custom property value into a function.
    Attr,
    ///
    /// href: https://drafts.csswg.org/filter-effects-1/#funcdef-filter-blur
    /// syntax: blur( <length>? )
    /// for_parents: [filter]
    Blur,
    ///
    /// href: https://drafts.csswg.org/filter-effects-1/#funcdef-filter-brightness
    /// syntax: brightness( [ <number> | <percentage> ]? )
    /// for_parents: [filter]
    Brightness,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-calc-interpolate
    /// syntax: calc-interpolate( [ <progress-source> && [ by <easing-function> ]? && <easing-function>? ] , <input-position>{1,2} : <calc-sum> , [ <easing-function>? , <input-position>{1,2} : <calc-sum> ]#? )
    /// prose: The calc-interpolate() interpolation notation represents an interpolated numeric or dimensional value. Like calc(), it is a math function, with the following syntax:
    CalcInterpolate,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-calc-mix
    /// syntax: calc-mix( [ <calc-sum> <percentage [0,100]>? ]# )
    /// prose: The calc-mix() mix notation represents a weighted average of numeric or dimensional value. Like calc(), it is a math function, with the following syntactic form:
    CalcMix,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-calc-size
    /// syntax: calc-size( <calc-size-basis>, <calc-sum> )
    CalcSize,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-calc
    /// syntax: calc( <calc-sum> )
    /// prose: The calc() function is a math function that allows basic arithmetic to be performed on numerical values, using addition (+), subtraction (-), multiplication (*), division (/), and parentheses.
    Calc,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#funcdef-basic-shape-circle
    /// syntax: circle( <radial-size>? [ at <position> ]? )
    /// for_parents: [<basic-shape>]
    Circle,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-clamp
    /// syntax: clamp( [ <calc-sum> | none ], <calc-sum>, [ <calc-sum> | none ] )
    /// prose: The clamp() function takes three calculations—​a minimum value, a central value, and a maximum value—​and represents its central calculation, clamped according to its min and max calculations, favoring the min calculation if it conflicts with the max. (That is, given clamp(MIN, VAL, MAX), it represents exactly the same value as max(MIN, min(VAL, MAX))).
    Clamp,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-color-interpolate
    /// syntax: color-interpolate( [ <progress-source> && [ by <easing-function> ]? && <easing-function>? && <color-interpolation-method>? ] , <input-position>{1,2} : <color>, [ [ <easing-function> || <color-interpolation-method> ]?, <input-position>{1,2} : <color> ]#? )
    /// prose: The color-interpolate() interpolation notation represents an interpolated <color> value, with the following syntax:
    ColorInterpolate,
    ///
    /// href: https://drafts.csswg.org/css-color-6/#funcdef-color-layers
    /// syntax: color-layers([ <blend-mode>, ]? <color># )
    /// prose: The color-layers() functional notation takes an optional <blend-mode> followed by a list of two or more <color> layers.
    ColorLayers,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#funcdef-color-mix
    /// syntax: color-mix( <color-interpolation-method>? , [ <color> && <percentage [0,100]>? ]#)
    ColorMix,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#funcdef-color
    /// syntax: color( [from <color>]? <colorspace-params> [ / [ <alpha-value> | none ] ]? )
    Color,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#funcdef-conic-gradient
    /// syntax: conic-gradient( [ <conic-gradient-syntax> ] )
    ConicGradient,
    ///
    /// href: https://drafts.csswg.org/css-content-3/#funcdef-content
    /// syntax: content( [ text | before | after | first-letter | marker ]? )
    Content,
    ///
    /// href: https://drafts.csswg.org/css-color-6/#funcdef-contrast-color
    /// syntax: contrast-color( [ [ <color> && [ tbd-fg | tbd-bg ] && <target-contrast>? ] | [ <color> && [ tbd-fg | tbd-bg ] && <target-contrast>, <color># ] ] )
    /// prose: The contrast-color() functional notation identifies a sufficiently contrasting color against a specified background or foreground color without requiring manual computation.
    ContrastColor,
    ///
    /// href: https://drafts.csswg.org/filter-effects-1/#funcdef-filter-contrast
    /// syntax: contrast( [ <number> | <percentage> ]? )
    /// for_parents: [filter]
    Contrast,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#funcdef-control-value
    /// syntax: control-value( <type>? )
    /// prose: The control-value() function computes to the current value of the form control it is on. If it is used on an element that is not a form control, it returns an empty string.
    ControlValue,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-cos
    /// syntax: cos( <calc-sum> )
    Cos,
    ///
    /// href: https://drafts.csswg.org/css-lists-3/#funcdef-counter
    /// syntax: counter( <counter-name>, <counter-style>? )
    /// prose: Counters have no visible effect by themselves, but their values can be used with the counter() and counters() functions, whose used values represent counter values as strings or images. They are defined as follows:
    Counter,
    ///
    /// href: https://drafts.csswg.org/css-lists-3/#funcdef-counters
    /// syntax: counters( <counter-name>, <string>, <counter-style>? )
    /// prose: Counters have no visible effect by themselves, but their values can be used with the counter() and counters() functions, whose used values represent counter values as strings or images. They are defined as follows:
    Counters,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#funcdef-cross-fade
    /// syntax: cross-fade( <cf-image># )
    CrossFade,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-request-url-modifier-cross-origin
    /// prose: <cross-origin-modifier> = cross-origin(anonymous | use-credentials)
    /// for_parents: [<request-url-modifier>]
    CrossOrigin,
    ///
    /// href: https://drafts.csswg.org/css-easing-2/#funcdef-cubic-bezier
    /// syntax: cubic-bezier( [ <number [0,1]>, <number> ]#{2} )
    CubicBezier,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#funcdef-device-cmyk
    /// syntax: <legacy-device-cmyk-syntax> | <modern-device-cmyk-syntax>
    DeviceCmyk,
    ///
    /// href: https://drafts.csswg.org/filter-effects-1/#funcdef-filter-drop-shadow
    /// syntax: drop-shadow( [ <color>? && <length>{2,3} ] )
    /// for_parents: [filter]
    DropShadow,
    ///
    /// href: https://drafts.csswg.org/css-color-hdr-1/#funcdef-dynamic-range-limit-mix
    /// syntax: dynamic-range-limit-mix( [ <'dynamic-range-limit'> && <percentage [0,100]> ]#{2,} )
    DynamicRangeLimitMix,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#funcdef-element
    /// syntax: element( <id-selector> )
    Element,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#funcdef-basic-shape-ellipse
    /// syntax: ellipse( <radial-size>? [ at <position> ]? )
    /// for_parents: [<basic-shape>]
    Ellipse,
    ///
    /// href: https://drafts.csswg.org/css-env-1/#funcdef-env
    /// syntax: env( <custom-ident> <integer [0,∞]>*, <declaration-value>? )
    Env,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-exp
    /// syntax: exp( <calc-sum> )
    Exp,
    ///
    /// href: https://drafts.csswg.org/css-overflow-4/#funcdef-text-overflow-fade
    /// syntax: fade( [ <length-percentage> ] )
    /// for_parents: [text-overflow]
    Fade,
    ///
    /// href: https://drafts.csswg.org/filter-effects-1/#funcdef-filter
    /// syntax: filter( [ <image> | <string> ], <filter-value-list> )
    Filter,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-first-valid
    /// syntax: first-valid( <declaration-value># )
    /// prose: The first-valid() functional notation inlines the fallback behavior intrinsic to parsing declarations. Unlike most notations, it can accept any valid or invalid syntax in its arguments, and represents the first value among its arguments that is supported (parsed as valid) by the UA as the whole value of the property it’s used in.
    FirstValid,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#funcdef-grid-template-columns-fit-content
    /// syntax: fit-content( <length-percentage> )
    /// for_parents: [grid-template-columns, grid-template-rows]
    FitContentFuncdefGridTemplateColumnsFitC,
    ///
    /// href: https://drafts.csswg.org/css-sizing-3/#funcdef-width-fit-content
    /// syntax: fit-content(<length-percentage [0,∞]>)
    /// for_parents: [height, max-height, max-width, min-height, min-width, width]
    FitContentFuncdefWidthFitContent,
    ///
    /// href: https://drafts.csswg.org/filter-effects-1/#funcdef-filter-grayscale
    /// syntax: grayscale( [ <number> | <percentage> ]? )
    /// for_parents: [filter]
    Grayscale,
    ///
    /// href: https://drafts.csswg.org/css-color-hdr-1/#funcdef-hdr-color
    /// syntax: color-hdr([ <color> && <number [0,∞]>? ]#{2})
    HdrColor,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#funcdef-hsl
    /// syntax: [ <legacy-hsl-syntax> | <modern-hsl-syntax> ]
    Hsl,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#funcdef-hsla
    /// syntax: [ <legacy-hsla-syntax> | <modern-hsla-syntax> ]
    Hsla,
    ///
    /// href: https://drafts.csswg.org/filter-effects-1/#funcdef-filter-hue-rotate
    /// syntax: hue-rotate( [ <angle> | <zero> ]? )
    /// for_parents: [filter]
    HueRotate,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#funcdef-hwb
    /// syntax: hwb([from <color>]? [<hue> | none] [<percentage> | <number> | none] [<percentage> | <number> | none] [ / [<alpha-value> | none] ]? )
    Hwb,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-hypot
    /// syntax: hypot( <calc-sum># )
    Hypot,
    ///
    /// href: https://drafts.csswg.org/css-color-hdr-1/#funcdef-ictcp
    /// syntax: ictcp([from <color>]? [<percentage> | <number> | none] [<percentage> | <number> | none] [<percentage> | <number> | none] [ / [<alpha-value> | none] ]? )
    Ictcp,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-ident
    /// syntax: ident( <ident-arg>+ )
    /// prose: The ident() function represents an <ident>, and can be used to manually construct <custom-ident> values from several parts.
    Ident,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-if
    /// syntax: if( [ <if-branch> ; ]* <if-branch> ;? )
    /// prose: The if() function is an arbitrary substitution function that represents conditional values. Its argument consists of an ordered semi-colon–separated list of statements, each consisting of a condition followed by a colon followed by a value. An if() function represents the value corresponding to the first condition in its argument list to be true; if no condition matches, then the if() function represents an empty token stream.
    If,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#funcdef-image-set
    /// syntax: image-set( <image-set-option># )
    /// prose: Delivering the most appropriate image resolution for a user’s device can be a difficult task. Ideally, images should be in the same resolution as the device they’re being viewed in, which can vary between users. However, other factors can factor into the decision of which image to send; for example, if the user is on a slow mobile connection, they may prefer to receive lower-res images rather than waiting for a large proper-res image to load. The image-set() function allows an author to ignore most of these issues, simply providing multiple resolutions of an image and letting the UA decide which is most appropriate in a given situation.
    ImageSet,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#funcdef-image
    /// syntax: image( <image-tags>? [ <image-src>? , <color>? ]! )
    Image,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-inherit
    /// syntax: inherit( <custom-property-name>, <declaration-value>? )
    /// prose: Like the inherit keyword, the inherit() functional notation resolves to the computed value of a property on the parent. Rather than resolving to the value of the same property, however, it resolves to a sequence of component values representing the computed value of the property specified as its first argument. Its second argument, if present, is used as a fallback in case the first argument resolves to the guaranteed-invalid value.
    Inherit,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#funcdef-basic-shape-inset
    /// syntax: inset( <length-percentage>{1,4} [ round <'border-radius'> ]? )
    /// for_parents: [<basic-shape>]
    Inset,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-request-url-modifier-integrity
    /// prose: <integrity-modifier> = integrity(<string>)
    /// for_parents: [<request-url-modifier>]
    Integrity,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-interpolate
    /// syntax: interpolate( [ <progress-source> && [ by <easing-function> ]? && <easing-function>? ] , <input-position>{1,2} : <whole-value>, [ <easing-function>?, <input-position>{1,2} : <whole-value> ]#? ) | interpolate( <progress-source> && [ by <easing-function> ]? && <easing-function>? of <keyframes-name> )
    /// prose: The interpolate() interpolation notation represents the interpolation of entire property values, which supports two alternative syntax patterns:
    Interpolate,
    ///
    /// href: https://drafts.csswg.org/filter-effects-1/#funcdef-filter-invert
    /// syntax: invert( [ <number> | <percentage> ]? )
    /// for_parents: [filter]
    Invert,
    ///
    /// href: https://drafts.csswg.org/css-color-hdr-1/#funcdef-jzazbz
    /// syntax: jzazbz([from <color>]? [<percentage> | <number> | none] [<percentage> | <number> | none] [<percentage> | <number> | none] [ / [<alpha-value> | none] ]? )
    Jzazbz,
    ///
    /// href: https://drafts.csswg.org/css-color-hdr-1/#funcdef-jzczhz
    /// syntax: jzczhz([from <color>]? [<percentage> | <number> | none] [<percentage> | <number> | none] [<hue> | none] [ / [<alpha-value> | none] ]? )
    Jzczhz,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#funcdef-lab
    /// syntax: lab([from <color>]? [<percentage> | <number> | none] [<percentage> | <number> | none] [<percentage> | <number> | none] [ / [<alpha-value> | none] ]? )
    Lab,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#funcdef-lch
    /// syntax: lch([from <color>]? [<percentage> | <number> | none] [<percentage> | <number> | none] [<hue> | none] [ / [<alpha-value> | none] ]? )
    Lch,
    ///
    /// href: https://drafts.csswg.org/css-content-3/#funcdef-leader
    /// syntax: leader( <leader-type> )
    Leader,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#funcdef-light-dark
    /// syntax: <light-dark-color> | <light-dark-image>
    LightDark,
    ///
    /// href: https://drafts.csswg.org/css-images-3/#funcdef-linear-gradient
    /// syntax: linear-gradient( [ <linear-gradient-syntax> ] )
    /// prose: The linear-gradient() notation specifies a linear gradient in CSS. Its syntax is as follows:
    LinearGradient,
    ///
    /// href: https://drafts.csswg.org/css-easing-2/#funcdef-linear
    /// syntax: linear( [ <number> && <percentage>{0,2} ]# )
    Linear,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-log
    /// syntax: log( <calc-sum>, <calc-sum>? )
    Log,
    ///
    /// href: https://drafts.csswg.org/css-transforms-1/#funcdef-transform-matrix
    /// syntax: matrix( <number>#{6} )
    /// for_parents: [transform]
    Matrix,
    ///
    /// href: https://drafts.csswg.org/css-transforms-2/#funcdef-matrix3d
    /// syntax: matrix3d( <number>#{16} )
    Matrix3d,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-max
    /// syntax: max( <calc-sum># )
    /// prose: The min() or max() functions contain one or more comma-separated calculations, and represent the smallest (most negative) or largest (most positive) of them, respectively.
    Max,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#funcdef-media
    /// syntax: media( [ <mf-plain> | <mf-boolean> | <mf-range> ] )
    Media,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-min
    /// syntax: min( <calc-sum># )
    /// prose: The min() or max() functions contain one or more comma-separated calculations, and represent the smallest (most negative) or largest (most positive) of them, respectively.
    Min,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#funcdef-grid-template-columns-minmax
    /// syntax: minmax(min, max)
    /// for_parents: [grid-template-columns, grid-template-rows]
    Minmax,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-mod
    /// syntax: mod( <calc-sum>, <calc-sum> )
    Mod,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#funcdef-oklab
    /// syntax: oklab([from <color>]? [<percentage> | <number> | none] [<percentage> | <number> | none] [<percentage> | <number> | none] [ / [<alpha-value> | none] ]? )
    Oklab,
    ///
    /// href: https://drafts.csswg.org/css-color-5/#funcdef-oklch
    /// syntax: oklch([from <color>]? [<percentage> | <number> | none] [<percentage> | <number> | none] [<hue> | none] [ / [<alpha-value> | none] ]? )
    Oklch,
    ///
    /// href: https://drafts.csswg.org/filter-effects-1/#funcdef-filter-opacity
    /// syntax: opacity( [ <number> | <percentage> ]? )
    /// for_parents: [filter]
    Opacity,
    ///
    /// href: https://drafts.css-houdini.org/css-paint-api-1/#funcdef-paint
    /// syntax: paint( <ident>, <declaration-value>? )
    Paint,
    ///
    /// href: https://drafts.csswg.org/css-fonts-4/#funcdef-palette-mix
    /// syntax: palette-mix(<color-interpolation-method> , [ [normal | light | dark | <palette-identifier> | <palette-mix()> ] && <percentage [0,100]>? ]#{2})
    /// prose: With the palette-mix() function defined as follows:
    PaletteMix,
    ///
    /// href: https://drafts.csswg.org/css-link-params-1/#funcdef-param
    /// syntax: param( <dashed-ident> , <declaration-value>? )
    /// prose: The param() function specifies a link parameter, with a key of the <dashed-ident>, and a value of the <declaration-value>?. (If the <declaration-value> is omitted, it represents an empty value.) It has the syntax:
    Param,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#funcdef-basic-shape-path
    /// syntax: path( <'fill-rule'>? , <string> )
    /// for_parents: [<basic-shape>]
    Path,
    ///
    /// href: https://drafts.csswg.org/css-transforms-2/#funcdef-perspective
    /// syntax: perspective( [ <length [0,∞]> | none ] )
    Perspective,
    ///
    /// href: https://drafts.csswg.org/pointer-animations-1/#funcdef-pointer
    /// syntax: pointer( [ <pointer-source> || <pointer-axis> ]? )
    /// prose: The pointer() functional notation can be used as a <single-animation-timeline> value in animation-timeline and specifies a pointer progress timeline. Its syntax is
    Pointer,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#funcdef-basic-shape-polygon
    /// syntax: polygon( <'fill-rule'>? [ round <length> ]? , [<length-percentage> <length-percentage>]# )
    /// for_parents: [<basic-shape>]
    Polygon,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-pow
    /// syntax: pow( <calc-sum>, <calc-sum> )
    Pow,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-progress
    /// syntax: progress(<calc-sum>, <calc-sum>, <calc-sum>)
    /// prose: The progress() functional notation represents the proportional distance of a given value (the progress value) from one value (the progress start value) to another value (the progress end value), each represented as a calculation. It is a math function, and can be input into other calculations such as a math function or a mix notation.
    Progress,
    ///
    /// href: https://drafts.csswg.org/css-images-3/#funcdef-radial-gradient
    /// syntax: radial-gradient( [ <radial-gradient-syntax> ] )
    /// prose: The radial-gradient() notation specifies a radial gradient by indicating the center of the gradient (where the 0% ellipse will be) and the size and shape of the ending shape (the 100% ellipse). Color stops are given as a list, just as for linear-gradient(). Starting from the gradient center and progressing towards (and potentially beyond) the ending shape, uniformly-scaled concentric ellipses are drawn and colored according to the specified color stops.
    RadialGradient,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-random-item
    /// syntax: random-item( <random-key> , [ <declaration-value>? ]# )
    /// prose: The random-item() function resolves to a random item from among its list of items.
    RandomItem,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-random
    /// syntax: random( <random-key>? , <calc-sum>, <calc-sum>, <calc-sum>? )
    /// prose: The random() function is a math function that represents a random value between a minimum and maximum value, drawn from a uniform distribution, optionally limiting the possible values to a step between those limits:
    Random,
    ///
    /// href: https://drafts.csswg.org/motion-1/#funcdef-ray
    /// syntax: ray( <angle> && <ray-size>? && contain? && [at <position>]? )
    Ray,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#funcdef-basic-shape-rect
    /// syntax: rect( [ <length-percentage> | auto ]{4} [ round <'border-radius'> ]? )
    /// for_parents: [<basic-shape>]
    RectFuncdefBasicShapeRect,
    ///
    /// href: https://drafts.csswg.org/css-masking-1/#funcdef-clip-rect
    /// syntax: rect( <top>, <right>, <bottom>, <left> )
    /// for_parents: [clip]
    RectFuncdefClipRect,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-request-url-modifier-referrer-policy
    /// prose: <referrer-policy-modifier> = referrer-policy(no-referrer | no-referrer-when-downgrade | same-origin | origin | strict-origin | origin-when-cross-origin | strict-origin-when-cross-origin | unsafe-url)
    /// for_parents: [<request-url-modifier>]
    ReferrerPolicy,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-rem
    /// syntax: rem( <calc-sum>, <calc-sum> )
    Rem,
    ///
    /// href: https://drafts.csswg.org/css-gaps-1/#funcdef-repeat-line-color-repeat
    /// prose: Such a list may contain repeat() notations. Similar to CSS Grid Layout 1 § 7.2.3 Repeating Rows and Columns: the repeat() notation, these notations allow a series of gap decorations that exhibit a recurring pattern to be written in a more compact form.
    /// for_parents: [<auto-repeat-line-color>, <auto-repeat-line-style>, <auto-repeat-line-width>, <repeat-line-color>, <repeat-line-style>, <repeat-line-width>]
    RepeatFuncdefRepeatLineColorRepeat,
    ///
    /// href: https://drafts.csswg.org/css-grid-2/#funcdef-track-repeat-repeat
    /// prose: The repeat() notation represents a repeated fragment of the track list, allowing a large number of columns or rows that exhibit a recurring pattern to be written in a more compact form.
    /// for_parents: [<auto-repeat>, <fixed-repeat>, <track-repeat>]
    RepeatFuncdefTrackRepeatRepeat,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#funcdef-repeating-conic-gradient
    /// syntax: repeating-conic-gradient( [ <conic-gradient-syntax> ] )
    /// prose: In addition to linear-gradient(), radial-gradient(), and conic-gradient(), this specification defines repeating-linear-gradient(), repeating-radial-gradient(), and repeating-conic-gradient() values. These notations take the same values and are interpreted the same as their respective non-repeating siblings defined previously.
    RepeatingConicGradient,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#funcdef-repeating-linear-gradient
    /// syntax: repeating-linear-gradient( [ <linear-gradient-syntax> ] )
    /// prose: In addition to linear-gradient(), radial-gradient(), and conic-gradient(), this specification defines repeating-linear-gradient(), repeating-radial-gradient(), and repeating-conic-gradient() values. These notations take the same values and are interpreted the same as their respective non-repeating siblings defined previously.
    RepeatingLinearGradient,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#funcdef-repeating-radial-gradient
    /// syntax: repeating-radial-gradient( [ <radial-gradient-syntax> ] )
    /// prose: In addition to linear-gradient(), radial-gradient(), and conic-gradient(), this specification defines repeating-linear-gradient(), repeating-radial-gradient(), and repeating-conic-gradient() values. These notations take the same values and are interpreted the same as their respective non-repeating siblings defined previously.
    RepeatingRadialGradient,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#funcdef-rgb
    /// syntax: [ <legacy-rgb-syntax> | <modern-rgb-syntax> ]
    Rgb,
    ///
    /// href: https://drafts.csswg.org/css-color-4/#funcdef-rgba
    /// syntax: [ <legacy-rgba-syntax> | <modern-rgba-syntax> ]
    Rgba,
    ///
    /// href: https://drafts.csswg.org/css-transforms-1/#funcdef-transform-rotate
    /// syntax: rotate( [ <angle> | <zero> ] )
    /// for_parents: [transform]
    Rotate,
    ///
    /// href: https://drafts.csswg.org/css-transforms-2/#funcdef-rotate3d
    /// syntax: rotate3d( <number> , <number> , <number> , [ <angle> | <zero> ] )
    Rotate3d,
    ///
    /// href: https://drafts.csswg.org/css-transforms-2/#funcdef-rotatex
    /// syntax: rotateX( [ <angle> | <zero> ] )
    Rotatex,
    ///
    /// href: https://drafts.csswg.org/css-transforms-2/#funcdef-rotatey
    /// syntax: rotateY( [ <angle> | <zero> ] )
    Rotatey,
    ///
    /// href: https://drafts.csswg.org/css-transforms-2/#funcdef-rotatez
    /// syntax: rotateZ( [ <angle> | <zero> ] )
    Rotatez,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-round
    /// syntax: round( <rounding-strategy>?, <calc-sum>, <calc-sum>? )
    Round,
    ///
    /// href: https://drafts.csswg.org/css-gcpm-3/#funcdef-running
    /// syntax: running( <custom-ident> )
    Running,
    ///
    /// href: https://drafts.csswg.org/filter-effects-1/#funcdef-filter-saturate
    /// syntax: saturate( [ <number> | <percentage> ]? )
    /// for_parents: [filter]
    Saturate,
    ///
    /// href: https://drafts.csswg.org/css-transforms-2/#funcdef-scale
    /// syntax: scale( [ <number> | <percentage> ]#{1,2} )
    ScaleFuncdefScale,
    ///
    /// href: https://drafts.csswg.org/css-transforms-1/#funcdef-transform-scale
    /// syntax: scale( <number> , <number>? )
    /// for_parents: [transform]
    ScaleFuncdefTransformScale,
    ///
    /// href: https://drafts.csswg.org/css-transforms-2/#funcdef-scale3d
    /// syntax: scale3d( [ <number> | <percentage> ]#{3} )
    Scale3d,
    ///
    /// href: https://drafts.csswg.org/css-transforms-2/#funcdef-scalex
    /// syntax: scaleX( [ <number> | <percentage> ] )
    ScalexFuncdefScalex,
    ///
    /// href: https://drafts.csswg.org/css-transforms-1/#funcdef-transform-scalex
    /// syntax: scaleX( <number> )
    /// for_parents: [transform]
    ScalexFuncdefTransformScalex,
    ///
    /// href: https://drafts.csswg.org/css-transforms-2/#funcdef-scaley
    /// syntax: scaleY( [ <number> | <percentage> ] )
    ScaleyFuncdefScaley,
    ///
    /// href: https://drafts.csswg.org/css-transforms-1/#funcdef-transform-scaley
    /// syntax: scaleY( <number> )
    /// for_parents: [transform]
    ScaleyFuncdefTransformScaley,
    ///
    /// href: https://drafts.csswg.org/css-transforms-2/#funcdef-scalez
    /// syntax: scaleZ( [ <number> | <percentage> ] )
    Scalez,
    ///
    /// href: https://drafts.csswg.org/scroll-animations-1/#funcdef-scroll
    /// syntax: scroll( [ <scroller> || <axis> ]? )
    /// prose: The scroll() functional notation can be used as a <single-animation-timeline> value in animation-timeline and specifies a scroll progress timeline. Its syntax is
    Scroll,
    ///
    /// href: https://drafts.csswg.org/filter-effects-1/#funcdef-filter-sepia
    /// syntax: sepia( [ <number> | <percentage> ]? )
    /// for_parents: [filter]
    Sepia,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#funcdef-basic-shape-shape
    /// syntax: shape( <'fill-rule'>? from <position> , <shape-command># )
    /// for_parents: [<basic-shape>]
    Shape,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-sibling-count
    /// prose: The sibling-count() functional notation represents, as an <integer>, the total number of child elements in the parent of the element on which the notation is used.
    SiblingCount,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-sibling-index
    /// prose: The sibling-index() functional notation represents, as an <integer>, the index of the element on which the notation is used among its inclusive siblings. Like :nth-child(), sibling-index() is 1-indexed.
    SiblingIndex,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-sign
    /// syntax: sign( <calc-sum> )
    Sign,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-sin
    /// syntax: sin( <calc-sum> )
    Sin,
    ///
    /// href: https://drafts.csswg.org/css-transforms-1/#funcdef-transform-skew
    /// syntax: skew( [ <angle> | <zero> ] , [ <angle> | <zero> ]? )
    /// for_parents: [transform]
    Skew,
    ///
    /// href: https://drafts.csswg.org/css-transforms-1/#funcdef-transform-skewx
    /// syntax: skewX( [ <angle> | <zero> ] )
    /// for_parents: [transform]
    Skewx,
    ///
    /// href: https://drafts.csswg.org/css-transforms-1/#funcdef-transform-skewy
    /// syntax: skewY( [ <angle> | <zero> ] )
    /// for_parents: [transform]
    Skewy,
    ///
    /// href: https://drafts.csswg.org/css-page-floats-3/#funcdef-float-snap-block
    /// syntax: snap-block( <length> , [ start | end | near ]? )
    /// for_parents: [float]
    SnapBlock,
    ///
    /// href: https://drafts.csswg.org/css-page-floats-3/#funcdef-float-snap-inline
    /// syntax: snap-inline( <length> , [ left | right | near ]? )
    /// for_parents: [float]
    SnapInline,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-sqrt
    /// syntax: sqrt( <calc-sum> )
    Sqrt,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-src
    /// syntax: src( <string> <url-modifier>* )
    /// prose: The <url> type, written with the url() and src() functions, represents a URL, which is a pointer to a resource.
    Src,
    ///
    /// href: https://drafts.csswg.org/css-easing-2/#funcdef-steps
    /// syntax: steps( <integer>, <step-position>?)
    Steps,
    ///
    /// href: https://drafts.csswg.org/css-content-3/#funcdef-string
    /// syntax: string( <custom-ident> , [ first | start | last | first-except ]? )
    String,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#funcdef-stripes
    /// syntax: stripes( <color-stripe># )
    /// prose: The stripes() function defines a 1D image as a comma-separated list of colored stripes, each placed end-to-end on the paint line in the order given.
    Stripes,
    ///
    /// href: https://drafts.csswg.org/css-borders-4/#funcdef-superellipse
    /// syntax: superellipse(<number> | infinity | -infinity)
    Superellipse,
    ///
    /// href: https://drafts.csswg.org/css-conditional-5/#funcdef-supports
    /// syntax: supports( <declaration> )
    Supports,
    ///
    /// href: https://drafts.csswg.org/css-counter-styles-3/#funcdef-symbols
    /// syntax: symbols( <symbols-type>? [ <string> | <image> ]+ )
    Symbols,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-tan
    /// syntax: tan( <calc-sum> )
    Tan,
    ///
    /// href: https://drafts.csswg.org/css-content-3/#funcdef-target-counter
    /// syntax: target-counter( [ <string> | <url> ] , <custom-ident> , <counter-style>? )
    TargetCounter,
    ///
    /// href: https://drafts.csswg.org/css-content-3/#funcdef-target-counters
    /// syntax: target-counters( [ <string> | <url> ] , <custom-ident> , <string> , <counter-style>? )
    TargetCounters,
    ///
    /// href: https://drafts.csswg.org/css-content-3/#target-text-function
    /// syntax: target-text( [ <string> | <url> ] , [ content | before | after | first-letter ]? )
    TargetText,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-toggle
    /// syntax: toggle( <whole-value># )
    /// prose: The toggle() expression allows descendant elements to cycle over a list of values instead of inheriting the same value.
    Toggle,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-transform-interpolate
    /// syntax: transform-interpolate( [ <progress-source> && [ by <easing-function> ]? && <easing-function>? ], <input-position>{1,2} : <transform-list>, [ <easing-function>?, <input-position>{1,2} : <transform-list> ]#? )
    /// prose: The transform-interpolate() interpolation notation represents an interpolated <transform-list>, with the following syntax:
    TransformInterpolate,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-transform-mix
    /// syntax: transform-mix( [ <transform-list> && <percentage [0,100]> ]# )
    /// prose: The transform-mix() mix notation represents a weighted average of <transform-list>, with the following syntactic form:
    TransformMix,
    ///
    /// href: https://drafts.csswg.org/css-transforms-1/#funcdef-transform-translate
    /// syntax: translate( <length-percentage> , <length-percentage>? )
    /// for_parents: [transform]
    Translate,
    ///
    /// href: https://drafts.csswg.org/css-transforms-2/#funcdef-translate3d
    /// syntax: translate3d( <length-percentage> , <length-percentage> , <length> )
    Translate3d,
    ///
    /// href: https://drafts.csswg.org/css-transforms-1/#funcdef-transform-translatex
    /// syntax: translateX( <length-percentage> )
    /// for_parents: [transform]
    Translatex,
    ///
    /// href: https://drafts.csswg.org/css-transforms-1/#funcdef-transform-translatey
    /// syntax: translateY( <length-percentage> )
    /// for_parents: [transform]
    Translatey,
    ///
    /// href: https://drafts.csswg.org/css-transforms-2/#funcdef-translatez
    /// syntax: translateZ( <length> )
    Translatez,
    ///
    /// href: https://drafts.csswg.org/css-mixins-1/#funcdef-function-type
    /// syntax: type( <syntax> )
    /// for_parents: [@function]
    TypeFuncdefFunctionType,
    ///
    /// href: https://drafts.csswg.org/css-values-5/#funcdef-attr-type
    /// syntax: type( <syntax> )
    /// for_parents: [attr()]
    TypeFuncdefAttrType,
    ///
    /// href: https://drafts.csswg.org/css-images-4/#funcdef-image-set-type
    /// syntax: type( <string> )
    /// for_parents: [image-set()]
    TypeFuncdefImageSetType,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#funcdef-url-pattern
    /// syntax: url-pattern( <string> )
    /// prose: The url-pattern() function represents a URL pattern, which can be used to match URLs.
    UrlPattern,
    ///
    /// href: https://drafts.csswg.org/css-values-4/#funcdef-url
    /// syntax: url( <string> <url-modifier>* ) | <url-token>
    /// prose: The <url> type, written with the url() and src() functions, represents a URL, which is a pointer to a resource.
    Url,
    ///
    /// href: https://drafts.csswg.org/css-variables-2/#funcdef-var
    /// syntax: var( <custom-property-name> , <declaration-value>? )
    Var,
    ///
    /// href: https://drafts.csswg.org/scroll-animations-1/#funcdef-view
    /// syntax: view( [ <axis> || <'view-timeline-inset'> ]? )
    /// prose: The view() functional notation can be used as a <single-animation-timeline> value in animation-timeline and specifies a view progress timeline in reference to the nearest ancestor scroll container. Its syntax is
    View,
    ///
    /// href: https://drafts.csswg.org/css-color-6/#funcdef-contrast-color-wcag2
    /// prose: The wcag2 keyword and wcag2() functional notations indicate use of the [WCAG21] luminance contrast algorithm. Their syntax is:
    /// for_parents: [contrast-color()]
    Wcag2,
    ///
    /// href: https://drafts.csswg.org/css-shapes-1/#funcdef-basic-shape-xywh
    /// syntax: xywh( <length-percentage>{2} <length-percentage [0,∞]>{2} [ round <'border-radius'> ]? )
    /// for_parents: [<basic-shape>]
    Xywh,
    Unknown(String),
}

impl CssFunction {
    pub fn name(&self) -> &'static str {
        match self {
            CssFunction::WebkitImageSet => "-webkit-image-set",
            CssFunction::Abs => "abs",
            CssFunction::Acos => "acos",
            CssFunction::Alpha => "alpha",
            CssFunction::AnchorSize => "anchor-size",
            CssFunction::Anchor => "anchor",
            CssFunction::Asin => "asin",
            CssFunction::Atan => "atan",
            CssFunction::Atan2 => "atan2",
            CssFunction::Attr => "attr",
            CssFunction::Blur => "blur",
            CssFunction::Brightness => "brightness",
            CssFunction::CalcInterpolate => "calc-interpolate",
            CssFunction::CalcMix => "calc-mix",
            CssFunction::CalcSize => "calc-size",
            CssFunction::Calc => "calc",
            CssFunction::Circle => "circle",
            CssFunction::Clamp => "clamp",
            CssFunction::ColorInterpolate => "color-interpolate",
            CssFunction::ColorLayers => "color-layers",
            CssFunction::ColorMix => "color-mix",
            CssFunction::Color => "color",
            CssFunction::ConicGradient => "conic-gradient",
            CssFunction::Content => "content",
            CssFunction::ContrastColor => "contrast-color",
            CssFunction::Contrast => "contrast",
            CssFunction::ControlValue => "control-value",
            CssFunction::Cos => "cos",
            CssFunction::Counter => "counter",
            CssFunction::Counters => "counters",
            CssFunction::CrossFade => "cross-fade",
            CssFunction::CrossOrigin => "cross-origin",
            CssFunction::CubicBezier => "cubic-bezier",
            CssFunction::DeviceCmyk => "device-cmyk",
            CssFunction::DropShadow => "drop-shadow",
            CssFunction::DynamicRangeLimitMix => "dynamic-range-limit-mix",
            CssFunction::Element => "element",
            CssFunction::Ellipse => "ellipse",
            CssFunction::Env => "env",
            CssFunction::Exp => "exp",
            CssFunction::Fade => "fade",
            CssFunction::Filter => "filter",
            CssFunction::FirstValid => "first-valid",
            CssFunction::FitContentFuncdefGridTemplateColumnsFitC => "fit-content",
            CssFunction::FitContentFuncdefWidthFitContent => "fit-content",
            CssFunction::Grayscale => "grayscale",
            CssFunction::HdrColor => "hdr-color",
            CssFunction::Hsl => "hsl",
            CssFunction::Hsla => "hsla",
            CssFunction::HueRotate => "hue-rotate",
            CssFunction::Hwb => "hwb",
            CssFunction::Hypot => "hypot",
            CssFunction::Ictcp => "ictcp",
            CssFunction::Ident => "ident",
            CssFunction::If => "if",
            CssFunction::ImageSet => "image-set",
            CssFunction::Image => "image",
            CssFunction::Inherit => "inherit",
            CssFunction::Inset => "inset",
            CssFunction::Integrity => "integrity",
            CssFunction::Interpolate => "interpolate",
            CssFunction::Invert => "invert",
            CssFunction::Jzazbz => "jzazbz",
            CssFunction::Jzczhz => "jzczhz",
            CssFunction::Lab => "lab",
            CssFunction::Lch => "lch",
            CssFunction::Leader => "leader",
            CssFunction::LightDark => "light-dark",
            CssFunction::LinearGradient => "linear-gradient",
            CssFunction::Linear => "linear",
            CssFunction::Log => "log",
            CssFunction::Matrix => "matrix",
            CssFunction::Matrix3d => "matrix3d",
            CssFunction::Max => "max",
            CssFunction::Media => "media",
            CssFunction::Min => "min",
            CssFunction::Minmax => "minmax",
            CssFunction::Mod => "mod",
            CssFunction::Oklab => "oklab",
            CssFunction::Oklch => "oklch",
            CssFunction::Opacity => "opacity",
            CssFunction::Paint => "paint",
            CssFunction::PaletteMix => "palette-mix",
            CssFunction::Param => "param",
            CssFunction::Path => "path",
            CssFunction::Perspective => "perspective",
            CssFunction::Pointer => "pointer",
            CssFunction::Polygon => "polygon",
            CssFunction::Pow => "pow",
            CssFunction::Progress => "progress",
            CssFunction::RadialGradient => "radial-gradient",
            CssFunction::RandomItem => "random-item",
            CssFunction::Random => "random",
            CssFunction::Ray => "ray",
            CssFunction::RectFuncdefBasicShapeRect => "rect",
            CssFunction::RectFuncdefClipRect => "rect",
            CssFunction::ReferrerPolicy => "referrer-policy",
            CssFunction::Rem => "rem",
            CssFunction::RepeatFuncdefRepeatLineColorRepeat => "repeat",
            CssFunction::RepeatFuncdefTrackRepeatRepeat => "repeat",
            CssFunction::RepeatingConicGradient => "repeating-conic-gradient",
            CssFunction::RepeatingLinearGradient => "repeating-linear-gradient",
            CssFunction::RepeatingRadialGradient => "repeating-radial-gradient",
            CssFunction::Rgb => "rgb",
            CssFunction::Rgba => "rgba",
            CssFunction::Rotate => "rotate",
            CssFunction::Rotate3d => "rotate3d",
            CssFunction::Rotatex => "rotateX",
            CssFunction::Rotatey => "rotateY",
            CssFunction::Rotatez => "rotateZ",
            CssFunction::Round => "round",
            CssFunction::Running => "running",
            CssFunction::Saturate => "saturate",
            CssFunction::ScaleFuncdefScale => "scale",
            CssFunction::ScaleFuncdefTransformScale => "scale",
            CssFunction::Scale3d => "scale3d",
            CssFunction::ScalexFuncdefScalex => "scaleX",
            CssFunction::ScalexFuncdefTransformScalex => "scaleX",
            CssFunction::ScaleyFuncdefScaley => "scaleY",
            CssFunction::ScaleyFuncdefTransformScaley => "scaleY",
            CssFunction::Scalez => "scaleZ",
            CssFunction::Scroll => "scroll",
            CssFunction::Sepia => "sepia",
            CssFunction::Shape => "shape",
            CssFunction::SiblingCount => "sibling-count",
            CssFunction::SiblingIndex => "sibling-index",
            CssFunction::Sign => "sign",
            CssFunction::Sin => "sin",
            CssFunction::Skew => "skew",
            CssFunction::Skewx => "skewX",
            CssFunction::Skewy => "skewY",
            CssFunction::SnapBlock => "snap-block",
            CssFunction::SnapInline => "snap-inline",
            CssFunction::Sqrt => "sqrt",
            CssFunction::Src => "src",
            CssFunction::Steps => "steps",
            CssFunction::String => "string",
            CssFunction::Stripes => "stripes",
            CssFunction::Superellipse => "superellipse",
            CssFunction::Supports => "supports",
            CssFunction::Symbols => "symbols",
            CssFunction::Tan => "tan",
            CssFunction::TargetCounter => "target-counter",
            CssFunction::TargetCounters => "target-counters",
            CssFunction::TargetText => "target-text",
            CssFunction::Toggle => "toggle",
            CssFunction::TransformInterpolate => "transform-interpolate",
            CssFunction::TransformMix => "transform-mix",
            CssFunction::Translate => "translate",
            CssFunction::Translate3d => "translate3d",
            CssFunction::Translatex => "translateX",
            CssFunction::Translatey => "translateY",
            CssFunction::Translatez => "translateZ",
            CssFunction::TypeFuncdefFunctionType => "type",
            CssFunction::TypeFuncdefAttrType => "type",
            CssFunction::TypeFuncdefImageSetType => "type",
            CssFunction::UrlPattern => "url-pattern",
            CssFunction::Url => "url",
            CssFunction::Var => "var",
            CssFunction::View => "view",
            CssFunction::Wcag2 => "wcag2",
            CssFunction::Xywh => "xywh",
        CssFunction::Unknown(_) => "",
}
    }

    pub fn from_name(name: &str) -> Self {
        const ENTRIES: &[(&str, CssFunction)] = &[
            ("-webkit-image-set", CssFunction::WebkitImageSet),
            ("abs", CssFunction::Abs),
            ("acos", CssFunction::Acos),
            ("alpha", CssFunction::Alpha),
            ("anchor", CssFunction::Anchor),
            ("anchor-size", CssFunction::AnchorSize),
            ("asin", CssFunction::Asin),
            ("atan", CssFunction::Atan),
            ("atan2", CssFunction::Atan2),
            ("attr", CssFunction::Attr),
            ("blur", CssFunction::Blur),
            ("brightness", CssFunction::Brightness),
            ("calc", CssFunction::Calc),
            ("calc-interpolate", CssFunction::CalcInterpolate),
            ("calc-mix", CssFunction::CalcMix),
            ("calc-size", CssFunction::CalcSize),
            ("circle", CssFunction::Circle),
            ("clamp", CssFunction::Clamp),
            ("color", CssFunction::Color),
            ("color-interpolate", CssFunction::ColorInterpolate),
            ("color-layers", CssFunction::ColorLayers),
            ("color-mix", CssFunction::ColorMix),
            ("conic-gradient", CssFunction::ConicGradient),
            ("content", CssFunction::Content),
            ("contrast", CssFunction::Contrast),
            ("contrast-color", CssFunction::ContrastColor),
            ("control-value", CssFunction::ControlValue),
            ("cos", CssFunction::Cos),
            ("counter", CssFunction::Counter),
            ("counters", CssFunction::Counters),
            ("cross-fade", CssFunction::CrossFade),
            ("cross-origin", CssFunction::CrossOrigin),
            ("cubic-bezier", CssFunction::CubicBezier),
            ("device-cmyk", CssFunction::DeviceCmyk),
            ("drop-shadow", CssFunction::DropShadow),
            ("dynamic-range-limit-mix", CssFunction::DynamicRangeLimitMix),
            ("element", CssFunction::Element),
            ("ellipse", CssFunction::Ellipse),
            ("env", CssFunction::Env),
            ("exp", CssFunction::Exp),
            ("fade", CssFunction::Fade),
            ("filter", CssFunction::Filter),
            ("first-valid", CssFunction::FirstValid),
            ("fit-content", CssFunction::FitContentFuncdefGridTemplateColumnsFitC),
            ("fit-content", CssFunction::FitContentFuncdefWidthFitContent),
            ("grayscale", CssFunction::Grayscale),
            ("hdr-color", CssFunction::HdrColor),
            ("hsl", CssFunction::Hsl),
            ("hsla", CssFunction::Hsla),
            ("hue-rotate", CssFunction::HueRotate),
            ("hwb", CssFunction::Hwb),
            ("hypot", CssFunction::Hypot),
            ("ictcp", CssFunction::Ictcp),
            ("ident", CssFunction::Ident),
            ("if", CssFunction::If),
            ("image", CssFunction::Image),
            ("image-set", CssFunction::ImageSet),
            ("inherit", CssFunction::Inherit),
            ("inset", CssFunction::Inset),
            ("integrity", CssFunction::Integrity),
            ("interpolate", CssFunction::Interpolate),
            ("invert", CssFunction::Invert),
            ("jzazbz", CssFunction::Jzazbz),
            ("jzczhz", CssFunction::Jzczhz),
            ("lab", CssFunction::Lab),
            ("lch", CssFunction::Lch),
            ("leader", CssFunction::Leader),
            ("light-dark", CssFunction::LightDark),
            ("linear", CssFunction::Linear),
            ("linear-gradient", CssFunction::LinearGradient),
            ("log", CssFunction::Log),
            ("matrix", CssFunction::Matrix),
            ("matrix3d", CssFunction::Matrix3d),
            ("max", CssFunction::Max),
            ("media", CssFunction::Media),
            ("min", CssFunction::Min),
            ("minmax", CssFunction::Minmax),
            ("mod", CssFunction::Mod),
            ("oklab", CssFunction::Oklab),
            ("oklch", CssFunction::Oklch),
            ("opacity", CssFunction::Opacity),
            ("paint", CssFunction::Paint),
            ("palette-mix", CssFunction::PaletteMix),
            ("param", CssFunction::Param),
            ("path", CssFunction::Path),
            ("perspective", CssFunction::Perspective),
            ("pointer", CssFunction::Pointer),
            ("polygon", CssFunction::Polygon),
            ("pow", CssFunction::Pow),
            ("progress", CssFunction::Progress),
            ("radial-gradient", CssFunction::RadialGradient),
            ("random", CssFunction::Random),
            ("random-item", CssFunction::RandomItem),
            ("ray", CssFunction::Ray),
            ("rect", CssFunction::RectFuncdefBasicShapeRect),
            ("rect", CssFunction::RectFuncdefClipRect),
            ("referrer-policy", CssFunction::ReferrerPolicy),
            ("rem", CssFunction::Rem),
            ("repeat", CssFunction::RepeatFuncdefRepeatLineColorRepeat),
            ("repeat", CssFunction::RepeatFuncdefTrackRepeatRepeat),
            ("repeating-conic-gradient", CssFunction::RepeatingConicGradient),
            ("repeating-linear-gradient", CssFunction::RepeatingLinearGradient),
            ("repeating-radial-gradient", CssFunction::RepeatingRadialGradient),
            ("rgb", CssFunction::Rgb),
            ("rgba", CssFunction::Rgba),
            ("rotate", CssFunction::Rotate),
            ("rotate3d", CssFunction::Rotate3d),
            ("rotateX", CssFunction::Rotatex),
            ("rotateY", CssFunction::Rotatey),
            ("rotateZ", CssFunction::Rotatez),
            ("round", CssFunction::Round),
            ("running", CssFunction::Running),
            ("saturate", CssFunction::Saturate),
            ("scale", CssFunction::ScaleFuncdefScale),
            ("scale", CssFunction::ScaleFuncdefTransformScale),
            ("scale3d", CssFunction::Scale3d),
            ("scaleX", CssFunction::ScalexFuncdefScalex),
            ("scaleX", CssFunction::ScalexFuncdefTransformScalex),
            ("scaleY", CssFunction::ScaleyFuncdefScaley),
            ("scaleY", CssFunction::ScaleyFuncdefTransformScaley),
            ("scaleZ", CssFunction::Scalez),
            ("scroll", CssFunction::Scroll),
            ("sepia", CssFunction::Sepia),
            ("shape", CssFunction::Shape),
            ("sibling-count", CssFunction::SiblingCount),
            ("sibling-index", CssFunction::SiblingIndex),
            ("sign", CssFunction::Sign),
            ("sin", CssFunction::Sin),
            ("skew", CssFunction::Skew),
            ("skewX", CssFunction::Skewx),
            ("skewY", CssFunction::Skewy),
            ("snap-block", CssFunction::SnapBlock),
            ("snap-inline", CssFunction::SnapInline),
            ("sqrt", CssFunction::Sqrt),
            ("src", CssFunction::Src),
            ("steps", CssFunction::Steps),
            ("string", CssFunction::String),
            ("stripes", CssFunction::Stripes),
            ("superellipse", CssFunction::Superellipse),
            ("supports", CssFunction::Supports),
            ("symbols", CssFunction::Symbols),
            ("tan", CssFunction::Tan),
            ("target-counter", CssFunction::TargetCounter),
            ("target-counters", CssFunction::TargetCounters),
            ("target-text", CssFunction::TargetText),
            ("toggle", CssFunction::Toggle),
            ("transform-interpolate", CssFunction::TransformInterpolate),
            ("transform-mix", CssFunction::TransformMix),
            ("translate", CssFunction::Translate),
            ("translate3d", CssFunction::Translate3d),
            ("translateX", CssFunction::Translatex),
            ("translateY", CssFunction::Translatey),
            ("translateZ", CssFunction::Translatez),
            ("type", CssFunction::TypeFuncdefFunctionType),
            ("type", CssFunction::TypeFuncdefAttrType),
            ("type", CssFunction::TypeFuncdefImageSetType),
            ("url", CssFunction::Url),
            ("url-pattern", CssFunction::UrlPattern),
            ("var", CssFunction::Var),
            ("view", CssFunction::View),
            ("wcag2", CssFunction::Wcag2),
            ("xywh", CssFunction::Xywh),
        ];
        match ENTRIES.binary_search_by_key(&name, |(n, _)| n) {
            Ok(i) => ENTRIES[i].1.clone(),
            Err(_) => CssFunction::Unknown(name.to_string()),
        }
    }}
