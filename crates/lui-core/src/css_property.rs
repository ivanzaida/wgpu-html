// Auto-generated from properties.json. DO NOT EDIT.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CssProperty {
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-align-content
  /// syntax: normal | <baseline-position> | <content-distribution> | <overflow-position>? <content-position>
  /// legacy_alias_of: align-content
  WebkitAlignContent,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-align-items
  /// syntax: normal | stretch | <baseline-position> | <overflow-position>? <self-position>
  /// legacy_alias_of: align-items
  WebkitAlignItems,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-align-self
  /// syntax: auto | <overflow-position>? [ normal | <self-position> ]| stretch | <baseline-position>
  /// legacy_alias_of: align-self
  WebkitAlignSelf,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-animation
  /// syntax: <single-animation>#
  /// longhands: [animation-name, animation-duration, animation-timing-function, animation-delay,
  /// animation-iteration-count, animation-direction, animation-fill-mode, animation-play-state, animation-timeline]
  /// legacy_alias_of: animation
  WebkitAnimation,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-animation-delay
  /// syntax: <time>#
  /// legacy_alias_of: animation-delay
  WebkitAnimationDelay,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-animation-direction
  /// syntax: <single-animation-direction>#
  /// legacy_alias_of: animation-direction
  WebkitAnimationDirection,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-animation-duration
  /// syntax: <time [0s,∞]>#
  /// legacy_alias_of: animation-duration
  WebkitAnimationDuration,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-animation-fill-mode
  /// syntax: <single-animation-fill-mode>#
  /// legacy_alias_of: animation-fill-mode
  WebkitAnimationFillMode,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-animation-iteration-count
  /// syntax: <single-animation-iteration-count>#
  /// legacy_alias_of: animation-iteration-count
  WebkitAnimationIterationCount,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-animation-name
  /// syntax: [ none | <keyframes-name> ]#
  /// legacy_alias_of: animation-name
  WebkitAnimationName,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-animation-play-state
  /// syntax: <single-animation-play-state>#
  /// legacy_alias_of: animation-play-state
  WebkitAnimationPlayState,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-animation-timing-function
  /// syntax: <easing-function>#
  /// legacy_alias_of: animation-timing-function
  WebkitAnimationTimingFunction,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef--webkit-appearance
  /// syntax: none | auto | base | base-select | <compat-auto> | <compat-special> | base
  /// legacy_alias_of: appearance
  WebkitAppearance,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-backface-visibility
  /// syntax: visible | hidden
  /// legacy_alias_of: backface-visibility
  WebkitBackfaceVisibility,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-background-clip
  /// syntax: <visual-box>#
  /// legacy_alias_of: background-clip
  WebkitBackgroundClip,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-background-origin
  /// syntax: <visual-box>#
  /// legacy_alias_of: background-origin
  WebkitBackgroundOrigin,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-background-size
  /// syntax: <bg-size>#
  /// legacy_alias_of: background-size
  WebkitBackgroundSize,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-border-bottom-left-radius
  /// syntax: <border-radius>
  /// legacy_alias_of: border-bottom-left-radius
  WebkitBorderBottomLeftRadius,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-border-bottom-right-radius
  /// syntax: <border-radius>
  /// legacy_alias_of: border-bottom-right-radius
  WebkitBorderBottomRightRadius,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-border-radius
  /// syntax: <length-percentage [0,∞]>{1,4} [ / <length-percentage [0,∞]>{1,4} ]?
  /// longhands: [border-top-left-radius, border-top-right-radius, border-bottom-right-radius,
  /// border-bottom-left-radius] legacy_alias_of: border-radius
  WebkitBorderRadius,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-border-top-left-radius
  /// syntax: <border-radius>
  /// legacy_alias_of: border-top-left-radius
  WebkitBorderTopLeftRadius,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-border-top-right-radius
  /// syntax: <border-radius>
  /// legacy_alias_of: border-top-right-radius
  WebkitBorderTopRightRadius,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-box-align
  WebkitBoxAlign,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-box-flex
  WebkitBoxFlex,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-box-ordinal-group
  WebkitBoxOrdinalGroup,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-box-orient
  WebkitBoxOrient,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-box-pack
  WebkitBoxPack,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-box-shadow
  /// syntax: <spread-shadow>#
  /// longhands: [box-shadow-color, box-shadow-offset, box-shadow-blur, box-shadow-spread, box-shadow-position]
  /// legacy_alias_of: box-shadow
  WebkitBoxShadow,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-box-sizing
  /// syntax: content-box | border-box
  /// legacy_alias_of: box-sizing
  WebkitBoxSizing,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-filter
  /// syntax: none | <filter-value-list>
  /// legacy_alias_of: filter
  WebkitFilter,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-flex-propdef
  /// syntax: none | [ <'flex-grow'> <'flex-shrink'>? || <'flex-basis'> ]
  /// longhands: [flex-grow, flex-shrink, flex-basis]
  /// legacy_alias_of: flex
  WebkitFlex,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-flex-basis
  /// syntax: content | <'width'>
  /// legacy_alias_of: flex-basis
  WebkitFlexBasis,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-flex-direction
  /// syntax: row | row-reverse | column | column-reverse
  /// legacy_alias_of: flex-direction
  WebkitFlexDirection,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-flex-flow
  /// syntax: <'flex-direction'> || <'flex-wrap'>
  /// longhands: [flex-direction, flex-wrap]
  /// legacy_alias_of: flex-flow
  WebkitFlexFlow,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-flex-grow
  /// syntax: <number [0,∞]>
  /// legacy_alias_of: flex-grow
  WebkitFlexGrow,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-flex-shrink
  /// syntax: <number [0,∞]>
  /// legacy_alias_of: flex-shrink
  WebkitFlexShrink,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-flex-wrap
  /// syntax: nowrap | wrap | wrap-reverse
  /// legacy_alias_of: flex-wrap
  WebkitFlexWrap,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-justify-content
  /// syntax: normal | <content-distribution> | <overflow-position>? [ <content-position> | left | right ]
  /// legacy_alias_of: justify-content
  WebkitJustifyContent,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef--webkit-line-clamp
  /// syntax: none | <integer [1,∞]>
  /// initial: none
  /// longhands: [max-lines, block-ellipsis, continue]
  WebkitLineClamp,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-mask
  /// syntax: <mask-layer>#
  /// longhands: [mask-image, mask-position, mask-size, mask-repeat, mask-origin, mask-clip, mask-composite, mask-mode]
  /// legacy_alias_of: mask
  WebkitMask,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-mask-box-image
  /// syntax: <'mask-border-source'> || <'mask-border-slice'> [ / <'mask-border-width'>? [ / <'mask-border-outset'> ]?
  /// ]? || <'mask-border-repeat'> || <'mask-border-mode'> longhands: [mask-border-source, mask-border-slice,
  /// mask-border-width, mask-border-outset, mask-border-repeat, mask-border-mode] legacy_alias_of: mask-border
  WebkitMaskBoxImage,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-mask-box-image-outset
  /// syntax: <'border-image-outset'>
  /// legacy_alias_of: mask-border-outset
  WebkitMaskBoxImageOutset,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-mask-box-image-repeat
  /// syntax: <'border-image-repeat'>
  /// legacy_alias_of: mask-border-repeat
  WebkitMaskBoxImageRepeat,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-mask-box-image-slice
  /// syntax: <'border-image-slice'>
  /// legacy_alias_of: mask-border-slice
  WebkitMaskBoxImageSlice,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-mask-box-image-source
  /// syntax: <'border-image-source'>
  /// legacy_alias_of: mask-border-source
  WebkitMaskBoxImageSource,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-mask-box-image-width
  /// syntax: <'border-image-width'>
  /// legacy_alias_of: mask-border-width
  WebkitMaskBoxImageWidth,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-mask-clip
  /// syntax: [ <coord-box> | no-clip ]#
  /// legacy_alias_of: mask-clip
  WebkitMaskClip,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-mask-composite
  /// syntax: <compositing-operator>#
  /// legacy_alias_of: mask-composite
  WebkitMaskComposite,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-mask-image
  /// syntax: <mask-reference>#
  /// legacy_alias_of: mask-image
  WebkitMaskImage,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-mask-origin
  /// syntax: <coord-box>#
  /// legacy_alias_of: mask-origin
  WebkitMaskOrigin,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-mask-position
  /// syntax: <position>#
  /// legacy_alias_of: mask-position
  WebkitMaskPosition,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-mask-repeat
  /// syntax: <repeat-style>#
  /// legacy_alias_of: mask-repeat
  WebkitMaskRepeat,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-mask-size
  /// syntax: <bg-size>#
  /// legacy_alias_of: mask-size
  WebkitMaskSize,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-order
  /// syntax: <integer>
  /// legacy_alias_of: order
  WebkitOrder,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-perspective
  /// syntax: none | <length [0,∞]>
  /// legacy_alias_of: perspective
  WebkitPerspective,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-perspective-origin
  /// syntax: <position>
  /// legacy_alias_of: perspective-origin
  WebkitPerspectiveOrigin,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-text-fill-color
  /// syntax: <color>
  /// initial: currentcolor
  /// inherited: yes
  WebkitTextFillColor,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-text-size-adjust
  /// syntax: auto | none | <percentage [0,∞]>
  /// legacy_alias_of: text-size-adjust
  WebkitTextSizeAdjust,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-text-stroke
  /// syntax: <line-width> || <color>
  /// initial: See individual properties
  /// inherited: yes
  /// longhands: [-webkit-text-stroke-width, -webkit-text-stroke-color]
  WebkitTextStroke,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-text-stroke-color
  /// syntax: <color>
  /// initial: currentcolor
  /// inherited: yes
  WebkitTextStrokeColor,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-text-stroke-width
  /// syntax: <line-width>
  /// initial: 0
  /// inherited: yes
  WebkitTextStrokeWidth,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-transform
  /// syntax: none | <transform-list>
  /// legacy_alias_of: transform
  WebkitTransform,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-transform-origin
  /// syntax: [ left | center | right | top | bottom | <length-percentage> ] | [ left | center | right |
  /// <length-percentage> ] [ top | center | bottom | <length-percentage> ] <length>? | [ [ center | left | right ] && [
  /// center | top | bottom ] ] <length>? legacy_alias_of: transform-origin
  WebkitTransformOrigin,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-transform-style
  /// syntax: flat | preserve-3d
  /// legacy_alias_of: transform-style
  WebkitTransformStyle,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-transition
  /// syntax: <single-transition>#
  /// longhands: [transition-property, transition-duration, transition-timing-function, transition-delay,
  /// transition-behavior] legacy_alias_of: transition
  WebkitTransition,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-transition-delay
  /// syntax: <time>#
  /// legacy_alias_of: transition-delay
  WebkitTransitionDelay,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-transition-duration
  /// syntax: <time [0s,∞]>#
  /// legacy_alias_of: transition-duration
  WebkitTransitionDuration,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-transition-property
  /// syntax: none | <single-transition-property>#
  /// legacy_alias_of: transition-property
  WebkitTransitionProperty,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef--webkit-transition-timing-function
  /// syntax: <easing-function>#
  /// legacy_alias_of: transition-timing-function
  WebkitTransitionTimingFunction,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef--webkit-user-select
  /// syntax: auto | text | none | contain | all
  WebkitUserSelect,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-accent-color
  /// syntax: auto | <color>
  /// initial: auto
  /// inherited: yes
  AccentColor,
  ///
  /// href: https://drafts.csswg.org/css-align-3/#propdef-align-content
  /// syntax: normal | <baseline-position> | <content-distribution> | <overflow-position>? <content-position>
  /// initial: normal
  AlignContent,
  ///
  /// href: https://drafts.csswg.org/css-align-3/#propdef-align-items
  /// syntax: normal | stretch | <baseline-position> | <overflow-position>? <self-position>
  /// initial: normal
  AlignItems,
  ///
  /// href: https://drafts.csswg.org/css-align-3/#propdef-align-self
  /// syntax: auto | <overflow-position>? [ normal | <self-position> ]| stretch | <baseline-position> | anchor-center
  /// initial: auto
  AlignSelf,
  ///
  /// href: https://drafts.csswg.org/css-inline-3/#propdef-alignment-baseline
  /// syntax: baseline | <baseline-metric>
  /// initial: baseline
  AlignmentBaseline,
  ///
  /// href: https://drafts.csswg.org/css-cascade-5/#propdef-all
  /// syntax: initial | inherit | unset | revert | revert-layer | revert-rule
  /// initial: see individual properties
  All,
  ///
  /// href: https://drafts.csswg.org/css-anchor-position-1/#propdef-anchor-name
  /// syntax: none | <anchor-name>#
  /// initial: none
  AnchorName,
  ///
  /// href: https://drafts.csswg.org/css-anchor-position-1/#propdef-anchor-scope
  /// syntax: none | all | <anchor-name>#
  /// initial: none
  AnchorScope,
  ///
  /// href: https://drafts.csswg.org/css-animations-1/#propdef-animation
  /// syntax: <single-animation>#
  /// initial: see individual properties
  /// longhands: [animation-name, animation-duration, animation-timing-function, animation-delay,
  /// animation-iteration-count, animation-direction, animation-fill-mode, animation-play-state, animation-timeline]
  Animation,
  ///
  /// href: https://drafts.csswg.org/css-animations-2/#propdef-animation-composition
  /// syntax: <single-animation-composition>#
  /// initial: replace
  AnimationComposition,
  ///
  /// href: https://drafts.csswg.org/css-animations-1/#propdef-animation-delay
  /// syntax: <time>#
  /// initial: 0s
  AnimationDelay,
  ///
  /// href: https://drafts.csswg.org/css-animations-1/#propdef-animation-direction
  /// syntax: <single-animation-direction>#
  /// initial: normal
  AnimationDirection,
  ///
  /// href: https://drafts.csswg.org/css-animations-2/#propdef-animation-duration
  /// syntax: [ auto | <time [0s,∞]> ]#
  /// initial: auto
  AnimationDuration,
  ///
  /// href: https://drafts.csswg.org/css-animations-1/#propdef-animation-fill-mode
  /// syntax: <single-animation-fill-mode>#
  /// initial: none
  AnimationFillMode,
  ///
  /// href: https://drafts.csswg.org/css-animations-1/#propdef-animation-iteration-count
  /// syntax: <single-animation-iteration-count>#
  /// initial: 1
  AnimationIterationCount,
  ///
  /// href: https://drafts.csswg.org/css-animations-1/#propdef-animation-name
  /// syntax: [ none | <keyframes-name> ]#
  /// initial: none
  AnimationName,
  ///
  /// href: https://drafts.csswg.org/css-animations-1/#propdef-animation-play-state
  /// syntax: <single-animation-play-state>#
  /// initial: running
  AnimationPlayState,
  ///
  /// href: https://drafts.csswg.org/scroll-animations-1/#propdef-animation-range
  /// syntax: [ <'animation-range-start'> <'animation-range-end'>? ]#
  /// initial: see individual properties
  /// longhands: [animation-range-start, animation-range-end]
  AnimationRange,
  ///
  /// href: https://drafts.csswg.org/pointer-animations-1/#propdef-animation-range-center
  /// syntax: [ normal | [ <length-percentage> | <timeline-range-center-subject> <length-percentage>? ] ]#
  /// initial: normal
  AnimationRangeCenter,
  ///
  /// href: https://drafts.csswg.org/scroll-animations-1/#propdef-animation-range-end
  /// syntax: [ normal | <length-percentage> | <timeline-range-name> <length-percentage>? ]#
  /// initial: normal
  AnimationRangeEnd,
  ///
  /// href: https://drafts.csswg.org/scroll-animations-1/#propdef-animation-range-start
  /// syntax: [ normal | <length-percentage> | <timeline-range-name> <length-percentage>? ]#
  /// initial: normal
  AnimationRangeStart,
  ///
  /// href: https://drafts.csswg.org/css-animations-2/#propdef-animation-timeline
  /// syntax: <single-animation-timeline>#
  /// initial: auto
  AnimationTimeline,
  ///
  /// href: https://drafts.csswg.org/css-animations-1/#propdef-animation-timing-function
  /// syntax: <easing-function>#
  /// initial: ease
  AnimationTimingFunction,
  ///
  /// href: https://drafts.csswg.org/animation-triggers-1/#propdef-animation-trigger
  /// syntax: [ none | [ <dashed-ident> <animation-action>+ ]+ ]#
  /// initial: none
  AnimationTrigger,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-appearance
  /// syntax: none | auto | base | base-select | <compat-auto> | <compat-special> | base
  /// initial: none
  Appearance,
  ///
  /// href: https://drafts.csswg.org/css-sizing-4/#propdef-aspect-ratio
  /// syntax: auto || <ratio>
  /// initial: auto
  AspectRatio,
  ///
  /// href: https://drafts.csswg.org/filter-effects-2/#propdef-backdrop-filter
  /// syntax: none | <filter-value-list>
  /// initial: none
  BackdropFilter,
  ///
  /// href: https://drafts.csswg.org/css-transforms-2/#propdef-backface-visibility
  /// syntax: visible | hidden
  /// initial: visible
  BackfaceVisibility,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background
  /// syntax: <bg-layer>#? , <final-bg-layer>
  /// initial: see individual properties
  /// longhands: [background-image, background-position, background-size, background-repeat, background-attachment,
  /// background-origin, background-clip, background-color]
  Background,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-attachment
  /// syntax: <attachment>#
  /// initial: scroll
  BackgroundAttachment,
  ///
  /// href: https://drafts.csswg.org/compositing-2/#propdef-background-blend-mode
  /// syntax: <'mix-blend-mode'>#
  /// initial: normal
  BackgroundBlendMode,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-clip
  /// syntax: <bg-clip>#
  /// initial: border-box
  BackgroundClip,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-color
  /// syntax: <color>
  /// initial: transparent
  BackgroundColor,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-image
  /// syntax: <bg-image>#
  /// initial: none
  BackgroundImage,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-origin
  /// syntax: <visual-box>#
  /// initial: padding-box
  BackgroundOrigin,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-position
  /// syntax: <bg-position>#
  /// initial: 0% 0%
  /// longhands: [background-position-x, background-position-y]
  BackgroundPosition,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-position-block
  /// syntax: [ center | [ [ start | end ]? <length-percentage>? ]! ]#
  /// initial: 0%
  BackgroundPositionBlock,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-position-inline
  /// syntax: [ center | [ [ start | end ]? <length-percentage>? ]! ]#
  /// initial: 0%
  BackgroundPositionInline,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-position-x
  /// syntax: [ center | [ [ left | right | x-start | x-end ]? <length-percentage>? ]! ]#
  /// initial: 0%
  BackgroundPositionX,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-position-y
  /// syntax: [ center | [ [ top | bottom | y-start | y-end ]? <length-percentage>? ]! ]#
  /// initial: 0%
  BackgroundPositionY,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-repeat
  /// syntax: <repeat-style>#
  /// initial: repeat
  BackgroundRepeat,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-repeat-block
  /// syntax: <repetition>#
  /// initial: repeat
  BackgroundRepeatBlock,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-repeat-inline
  /// syntax: <repetition>#
  /// initial: repeat
  BackgroundRepeatInline,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-repeat-x
  /// syntax: <repetition>#
  /// initial: repeat
  BackgroundRepeatX,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-repeat-y
  /// syntax: <repetition>#
  /// initial: repeat
  BackgroundRepeatY,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-size
  /// syntax: <bg-size>#
  /// initial: auto
  BackgroundSize,
  ///
  /// href: https://drafts.csswg.org/css-backgrounds-4/#propdef-background-tbd
  /// syntax: <bg-layer>#
  /// initial: see individual properties
  BackgroundTbd,
  ///
  /// href: https://drafts.csswg.org/css-inline-3/#propdef-baseline-shift
  /// syntax: <length-percentage> | sub | super | top | center | bottom
  /// initial: 0
  BaselineShift,
  ///
  /// href: https://drafts.csswg.org/css-inline-3/#propdef-baseline-source
  /// syntax: auto | first | last
  /// initial: auto
  BaselineSource,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-block-ellipsis
  /// syntax: no-ellipsis | auto | <string>
  /// initial: no-ellipsis
  /// inherited: yes
  BlockEllipsis,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-block-size
  /// syntax: <'width'>
  /// initial: auto
  BlockSize,
  ///
  /// href: https://drafts.csswg.org/css-rhythm-1/#propdef-block-step
  /// syntax: <'block-step-size'> || <'block-step-insert'> || <'block-step-align'> || <'block-step-round'>
  /// initial: see individual properties
  /// longhands: [block-step-size, block-step-insert, block-step-align, block-step-round]
  BlockStep,
  ///
  /// href: https://drafts.csswg.org/css-rhythm-1/#propdef-block-step-align
  /// syntax: auto | center | start | end
  /// initial: auto
  BlockStepAlign,
  ///
  /// href: https://drafts.csswg.org/css-rhythm-1/#propdef-block-step-insert
  /// syntax: margin-box | padding-box | content-box
  /// initial: margin-box
  BlockStepInsert,
  ///
  /// href: https://drafts.csswg.org/css-rhythm-1/#propdef-block-step-round
  /// syntax: up | down | nearest
  /// initial: up
  BlockStepRound,
  ///
  /// href: https://drafts.csswg.org/css-rhythm-1/#propdef-block-step-size
  /// syntax: none | <length [0,∞]>
  /// initial: none
  BlockStepSize,
  ///
  /// href: https://drafts.csswg.org/css-content-3/#propdef-bookmark-label
  /// syntax: <content-list>
  /// initial: content(text)
  BookmarkLabel,
  ///
  /// href: https://drafts.csswg.org/css-content-3/#propdef-bookmark-level
  /// syntax: none | <integer [1,∞]>
  /// initial: none
  BookmarkLevel,
  ///
  /// href: https://drafts.csswg.org/css-content-3/#propdef-bookmark-state
  /// syntax: open | closed
  /// initial: open
  BookmarkState,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border
  /// syntax: <line-width> || <line-style> || <color>
  /// initial: see individual properties
  /// longhands: [border-width, border-style, border-color]
  Border,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block
  /// syntax: <'border-block-start'>
  /// initial: see individual properties
  /// longhands: [border-block-start, border-block-end]
  BorderBlock,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-clip
  /// syntax: <'border-top-clip'>
  /// initial: see individual properties
  /// longhands: [border-block-start-clip, border-block-end-clip]
  BorderBlockClip,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-color
  /// syntax: <'border-top-color'>{1,2}
  /// initial: see individual properties
  /// longhands: [border-block-start-color, border-block-end-color]
  BorderBlockColor,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-end
  /// syntax: <line-width> || <line-style> || <color>
  /// initial: See individual properties
  /// longhands: [border-block-end-width, border-block-end-style, border-block-end-color]
  BorderBlockEnd,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-end-clip
  /// syntax: none | [ <length-percentage [0,∞]> | <flex> ]+
  /// initial: none
  BorderBlockEndClip,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-end-color
  /// syntax: <color> | <image-1D>
  /// initial: currentcolor
  BorderBlockEndColor,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-end-radius
  /// syntax: <length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?
  /// initial: 0
  /// longhands: [border-end-start-radius, border-end-end-radius]
  BorderBlockEndRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-end-style
  /// syntax: <line-style>
  /// initial: none
  BorderBlockEndStyle,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-end-width
  /// syntax: <line-width>
  /// initial: medium
  BorderBlockEndWidth,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-start
  /// syntax: <line-width> || <line-style> || <color>
  /// initial: See individual properties
  /// longhands: [border-block-start-width, border-block-start-style, border-block-start-color]
  BorderBlockStart,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-start-clip
  /// syntax: none | [ <length-percentage [0,∞]> | <flex> ]+
  /// initial: none
  BorderBlockStartClip,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-start-color
  /// syntax: <color> | <image-1D>
  /// initial: currentcolor
  BorderBlockStartColor,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-start-radius
  /// syntax: <length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?
  /// initial: 0
  /// longhands: [border-start-start-radius, border-start-end-radius]
  BorderBlockStartRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-start-style
  /// syntax: <line-style>
  /// initial: none
  BorderBlockStartStyle,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-start-width
  /// syntax: <line-width>
  /// initial: medium
  BorderBlockStartWidth,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-style
  /// syntax: <'border-top-style'>{1,2}
  /// initial: see individual properties
  /// longhands: [border-block-start-style, border-block-end-style]
  BorderBlockStyle,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-block-width
  /// syntax: <'border-top-width'>{1,2}
  /// initial: see individual properties
  /// longhands: [border-block-start-width, border-block-end-width]
  BorderBlockWidth,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-bottom
  /// syntax: <line-width> || <line-style> || <color>
  /// initial: See individual properties
  /// longhands: [border-bottom-width, border-bottom-style, border-bottom-color]
  BorderBottom,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-bottom-clip
  /// syntax: none | [ <length-percentage [0,∞]> | <flex> ]+
  /// initial: none
  BorderBottomClip,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-bottom-color
  /// syntax: <color> | <image-1D>
  /// initial: currentcolor
  BorderBottomColor,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-bottom-left-radius
  /// syntax: <border-radius>
  /// initial: 0
  BorderBottomLeftRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-bottom-radius
  /// syntax: <length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?
  /// initial: 0
  /// longhands: [border-bottom-left-radius, border-bottom-right-radius]
  BorderBottomRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-bottom-right-radius
  /// syntax: <border-radius>
  /// initial: 0
  BorderBottomRightRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-bottom-style
  /// syntax: <line-style>
  /// initial: none
  BorderBottomStyle,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-bottom-width
  /// syntax: <line-width>
  /// initial: medium
  BorderBottomWidth,
  ///
  /// href: https://drafts.csswg.org/css-round-display-1/#propdef-border-boundary
  /// syntax: none | parent | display
  /// initial: none
  /// inherited: yes
  BorderBoundary,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-clip
  /// syntax: <'border-top-clip'>
  /// initial: see individual properties
  /// longhands: [border-top-clip, border-right-clip, border-bottom-clip, border-left-clip]
  BorderClip,
  ///
  /// href: https://drafts.csswg.org/css-tables-3/#propdef-border-collapse
  /// syntax: separate | collapse
  /// initial: separate
  /// inherited: yes
  BorderCollapse,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-color
  /// syntax: [ <color> | <image-1D> ]{1,4}
  /// initial: see individual properties
  /// longhands: [border-top-color, border-right-color, border-bottom-color, border-left-color]
  BorderColor,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-end-end-radius
  /// syntax: <border-radius>
  /// initial: 0
  BorderEndEndRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-end-start-radius
  /// syntax: <border-radius>
  /// initial: 0
  BorderEndStartRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-image
  /// syntax: <'border-image-source'> || <'border-image-slice'> [ / <'border-image-width'> | / <'border-image-width'>? /
  /// <'border-image-outset'> ]? || <'border-image-repeat'> initial: See individual properties
  /// longhands: [border-image-source, border-image-slice, border-image-width, border-image-outset, border-image-repeat]
  BorderImage,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-image-outset
  /// syntax: [ <length [0,∞]> | <number [0,∞]> ]{1,4}
  /// initial: 0
  BorderImageOutset,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-image-repeat
  /// syntax: [ stretch | repeat | round | space ]{1,2}
  /// initial: stretch
  BorderImageRepeat,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-image-slice
  /// syntax: [<number [0,∞]> | <percentage [0,∞]>]{1,4} && fill?
  /// initial: 100%
  BorderImageSlice,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-image-source
  /// syntax: none | <image>
  /// initial: none
  BorderImageSource,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-image-width
  /// syntax: [ <length-percentage [0,∞]> | <number [0,∞]> | auto ]{1,4}
  /// initial: 1
  BorderImageWidth,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline
  /// syntax: <'border-block-start'>
  /// initial: see individual properties
  /// longhands: [border-inline-start, border-inline-end]
  BorderInline,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-clip
  /// syntax: <'border-top-clip'>
  /// initial: see individual properties
  /// longhands: [border-inline-start-clip, border-inline-end-clip]
  BorderInlineClip,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-color
  /// syntax: <'border-top-color'>{1,2}
  /// initial: see individual properties
  /// longhands: [border-inline-start-color, border-inline-end-color]
  BorderInlineColor,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-end
  /// syntax: <line-width> || <line-style> || <color>
  /// initial: See individual properties
  /// longhands: [border-inline-end-width, border-inline-end-style, border-inline-end-color]
  BorderInlineEnd,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-end-clip
  /// syntax: none | [ <length-percentage [0,∞]> | <flex> ]+
  /// initial: none
  BorderInlineEndClip,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-end-color
  /// syntax: <color> | <image-1D>
  /// initial: currentcolor
  BorderInlineEndColor,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-end-radius
  /// syntax: <length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?
  /// initial: 0
  /// longhands: [border-start-end-radius, border-end-end-radius]
  BorderInlineEndRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-end-style
  /// syntax: <line-style>
  /// initial: none
  BorderInlineEndStyle,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-end-width
  /// syntax: <line-width>
  /// initial: medium
  BorderInlineEndWidth,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-start
  /// syntax: <line-width> || <line-style> || <color>
  /// initial: See individual properties
  /// longhands: [border-inline-start-width, border-inline-start-style, border-inline-start-color]
  BorderInlineStart,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-start-clip
  /// syntax: none | [ <length-percentage [0,∞]> | <flex> ]+
  /// initial: none
  BorderInlineStartClip,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-start-color
  /// syntax: <color> | <image-1D>
  /// initial: currentcolor
  BorderInlineStartColor,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-start-radius
  /// syntax: <length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?
  /// initial: 0
  /// longhands: [border-start-start-radius, border-end-start-radius]
  BorderInlineStartRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-start-style
  /// syntax: <line-style>
  /// initial: none
  BorderInlineStartStyle,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-start-width
  /// syntax: <line-width>
  /// initial: medium
  BorderInlineStartWidth,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-style
  /// syntax: <'border-top-style'>{1,2}
  /// initial: see individual properties
  /// longhands: [border-inline-start-style, border-inline-end-style]
  BorderInlineStyle,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-inline-width
  /// syntax: <'border-top-width'>{1,2}
  /// initial: see individual properties
  /// longhands: [border-inline-start-width, border-inline-end-width]
  BorderInlineWidth,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-left
  /// syntax: <line-width> || <line-style> || <color>
  /// initial: See individual properties
  /// longhands: [border-left-width, border-left-style, border-left-color]
  BorderLeft,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-left-clip
  /// syntax: none | [ <length-percentage [0,∞]> | <flex> ]+
  /// initial: none
  BorderLeftClip,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-left-color
  /// syntax: <color> | <image-1D>
  /// initial: currentcolor
  BorderLeftColor,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-left-radius
  /// syntax: <length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?
  /// initial: 0
  /// longhands: [border-top-left-radius, border-bottom-left-radius]
  BorderLeftRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-left-style
  /// syntax: <line-style>
  /// initial: none
  BorderLeftStyle,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-left-width
  /// syntax: <line-width>
  /// initial: medium
  BorderLeftWidth,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-limit
  /// syntax: all | [ sides | corners ] <length-percentage [0,∞]>? | [ top | right | bottom | left ] <length-percentage
  /// [0,∞]> initial: all
  BorderLimit,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-radius
  /// syntax: <length-percentage [0,∞]>{1,4} [ / <length-percentage [0,∞]>{1,4} ]?
  /// initial: see individual properties
  /// longhands: [border-top-left-radius, border-top-right-radius, border-bottom-right-radius,
  /// border-bottom-left-radius]
  BorderRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-right
  /// syntax: <line-width> || <line-style> || <color>
  /// initial: See individual properties
  /// longhands: [border-right-width, border-right-style, border-right-color]
  BorderRight,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-right-clip
  /// syntax: none | [ <length-percentage [0,∞]> | <flex> ]+
  /// initial: none
  BorderRightClip,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-right-color
  /// syntax: <color> | <image-1D>
  /// initial: currentcolor
  BorderRightColor,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-right-radius
  /// syntax: <length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?
  /// initial: 0
  /// longhands: [border-top-right-radius, border-bottom-right-radius]
  BorderRightRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-right-style
  /// syntax: <line-style>
  /// initial: none
  BorderRightStyle,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-right-width
  /// syntax: <line-width>
  /// initial: medium
  BorderRightWidth,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-shape
  /// syntax: none | [ <basic-shape> <geometry-box>?]{1,2}
  /// initial: none
  BorderShape,
  ///
  /// href: https://drafts.csswg.org/css-tables-3/#propdef-border-spacing
  /// syntax: <length [0,∞]>{1,2}
  /// initial: 0px 0px
  /// inherited: yes
  BorderSpacing,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-start-end-radius
  /// syntax: <border-radius>
  /// initial: 0
  BorderStartEndRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-start-start-radius
  /// syntax: <border-radius>
  /// initial: 0
  BorderStartStartRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-style
  /// syntax: <'border-top-style'>{1,4}
  /// initial: see individual properties
  /// longhands: [border-top-style, border-right-style, border-bottom-style, border-left-style]
  BorderStyle,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-top
  /// syntax: <line-width> || <line-style> || <color>
  /// initial: See individual properties
  /// longhands: [border-top-width, border-top-style, border-top-color]
  BorderTop,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-top-clip
  /// syntax: none | [ <length-percentage [0,∞]> | <flex> ]+
  /// initial: none
  BorderTopClip,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-top-color
  /// syntax: <color> | <image-1D>
  /// initial: currentcolor
  BorderTopColor,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-top-left-radius
  /// syntax: <border-radius>
  /// initial: 0
  BorderTopLeftRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-top-radius
  /// syntax: <length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?
  /// initial: 0
  /// longhands: [border-top-left-radius, border-top-right-radius]
  BorderTopRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-top-right-radius
  /// syntax: <border-radius>
  /// initial: 0
  BorderTopRightRadius,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-top-style
  /// syntax: <line-style>
  /// initial: none
  BorderTopStyle,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-top-width
  /// syntax: <line-width>
  /// initial: medium
  BorderTopWidth,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-border-width
  /// syntax: <'border-top-width'>{1,4}
  /// initial: see individual properties
  /// longhands: [border-top-width, border-right-width, border-bottom-width, border-left-width]
  BorderWidth,
  ///
  /// href: https://drafts.csswg.org/css-position-3/#propdef-bottom
  /// syntax: auto | <length-percentage> | <anchor()> | <anchor-size()>
  /// initial: auto
  Bottom,
  ///
  /// href: https://drafts.csswg.org/css-break-4/#propdef-box-decoration-break
  /// syntax: slice | clone
  /// initial: slice
  BoxDecorationBreak,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-box-shadow
  /// syntax: <spread-shadow>#
  /// initial: none
  /// longhands: [box-shadow-color, box-shadow-offset, box-shadow-blur, box-shadow-spread, box-shadow-position]
  BoxShadow,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-box-shadow-blur
  /// syntax: <length [0,∞]>#
  /// initial: 0
  BoxShadowBlur,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-box-shadow-color
  /// syntax: <color>#
  /// initial: currentcolor
  BoxShadowColor,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-box-shadow-offset
  /// syntax: [ none | <length>{1,2} ]#
  /// initial: none
  BoxShadowOffset,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-box-shadow-position
  /// syntax: [ outset | inset ]#
  /// initial: outset
  BoxShadowPosition,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-box-shadow-spread
  /// syntax: <length>#
  /// initial: 0
  BoxShadowSpread,
  ///
  /// href: https://drafts.csswg.org/css-sizing-3/#propdef-box-sizing
  /// syntax: content-box | border-box
  /// initial: content-box
  BoxSizing,
  ///
  /// href: https://drafts.csswg.org/css-line-grid-1/#propdef-box-snap
  /// syntax: none | block-start | block-end | center | baseline | last-baseline
  /// initial: none
  /// inherited: yes
  BoxSnap,
  ///
  /// href: https://drafts.csswg.org/css-break-4/#propdef-break-after
  /// syntax: auto | avoid | always | all | avoid-page | page | left | right | recto | verso | avoid-column | column |
  /// avoid-region | region initial: auto
  BreakAfter,
  ///
  /// href: https://drafts.csswg.org/css-break-4/#propdef-break-before
  /// syntax: auto | avoid | always | all | avoid-page | page | left | right | recto | verso | avoid-column | column |
  /// avoid-region | region initial: auto
  BreakBefore,
  ///
  /// href: https://drafts.csswg.org/css-break-4/#propdef-break-inside
  /// syntax: auto | avoid | avoid-page | avoid-column | avoid-region
  /// initial: auto
  BreakInside,
  ///
  /// href: https://drafts.csswg.org/css-tables-3/#propdef-caption-side
  /// syntax: top | bottom
  /// initial: top
  /// inherited: yes
  CaptionSide,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-caret
  /// syntax: <'caret-color'> || <'caret-animation'> || <'caret-shape'>
  /// initial: auto
  /// inherited: yes
  /// longhands: [caret-color, caret-animation, caret-shape]
  Caret,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-caret-animation
  /// syntax: auto | manual
  /// initial: auto
  /// inherited: yes
  CaretAnimation,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-caret-color
  /// syntax: auto | <color> [auto | <color>]?
  /// initial: auto
  /// inherited: yes
  CaretColor,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-caret-shape
  /// syntax: auto | bar | block | underscore
  /// initial: auto
  /// inherited: yes
  CaretShape,
  ///
  /// href: https://drafts.csswg.org/css-page-floats-3/#propdef-clear
  /// syntax: inline-start | inline-end | block-start | block-end | left | right | top | bottom | both-inline |
  /// both-block | both | none initial: none
  Clear,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-clip
  /// syntax: <rect()> | auto
  /// initial: auto
  Clip,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-clip-path
  /// syntax: <clip-source> | [ <basic-shape> || <geometry-box> ] | none
  /// initial: none
  ClipPath,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-clip-rule
  /// syntax: nonzero | evenodd
  /// initial: nonzero
  /// inherited: yes
  ClipRule,
  ///
  /// href: https://drafts.csswg.org/css-color-4/#propdef-color
  /// syntax: <color>
  /// initial: CanvasText
  /// inherited: yes
  Color,
  ///
  /// href: https://drafts.csswg.org/css-color-adjust-1/#propdef-color-adjust
  /// syntax: <'print-color-adjust'>
  /// initial: see individual properties
  /// longhands: [print-color-adjust]
  ColorAdjust,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/painting.html#ColorInterpolationProperty
  /// syntax: auto | sRGB | linearRGB
  /// initial: sRGB
  /// inherited: yes
  ColorInterpolation,
  ///
  /// href: https://drafts.csswg.org/filter-effects-1/#propdef-color-interpolation-filters
  /// syntax: auto | sRGB | linearRGB
  /// initial: linearRGB
  /// inherited: yes
  ColorInterpolationFilters,
  ///
  /// href: https://drafts.csswg.org/css-color-adjust-1/#propdef-color-scheme
  /// syntax: normal | [ light | dark | <custom-ident> ]+ && only?
  /// initial: normal
  /// inherited: yes
  ColorScheme,
  ///
  /// href: https://drafts.csswg.org/css-multicol-2/#propdef-column-count
  /// syntax: auto | <integer [1,∞]>
  /// initial: auto
  ColumnCount,
  ///
  /// href: https://drafts.csswg.org/css-multicol-2/#propdef-column-fill
  /// syntax: auto | balance | balance-all
  /// initial: balance
  ColumnFill,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-gap
  /// syntax: normal | <length-percentage [0,∞]> | <line-width>
  /// initial: normal
  ColumnGap,
  ///
  /// href: https://drafts.csswg.org/css-multicol-2/#propdef-column-height
  /// syntax: auto | <length [0,∞]>
  /// initial: auto
  ColumnHeight,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-rule
  /// syntax: <gap-rule-list> | <gap-auto-rule-list>
  /// initial: see individual properties
  /// longhands: [column-rule-width, column-rule-style, column-rule-color]
  ColumnRule,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-rule-break
  /// syntax: none | normal | intersection
  /// initial: normal
  ColumnRuleBreak,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-rule-color
  /// syntax: <line-color-list> | <auto-line-color-list>
  /// initial: currentcolor
  ColumnRuleColor,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-rule-inset
  /// syntax: <'column-rule-inset-cap'> [ / <'column-rule-inset-junction'> ]?
  /// initial: see individual properties
  /// longhands: [column-rule-inset-cap-start, column-rule-inset-cap-end, column-rule-inset-junction-start,
  /// column-rule-inset-junction-end]
  ColumnRuleInset,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-rule-inset-cap
  /// syntax: <length-percentage> [ <length-percentage> ]?
  /// initial: see individual properties
  /// longhands: [column-rule-inset-cap-start, column-rule-inset-cap-end]
  ColumnRuleInsetCap,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-rule-inset-cap-end
  /// syntax: <length-percentage>
  /// initial: 0
  ColumnRuleInsetCapEnd,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-rule-inset-cap-start
  /// syntax: <length-percentage>
  /// initial: 0
  ColumnRuleInsetCapStart,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-rule-inset-end
  /// syntax: <length-percentage>
  /// initial: see individual properties
  /// longhands: [column-rule-inset-cap-end, column-rule-inset-junction-end]
  ColumnRuleInsetEnd,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-rule-inset-junction
  /// syntax: <length-percentage> [ <length-percentage> ]?
  /// initial: see individual properties
  /// longhands: [column-rule-inset-junction-start, column-rule-inset-junction-end]
  ColumnRuleInsetJunction,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-rule-inset-junction-end
  /// syntax: <length-percentage>
  /// initial: 0
  ColumnRuleInsetJunctionEnd,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-rule-inset-junction-start
  /// syntax: <length-percentage>
  /// initial: 0
  ColumnRuleInsetJunctionStart,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-rule-inset-start
  /// syntax: <length-percentage>
  /// initial: see individual properties
  /// longhands: [column-rule-inset-cap-start, column-rule-inset-junction-start]
  ColumnRuleInsetStart,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-rule-style
  /// syntax: <line-style-list> | <auto-line-style-list>
  /// initial: none
  ColumnRuleStyle,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-rule-visibility-items
  /// syntax: all | around | between | normal
  /// initial: normal
  ColumnRuleVisibilityItems,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-column-rule-width
  /// syntax: <line-width-list> | <auto-line-width-list>
  /// initial: medium
  ColumnRuleWidth,
  ///
  /// href: https://drafts.csswg.org/css-multicol-2/#propdef-column-span
  /// syntax: none | <integer [1,∞]> | all | auto
  /// initial: none
  ColumnSpan,
  ///
  /// href: https://drafts.csswg.org/css-multicol-2/#propdef-column-width
  /// syntax: auto | <length [0,∞]> | min-content | max-content | fit-content(<length-percentage>)
  /// initial: auto
  ColumnWidth,
  ///
  /// href: https://drafts.csswg.org/css-multicol-2/#propdef-column-wrap
  /// syntax: auto | nowrap | wrap
  /// initial: auto
  ColumnWrap,
  ///
  /// href: https://drafts.csswg.org/css-multicol-2/#propdef-columns
  /// syntax: [ <'column-width'> || <'column-count'> ] [ / <'column-height'> ]?
  /// initial: see individual properties
  /// longhands: [column-width, column-count, column-height]
  Columns,
  ///
  /// href: https://drafts.csswg.org/css-contain-2/#propdef-contain
  /// syntax: none | strict | content | [ [size | inline-size] || layout || style || paint ]
  /// initial: none
  Contain,
  ///
  /// href: https://drafts.csswg.org/css-sizing-4/#propdef-contain-intrinsic-block-size
  /// syntax: auto? [ none | <length [0,∞]> ]
  /// initial: none
  ContainIntrinsicBlockSize,
  ///
  /// href: https://drafts.csswg.org/css-sizing-4/#propdef-contain-intrinsic-height
  /// syntax: auto? [ none | <length [0,∞]> ]
  /// initial: none
  ContainIntrinsicHeight,
  ///
  /// href: https://drafts.csswg.org/css-sizing-4/#propdef-contain-intrinsic-inline-size
  /// syntax: auto? [ none | <length [0,∞]> ]
  /// initial: none
  ContainIntrinsicInlineSize,
  ///
  /// href: https://drafts.csswg.org/css-sizing-4/#propdef-contain-intrinsic-size
  /// syntax: [ auto? [ none | <length [0,∞]> ] ]{1,2}
  /// initial: see individual properties
  /// longhands: [contain-intrinsic-width, contain-intrinsic-height]
  ContainIntrinsicSize,
  ///
  /// href: https://drafts.csswg.org/css-sizing-4/#propdef-contain-intrinsic-width
  /// syntax: auto? [ none | <length [0,∞]> ]
  /// initial: none
  ContainIntrinsicWidth,
  ///
  /// href: https://drafts.csswg.org/css-conditional-5/#propdef-container
  /// syntax: <'container-name'> [ / <'container-type'> ]?
  /// initial: see individual properties
  /// longhands: [container-name, container-type]
  Container,
  ///
  /// href: https://drafts.csswg.org/css-conditional-5/#propdef-container-name
  /// syntax: none | <custom-ident>+
  /// initial: none
  ContainerName,
  ///
  /// href: https://drafts.csswg.org/css-conditional-5/#propdef-container-type
  /// syntax: normal | [ [ size | inline-size ] || scroll-state ]
  /// initial: normal
  ContainerType,
  ///
  /// href: https://drafts.csswg.org/css-content-3/#propdef-content
  /// syntax: normal | none | [ <content-replacement> | <content-list> ] [/ [ <string> | <counter> | <attr()> ]+ ]? |
  /// <element()> initial: normal
  Content,
  ///
  /// href: https://drafts.csswg.org/css-contain-2/#propdef-content-visibility
  /// syntax: visible | auto | hidden
  /// initial: visible
  ContentVisibility,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-continue
  /// syntax: auto | discard | collapse | -webkit-legacy | overflow | paginate | fragments
  /// initial: auto
  Continue,
  ///
  /// href: https://drafts.csswg.org/css-gcpm-4/#propdef-copy-into
  /// syntax: none | [ [ <custom-ident> <content-level>] [, <custom-ident> <content-level>]* ]?
  /// initial: none
  CopyInto,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner
  /// syntax: <'border-radius'> || <'corner-shape'>
  /// initial: 0
  /// longhands: [border-radius, corner-shape]
  Corner,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-block-end
  /// syntax: <'border-top-radius'> || <'corner-top-shape'>
  /// initial: 0
  /// longhands: [border-block-end-radius, corner-block-end-shape]
  CornerBlockEnd,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-block-end-shape
  /// syntax: <'corner-top-left-shape'>{1,2}
  /// initial: see individual properties
  /// longhands: [corner-end-start-shape, corner-end-end-shape]
  CornerBlockEndShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-block-start
  /// syntax: <'border-top-radius'> || <'corner-top-shape'>
  /// initial: 0
  /// longhands: [border-block-start-radius, corner-block-start-shape]
  CornerBlockStart,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-block-start-shape
  /// syntax: <'corner-top-left-shape'>{1,2}
  /// initial: see individual properties
  /// longhands: [corner-start-start-shape, corner-start-end-shape]
  CornerBlockStartShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-bottom
  /// syntax: <'border-top-radius'> || <'corner-top-shape'>
  /// initial: 0
  /// longhands: [border-bottom-radius, corner-bottom-shape]
  CornerBottom,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-bottom-left
  /// syntax: <'border-top-left-radius'> || <'corner-top-left-shape'>
  /// initial: 0
  /// longhands: [border-bottom-left-radius, corner-bottom-left-shape]
  CornerBottomLeft,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-bottom-left-shape
  /// syntax: <corner-shape-value>
  /// initial: round
  CornerBottomLeftShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-bottom-right
  /// syntax: <'border-top-left-radius'> || <'corner-top-left-shape'>
  /// initial: 0
  /// longhands: [border-bottom-right-radius, corner-bottom-right-shape]
  CornerBottomRight,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-bottom-right-shape
  /// syntax: <corner-shape-value>
  /// initial: round
  CornerBottomRightShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-bottom-shape
  /// syntax: <'corner-top-left-shape'>{1,2}
  /// initial: see individual properties
  /// longhands: [corner-bottom-left-shape, corner-bottom-right-shape]
  CornerBottomShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-end-end
  /// syntax: <'border-top-left-radius'> || <'corner-top-left-shape'>
  /// initial: 0
  /// longhands: [border-end-end-radius, corner-end-end-shape]
  CornerEndEnd,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-end-end-shape
  /// syntax: <corner-shape-value>
  /// initial: round
  CornerEndEndShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-end-start
  /// syntax: <'border-top-left-radius'> || <'corner-top-left-shape'>
  /// initial: 0
  /// longhands: [border-end-start-radius, corner-end-start-shape]
  CornerEndStart,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-end-start-shape
  /// syntax: <corner-shape-value>
  /// initial: round
  CornerEndStartShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-inline-end
  /// syntax: <'border-top-radius'> || <'corner-top-shape'>
  /// initial: 0
  /// longhands: [border-inline-end-radius, corner-inline-end-shape]
  CornerInlineEnd,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-inline-end-shape
  /// syntax: <'corner-top-left-shape'>{1,2}
  /// initial: see individual properties
  /// longhands: [corner-start-end-shape, corner-end-end-shape]
  CornerInlineEndShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-inline-start
  /// syntax: <'border-top-radius'> || <'corner-top-shape'>
  /// initial: 0
  /// longhands: [border-inline-start-radius, corner-inline-start-shape]
  CornerInlineStart,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-inline-start-shape
  /// syntax: <'corner-top-left-shape'>{1,2}
  /// initial: see individual properties
  /// longhands: [corner-start-start-shape, corner-end-start-shape]
  CornerInlineStartShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-left
  /// syntax: <'border-top-radius'> || <'corner-top-shape'>
  /// initial: 0
  /// longhands: [border-left-radius, corner-left-shape]
  CornerLeft,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-left-shape
  /// syntax: <'corner-top-left-shape'>{1,2}
  /// initial: see individual properties
  /// longhands: [corner-top-left-shape, corner-bottom-left-shape]
  CornerLeftShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-right
  /// syntax: <'border-top-radius'> || <'corner-top-shape'>
  /// initial: 0
  /// longhands: [border-right-radius, corner-right-shape]
  CornerRight,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-right-shape
  /// syntax: <'corner-top-left-shape'>{1,2}
  /// initial: see individual properties
  /// longhands: [corner-top-right-shape, corner-bottom-right-shape]
  CornerRightShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-shape
  /// syntax: <'corner-top-left-shape'>{1,4}
  /// initial: round
  /// longhands: [corner-top-left-shape, corner-top-right-shape, corner-bottom-right-shape, corner-bottom-left-shape]
  CornerShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-start-end
  /// syntax: <'border-top-left-radius'> || <'corner-top-left-shape'>
  /// initial: 0
  /// longhands: [border-start-end-radius, corner-start-end-shape]
  CornerStartEnd,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-start-end-shape
  /// syntax: <corner-shape-value>
  /// initial: round
  CornerStartEndShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-start-start
  /// syntax: <'border-top-left-radius'> || <'corner-top-left-shape'>
  /// initial: 0
  /// longhands: [border-start-start-radius, corner-start-start-shape]
  CornerStartStart,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-start-start-shape
  /// syntax: <corner-shape-value>
  /// initial: round
  CornerStartStartShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-top
  /// syntax: <'border-top-radius'> || <'corner-top-shape'>
  /// initial: 0
  /// longhands: [border-top-radius, corner-top-shape]
  CornerTop,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-top-left
  /// syntax: <'border-top-left-radius'> || <'corner-top-left-shape'>
  /// initial: 0
  /// longhands: [border-top-left-radius, corner-top-left-shape]
  CornerTopLeft,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-top-left-shape
  /// syntax: <corner-shape-value>
  /// initial: round
  CornerTopLeftShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-top-right
  /// syntax: <'border-top-left-radius'> || <'corner-top-left-shape'>
  /// initial: 0
  /// longhands: [border-top-right-radius, corner-top-right-shape]
  CornerTopRight,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-top-right-shape
  /// syntax: <corner-shape-value>
  /// initial: round
  CornerTopRightShape,
  ///
  /// href: https://drafts.csswg.org/css-borders-4/#propdef-corner-top-shape
  /// syntax: <'corner-top-left-shape'>{1,2}
  /// initial: see individual properties
  /// longhands: [corner-top-left-shape, corner-top-right-shape]
  CornerTopShape,
  ///
  /// href: https://drafts.csswg.org/css-lists-3/#propdef-counter-increment
  /// syntax: [ <counter-name> <integer>? ]+ | none
  /// initial: none
  CounterIncrement,
  ///
  /// href: https://drafts.csswg.org/css-lists-3/#propdef-counter-reset
  /// syntax: [ <counter-name> <integer>? | <reversed-counter-name> <integer>? ]+ | none
  /// initial: none
  CounterReset,
  ///
  /// href: https://drafts.csswg.org/css-lists-3/#propdef-counter-set
  /// syntax: [ <counter-name> <integer>? ]+ | none
  /// initial: none
  CounterSet,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-cue
  /// syntax: <'cue-before'> <'cue-after'>?
  /// initial: see individual properties
  /// longhands: [cue-before, cue-after]
  Cue,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-cue-after
  /// syntax: <url> <decibel>? | none
  /// initial: none
  CueAfter,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-cue-before
  /// syntax: <url> <decibel>? | none
  /// initial: none
  CueBefore,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-cursor
  /// syntax: [<cursor-image>,]* <cursor-predefined>
  /// initial: auto
  /// inherited: yes
  Cursor,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/geometry.html#CxProperty
  /// syntax: <length-percentage>
  /// initial: 0
  Cx,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/geometry.html#CyProperty
  /// syntax: <length-percentage>
  /// initial: 0
  Cy,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/paths.html#DProperty
  /// syntax: none | <string>
  /// initial: none
  D,
  ///
  /// href: https://drafts.csswg.org/css-writing-modes-4/#propdef-direction
  /// syntax: ltr | rtl
  /// initial: ltr
  /// inherited: yes
  Direction,
  ///
  /// href: https://drafts.csswg.org/css-display-4/#propdef-display
  /// syntax: [ <display-outside> || <display-inside> ] | <display-listitem> | <display-internal> | <display-box> |
  /// <display-legacy> | grid-lanes | inline-grid-lanes | <display-outside> || [ <display-inside> | math ]
  /// initial: inline
  Display,
  ///
  /// href: https://drafts.csswg.org/css-inline-3/#propdef-dominant-baseline
  /// syntax: auto | <baseline-metric>
  /// initial: auto
  /// inherited: yes
  DominantBaseline,
  ///
  /// href: https://drafts.csswg.org/css-color-hdr-1/#propdef-dynamic-range-limit
  /// syntax: standard | no-limit | constrained | <dynamic-range-limit-mix()>
  /// initial: no-limit
  /// inherited: yes
  DynamicRangeLimit,
  ///
  /// href: https://drafts.csswg.org/css-tables-3/#propdef-empty-cells
  /// syntax: show | hide
  /// initial: show
  /// inherited: yes
  EmptyCells,
  ///
  /// href: https://drafts.csswg.org/animation-triggers-1/#propdef-event-trigger
  /// syntax: none | [ <'event-trigger-name'> <'event-trigger-source'> ]#
  /// initial: none
  EventTrigger,
  ///
  /// href: https://drafts.csswg.org/animation-triggers-1/#propdef-event-trigger-name
  /// syntax: none | <dashed-ident>#
  /// initial: none
  EventTriggerName,
  ///
  /// href: https://drafts.csswg.org/animation-triggers-1/#propdef-event-trigger-source
  /// syntax: [ none | <event-trigger-event>+ [ / <event-trigger-event>+ ]? ]#
  /// initial: none
  EventTriggerSource,
  ///
  /// href: https://drafts.csswg.org/css-forms-1/#propdef-field-sizing
  /// syntax: fixed | content
  /// initial: fixed
  FieldSizing,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/painting.html#FillProperty
  /// syntax: <paint>
  /// initial: black
  /// inherited: yes
  Fill,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-fill-break
  /// syntax: bounding-box | slice | clone
  /// initial: bounding-box
  FillBreak,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-fill-color
  /// syntax: <color>
  /// initial: currentcolor
  /// inherited: yes
  FillColor,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-fill-image
  /// syntax: <paint>#
  /// initial: none
  /// inherited: yes
  FillImage,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-fill-opacity
  /// syntax: <'opacity'>
  /// initial: 1
  /// inherited: yes
  FillOpacity,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-fill-origin
  /// syntax: match-parent | fill-box | stroke-box | content-box | padding-box | border-box
  /// initial: match-parent
  FillOrigin,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-fill-position
  /// syntax: <position>#
  /// initial: 0% 0%
  /// inherited: yes
  FillPosition,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-fill-repeat
  /// syntax: <repeat-style>#
  /// initial: repeat
  /// inherited: yes
  FillRepeat,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-fill-rule
  /// syntax: nonzero | evenodd
  /// initial: nonzero
  /// inherited: yes
  FillRule,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-fill-size
  /// syntax: <bg-size>#
  /// initial: auto
  /// inherited: yes
  FillSize,
  ///
  /// href: https://drafts.csswg.org/filter-effects-1/#propdef-filter
  /// syntax: none | <filter-value-list>
  /// initial: none
  Filter,
  ///
  /// href: https://drafts.csswg.org/css-flexbox-1/#propdef-flex
  /// syntax: none | [ <'flex-grow'> <'flex-shrink'>? || <'flex-basis'> ]
  /// initial: 0 1 auto
  /// longhands: [flex-grow, flex-shrink, flex-basis]
  Flex,
  ///
  /// href: https://drafts.csswg.org/css-flexbox-1/#propdef-flex-basis
  /// syntax: content | <'width'>
  /// initial: auto
  FlexBasis,
  ///
  /// href: https://drafts.csswg.org/css-flexbox-1/#propdef-flex-direction
  /// syntax: row | row-reverse | column | column-reverse
  /// initial: row
  FlexDirection,
  ///
  /// href: https://drafts.csswg.org/css-flexbox-1/#propdef-flex-flow
  /// syntax: <'flex-direction'> || <'flex-wrap'>
  /// initial: see individual properties
  /// longhands: [flex-direction, flex-wrap]
  FlexFlow,
  ///
  /// href: https://drafts.csswg.org/css-flexbox-1/#propdef-flex-grow
  /// syntax: <number [0,∞]>
  /// initial: 0
  FlexGrow,
  ///
  /// href: https://drafts.csswg.org/css-flexbox-1/#propdef-flex-shrink
  /// syntax: <number [0,∞]>
  /// initial: 1
  FlexShrink,
  ///
  /// href: https://drafts.csswg.org/css-flexbox-1/#propdef-flex-wrap
  /// syntax: nowrap | wrap | wrap-reverse
  /// initial: nowrap
  FlexWrap,
  ///
  /// href: https://drafts.csswg.org/css-page-floats-3/#propdef-float
  /// syntax: block-start | block-end | inline-start | inline-end | snap-block | <snap-block()> | snap-inline |
  /// <snap-inline()> | left | right | top | bottom | none | footnote initial: none
  Float,
  ///
  /// href: https://drafts.csswg.org/css-page-floats-3/#propdef-float-defer
  /// syntax: <integer> | last | none
  /// initial: none
  FloatDefer,
  ///
  /// href: https://drafts.csswg.org/css-page-floats-3/#propdef-float-offset
  /// syntax: <length-percentage>
  /// initial: 0
  FloatOffset,
  ///
  /// href: https://drafts.csswg.org/css-page-floats-3/#propdef-float-reference
  /// syntax: inline | column | region | page
  /// initial: inline
  FloatReference,
  ///
  /// href: https://drafts.csswg.org/filter-effects-1/#propdef-flood-color
  /// syntax: <color>
  /// initial: black
  FloodColor,
  ///
  /// href: https://drafts.csswg.org/filter-effects-1/#propdef-flood-opacity
  /// syntax: <'opacity'>
  /// initial: 1
  FloodOpacity,
  ///
  /// href: https://drafts.csswg.org/css-regions-1/#propdef-flow-from
  /// syntax: <custom-ident> | none
  /// initial: none
  FlowFrom,
  ///
  /// href: https://drafts.csswg.org/css-regions-1/#propdef-flow-into
  /// syntax: none | <custom-ident> [element | content]?
  /// initial: none
  FlowInto,
  ///
  /// href: https://drafts.csswg.org/css-grid-3/#propdef-flow-tolerance
  /// syntax: normal | <length-percentage> | infinite
  /// initial: normal
  FlowTolerance,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font
  /// syntax: [ [ <'font-style'> || <font-variant-css2> || <'font-weight'> || <font-width-css3> ]? <'font-size'> [ /
  /// <'line-height'> ]? <'font-family'># ] | <system-font-family-name> initial: see individual properties
  /// inherited: yes
  /// longhands: [font-style, font-variant, font-weight, font-stretch, font-size, line-height, font-family]
  Font,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-family
  /// syntax: [ <font-family-name> | <generic-font-family> ]#
  /// initial: depends on user agent
  /// inherited: yes
  FontFamily,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-feature-settings
  /// syntax: normal | <feature-tag-value>#
  /// initial: normal
  /// inherited: yes
  FontFeatureSettings,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-kerning
  /// syntax: auto | normal | none
  /// initial: auto
  /// inherited: yes
  FontKerning,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-language-override
  /// syntax: normal | <string>
  /// initial: normal
  /// inherited: yes
  FontLanguageOverride,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-optical-sizing
  /// syntax: auto | none
  /// initial: auto
  /// inherited: yes
  FontOpticalSizing,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-palette
  /// syntax: normal | light | dark | <palette-identifier> | <palette-mix()>
  /// initial: normal
  /// inherited: yes
  FontPalette,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-size
  /// syntax: <absolute-size> | <relative-size> | <length-percentage [0,∞]> | math
  /// initial: medium
  /// inherited: yes
  FontSize,
  ///
  /// href: https://drafts.csswg.org/css-fonts-5/#propdef-font-size-adjust
  /// syntax: none | [ ex-height | cap-height | ch-width | ic-width | ic-height ]? [ from-font | <number [0,∞]> ]
  /// initial: none
  /// inherited: yes
  FontSizeAdjust,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-stretch
  /// syntax: normal | <percentage [0,∞]> | ultra-condensed | extra-condensed | condensed | semi-condensed |
  /// semi-expanded | expanded | extra-expanded | ultra-expanded legacy_alias_of: font-width
  FontStretch,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-style
  /// syntax: normal | italic | left | right | oblique <angle [-90deg,90deg]>?
  /// initial: normal
  /// inherited: yes
  FontStyle,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-synthesis
  /// syntax: none | [ weight || style || small-caps || position]
  /// initial: weight style small-caps position
  /// inherited: yes
  /// longhands: [font-synthesis-weight, font-synthesis-style, font-synthesis-small-caps]
  FontSynthesis,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-synthesis-position
  /// syntax: auto | none
  /// initial: auto
  /// inherited: yes
  FontSynthesisPosition,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-synthesis-small-caps
  /// syntax: auto | none
  /// initial: auto
  /// inherited: yes
  FontSynthesisSmallCaps,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-synthesis-style
  /// syntax: auto | none | oblique-only
  /// initial: auto
  /// inherited: yes
  FontSynthesisStyle,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-synthesis-weight
  /// syntax: auto | none
  /// initial: auto
  /// inherited: yes
  FontSynthesisWeight,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-variant
  /// syntax: normal | none | [ [ <common-lig-values> || <discretionary-lig-values> || <historical-lig-values> ||
  /// <contextual-alt-values> ] || [ small-caps | all-small-caps | petite-caps | all-petite-caps | unicase |
  /// titling-caps ] || [ stylistic(<font-feature-value-name>) || historical-forms ||
  /// styleset(<font-feature-value-name>#) || character-variant(<font-feature-value-name>#) ||
  /// swash(<font-feature-value-name>) || ornaments(<font-feature-value-name>) || annotation(<font-feature-value-name>)
  /// ] || [ <numeric-figure-values> || <numeric-spacing-values> || <numeric-fraction-values> || ordinal || slashed-zero
  /// ] || [ <east-asian-variant-values> || <east-asian-width-values> || ruby ] || [ sub | super ] || [ text | emoji |
  /// unicode ] ] initial: normal
  /// inherited: yes
  /// longhands: [font-variant-ligatures, font-variant-caps, font-variant-alternates, font-variant-numeric,
  /// font-variant-east-asian, font-variant-position, font-variant-emoji]
  FontVariant,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-variant-alternates
  /// syntax: normal | [ stylistic(<font-feature-value-name>) || historical-forms ||
  /// styleset(<font-feature-value-name>#) || character-variant(<font-feature-value-name>#) ||
  /// swash(<font-feature-value-name>) || ornaments(<font-feature-value-name>) || annotation(<font-feature-value-name>)
  /// ] initial: normal
  /// inherited: yes
  FontVariantAlternates,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-variant-caps
  /// syntax: normal | small-caps | all-small-caps | petite-caps | all-petite-caps | unicase | titling-caps
  /// initial: normal
  /// inherited: yes
  FontVariantCaps,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-variant-east-asian
  /// syntax: normal | [ <east-asian-variant-values> || <east-asian-width-values> || ruby ]
  /// initial: normal
  /// inherited: yes
  FontVariantEastAsian,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-variant-emoji
  /// syntax: normal | text | emoji | unicode
  /// initial: normal
  /// inherited: yes
  FontVariantEmoji,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-variant-ligatures
  /// syntax: normal | none | [ <common-lig-values> || <discretionary-lig-values> || <historical-lig-values> ||
  /// <contextual-alt-values> ] initial: normal
  /// inherited: yes
  FontVariantLigatures,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-variant-numeric
  /// syntax: normal | [ <numeric-figure-values> || <numeric-spacing-values> || <numeric-fraction-values> || ordinal ||
  /// slashed-zero ] initial: normal
  /// inherited: yes
  FontVariantNumeric,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-variant-position
  /// syntax: normal | sub | super
  /// initial: normal
  /// inherited: yes
  FontVariantPosition,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-variation-settings
  /// syntax: normal | [ <opentype-tag> <number> ]#
  /// initial: normal
  /// inherited: yes
  FontVariationSettings,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-weight
  /// syntax: <font-weight-absolute> | bolder | lighter
  /// initial: normal
  /// inherited: yes
  FontWeight,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#propdef-font-width
  /// syntax: normal | <percentage [0,∞]> | ultra-condensed | extra-condensed | condensed | semi-condensed |
  /// semi-expanded | expanded | extra-expanded | ultra-expanded initial: normal
  /// inherited: yes
  FontWidth,
  ///
  /// href: https://drafts.csswg.org/css-gcpm-3/#propdef-footnote-display
  /// syntax: block | inline | compact
  /// initial: block
  FootnoteDisplay,
  ///
  /// href: https://drafts.csswg.org/css-gcpm-3/#propdef-footnote-policy
  /// syntax: auto | line | block
  /// initial: auto
  FootnotePolicy,
  ///
  /// href: https://drafts.csswg.org/css-color-adjust-1/#propdef-forced-color-adjust
  /// syntax: auto | none | preserve-parent-color
  /// initial: auto
  /// inherited: yes
  ForcedColorAdjust,
  ///
  /// href: https://drafts.csswg.org/css-sizing-4/#propdef-frame-sizing
  /// syntax: auto | content-width | content-height | content-block-size | content-inline-size
  /// initial: auto
  FrameSizing,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-gap
  /// syntax: <'row-gap'> <'column-gap'>?
  /// initial: see individual properties
  /// longhands: [row-gap, column-gap]
  Gap,
  ///
  /// href: https://drafts.csswg.org/css-writing-modes-4/#propdef-glyph-orientation-vertical
  /// syntax: auto | 0deg | 90deg | 0 | 90
  /// initial: n/a
  GlyphOrientationVertical,
  ///
  /// href: https://drafts.csswg.org/css-grid-2/#propdef-grid
  /// syntax: <'grid-template'> | <'grid-template-rows'> / [ auto-flow && dense? ] <'grid-auto-columns'>? | [ auto-flow
  /// && dense? ] <'grid-auto-rows'>? / <'grid-template-columns'> initial: none
  /// longhands: [grid-template-rows, grid-template-columns, grid-template-areas, grid-auto-rows, grid-auto-columns,
  /// grid-auto-flow]
  Grid,
  ///
  /// href: https://drafts.csswg.org/css-grid-2/#propdef-grid-area
  /// syntax: <grid-line> [ / <grid-line> ]{0,3}
  /// initial: auto
  /// longhands: [grid-row-start, grid-column-start, grid-row-end, grid-column-end]
  GridArea,
  ///
  /// href: https://drafts.csswg.org/css-grid-2/#propdef-grid-auto-columns
  /// syntax: <track-size>+
  /// initial: auto
  GridAutoColumns,
  ///
  /// href: https://drafts.csswg.org/css-grid-2/#propdef-grid-auto-flow
  /// syntax: [ row | column ] || dense
  /// initial: row
  GridAutoFlow,
  ///
  /// href: https://drafts.csswg.org/css-grid-2/#propdef-grid-auto-rows
  /// syntax: <track-size>+
  /// initial: auto
  GridAutoRows,
  ///
  /// href: https://drafts.csswg.org/css-grid-2/#propdef-grid-column
  /// syntax: <grid-line> [ / <grid-line> ]?
  /// initial: auto
  /// longhands: [grid-column-start, grid-column-end]
  GridColumn,
  ///
  /// href: https://drafts.csswg.org/css-grid-2/#propdef-grid-column-end
  /// syntax: <grid-line>
  /// initial: auto
  GridColumnEnd,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-grid-column-gap
  /// syntax: normal | <length-percentage [0,∞]> | <line-width>
  /// legacy_alias_of: column-gap
  GridColumnGap,
  ///
  /// href: https://drafts.csswg.org/css-grid-2/#propdef-grid-column-start
  /// syntax: <grid-line>
  /// initial: auto
  GridColumnStart,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-grid-gap
  /// syntax: <'row-gap'> <'column-gap'>?
  /// longhands: [row-gap, column-gap]
  /// legacy_alias_of: gap
  GridGap,
  ///
  /// href: https://drafts.csswg.org/css-grid-2/#propdef-grid-row
  /// syntax: <grid-line> [ / <grid-line> ]?
  /// initial: auto
  /// longhands: [grid-row-start, grid-row-end]
  GridRow,
  ///
  /// href: https://drafts.csswg.org/css-grid-2/#propdef-grid-row-end
  /// syntax: <grid-line>
  /// initial: auto
  GridRowEnd,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-grid-row-gap
  /// syntax: normal | <length-percentage [0,∞]> | <line-width>
  /// legacy_alias_of: row-gap
  GridRowGap,
  ///
  /// href: https://drafts.csswg.org/css-grid-2/#propdef-grid-row-start
  /// syntax: <grid-line>
  /// initial: auto
  GridRowStart,
  ///
  /// href: https://drafts.csswg.org/css-grid-2/#propdef-grid-template
  /// syntax: none | [ <'grid-template-rows'> / <'grid-template-columns'> ] | [ <line-names>? <string> <track-size>?
  /// <line-names>? ]+ [ / <explicit-track-list> ]? initial: none
  /// longhands: [grid-template-rows, grid-template-columns, grid-template-areas]
  GridTemplate,
  ///
  /// href: https://drafts.csswg.org/css-grid-2/#propdef-grid-template-areas
  /// syntax: none | <string>+
  /// initial: none
  GridTemplateAreas,
  ///
  /// href: https://drafts.csswg.org/css-grid-2/#propdef-grid-template-columns
  /// syntax: none | <track-list> | <auto-track-list> | subgrid <line-name-list>?
  /// initial: none
  GridTemplateColumns,
  ///
  /// href: https://drafts.csswg.org/css-grid-2/#propdef-grid-template-rows
  /// syntax: none | <track-list> | <auto-track-list> | subgrid <line-name-list>?
  /// initial: none
  GridTemplateRows,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-hanging-punctuation
  /// syntax: none | [ first || [ force-end | allow-end ] || last ]
  /// initial: none
  /// inherited: yes
  HangingPunctuation,
  ///
  /// href: https://drafts.csswg.org/css-sizing-3/#propdef-height
  /// syntax: auto | <length-percentage [0,∞]> | min-content | max-content | fit-content(<length-percentage [0,∞]>) |
  /// <calc-size()> | <anchor-size()> | stretch | fit-content | contain initial: auto
  Height,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-hyphenate-character
  /// syntax: auto | <string>
  /// initial: auto
  /// inherited: yes
  HyphenateCharacter,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-hyphenate-limit-chars
  /// syntax: [ auto | <integer> ]{1,3}
  /// initial: auto
  /// inherited: yes
  HyphenateLimitChars,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-hyphenate-limit-last
  /// syntax: none | always | column | page | spread
  /// initial: none
  /// inherited: yes
  HyphenateLimitLast,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-hyphenate-limit-lines
  /// syntax: no-limit | <integer>
  /// initial: no-limit
  /// inherited: yes
  HyphenateLimitLines,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-hyphenate-limit-zone
  /// syntax: <length-percentage>
  /// initial: 0
  /// inherited: yes
  HyphenateLimitZone,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-hyphens
  /// syntax: none | manual | auto
  /// initial: manual
  /// inherited: yes
  Hyphens,
  ///
  /// href: https://drafts.csswg.org/css-image-animation-1/#propdef-image-animation
  /// syntax: normal | paused | stopped | running
  /// initial: normal
  /// inherited: yes
  ImageAnimation,
  ///
  /// href: https://drafts.csswg.org/css-images-3/#propdef-image-orientation
  /// syntax: from-image | none | [ <angle> || flip ]
  /// initial: from-image
  /// inherited: yes
  ImageOrientation,
  ///
  /// href: https://drafts.csswg.org/css-images-3/#propdef-image-rendering
  /// syntax: auto | smooth | high-quality | pixelated | crisp-edges
  /// initial: auto
  /// inherited: yes
  ImageRendering,
  ///
  /// href: https://drafts.csswg.org/css-images-4/#propdef-image-resolution
  /// syntax: [ from-image || <resolution> ] && snap?
  /// initial: 1dppx
  /// inherited: yes
  ImageResolution,
  ///
  /// href: https://drafts.csswg.org/css-inline-3/#propdef-initial-letter
  /// syntax: normal | <number [1,∞]> <integer [1,∞]> | <number [1,∞]> && [ drop | raise ]?
  /// initial: normal
  InitialLetter,
  ///
  /// href: https://drafts.csswg.org/css-inline-3/#propdef-initial-letter-align
  /// syntax: [ border-box? [ alphabetic | ideographic | hanging | leading ]? ]!
  /// initial: alphabetic
  /// inherited: yes
  InitialLetterAlign,
  ///
  /// href: https://drafts.csswg.org/css-inline-3/#propdef-initial-letter-wrap
  /// syntax: none | first | all | grid | <length-percentage>
  /// initial: none
  /// inherited: yes
  InitialLetterWrap,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-inline-size
  /// syntax: <'width'>
  /// initial: auto
  InlineSize,
  ///
  /// href: https://drafts.csswg.org/css-inline-3/#propdef-inline-sizing
  /// syntax: normal | stretch
  /// initial: normal
  /// inherited: yes
  InlineSizing,
  ///
  /// href: https://drafts.csswg.org/css-forms-1/#propdef-input-security
  /// syntax: auto | none
  /// initial: auto
  InputSecurity,
  ///
  /// href: https://drafts.csswg.org/css-position-3/#propdef-inset
  /// syntax: <'top'>{1,4}
  /// initial: auto
  /// longhands: [top, right, bottom, left]
  Inset,
  ///
  /// href: https://drafts.csswg.org/css-position-3/#propdef-inset-block
  /// syntax: <'top'>{1,2}
  /// initial: auto
  /// longhands: [inset-block-start, inset-block-end]
  InsetBlock,
  ///
  /// href: https://drafts.csswg.org/css-position-3/#propdef-inset-block-end
  /// syntax: auto | <length-percentage>
  /// initial: auto
  InsetBlockEnd,
  ///
  /// href: https://drafts.csswg.org/css-position-3/#propdef-inset-block-start
  /// syntax: auto | <length-percentage>
  /// initial: auto
  InsetBlockStart,
  ///
  /// href: https://drafts.csswg.org/css-position-3/#propdef-inset-inline
  /// syntax: <'top'>{1,2}
  /// initial: auto
  /// longhands: [inset-inline-start, inset-inline-end]
  InsetInline,
  ///
  /// href: https://drafts.csswg.org/css-position-3/#propdef-inset-inline-end
  /// syntax: auto | <length-percentage>
  /// initial: auto
  InsetInlineEnd,
  ///
  /// href: https://drafts.csswg.org/css-position-3/#propdef-inset-inline-start
  /// syntax: auto | <length-percentage>
  /// initial: auto
  InsetInlineStart,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-interactivity
  /// syntax: auto | inert
  /// initial: auto
  /// inherited: yes
  Interactivity,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-interest-delay
  /// syntax: <'interest-delay-start'>{1,2}
  /// initial: see individual properties
  /// longhands: [interest-delay-start, interest-delay-end]
  InterestDelay,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-interest-delay-end
  /// syntax: normal | <time>
  /// initial: normal
  /// inherited: yes
  InterestDelayEnd,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-interest-delay-start
  /// syntax: normal | <time>
  /// initial: normal
  /// inherited: yes
  InterestDelayStart,
  ///
  /// href: https://drafts.csswg.org/css-values-5/#propdef-interpolate-size
  /// syntax: numeric-only | allow-keywords
  /// initial: numeric-only
  /// inherited: yes
  InterpolateSize,
  ///
  /// href: https://drafts.csswg.org/compositing-2/#propdef-isolation
  /// syntax: <isolation-mode>
  /// initial: auto
  Isolation,
  ///
  /// href: https://drafts.csswg.org/css-align-3/#propdef-justify-content
  /// syntax: normal | <content-distribution> | <overflow-position>? [ <content-position> | left | right ]
  /// initial: normal
  JustifyContent,
  ///
  /// href: https://drafts.csswg.org/css-align-3/#propdef-justify-items
  /// syntax: normal | stretch | <baseline-position> | <overflow-position>? [ <self-position> | left | right ] | legacy
  /// | legacy && [ left | right | center ] initial: legacy
  JustifyItems,
  ///
  /// href: https://drafts.csswg.org/css-align-3/#propdef-justify-self
  /// syntax: auto | <overflow-position>? [ normal | <self-position> | left | right ] | stretch | <baseline-position> |
  /// anchor-center initial: auto
  JustifySelf,
  ///
  /// href: https://drafts.csswg.org/css-position-3/#propdef-left
  /// syntax: auto | <length-percentage> | <anchor()> | <anchor-size()>
  /// initial: auto
  Left,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-letter-spacing
  /// syntax: normal | <length-percentage>
  /// initial: normal
  /// inherited: yes
  LetterSpacing,
  ///
  /// href: https://drafts.csswg.org/filter-effects-1/#propdef-lighting-color
  /// syntax: <color>
  /// initial: white
  LightingColor,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-line-break
  /// syntax: auto | loose | normal | strict | anywhere
  /// initial: auto
  /// inherited: yes
  LineBreak,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-line-clamp
  /// syntax: none | [<integer [1,∞]> || <'block-ellipsis'>] -webkit-legacy?
  /// initial: none
  /// longhands: [max-lines, block-ellipsis, continue]
  LineClamp,
  ///
  /// href: https://drafts.csswg.org/css-inline-3/#propdef-line-fit-edge
  /// syntax: leading | <text-edge>
  /// initial: leading
  /// inherited: yes
  LineFitEdge,
  ///
  /// href: https://drafts.csswg.org/css-line-grid-1/#propdef-line-grid
  /// syntax: match-parent | create
  /// initial: match-parent
  LineGrid,
  ///
  /// href: https://drafts.csswg.org/css-inline-3/#propdef-line-height
  /// syntax: normal | <number [0,∞]> | <length-percentage [0,∞]>
  /// initial: normal
  /// inherited: yes
  LineHeight,
  ///
  /// href: https://drafts.csswg.org/css-rhythm-1/#propdef-line-height-step
  /// syntax: <length [0,∞]>
  /// initial: 0
  /// inherited: yes
  LineHeightStep,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-line-padding
  /// syntax: <length>
  /// initial: 0
  /// inherited: yes
  LinePadding,
  ///
  /// href: https://drafts.csswg.org/css-line-grid-1/#propdef-line-snap
  /// syntax: none | baseline | contain
  /// initial: none
  /// inherited: yes
  LineSnap,
  ///
  /// href: https://drafts.csswg.org/css-link-params-1/#propdef-link-parameters
  /// syntax: none | <param()>#
  /// initial: none
  LinkParameters,
  ///
  /// href: https://drafts.csswg.org/css-lists-3/#propdef-list-style
  /// syntax: <'list-style-position'> || <'list-style-image'> || <'list-style-type'>
  /// initial: see individual properties
  /// longhands: [list-style-type, list-style-position, list-style-image]
  ListStyle,
  ///
  /// href: https://drafts.csswg.org/css-lists-3/#propdef-list-style-image
  /// syntax: <image> | none
  /// initial: none
  /// inherited: yes
  ListStyleImage,
  ///
  /// href: https://drafts.csswg.org/css-lists-3/#propdef-list-style-position
  /// syntax: inside | outside
  /// initial: outside
  /// inherited: yes
  ListStylePosition,
  ///
  /// href: https://drafts.csswg.org/css-lists-3/#propdef-list-style-type
  /// syntax: <counter-style> | <string> | none
  /// initial: disc
  /// inherited: yes
  ListStyleType,
  ///
  /// href: https://drafts.csswg.org/css-box-4/#propdef-margin
  /// syntax: <'margin-top'>{1,4}
  /// initial: 0
  /// longhands: [margin-top, margin-right, margin-bottom, margin-left]
  Margin,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-margin-block
  /// syntax: <'margin-top'>{1,2}
  /// initial: see individual properties
  /// longhands: [margin-block-start, margin-block-end]
  MarginBlock,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-margin-block-end
  /// syntax: <'margin-top'>
  /// initial: 0
  MarginBlockEnd,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-margin-block-start
  /// syntax: <'margin-top'>
  /// initial: 0
  MarginBlockStart,
  ///
  /// href: https://drafts.csswg.org/css-box-4/#propdef-margin-bottom
  /// syntax: <length-percentage> | auto | <anchor-size()>
  /// initial: 0
  MarginBottom,
  ///
  /// href: https://drafts.csswg.org/css-break-4/#propdef-margin-break
  /// syntax: auto | keep | discard
  /// initial: auto
  MarginBreak,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-margin-inline
  /// syntax: <'margin-top'>{1,2}
  /// initial: see individual properties
  /// longhands: [margin-inline-start, margin-inline-end]
  MarginInline,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-margin-inline-end
  /// syntax: <'margin-top'>
  /// initial: 0
  MarginInlineEnd,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-margin-inline-start
  /// syntax: <'margin-top'>
  /// initial: 0
  MarginInlineStart,
  ///
  /// href: https://drafts.csswg.org/css-box-4/#propdef-margin-left
  /// syntax: <length-percentage> | auto | <anchor-size()>
  /// initial: 0
  MarginLeft,
  ///
  /// href: https://drafts.csswg.org/css-box-4/#propdef-margin-right
  /// syntax: <length-percentage> | auto | <anchor-size()>
  /// initial: 0
  MarginRight,
  ///
  /// href: https://drafts.csswg.org/css-box-4/#propdef-margin-top
  /// syntax: <length-percentage> | auto | <anchor-size()>
  /// initial: 0
  MarginTop,
  ///
  /// href: https://drafts.csswg.org/css-box-4/#propdef-margin-trim
  /// syntax: none | [ block || inline ] | [ block-start || inline-start || block-end || inline-end ]
  /// initial: none
  MarginTrim,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/painting.html#MarkerProperty
  /// syntax: none | <marker-ref>
  /// initial: not defined for shorthand properties
  /// inherited: yes
  /// longhands: [marker-start, marker-mid, marker-end]
  Marker,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/painting.html#MarkerEndProperty
  /// syntax: none | <marker-ref>
  /// initial: none
  /// inherited: yes
  MarkerEnd,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/painting.html#MarkerMidProperty
  /// syntax: none | <marker-ref>
  /// initial: none
  /// inherited: yes
  MarkerMid,
  ///
  /// href: https://drafts.csswg.org/css-lists-3/#propdef-marker-side
  /// syntax: match-self | match-parent
  /// initial: match-self
  /// inherited: yes
  MarkerSide,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/painting.html#MarkerStartProperty
  /// syntax: none | <marker-ref>
  /// initial: none
  /// inherited: yes
  MarkerStart,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask
  /// syntax: <mask-layer>#
  /// initial: see individual properties
  /// longhands: [mask-image, mask-position, mask-size, mask-repeat, mask-origin, mask-clip, mask-composite, mask-mode]
  Mask,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-border
  /// syntax: <'mask-border-source'> || <'mask-border-slice'> [ / <'mask-border-width'>? [ / <'mask-border-outset'> ]?
  /// ]? || <'mask-border-repeat'> || <'mask-border-mode'> initial: See individual properties
  /// longhands: [mask-border-source, mask-border-slice, mask-border-width, mask-border-outset, mask-border-repeat,
  /// mask-border-mode]
  MaskBorder,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-border-mode
  /// syntax: luminance | alpha
  /// initial: alpha
  MaskBorderMode,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-border-outset
  /// syntax: <'border-image-outset'>
  /// initial: 0
  MaskBorderOutset,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-border-repeat
  /// syntax: <'border-image-repeat'>
  /// initial: stretch
  MaskBorderRepeat,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-border-slice
  /// syntax: <'border-image-slice'>
  /// initial: 0
  MaskBorderSlice,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-border-source
  /// syntax: <'border-image-source'>
  /// initial: none
  MaskBorderSource,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-border-width
  /// syntax: <'border-image-width'>
  /// initial: auto
  MaskBorderWidth,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-clip
  /// syntax: [ <coord-box> | no-clip ]#
  /// initial: border-box
  MaskClip,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-composite
  /// syntax: <compositing-operator>#
  /// initial: add
  MaskComposite,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-image
  /// syntax: <mask-reference>#
  /// initial: none
  MaskImage,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-mode
  /// syntax: <masking-mode>#
  /// initial: match-source
  MaskMode,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-origin
  /// syntax: <coord-box>#
  /// initial: border-box
  MaskOrigin,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-position
  /// syntax: <position>#
  /// initial: 0% 0%
  MaskPosition,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-repeat
  /// syntax: <repeat-style>#
  /// initial: repeat
  MaskRepeat,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-size
  /// syntax: <bg-size>#
  /// initial: auto
  MaskSize,
  ///
  /// href: https://drafts.csswg.org/css-masking-1/#propdef-mask-type
  /// syntax: luminance | alpha
  /// initial: luminance
  MaskType,
  ///
  /// href: https://w3c.github.io/mathml-core/#propdef-math-depth
  /// syntax: auto-add | add(<integer>) | <integer>
  /// initial: 0
  /// inherited: yes
  MathDepth,
  ///
  /// href: https://w3c.github.io/mathml-core/#propdef-math-shift
  /// syntax: normal | compact
  /// initial: normal
  /// inherited: yes
  MathShift,
  ///
  /// href: https://w3c.github.io/mathml-core/#propdef-math-style
  /// syntax: normal | compact
  /// initial: normal
  /// inherited: yes
  MathStyle,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-max-block-size
  /// syntax: <'max-width'>
  /// initial: none
  MaxBlockSize,
  ///
  /// href: https://drafts.csswg.org/css-sizing-3/#propdef-max-height
  /// syntax: none | <length-percentage [0,∞]> | min-content | max-content | fit-content(<length-percentage [0,∞]>) |
  /// <calc-size()> | <anchor-size()> | stretch | fit-content | contain initial: none
  MaxHeight,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-max-inline-size
  /// syntax: <'max-width'>
  /// initial: none
  MaxInlineSize,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-max-lines
  /// syntax: none | <integer [1,∞]>
  /// initial: none
  MaxLines,
  ///
  /// href: https://drafts.csswg.org/css-sizing-3/#propdef-max-width
  /// syntax: none | <length-percentage [0,∞]> | min-content | max-content | fit-content(<length-percentage [0,∞]>) |
  /// <calc-size()> | <anchor-size()> | stretch | fit-content | contain initial: none
  MaxWidth,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-min-block-size
  /// syntax: <'min-width'>
  /// initial: 0
  MinBlockSize,
  ///
  /// href: https://drafts.csswg.org/css-sizing-3/#propdef-min-height
  /// syntax: auto | <length-percentage [0,∞]> | min-content | max-content | fit-content(<length-percentage [0,∞]>) |
  /// <calc-size()> | <anchor-size()> | stretch | fit-content | contain initial: auto
  MinHeight,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-min-inline-size
  /// syntax: <'min-width'>
  /// initial: 0
  MinInlineSize,
  ///
  /// href: https://drafts.csswg.org/css-sizing-4/#propdef-min-intrinsic-sizing
  /// syntax: legacy | zero-if-scroll || zero-if-extrinsic
  /// initial: legacy
  MinIntrinsicSizing,
  ///
  /// href: https://drafts.csswg.org/css-sizing-3/#propdef-min-width
  /// syntax: auto | <length-percentage [0,∞]> | min-content | max-content | fit-content(<length-percentage [0,∞]>) |
  /// <calc-size()> | <anchor-size()> | stretch | fit-content | contain initial: auto
  MinWidth,
  ///
  /// href: https://drafts.csswg.org/compositing-2/#propdef-mix-blend-mode
  /// syntax: <blend-mode> | plus-lighter
  /// initial: normal
  MixBlendMode,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-nav-down
  /// syntax: auto | <id> [ current | root | <target-name> ]?
  /// initial: auto
  NavDown,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-nav-left
  /// syntax: auto | <id> [ current | root | <target-name> ]?
  /// initial: auto
  NavLeft,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-nav-right
  /// syntax: auto | <id> [ current | root | <target-name> ]?
  /// initial: auto
  NavRight,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-nav-up
  /// syntax: auto | <id> [ current | root | <target-name> ]?
  /// initial: auto
  NavUp,
  ///
  /// href: https://drafts.csswg.org/css-images-4/#propdef-object-fit
  /// syntax: fill | none | [contain | cover] || scale-down
  /// initial: fill
  ObjectFit,
  ///
  /// href: https://drafts.csswg.org/css-images-3/#propdef-object-position
  /// syntax: <position>
  /// initial: 50% 50%
  ObjectPosition,
  ///
  /// href: https://drafts.csswg.org/css-images-5/#propdef-object-view-box
  /// syntax: none | <basic-shape-rect>
  /// initial: none
  ObjectViewBox,
  ///
  /// href: https://drafts.csswg.org/motion-1/#propdef-offset
  /// syntax: [ <'offset-position'>? [ <'offset-path'> [ <'offset-distance'> || <'offset-rotate'> ]? ]? ]! [ /
  /// <'offset-anchor'> ]? initial: see individual properties
  /// longhands: [offset-path, offset-distance, offset-rotate, offset-anchor, offset-position]
  Offset,
  ///
  /// href: https://drafts.csswg.org/motion-1/#propdef-offset-anchor
  /// syntax: auto | <position>
  /// initial: auto
  OffsetAnchor,
  ///
  /// href: https://drafts.csswg.org/motion-1/#propdef-offset-distance
  /// syntax: <length-percentage>
  /// initial: 0
  OffsetDistance,
  ///
  /// href: https://drafts.csswg.org/motion-1/#propdef-offset-path
  /// syntax: none | <offset-path> || <coord-box>
  /// initial: none
  OffsetPath,
  ///
  /// href: https://drafts.csswg.org/motion-1/#propdef-offset-position
  /// syntax: normal | auto | <position>
  /// initial: normal
  OffsetPosition,
  ///
  /// href: https://drafts.csswg.org/motion-1/#propdef-offset-rotate
  /// syntax: [ auto | reverse ] || <angle>
  /// initial: auto
  OffsetRotate,
  ///
  /// href: https://drafts.csswg.org/css-color-4/#propdef-opacity
  /// syntax: <opacity-value>
  /// initial: 1
  Opacity,
  ///
  /// href: https://drafts.csswg.org/css-display-4/#propdef-order
  /// syntax: <integer>
  /// initial: 0
  Order,
  ///
  /// href: https://drafts.csswg.org/css-break-4/#propdef-orphans
  /// syntax: <integer [1,∞]>
  /// initial: 2
  /// inherited: yes
  Orphans,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-outline
  /// syntax: <'outline-width'> || <'outline-style'> || <'outline-color'>
  /// initial: see individual properties
  /// longhands: [outline-width, outline-style, outline-color]
  Outline,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-outline-color
  /// syntax: auto | <'border-top-color'>
  /// initial: auto
  OutlineColor,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-outline-offset
  /// syntax: <length>
  /// initial: 0
  OutlineOffset,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-outline-style
  /// syntax: auto | <outline-line-style>
  /// initial: none
  OutlineStyle,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-outline-width
  /// syntax: <line-width>
  /// initial: medium
  OutlineWidth,
  ///
  /// href: https://drafts.csswg.org/css-overflow-3/#propdef-overflow
  /// syntax: <'overflow-block'>{1,2}
  /// initial: visible
  /// longhands: [overflow-x, overflow-y]
  Overflow,
  ///
  /// href: https://drafts.csswg.org/css-scroll-anchoring-1/#propdef-overflow-anchor
  /// syntax: auto | none
  /// initial: auto
  OverflowAnchor,
  ///
  /// href: https://drafts.csswg.org/css-overflow-3/#propdef-overflow-block
  /// syntax: visible | hidden | clip | scroll | auto
  /// initial: visible
  OverflowBlock,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-overflow-clip-margin
  /// syntax: <visual-box> || <length>
  /// initial: 0px
  /// longhands: [overflow-clip-margin-top, overflow-clip-margin-right, overflow-clip-margin-bottom,
  /// overflow-clip-margin-left]
  OverflowClipMargin,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-overflow-clip-margin-block
  /// syntax: <visual-box> || <length>
  /// initial: 0px
  /// longhands: [overflow-clip-margin-block-start, overflow-clip-margin-block-end]
  OverflowClipMarginBlock,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-overflow-clip-margin-block-end
  /// syntax: <visual-box> || <length>
  /// initial: 0px
  OverflowClipMarginBlockEnd,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-overflow-clip-margin-block-start
  /// syntax: <visual-box> || <length>
  /// initial: 0px
  OverflowClipMarginBlockStart,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-overflow-clip-margin-bottom
  /// syntax: <visual-box> || <length>
  /// initial: 0px
  OverflowClipMarginBottom,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-overflow-clip-margin-inline
  /// syntax: <visual-box> || <length>
  /// initial: 0px
  /// longhands: [overflow-clip-margin-inline-start, overflow-clip-margin-inline-end]
  OverflowClipMarginInline,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-overflow-clip-margin-inline-end
  /// syntax: <visual-box> || <length>
  /// initial: 0px
  OverflowClipMarginInlineEnd,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-overflow-clip-margin-inline-start
  /// syntax: <visual-box> || <length>
  /// initial: 0px
  OverflowClipMarginInlineStart,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-overflow-clip-margin-left
  /// syntax: <visual-box> || <length>
  /// initial: 0px
  OverflowClipMarginLeft,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-overflow-clip-margin-right
  /// syntax: <visual-box> || <length>
  /// initial: 0px
  OverflowClipMarginRight,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-overflow-clip-margin-top
  /// syntax: <visual-box> || <length>
  /// initial: 0px
  OverflowClipMarginTop,
  ///
  /// href: https://drafts.csswg.org/css-overflow-3/#propdef-overflow-inline
  /// syntax: visible | hidden | clip | scroll | auto
  /// initial: visible
  OverflowInline,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-overflow-wrap
  /// syntax: normal | break-word | anywhere
  /// initial: normal
  /// inherited: yes
  OverflowWrap,
  ///
  /// href: https://drafts.csswg.org/css-overflow-3/#propdef-overflow-x
  /// syntax: visible | hidden | clip | scroll | auto
  /// initial: visible
  OverflowX,
  ///
  /// href: https://drafts.csswg.org/css-overflow-3/#propdef-overflow-y
  /// syntax: visible | hidden | clip | scroll | auto
  /// initial: visible
  OverflowY,
  ///
  /// href: https://drafts.csswg.org/css-position-4/#propdef-overlay
  /// syntax: none | auto
  /// initial: none
  Overlay,
  ///
  /// href: https://drafts.csswg.org/css-overscroll-1/#propdef-overscroll-behavior
  /// syntax: [ contain | none | auto | chain ]{1,2}
  /// initial: auto auto
  /// longhands: [overscroll-behavior-x, overscroll-behavior-y]
  OverscrollBehavior,
  ///
  /// href: https://drafts.csswg.org/css-overscroll-1/#propdef-overscroll-behavior-block
  /// syntax: contain | none | auto | chain
  /// initial: auto
  OverscrollBehaviorBlock,
  ///
  /// href: https://drafts.csswg.org/css-overscroll-1/#propdef-overscroll-behavior-inline
  /// syntax: contain | none | auto | chain
  /// initial: auto
  OverscrollBehaviorInline,
  ///
  /// href: https://drafts.csswg.org/css-overscroll-1/#propdef-overscroll-behavior-x
  /// syntax: contain | none | auto | chain
  /// initial: auto
  OverscrollBehaviorX,
  ///
  /// href: https://drafts.csswg.org/css-overscroll-1/#propdef-overscroll-behavior-y
  /// syntax: contain | none | auto | chain
  /// initial: auto
  OverscrollBehaviorY,
  ///
  /// href: https://drafts.csswg.org/css-box-4/#propdef-padding
  /// syntax: <'padding-top'>{1,4}
  /// initial: 0
  /// longhands: [padding-top, padding-right, padding-bottom, padding-left]
  Padding,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-padding-block
  /// syntax: <'padding-top'>{1,2}
  /// initial: see individual properties
  /// longhands: [padding-block-start, padding-block-end]
  PaddingBlock,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-padding-block-end
  /// syntax: <'padding-top'>
  /// initial: 0
  PaddingBlockEnd,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-padding-block-start
  /// syntax: <'padding-top'>
  /// initial: 0
  PaddingBlockStart,
  ///
  /// href: https://drafts.csswg.org/css-box-4/#propdef-padding-bottom
  /// syntax: <length-percentage [0,∞]>
  /// initial: 0
  PaddingBottom,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-padding-inline
  /// syntax: <'padding-top'>{1,2}
  /// initial: see individual properties
  /// longhands: [padding-inline-start, padding-inline-end]
  PaddingInline,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-padding-inline-end
  /// syntax: <'padding-top'>
  /// initial: 0
  PaddingInlineEnd,
  ///
  /// href: https://drafts.csswg.org/css-logical-1/#propdef-padding-inline-start
  /// syntax: <'padding-top'>
  /// initial: 0
  PaddingInlineStart,
  ///
  /// href: https://drafts.csswg.org/css-box-4/#propdef-padding-left
  /// syntax: <length-percentage [0,∞]>
  /// initial: 0
  PaddingLeft,
  ///
  /// href: https://drafts.csswg.org/css-box-4/#propdef-padding-right
  /// syntax: <length-percentage [0,∞]>
  /// initial: 0
  PaddingRight,
  ///
  /// href: https://drafts.csswg.org/css-box-4/#propdef-padding-top
  /// syntax: <length-percentage [0,∞]>
  /// initial: 0
  PaddingTop,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#propdef-page
  /// syntax: auto | <custom-ident>
  /// initial: auto
  Page,
  ///
  /// href: https://drafts.csswg.org/css2/#propdef-page-break-after
  /// syntax: auto | always | avoid | left | right | inherit
  /// initial: auto
  PageBreakAfter,
  ///
  /// href: https://drafts.csswg.org/css2/#propdef-page-break-before
  /// syntax: auto | always | avoid | left | right | inherit
  /// initial: auto
  PageBreakBefore,
  ///
  /// href: https://drafts.csswg.org/css2/#propdef-page-break-inside
  /// syntax: avoid | auto | inherit
  /// initial: auto
  PageBreakInside,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/painting.html#PaintOrderProperty
  /// syntax: normal | [ fill || stroke || markers ]
  /// initial: normal
  /// inherited: yes
  PaintOrder,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/paths.html#PathLengthCSSProperty
  /// syntax: none | @@ unknown symbol "number [0,∞]"
  /// initial: none
  PathLength,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-pause
  /// syntax: <'pause-before'> <'pause-after'>?
  /// initial: see individual properties
  /// longhands: [pause-before, pause-after]
  Pause,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-pause-after
  /// syntax: <time [0s,∞]> | none | x-weak | weak | medium | strong | x-strong
  /// initial: none
  PauseAfter,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-pause-before
  /// syntax: <time [0s,∞]> | none | x-weak | weak | medium | strong | x-strong
  /// initial: none
  PauseBefore,
  ///
  /// href: https://drafts.csswg.org/css-transforms-2/#propdef-perspective
  /// syntax: none | <length [0,∞]>
  /// initial: none
  Perspective,
  ///
  /// href: https://drafts.csswg.org/css-transforms-2/#propdef-perspective-origin
  /// syntax: <position>
  /// initial: 50% 50%
  PerspectiveOrigin,
  ///
  /// href: https://drafts.csswg.org/css-align-3/#propdef-place-content
  /// syntax: <'align-content'> <'justify-content'>?
  /// initial: normal
  /// longhands: [align-content, justify-content]
  PlaceContent,
  ///
  /// href: https://drafts.csswg.org/css-align-3/#propdef-place-items
  /// syntax: <'align-items'> <'justify-items'>?
  /// initial: see individual properties
  /// longhands: [align-items, justify-items]
  PlaceItems,
  ///
  /// href: https://drafts.csswg.org/css-align-3/#propdef-place-self
  /// syntax: <'align-self'> <'justify-self'>?
  /// initial: auto
  /// longhands: [align-self, justify-self]
  PlaceSelf,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/interact.html#PointerEventsProperty
  /// syntax: auto | bounding-box | visiblePainted | visibleFill | visibleStroke | visible | painted | fill | stroke |
  /// all | none initial: auto
  /// inherited: yes
  PointerEvents,
  ///
  /// href: https://drafts.csswg.org/pointer-animations-1/#propdef-pointer-timeline
  /// syntax: [ <'pointer-timeline-name'> <'pointer-timeline-axis'>? ]#
  /// initial: see individual properties
  /// longhands: [pointer-timeline-name, pointer-timeline-axis]
  PointerTimeline,
  ///
  /// href: https://drafts.csswg.org/pointer-animations-1/#propdef-pointer-timeline-axis
  /// syntax: [ block | inline | x | y ]#
  /// initial: block
  PointerTimelineAxis,
  ///
  /// href: https://drafts.csswg.org/pointer-animations-1/#propdef-pointer-timeline-name
  /// syntax: [ none | <dashed-ident> ]#
  /// initial: none
  PointerTimelineName,
  ///
  /// href: https://drafts.csswg.org/css-position-3/#propdef-position
  /// syntax: static | relative | absolute | sticky | fixed | <running()>
  /// initial: static
  Position,
  ///
  /// href: https://drafts.csswg.org/css-anchor-position-1/#propdef-position-anchor
  /// syntax: normal | none | auto | <anchor-name> | match-parent
  /// initial: normal
  PositionAnchor,
  ///
  /// href: https://drafts.csswg.org/css-anchor-position-1/#propdef-position-area
  /// syntax: none | <position-area>
  /// initial: none
  PositionArea,
  ///
  /// href: https://drafts.csswg.org/css-anchor-position-1/#propdef-position-try
  /// syntax: <'position-try-order'>? <'position-try-fallbacks'>
  /// initial: see individual properties
  /// longhands: [position-try-order, position-try-fallbacks]
  PositionTry,
  ///
  /// href: https://drafts.csswg.org/css-anchor-position-1/#propdef-position-try-fallbacks
  /// syntax: none | [ [<dashed-ident> || <try-tactic>] | <position-area> ]#
  /// initial: none
  PositionTryFallbacks,
  ///
  /// href: https://drafts.csswg.org/css-anchor-position-1/#propdef-position-try-order
  /// syntax: normal | <try-size>
  /// initial: normal
  PositionTryOrder,
  ///
  /// href: https://drafts.csswg.org/css-anchor-position-1/#propdef-position-visibility
  /// syntax: always | [ anchors-valid || anchors-visible || no-overflow ]
  /// initial: anchors-visible
  PositionVisibility,
  ///
  /// href: https://drafts.csswg.org/css-color-adjust-1/#propdef-print-color-adjust
  /// syntax: economy | exact
  /// initial: economy
  /// inherited: yes
  PrintColorAdjust,
  ///
  /// href: https://drafts.csswg.org/css-content-3/#propdef-quotes
  /// syntax: auto | none | match-parent | [ <string> <string> ]+
  /// initial: auto
  /// inherited: yes
  Quotes,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/geometry.html#RProperty
  /// syntax: <length-percentage>
  /// initial: 0
  R,
  ///
  /// href: https://drafts.csswg.org/css-display-4/#propdef-reading-flow
  /// syntax: normal | source-order | flex-visual | flex-flow | grid-rows | grid-columns | grid-order
  /// initial: normal
  ReadingFlow,
  ///
  /// href: https://drafts.csswg.org/css-display-4/#propdef-reading-order
  /// syntax: <integer>
  /// initial: 0
  ReadingOrder,
  ///
  /// href: https://drafts.csswg.org/css-regions-1/#propdef-region-fragment
  /// syntax: auto | break
  /// initial: auto
  RegionFragment,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-resize
  /// syntax: none | both | horizontal | vertical | block | inline
  /// initial: none
  Resize,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-rest
  /// syntax: <'rest-before'> <'rest-after'>?
  /// initial: see individual properties
  /// longhands: [rest-before, rest-after]
  Rest,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-rest-after
  /// syntax: <time [0s,∞]> | none | x-weak | weak | medium | strong | x-strong
  /// initial: none
  RestAfter,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-rest-before
  /// syntax: <time [0s,∞]> | none | x-weak | weak | medium | strong | x-strong
  /// initial: none
  RestBefore,
  ///
  /// href: https://drafts.csswg.org/css-position-3/#propdef-right
  /// syntax: auto | <length-percentage> | <anchor()> | <anchor-size()>
  /// initial: auto
  Right,
  ///
  /// href: https://drafts.csswg.org/css-transforms-2/#propdef-rotate
  /// syntax: none | <angle> | [ x | y | z | <number>{3} ] && <angle>
  /// initial: none
  Rotate,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-gap
  /// syntax: normal | <length-percentage [0,∞]> | <line-width>
  /// initial: normal
  RowGap,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-rule
  /// syntax: <gap-rule-list> | <gap-auto-rule-list>
  /// initial: see individual properties
  /// longhands: [row-rule-width, row-rule-style, row-rule-color]
  RowRule,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-rule-break
  /// syntax: none | normal | intersection
  /// initial: normal
  RowRuleBreak,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-rule-color
  /// syntax: <line-color-list> | <auto-line-color-list>
  /// initial: currentcolor
  RowRuleColor,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-rule-inset
  /// syntax: <'column-rule-inset-cap'> [ / <'column-rule-inset-junction'> ]?
  /// initial: see individual properties
  /// longhands: [row-rule-inset-cap-start, row-rule-inset-cap-end, row-rule-inset-junction-start,
  /// row-rule-inset-junction-end]
  RowRuleInset,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-rule-inset-cap
  /// syntax: <length-percentage> [ <length-percentage> ]?
  /// initial: see individual properties
  /// longhands: [row-rule-inset-cap-start, row-rule-inset-cap-end]
  RowRuleInsetCap,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-rule-inset-cap-end
  /// syntax: <length-percentage>
  /// initial: 0
  RowRuleInsetCapEnd,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-rule-inset-cap-start
  /// syntax: <length-percentage>
  /// initial: 0
  RowRuleInsetCapStart,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-rule-inset-end
  /// syntax: <length-percentage>
  /// initial: see individual properties
  /// longhands: [row-rule-inset-cap-end, row-rule-inset-junction-end]
  RowRuleInsetEnd,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-rule-inset-junction
  /// syntax: <length-percentage> [ <length-percentage> ]?
  /// initial: see individual properties
  /// longhands: [row-rule-inset-junction-start, row-rule-inset-junction-end]
  RowRuleInsetJunction,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-rule-inset-junction-end
  /// syntax: <length-percentage>
  /// initial: 0
  RowRuleInsetJunctionEnd,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-rule-inset-junction-start
  /// syntax: <length-percentage>
  /// initial: 0
  RowRuleInsetJunctionStart,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-rule-inset-start
  /// syntax: <length-percentage>
  /// initial: see individual properties
  /// longhands: [row-rule-inset-cap-start, row-rule-inset-junction-start]
  RowRuleInsetStart,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-rule-style
  /// syntax: <line-style-list> | <auto-line-style-list>
  /// initial: none
  RowRuleStyle,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-rule-visibility-items
  /// syntax: all | around | between | normal
  /// initial: normal
  RowRuleVisibilityItems,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-row-rule-width
  /// syntax: <line-width-list> | <auto-line-width-list>
  /// initial: medium
  RowRuleWidth,
  ///
  /// href: https://drafts.csswg.org/css-ruby-1/#propdef-ruby-align
  /// syntax: start | center | space-between | space-around
  /// initial: space-around
  /// inherited: yes
  RubyAlign,
  ///
  /// href: https://drafts.csswg.org/css-ruby-1/#propdef-ruby-merge
  /// syntax: separate | merge | auto
  /// initial: separate
  /// inherited: yes
  RubyMerge,
  ///
  /// href: https://drafts.csswg.org/css-ruby-1/#propdef-ruby-overhang
  /// syntax: auto | spaces
  /// initial: auto
  /// inherited: yes
  RubyOverhang,
  ///
  /// href: https://drafts.csswg.org/css-ruby-1/#propdef-ruby-position
  /// syntax: [ alternate || [ over | under ] ] | inter-character
  /// initial: alternate
  /// inherited: yes
  RubyPosition,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-rule
  /// syntax: <'column-rule'>
  /// initial: see individual properties
  /// longhands: [column-rule, row-rule]
  Rule,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-rule-break
  /// syntax: <'column-rule-break'>
  /// initial: see individual properties
  /// longhands: [column-rule-break, row-rule-break]
  RuleBreak,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-rule-color
  /// syntax: <'column-rule-color'>
  /// initial: see individual properties
  /// longhands: [column-rule-color, row-rule-color]
  RuleColor,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-rule-inset
  /// syntax: <'column-rule-inset'>
  /// initial: see individual properties
  /// longhands: [column-rule-inset, row-rule-inset]
  RuleInset,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-rule-inset-cap
  /// syntax: <'column-rule-inset-cap'>
  /// initial: see individual properties
  /// longhands: [column-rule-inset-cap]
  RuleInsetCap,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-rule-inset-end
  /// syntax: <'column-rule-inset-end'>
  /// initial: see individual properties
  /// longhands: [column-rule-inset-end, row-rule-inset-end]
  RuleInsetEnd,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-rule-inset-junction
  /// syntax: <'column-rule-inset-junction'>
  /// initial: see individual properties
  /// longhands: [column-rule-inset-junction]
  RuleInsetJunction,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-rule-inset-start
  /// syntax: <'column-rule-inset-start'>
  /// initial: see individual properties
  /// longhands: [column-rule-inset-start, row-rule-inset-start]
  RuleInsetStart,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-rule-overlap
  /// syntax: row-over-column | column-over-row
  /// initial: row-over-column
  RuleOverlap,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-rule-style
  /// syntax: <'column-rule-style'>
  /// initial: see individual properties
  /// longhands: [column-rule-style, row-rule-style]
  RuleStyle,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-rule-visibility-items
  /// syntax: <'column-rule-visibility-items'>
  /// initial: see individual properties
  /// longhands: [column-rule-visibility-items]
  RuleVisibilityItems,
  ///
  /// href: https://drafts.csswg.org/css-gaps-1/#propdef-rule-width
  /// syntax: <'column-rule-width'>
  /// initial: see individual properties
  /// longhands: [column-rule-width, row-rule-width]
  RuleWidth,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/geometry.html#RxProperty
  /// syntax: <length-percentage> | auto
  /// initial: auto
  Rx,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/geometry.html#RyProperty
  /// syntax: <length-percentage> | auto
  /// initial: auto
  Ry,
  ///
  /// href: https://drafts.csswg.org/css-transforms-2/#propdef-scale
  /// syntax: none | [ <number> | <percentage> ]{1,3}
  /// initial: none
  Scale,
  ///
  /// href: https://drafts.csswg.org/css-overflow-3/#propdef-scroll-behavior
  /// syntax: auto | smooth
  /// initial: auto
  ScrollBehavior,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-2/#propdef-scroll-initial-target
  /// syntax: none | nearest
  /// initial: none
  ScrollInitialTarget,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-margin
  /// syntax: <length>{1,4}
  /// initial: 0
  /// longhands: [scroll-margin-top, scroll-margin-right, scroll-margin-bottom, scroll-margin-left]
  ScrollMargin,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-margin-block
  /// syntax: <length>{1,2}
  /// initial: 0
  /// longhands: [scroll-margin-block-start, scroll-margin-block-end]
  ScrollMarginBlock,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-margin-block-end
  /// syntax: <length>
  /// initial: 0
  ScrollMarginBlockEnd,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-margin-block-start
  /// syntax: <length>
  /// initial: 0
  ScrollMarginBlockStart,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-margin-bottom
  /// syntax: <length>
  /// initial: 0
  ScrollMarginBottom,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-margin-inline
  /// syntax: <length>{1,2}
  /// initial: 0
  /// longhands: [scroll-margin-inline-start, scroll-margin-inline-end]
  ScrollMarginInline,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-margin-inline-end
  /// syntax: <length>
  /// initial: 0
  ScrollMarginInlineEnd,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-margin-inline-start
  /// syntax: <length>
  /// initial: 0
  ScrollMarginInlineStart,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-margin-left
  /// syntax: <length>
  /// initial: 0
  ScrollMarginLeft,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-margin-right
  /// syntax: <length>
  /// initial: 0
  ScrollMarginRight,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-margin-top
  /// syntax: <length>
  /// initial: 0
  ScrollMarginTop,
  ///
  /// href: https://drafts.csswg.org/css-overflow-5/#propdef-scroll-marker-group
  /// syntax: none | [ [ before | after ] || [ links | tabs ] ]
  /// initial: none
  ScrollMarkerGroup,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-padding
  /// syntax: [ auto | <length-percentage [0,∞]> ]{1,4}
  /// initial: auto
  /// longhands: [scroll-padding-top, scroll-padding-right, scroll-padding-bottom, scroll-padding-left]
  ScrollPadding,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-padding-block
  /// syntax: [ auto | <length-percentage [0,∞]> ]{1,2}
  /// initial: auto
  /// longhands: [scroll-padding-block-start, scroll-padding-block-end]
  ScrollPaddingBlock,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-padding-block-end
  /// syntax: auto | <length-percentage [0,∞]>
  /// initial: auto
  ScrollPaddingBlockEnd,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-padding-block-start
  /// syntax: auto | <length-percentage [0,∞]>
  /// initial: auto
  ScrollPaddingBlockStart,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-padding-bottom
  /// syntax: auto | <length-percentage [0,∞]>
  /// initial: auto
  ScrollPaddingBottom,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-padding-inline
  /// syntax: [ auto | <length-percentage [0,∞]> ]{1,2}
  /// initial: auto
  /// longhands: [scroll-padding-inline-start, scroll-padding-inline-end]
  ScrollPaddingInline,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-padding-inline-end
  /// syntax: auto | <length-percentage [0,∞]>
  /// initial: auto
  ScrollPaddingInlineEnd,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-padding-inline-start
  /// syntax: auto | <length-percentage [0,∞]>
  /// initial: auto
  ScrollPaddingInlineStart,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-padding-left
  /// syntax: auto | <length-percentage [0,∞]>
  /// initial: auto
  ScrollPaddingLeft,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-padding-right
  /// syntax: auto | <length-percentage [0,∞]>
  /// initial: auto
  ScrollPaddingRight,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-padding-top
  /// syntax: auto | <length-percentage [0,∞]>
  /// initial: auto
  ScrollPaddingTop,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-snap-align
  /// syntax: [ none | start | end | center ]{1,2}
  /// initial: none
  ScrollSnapAlign,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-snap-stop
  /// syntax: normal | always
  /// initial: normal
  ScrollSnapStop,
  ///
  /// href: https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-snap-type
  /// syntax: none | [ x | y | block | inline | both ] [ mandatory | proximity ]?
  /// initial: none
  ScrollSnapType,
  ///
  /// href: https://drafts.csswg.org/css-overflow-5/#propdef-scroll-target-group
  /// syntax: none | auto
  /// initial: none
  ScrollTargetGroup,
  ///
  /// href: https://drafts.csswg.org/scroll-animations-1/#propdef-scroll-timeline
  /// syntax: [ <'scroll-timeline-name'> <'scroll-timeline-axis'>? ]#
  /// initial: see individual properties
  /// longhands: [scroll-timeline-name, scroll-timeline-axis]
  ScrollTimeline,
  ///
  /// href: https://drafts.csswg.org/scroll-animations-1/#propdef-scroll-timeline-axis
  /// syntax: [ block | inline | x | y ]#
  /// initial: block
  ScrollTimelineAxis,
  ///
  /// href: https://drafts.csswg.org/scroll-animations-1/#propdef-scroll-timeline-name
  /// syntax: [ none | <dashed-ident> ]#
  /// initial: none
  ScrollTimelineName,
  ///
  /// href: https://drafts.csswg.org/css-scrollbars-1/#propdef-scrollbar-color
  /// syntax: auto | <color>{2}
  /// initial: auto
  /// inherited: yes
  ScrollbarColor,
  ///
  /// href: https://drafts.csswg.org/css-overflow-3/#propdef-scrollbar-gutter
  /// syntax: auto | stable && both-edges?
  /// initial: auto
  ScrollbarGutter,
  /// Engine-specific: inset from scrollbar container edges.
  /// syntax: <length>{1,4}
  /// initial: 0px
  ScrollbarInset,
  /// Engine-specific: minimum visual thumb length.
  /// syntax: <length>
  /// initial: 20px
  ScrollbarMinThumbSize,
  /// Engine-specific: scrollbar display mode.
  /// syntax: auto | classic | overlay | none
  /// initial: auto
  ScrollbarMode,
  ///
  /// href: https://drafts.csswg.org/css-scrollbars-1/#propdef-scrollbar-width
  /// syntax: auto | thin | none
  /// initial: auto
  ScrollbarWidth,
  ///
  /// href: https://drafts.csswg.org/css-shapes-1/#propdef-shape-image-threshold
  /// syntax: <opacity-value>
  /// initial: 0
  ShapeImageThreshold,
  ///
  /// href: https://drafts.csswg.org/css-shapes-2/#propdef-shape-inside
  /// syntax: auto | outside-shape | [ <basic-shape> || shape-box ] | <image> | display
  /// initial: auto
  ShapeInside,
  ///
  /// href: https://drafts.csswg.org/css-shapes-1/#propdef-shape-margin
  /// syntax: <length-percentage [0,∞]>
  /// initial: 0
  ShapeMargin,
  ///
  /// href: https://drafts.csswg.org/css-shapes-1/#propdef-shape-outside
  /// syntax: none | [ <basic-shape> || <shape-box> ] | <image>
  /// initial: none
  ShapeOutside,
  ///
  /// href: https://drafts.csswg.org/css-shapes-2/#propdef-shape-padding
  /// syntax: <length-percentage [0,∞]>
  /// initial: 0
  ShapePadding,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/painting.html#ShapeRenderingProperty
  /// syntax: auto | optimizeSpeed | crispEdges | geometricPrecision
  /// initial: auto
  /// inherited: yes
  ShapeRendering,
  ///
  /// href: https://drafts.csswg.org/css-forms-1/#propdef-slider-orientation
  /// syntax: auto | left-to-right | right-to-left | top-to-bottom | bottom-to-top
  /// initial: auto
  SliderOrientation,
  ///
  /// href: https://drafts.csswg.org/css-nav-1/#propdef-spatial-navigation-action
  /// syntax: auto | focus | scroll
  /// initial: auto
  SpatialNavigationAction,
  ///
  /// href: https://drafts.csswg.org/css-nav-1/#propdef-spatial-navigation-contain
  /// syntax: auto | contain
  /// initial: auto
  SpatialNavigationContain,
  ///
  /// href: https://drafts.csswg.org/css-nav-1/#propdef-spatial-navigation-function
  /// syntax: normal | grid
  /// initial: normal
  SpatialNavigationFunction,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-speak
  /// syntax: auto | never | always
  /// initial: auto
  /// inherited: yes
  Speak,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-speak-as
  /// syntax: normal | spell-out || digits || [ literal-punctuation | no-punctuation ]
  /// initial: normal
  /// inherited: yes
  SpeakAs,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/pservers.html#StopColorProperty
  /// syntax: <'color'>
  StopColor,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/pservers.html#StopOpacityProperty
  /// syntax: <'opacity'>
  StopOpacity,
  ///
  /// href: https://drafts.csswg.org/css-content-3/#propdef-string-set
  /// syntax: none | [ <custom-ident> <string>+ ]#
  /// initial: none
  StringSet,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/painting.html#StrokeProperty
  /// syntax: <paint>
  /// initial: none
  /// inherited: yes
  Stroke,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-align
  /// syntax: center | inset | outset
  /// initial: center
  /// inherited: yes
  StrokeAlign,
  ///
  /// href: https://svgwg.org/specs/strokes/#StrokeAlignmentProperty
  /// syntax: center | inner | outer
  /// initial: center
  /// inherited: yes
  StrokeAlignment,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-break
  /// syntax: bounding-box | slice | clone
  /// initial: bounding-box
  StrokeBreak,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-color
  /// syntax: <color>#
  /// initial: transparent
  /// inherited: yes
  StrokeColor,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-dash-corner
  /// syntax: none | <length>
  /// initial: none
  /// inherited: yes
  StrokeDashCornerPropdefStrokeDashCorner,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-dash-justify
  /// syntax: none | [ stretch | compress ] || [ dashes || gaps ]
  /// initial: none
  /// inherited: yes
  StrokeDashJustify,
  ///
  /// href: https://svgwg.org/specs/strokes/#StrokeDashadjustProperty
  /// syntax: none | [stretch | compress] [dashes | gaps]?
  /// initial: none
  /// inherited: yes
  StrokeDashadjust,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-dasharray
  /// syntax: none | [<length-percentage> | <number>]+#
  /// initial: none
  /// inherited: yes
  StrokeDasharray,
  ///
  /// href: https://svgwg.org/specs/strokes/#StrokeDashcornerProperty
  /// syntax: none | <length>
  /// initial: none
  /// inherited: yes
  StrokeDashcornerStrokedashcornerproperty,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-dashoffset
  /// syntax: <length-percentage> | <number>
  /// initial: 0
  /// inherited: yes
  StrokeDashoffset,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-image
  /// syntax: <paint>#
  /// initial: none
  /// inherited: yes
  StrokeImage,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-linecap
  /// syntax: butt | round | square
  /// initial: butt
  /// inherited: yes
  StrokeLinecap,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-linejoin
  /// syntax: [ crop | arcs | miter ] || [ bevel | round | fallback ]
  /// initial: miter
  /// inherited: yes
  StrokeLinejoin,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-miterlimit
  /// syntax: <number>
  /// initial: 4
  /// inherited: yes
  StrokeMiterlimit,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-opacity
  /// syntax: <'opacity'>
  /// initial: 1
  /// inherited: yes
  StrokeOpacity,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-origin
  /// syntax: match-parent | fill-box | stroke-box | content-box | padding-box | border-box
  /// initial: match-parent
  StrokeOrigin,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-position
  /// syntax: <position>#
  /// initial: 0% 0%
  /// inherited: yes
  StrokePosition,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-repeat
  /// syntax: <repeat-style>#
  /// initial: repeat
  /// inherited: yes
  StrokeRepeat,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-size
  /// syntax: <bg-size>#
  /// initial: auto
  /// inherited: yes
  StrokeSize,
  ///
  /// href: https://drafts.csswg.org/fill-stroke-3/#propdef-stroke-width
  /// syntax: [ <length-percentage> | <line-width> | <number> ]#
  /// initial: 1px
  /// inherited: yes
  StrokeWidth,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-tab-size
  /// syntax: <number [0,∞]> | <length [0,∞]>
  /// initial: 8
  /// inherited: yes
  TabSize,
  ///
  /// href: https://drafts.csswg.org/css-tables-3/#propdef-table-layout
  /// syntax: auto | fixed
  /// initial: auto
  TableLayout,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-text-align
  /// syntax: start | end | left | right | center | <string> | justify | match-parent | justify-all
  /// initial: start
  /// inherited: yes
  /// longhands: [text-align-all, text-align-last]
  TextAlign,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-text-align-all
  /// syntax: start | end | left | right | center | <string> | justify | match-parent
  /// initial: start
  /// inherited: yes
  TextAlignAll,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-text-align-last
  /// syntax: auto | start | end | left | right | center | justify | match-parent
  /// initial: auto
  /// inherited: yes
  TextAlignLast,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/text.html#TextAnchorProperty
  /// syntax: start | middle | end
  /// initial: start
  /// inherited: yes
  TextAnchor,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-text-autospace
  /// syntax: normal | <autospace> | auto
  /// initial: normal
  /// inherited: yes
  TextAutospace,
  ///
  /// href: https://drafts.csswg.org/css-inline-3/#propdef-text-box
  /// syntax: normal | <'text-box-trim'> || <'text-box-edge'>
  /// initial: normal
  TextBox,
  ///
  /// href: https://drafts.csswg.org/css-inline-3/#propdef-text-box-edge
  /// syntax: auto | <text-edge>
  /// initial: auto
  /// inherited: yes
  TextBoxEdge,
  ///
  /// href: https://drafts.csswg.org/css-inline-3/#propdef-text-box-trim
  /// syntax: none | trim-start | trim-end | trim-both
  /// initial: none
  TextBoxTrim,
  ///
  /// href: https://drafts.csswg.org/css-writing-modes-4/#propdef-text-combine-upright
  /// syntax: none | all | [ digits <integer [2,4]>? ]
  /// initial: none
  /// inherited: yes
  TextCombineUpright,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration
  /// syntax: <'text-decoration-line'> || <'text-decoration-thickness'> || <'text-decoration-style'> ||
  /// <'text-decoration-color'> initial: see individual properties
  /// longhands: [text-decoration-line, text-decoration-thickness, text-decoration-style, text-decoration-color]
  TextDecoration,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-color
  /// syntax: <color>
  /// initial: currentcolor
  TextDecorationColor,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-inset
  /// syntax: <length>{1,2} | auto
  /// initial: 0
  TextDecorationInset,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-line
  /// syntax: none | [ underline || overline || line-through || blink ] | spelling-error | grammar-error
  /// initial: none
  TextDecorationLine,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-skip
  /// syntax: none | auto
  /// initial: See individual properties
  /// inherited: yes
  /// longhands: [text-decoration-skip-self, text-decoration-skip-box, text-decoration-skip-spaces,
  /// text-decoration-skip-ink]
  TextDecorationSkip,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-skip-box
  /// syntax: none | all
  /// initial: none
  /// inherited: yes
  TextDecorationSkipBox,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-skip-ink
  /// syntax: auto | none | all
  /// initial: auto
  /// inherited: yes
  TextDecorationSkipInk,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-skip-self
  /// syntax: auto | skip-all | [ skip-underline || skip-overline || skip-line-through ] | no-skip
  /// initial: auto
  TextDecorationSkipSelf,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-skip-spaces
  /// syntax: none | all | [ start || end ]
  /// initial: start end
  /// inherited: yes
  TextDecorationSkipSpaces,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-style
  /// syntax: solid | double | dotted | dashed | wavy
  /// initial: solid
  TextDecorationStyle,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-decoration-thickness
  /// syntax: auto | from-font | <length-percentage> | <line-width>
  /// initial: auto
  TextDecorationThickness,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-emphasis
  /// syntax: <'text-emphasis-style'> || <'text-emphasis-color'>
  /// initial: see individual properties
  /// longhands: [text-emphasis-style, text-emphasis-color]
  TextEmphasis,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-emphasis-color
  /// syntax: <color>
  /// initial: currentcolor
  /// inherited: yes
  TextEmphasisColor,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-emphasis-position
  /// syntax: [ over | under ] && [ right | left ]?
  /// initial: over right
  /// inherited: yes
  TextEmphasisPosition,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-emphasis-skip
  /// syntax: spaces || punctuation || symbols || narrow
  /// initial: spaces punctuation
  /// inherited: yes
  TextEmphasisSkip,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-emphasis-style
  /// syntax: none | [ [ filled | open ] || [ dot | circle | double-circle | triangle | sesame ] ] | <string>
  /// initial: none
  /// inherited: yes
  TextEmphasisStyle,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-text-fit
  /// syntax: [ none | grow | shrink ] [consistent | per-line | per-line-all]? <percentage>?
  /// initial: none
  TextFit,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-text-group-align
  /// syntax: none | start | end | left | right | center
  /// initial: none
  TextGroupAlign,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-text-indent
  /// syntax: [ <length-percentage> ] && hanging? && each-line?
  /// initial: 0
  /// inherited: yes
  TextIndent,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-text-justify
  /// syntax: [ auto | none | inter-word | inter-character | ruby ] || no-compress
  /// initial: auto
  /// inherited: yes
  TextJustify,
  ///
  /// href: https://drafts.csswg.org/css-writing-modes-4/#propdef-text-orientation
  /// syntax: mixed | upright | sideways
  /// initial: mixed
  /// inherited: yes
  TextOrientation,
  ///
  /// href: https://drafts.csswg.org/css-overflow-4/#propdef-text-overflow
  /// syntax: [ clip | ellipsis | <string> | fade | <fade()> ]{1,2}
  /// initial: clip
  TextOverflow,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/painting.html#TextRenderingProperty
  /// syntax: auto | optimizeSpeed | optimizeLegibility | geometricPrecision
  /// initial: auto
  /// inherited: yes
  TextRendering,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-shadow
  /// syntax: none | <shadow>#
  /// initial: none
  /// inherited: yes
  TextShadow,
  ///
  /// href: https://drafts.csswg.org/css-size-adjust-1/#propdef-text-size-adjust
  /// syntax: auto | none | <percentage [0,∞]>
  /// initial: auto
  /// inherited: yes
  TextSizeAdjust,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-text-spacing
  /// syntax: none | auto | <spacing-trim> || <autospace>
  /// initial: see individual properties
  /// inherited: yes
  /// longhands: [text-spacing-trim, text-autospace]
  TextSpacing,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-text-spacing-trim
  /// syntax: <spacing-trim> | auto
  /// initial: normal
  /// inherited: yes
  TextSpacingTrim,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-text-transform
  /// syntax: none | [capitalize | uppercase | lowercase ] || full-width || full-size-kana | math-auto
  /// initial: none
  /// inherited: yes
  TextTransform,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-underline-offset
  /// syntax: auto | <length-percentage>
  /// initial: auto
  /// inherited: yes
  TextUnderlineOffset,
  ///
  /// href: https://drafts.csswg.org/css-text-decor-4/#propdef-text-underline-position
  /// syntax: auto | [ from-font | under ] || [ left | right ]
  /// initial: auto
  /// inherited: yes
  TextUnderlinePosition,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-text-wrap
  /// syntax: <'text-wrap-mode'> || <'text-wrap-style'>
  /// initial: wrap
  /// longhands: [text-wrap-mode, text-wrap-style]
  TextWrap,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-text-wrap-mode
  /// syntax: wrap | nowrap
  /// initial: wrap
  /// inherited: yes
  TextWrapMode,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-text-wrap-style
  /// syntax: auto | balance | stable | pretty | avoid-orphans
  /// initial: auto
  /// inherited: yes
  TextWrapStyle,
  ///
  /// href: https://drafts.csswg.org/scroll-animations-1/#propdef-timeline-scope
  /// syntax: none | all | <dashed-ident>#
  /// initial: none
  TimelineScope,
  ///
  /// href: https://drafts.csswg.org/animation-triggers-1/#propdef-timeline-trigger
  /// syntax: none | [ <'timeline-trigger-name'> <'timeline-trigger-source'> <'timeline-trigger-activation-range'> [ '/'
  /// <'timeline-trigger-active-range'> ]? ]# initial: see individual properties
  /// longhands: [timeline-trigger-name, timeline-trigger-source, timeline-trigger-activation-range,
  /// timeline-trigger-active-range]
  TimelineTrigger,
  ///
  /// href: https://drafts.csswg.org/animation-triggers-1/#propdef-timeline-trigger-activation-range
  /// syntax: [ <'timeline-trigger-activation-range-start'> <'timeline-trigger-activation-range-end'>? ]#
  /// initial: see individual properties
  /// longhands: [timeline-trigger-activation-range-start, timeline-trigger-activation-range-end]
  TimelineTriggerActivationRange,
  ///
  /// href: https://drafts.csswg.org/animation-triggers-1/#propdef-timeline-trigger-activation-range-end
  /// syntax: [ normal | <length-percentage> | <timeline-range-name> <length-percentage>? ]#
  /// initial: normal
  TimelineTriggerActivationRangeEnd,
  ///
  /// href: https://drafts.csswg.org/animation-triggers-1/#propdef-timeline-trigger-activation-range-start
  /// syntax: [ normal | <length-percentage> | <timeline-range-name> <length-percentage>? ]#
  /// initial: normal
  TimelineTriggerActivationRangeStart,
  ///
  /// href: https://drafts.csswg.org/animation-triggers-1/#propdef-timeline-trigger-active-range
  /// syntax: [ <'timeline-trigger-active-range-start'> <'timeline-trigger-active-range-end'>? ]#
  /// initial: see individual properties
  /// longhands: [timeline-trigger-active-range-start, timeline-trigger-active-range-end]
  TimelineTriggerActiveRange,
  ///
  /// href: https://drafts.csswg.org/animation-triggers-1/#propdef-timeline-trigger-active-range-end
  /// syntax: [ auto | normal | <length-percentage> | <timeline-range-name> <length-percentage>? ]#
  /// initial: auto
  TimelineTriggerActiveRangeEnd,
  ///
  /// href: https://drafts.csswg.org/animation-triggers-1/#propdef-timeline-trigger-active-range-start
  /// syntax: [ auto | normal | <length-percentage> | <timeline-range-name> <length-percentage>? ]#
  /// initial: auto
  TimelineTriggerActiveRangeStart,
  ///
  /// href: https://drafts.csswg.org/animation-triggers-1/#propdef-timeline-trigger-name
  /// syntax: none | <dashed-ident>#
  /// initial: none
  TimelineTriggerName,
  ///
  /// href: https://drafts.csswg.org/animation-triggers-1/#propdef-timeline-trigger-source
  /// syntax: <single-animation-timeline>#
  /// initial: auto
  TimelineTriggerSource,
  ///
  /// href: https://drafts.csswg.org/css-position-3/#propdef-top
  /// syntax: auto | <length-percentage> | <anchor()> | <anchor-size()>
  /// initial: auto
  Top,
  ///
  /// href: https://compat.spec.whatwg.org/#propdef-touch-action
  /// syntax: auto | none | [ [ pan-x | pan-left | pan-right ] || [ pan-y | pan-up | pan-down ] || pinch-zoom ] |
  /// manipulation initial: auto
  TouchAction,
  ///
  /// href: https://drafts.csswg.org/css-transforms-1/#propdef-transform
  /// syntax: none | <transform-list>
  /// initial: none
  Transform,
  ///
  /// href: https://drafts.csswg.org/css-transforms-1/#propdef-transform-box
  /// syntax: content-box | border-box | fill-box | stroke-box | view-box
  /// initial: view-box
  TransformBox,
  ///
  /// href: https://drafts.csswg.org/css-transforms-1/#propdef-transform-origin
  /// syntax: [ left | center | right | top | bottom | <length-percentage> ] | [ left | center | right |
  /// <length-percentage> ] [ top | center | bottom | <length-percentage> ] <length>? | [ [ center | left | right ] && [
  /// center | top | bottom ] ] <length>? initial: 50% 50%
  TransformOrigin,
  ///
  /// href: https://drafts.csswg.org/css-transforms-2/#propdef-transform-style
  /// syntax: flat | preserve-3d
  /// initial: flat
  TransformStyle,
  ///
  /// href: https://drafts.csswg.org/css-transitions-1/#propdef-transition
  /// syntax: <single-transition>#
  /// initial: see individual properties
  /// longhands: [transition-property, transition-duration, transition-timing-function, transition-delay,
  /// transition-behavior]
  Transition,
  ///
  /// href: https://drafts.csswg.org/css-transitions-2/#propdef-transition-behavior
  /// syntax: <transition-behavior-value>#
  /// initial: normal
  TransitionBehavior,
  ///
  /// href: https://drafts.csswg.org/css-transitions-1/#propdef-transition-delay
  /// syntax: <time>#
  /// initial: 0s
  TransitionDelay,
  ///
  /// href: https://drafts.csswg.org/css-transitions-1/#propdef-transition-duration
  /// syntax: <time [0s,∞]>#
  /// initial: 0s
  TransitionDuration,
  ///
  /// href: https://drafts.csswg.org/css-transitions-1/#propdef-transition-property
  /// syntax: none | <single-transition-property>#
  /// initial: all
  TransitionProperty,
  ///
  /// href: https://drafts.csswg.org/css-transitions-1/#propdef-transition-timing-function
  /// syntax: <easing-function>#
  /// initial: ease
  TransitionTimingFunction,
  ///
  /// href: https://drafts.csswg.org/css-transforms-2/#propdef-translate
  /// syntax: none | <length-percentage> [ <length-percentage> <length>? ]?
  /// initial: none
  Translate,
  ///
  /// href: https://drafts.csswg.org/animation-triggers-1/#propdef-trigger-scope
  /// syntax: none | all | <dashed-ident>#
  /// initial: none
  TriggerScope,
  ///
  /// href: https://drafts.csswg.org/css-writing-modes-4/#propdef-unicode-bidi
  /// syntax: normal | embed | isolate | bidi-override | isolate-override | plaintext
  /// initial: normal
  UnicodeBidi,
  ///
  /// href: https://drafts.csswg.org/css-ui-4/#propdef-user-select
  /// syntax: auto | text | none | contain | all
  /// initial: auto
  UserSelect,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/coords.html#VectorEffectProperty
  /// syntax: none | non-scaling-stroke | non-scaling-size | non-rotation | fixed-position
  /// initial: none
  VectorEffect,
  ///
  /// href: https://drafts.csswg.org/css-inline-3/#propdef-vertical-align
  /// syntax: [ first | last] || <'alignment-baseline'> || <'baseline-shift'>
  /// initial: baseline
  /// longhands: [alignment-baseline, baseline-shift, baseline-source]
  VerticalAlign,
  ///
  /// href: https://drafts.csswg.org/scroll-animations-1/#propdef-view-timeline
  /// syntax: [ <'view-timeline-name'> [ <'view-timeline-axis'> || <'view-timeline-inset'> ]? ]#
  /// initial: see individual properties
  /// longhands: [view-timeline-name, view-timeline-axis]
  ViewTimeline,
  ///
  /// href: https://drafts.csswg.org/scroll-animations-1/#propdef-view-timeline-axis
  /// syntax: [ block | inline | x | y ]#
  /// initial: block
  ViewTimelineAxis,
  ///
  /// href: https://drafts.csswg.org/scroll-animations-1/#propdef-view-timeline-inset
  /// syntax: [ [ auto | <length-percentage> ]{1,2} ]#
  /// initial: auto
  ViewTimelineInset,
  ///
  /// href: https://drafts.csswg.org/scroll-animations-1/#propdef-view-timeline-name
  /// syntax: [ none | <dashed-ident> ]#
  /// initial: none
  ViewTimelineName,
  ///
  /// href: https://drafts.csswg.org/css-view-transitions-2/#propdef-view-transition-class
  /// syntax: none | <custom-ident>+
  /// initial: none
  ViewTransitionClass,
  ///
  /// href: https://drafts.csswg.org/css-view-transitions-2/#propdef-view-transition-group
  /// syntax: normal | contain | nearest | <custom-ident>
  /// initial: normal
  ViewTransitionGroup,
  ///
  /// href: https://drafts.csswg.org/css-view-transitions-2/#propdef-view-transition-name
  /// syntax: none | <custom-ident>
  /// initial: none
  ViewTransitionName,
  ///
  /// href: https://drafts.csswg.org/css-view-transitions-2/#propdef-view-transition-scope
  /// syntax: none | all
  /// initial: none
  ViewTransitionScope,
  ///
  /// href: https://drafts.csswg.org/css-display-4/#propdef-visibility
  /// syntax: visible | hidden | force-hidden | collapse
  /// initial: visible
  /// inherited: yes
  Visibility,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-voice-balance
  /// syntax: <number> | left | center | right | leftwards | rightwards
  /// initial: center
  /// inherited: yes
  VoiceBalance,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-voice-duration
  /// syntax: auto | <time [0s,∞]>
  /// initial: auto
  VoiceDuration,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-voice-family
  /// syntax: [ <voice-family-name> | <generic-voice> ]# | preserve
  /// initial: implementation-dependent
  /// inherited: yes
  VoiceFamily,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-voice-pitch
  /// syntax: <frequency [0Hz,∞]> && absolute | [ [ x-low | low | medium | high | x-high ] || [ <frequency [0Hz,∞]> |
  /// <semitones> | <percentage> ] ] initial: medium
  /// inherited: yes
  VoicePitch,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-voice-range
  /// syntax: <frequency [0Hz,∞]> && absolute | [ [ x-low | low | medium | high | x-high ] || [ <frequency [0Hz,∞]> |
  /// <semitones> | <percentage> ] ] initial: medium
  /// inherited: yes
  VoiceRange,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-voice-rate
  /// syntax: [ normal | x-slow | slow | medium | fast | x-fast ] || <percentage [0,∞]>
  /// initial: normal
  /// inherited: yes
  VoiceRate,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-voice-stress
  /// syntax: normal | strong | moderate | none | reduced
  /// initial: normal
  /// inherited: yes
  VoiceStress,
  ///
  /// href: https://drafts.csswg.org/css-speech-1/#propdef-voice-volume
  /// syntax: silent | [ [ x-soft | soft | medium | loud | x-loud ] || <decibel> ]
  /// initial: medium
  /// inherited: yes
  VoiceVolume,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-white-space
  /// syntax: normal | pre | pre-wrap | pre-line | <'white-space-collapse'> || <'text-wrap-mode'> ||
  /// <'white-space-trim'> initial: normal
  /// longhands: [white-space-collapse, text-wrap-mode]
  WhiteSpace,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-white-space-collapse
  /// syntax: collapse | discard | preserve | preserve-breaks | preserve-spaces | break-spaces
  /// initial: collapse
  /// inherited: yes
  WhiteSpaceCollapse,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-white-space-trim
  /// syntax: none | discard-before || discard-after || discard-inner
  /// initial: none
  WhiteSpaceTrim,
  ///
  /// href: https://drafts.csswg.org/css-break-4/#propdef-widows
  /// syntax: <integer [1,∞]>
  /// initial: 2
  /// inherited: yes
  Widows,
  ///
  /// href: https://drafts.csswg.org/css-sizing-3/#propdef-width
  /// syntax: auto | <length-percentage [0,∞]> | min-content | max-content | fit-content(<length-percentage [0,∞]>) |
  /// <calc-size()> | <anchor-size()> | stretch | fit-content | contain initial: auto
  Width,
  ///
  /// href: https://drafts.csswg.org/css-will-change-1/#propdef-will-change
  /// syntax: auto | <animateable-feature>#
  /// initial: auto
  WillChange,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-word-break
  /// syntax: normal | break-all | keep-all | manual | auto-phrase | break-word
  /// initial: normal
  /// inherited: yes
  WordBreak,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-word-space-transform
  /// syntax: none | [ space | ideographic-space ] && auto-phrase?
  /// initial: none
  /// inherited: yes
  WordSpaceTransform,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-word-spacing
  /// syntax: normal | <length-percentage>
  /// initial: normal
  /// inherited: yes
  WordSpacing,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-word-wrap
  /// syntax: normal | break-word | anywhere
  /// initial: normal
  /// inherited: yes
  WordWrap,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-wrap-after
  /// syntax: auto | avoid | avoid-line | avoid-flex | line | flex
  /// initial: auto
  WrapAfter,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-wrap-before
  /// syntax: auto | avoid | avoid-line | avoid-flex | line | flex
  /// initial: auto
  WrapBefore,
  ///
  /// href: https://drafts.csswg.org/css-exclusions-1/#propdef-wrap-flow
  /// syntax: auto | both | start | end | minimum | maximum | clear
  /// initial: auto
  WrapFlow,
  ///
  /// href: https://drafts.csswg.org/css-text-4/#propdef-wrap-inside
  /// syntax: auto | avoid
  /// initial: auto
  WrapInside,
  ///
  /// href: https://drafts.csswg.org/css-exclusions-1/#propdef-wrap-through
  /// syntax: wrap | none
  /// initial: wrap
  WrapThrough,
  ///
  /// href: https://drafts.csswg.org/css-writing-modes-4/#propdef-writing-mode
  /// syntax: horizontal-tb | vertical-rl | vertical-lr | sideways-rl | sideways-lr
  /// initial: horizontal-tb
  /// inherited: yes
  WritingMode,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/geometry.html#XProperty
  /// syntax: <length-percentage>
  /// initial: 0
  X,
  ///
  /// href: https://w3c.github.io/svgwg/svg2-draft/geometry.html#YProperty
  /// syntax: <length-percentage>
  /// initial: 0
  Y,
  ///
  /// href: https://drafts.csswg.org/css2/#propdef-z-index
  /// syntax: auto | <integer> | inherit
  /// initial: auto
  ZIndex,
  ///
  /// href: https://drafts.csswg.org/css-viewport/#propdef-zoom
  /// syntax: <number [0,∞]> | <percentage [0,∞]>
  /// initial: 1
  Zoom,
  Unknown(String),
}

impl CssProperty {
  pub fn name(&self) -> &'static str {
    match self {
      CssProperty::WebkitAlignContent => "-webkit-align-content",
      CssProperty::WebkitAlignItems => "-webkit-align-items",
      CssProperty::WebkitAlignSelf => "-webkit-align-self",
      CssProperty::WebkitAnimation => "-webkit-animation",
      CssProperty::WebkitAnimationDelay => "-webkit-animation-delay",
      CssProperty::WebkitAnimationDirection => "-webkit-animation-direction",
      CssProperty::WebkitAnimationDuration => "-webkit-animation-duration",
      CssProperty::WebkitAnimationFillMode => "-webkit-animation-fill-mode",
      CssProperty::WebkitAnimationIterationCount => "-webkit-animation-iteration-count",
      CssProperty::WebkitAnimationName => "-webkit-animation-name",
      CssProperty::WebkitAnimationPlayState => "-webkit-animation-play-state",
      CssProperty::WebkitAnimationTimingFunction => "-webkit-animation-timing-function",
      CssProperty::WebkitAppearance => "-webkit-appearance",
      CssProperty::WebkitBackfaceVisibility => "-webkit-backface-visibility",
      CssProperty::WebkitBackgroundClip => "-webkit-background-clip",
      CssProperty::WebkitBackgroundOrigin => "-webkit-background-origin",
      CssProperty::WebkitBackgroundSize => "-webkit-background-size",
      CssProperty::WebkitBorderBottomLeftRadius => "-webkit-border-bottom-left-radius",
      CssProperty::WebkitBorderBottomRightRadius => "-webkit-border-bottom-right-radius",
      CssProperty::WebkitBorderRadius => "-webkit-border-radius",
      CssProperty::WebkitBorderTopLeftRadius => "-webkit-border-top-left-radius",
      CssProperty::WebkitBorderTopRightRadius => "-webkit-border-top-right-radius",
      CssProperty::WebkitBoxAlign => "-webkit-box-align",
      CssProperty::WebkitBoxFlex => "-webkit-box-flex",
      CssProperty::WebkitBoxOrdinalGroup => "-webkit-box-ordinal-group",
      CssProperty::WebkitBoxOrient => "-webkit-box-orient",
      CssProperty::WebkitBoxPack => "-webkit-box-pack",
      CssProperty::WebkitBoxShadow => "-webkit-box-shadow",
      CssProperty::WebkitBoxSizing => "-webkit-box-sizing",
      CssProperty::WebkitFilter => "-webkit-filter",
      CssProperty::WebkitFlex => "-webkit-flex",
      CssProperty::WebkitFlexBasis => "-webkit-flex-basis",
      CssProperty::WebkitFlexDirection => "-webkit-flex-direction",
      CssProperty::WebkitFlexFlow => "-webkit-flex-flow",
      CssProperty::WebkitFlexGrow => "-webkit-flex-grow",
      CssProperty::WebkitFlexShrink => "-webkit-flex-shrink",
      CssProperty::WebkitFlexWrap => "-webkit-flex-wrap",
      CssProperty::WebkitJustifyContent => "-webkit-justify-content",
      CssProperty::WebkitLineClamp => "-webkit-line-clamp",
      CssProperty::WebkitMask => "-webkit-mask",
      CssProperty::WebkitMaskBoxImage => "-webkit-mask-box-image",
      CssProperty::WebkitMaskBoxImageOutset => "-webkit-mask-box-image-outset",
      CssProperty::WebkitMaskBoxImageRepeat => "-webkit-mask-box-image-repeat",
      CssProperty::WebkitMaskBoxImageSlice => "-webkit-mask-box-image-slice",
      CssProperty::WebkitMaskBoxImageSource => "-webkit-mask-box-image-source",
      CssProperty::WebkitMaskBoxImageWidth => "-webkit-mask-box-image-width",
      CssProperty::WebkitMaskClip => "-webkit-mask-clip",
      CssProperty::WebkitMaskComposite => "-webkit-mask-composite",
      CssProperty::WebkitMaskImage => "-webkit-mask-image",
      CssProperty::WebkitMaskOrigin => "-webkit-mask-origin",
      CssProperty::WebkitMaskPosition => "-webkit-mask-position",
      CssProperty::WebkitMaskRepeat => "-webkit-mask-repeat",
      CssProperty::WebkitMaskSize => "-webkit-mask-size",
      CssProperty::WebkitOrder => "-webkit-order",
      CssProperty::WebkitPerspective => "-webkit-perspective",
      CssProperty::WebkitPerspectiveOrigin => "-webkit-perspective-origin",
      CssProperty::WebkitTextFillColor => "-webkit-text-fill-color",
      CssProperty::WebkitTextSizeAdjust => "-webkit-text-size-adjust",
      CssProperty::WebkitTextStroke => "-webkit-text-stroke",
      CssProperty::WebkitTextStrokeColor => "-webkit-text-stroke-color",
      CssProperty::WebkitTextStrokeWidth => "-webkit-text-stroke-width",
      CssProperty::WebkitTransform => "-webkit-transform",
      CssProperty::WebkitTransformOrigin => "-webkit-transform-origin",
      CssProperty::WebkitTransformStyle => "-webkit-transform-style",
      CssProperty::WebkitTransition => "-webkit-transition",
      CssProperty::WebkitTransitionDelay => "-webkit-transition-delay",
      CssProperty::WebkitTransitionDuration => "-webkit-transition-duration",
      CssProperty::WebkitTransitionProperty => "-webkit-transition-property",
      CssProperty::WebkitTransitionTimingFunction => "-webkit-transition-timing-function",
      CssProperty::WebkitUserSelect => "-webkit-user-select",
      CssProperty::AccentColor => "accent-color",
      CssProperty::AlignContent => "align-content",
      CssProperty::AlignItems => "align-items",
      CssProperty::AlignSelf => "align-self",
      CssProperty::AlignmentBaseline => "alignment-baseline",
      CssProperty::All => "all",
      CssProperty::AnchorName => "anchor-name",
      CssProperty::AnchorScope => "anchor-scope",
      CssProperty::Animation => "animation",
      CssProperty::AnimationComposition => "animation-composition",
      CssProperty::AnimationDelay => "animation-delay",
      CssProperty::AnimationDirection => "animation-direction",
      CssProperty::AnimationDuration => "animation-duration",
      CssProperty::AnimationFillMode => "animation-fill-mode",
      CssProperty::AnimationIterationCount => "animation-iteration-count",
      CssProperty::AnimationName => "animation-name",
      CssProperty::AnimationPlayState => "animation-play-state",
      CssProperty::AnimationRange => "animation-range",
      CssProperty::AnimationRangeCenter => "animation-range-center",
      CssProperty::AnimationRangeEnd => "animation-range-end",
      CssProperty::AnimationRangeStart => "animation-range-start",
      CssProperty::AnimationTimeline => "animation-timeline",
      CssProperty::AnimationTimingFunction => "animation-timing-function",
      CssProperty::AnimationTrigger => "animation-trigger",
      CssProperty::Appearance => "appearance",
      CssProperty::AspectRatio => "aspect-ratio",
      CssProperty::BackdropFilter => "backdrop-filter",
      CssProperty::BackfaceVisibility => "backface-visibility",
      CssProperty::Background => "background",
      CssProperty::BackgroundAttachment => "background-attachment",
      CssProperty::BackgroundBlendMode => "background-blend-mode",
      CssProperty::BackgroundClip => "background-clip",
      CssProperty::BackgroundColor => "background-color",
      CssProperty::BackgroundImage => "background-image",
      CssProperty::BackgroundOrigin => "background-origin",
      CssProperty::BackgroundPosition => "background-position",
      CssProperty::BackgroundPositionBlock => "background-position-block",
      CssProperty::BackgroundPositionInline => "background-position-inline",
      CssProperty::BackgroundPositionX => "background-position-x",
      CssProperty::BackgroundPositionY => "background-position-y",
      CssProperty::BackgroundRepeat => "background-repeat",
      CssProperty::BackgroundRepeatBlock => "background-repeat-block",
      CssProperty::BackgroundRepeatInline => "background-repeat-inline",
      CssProperty::BackgroundRepeatX => "background-repeat-x",
      CssProperty::BackgroundRepeatY => "background-repeat-y",
      CssProperty::BackgroundSize => "background-size",
      CssProperty::BackgroundTbd => "background-tbd",
      CssProperty::BaselineShift => "baseline-shift",
      CssProperty::BaselineSource => "baseline-source",
      CssProperty::BlockEllipsis => "block-ellipsis",
      CssProperty::BlockSize => "block-size",
      CssProperty::BlockStep => "block-step",
      CssProperty::BlockStepAlign => "block-step-align",
      CssProperty::BlockStepInsert => "block-step-insert",
      CssProperty::BlockStepRound => "block-step-round",
      CssProperty::BlockStepSize => "block-step-size",
      CssProperty::BookmarkLabel => "bookmark-label",
      CssProperty::BookmarkLevel => "bookmark-level",
      CssProperty::BookmarkState => "bookmark-state",
      CssProperty::Border => "border",
      CssProperty::BorderBlock => "border-block",
      CssProperty::BorderBlockClip => "border-block-clip",
      CssProperty::BorderBlockColor => "border-block-color",
      CssProperty::BorderBlockEnd => "border-block-end",
      CssProperty::BorderBlockEndClip => "border-block-end-clip",
      CssProperty::BorderBlockEndColor => "border-block-end-color",
      CssProperty::BorderBlockEndRadius => "border-block-end-radius",
      CssProperty::BorderBlockEndStyle => "border-block-end-style",
      CssProperty::BorderBlockEndWidth => "border-block-end-width",
      CssProperty::BorderBlockStart => "border-block-start",
      CssProperty::BorderBlockStartClip => "border-block-start-clip",
      CssProperty::BorderBlockStartColor => "border-block-start-color",
      CssProperty::BorderBlockStartRadius => "border-block-start-radius",
      CssProperty::BorderBlockStartStyle => "border-block-start-style",
      CssProperty::BorderBlockStartWidth => "border-block-start-width",
      CssProperty::BorderBlockStyle => "border-block-style",
      CssProperty::BorderBlockWidth => "border-block-width",
      CssProperty::BorderBottom => "border-bottom",
      CssProperty::BorderBottomClip => "border-bottom-clip",
      CssProperty::BorderBottomColor => "border-bottom-color",
      CssProperty::BorderBottomLeftRadius => "border-bottom-left-radius",
      CssProperty::BorderBottomRadius => "border-bottom-radius",
      CssProperty::BorderBottomRightRadius => "border-bottom-right-radius",
      CssProperty::BorderBottomStyle => "border-bottom-style",
      CssProperty::BorderBottomWidth => "border-bottom-width",
      CssProperty::BorderBoundary => "border-boundary",
      CssProperty::BorderClip => "border-clip",
      CssProperty::BorderCollapse => "border-collapse",
      CssProperty::BorderColor => "border-color",
      CssProperty::BorderEndEndRadius => "border-end-end-radius",
      CssProperty::BorderEndStartRadius => "border-end-start-radius",
      CssProperty::BorderImage => "border-image",
      CssProperty::BorderImageOutset => "border-image-outset",
      CssProperty::BorderImageRepeat => "border-image-repeat",
      CssProperty::BorderImageSlice => "border-image-slice",
      CssProperty::BorderImageSource => "border-image-source",
      CssProperty::BorderImageWidth => "border-image-width",
      CssProperty::BorderInline => "border-inline",
      CssProperty::BorderInlineClip => "border-inline-clip",
      CssProperty::BorderInlineColor => "border-inline-color",
      CssProperty::BorderInlineEnd => "border-inline-end",
      CssProperty::BorderInlineEndClip => "border-inline-end-clip",
      CssProperty::BorderInlineEndColor => "border-inline-end-color",
      CssProperty::BorderInlineEndRadius => "border-inline-end-radius",
      CssProperty::BorderInlineEndStyle => "border-inline-end-style",
      CssProperty::BorderInlineEndWidth => "border-inline-end-width",
      CssProperty::BorderInlineStart => "border-inline-start",
      CssProperty::BorderInlineStartClip => "border-inline-start-clip",
      CssProperty::BorderInlineStartColor => "border-inline-start-color",
      CssProperty::BorderInlineStartRadius => "border-inline-start-radius",
      CssProperty::BorderInlineStartStyle => "border-inline-start-style",
      CssProperty::BorderInlineStartWidth => "border-inline-start-width",
      CssProperty::BorderInlineStyle => "border-inline-style",
      CssProperty::BorderInlineWidth => "border-inline-width",
      CssProperty::BorderLeft => "border-left",
      CssProperty::BorderLeftClip => "border-left-clip",
      CssProperty::BorderLeftColor => "border-left-color",
      CssProperty::BorderLeftRadius => "border-left-radius",
      CssProperty::BorderLeftStyle => "border-left-style",
      CssProperty::BorderLeftWidth => "border-left-width",
      CssProperty::BorderLimit => "border-limit",
      CssProperty::BorderRadius => "border-radius",
      CssProperty::BorderRight => "border-right",
      CssProperty::BorderRightClip => "border-right-clip",
      CssProperty::BorderRightColor => "border-right-color",
      CssProperty::BorderRightRadius => "border-right-radius",
      CssProperty::BorderRightStyle => "border-right-style",
      CssProperty::BorderRightWidth => "border-right-width",
      CssProperty::BorderShape => "border-shape",
      CssProperty::BorderSpacing => "border-spacing",
      CssProperty::BorderStartEndRadius => "border-start-end-radius",
      CssProperty::BorderStartStartRadius => "border-start-start-radius",
      CssProperty::BorderStyle => "border-style",
      CssProperty::BorderTop => "border-top",
      CssProperty::BorderTopClip => "border-top-clip",
      CssProperty::BorderTopColor => "border-top-color",
      CssProperty::BorderTopLeftRadius => "border-top-left-radius",
      CssProperty::BorderTopRadius => "border-top-radius",
      CssProperty::BorderTopRightRadius => "border-top-right-radius",
      CssProperty::BorderTopStyle => "border-top-style",
      CssProperty::BorderTopWidth => "border-top-width",
      CssProperty::BorderWidth => "border-width",
      CssProperty::Bottom => "bottom",
      CssProperty::BoxDecorationBreak => "box-decoration-break",
      CssProperty::BoxShadow => "box-shadow",
      CssProperty::BoxShadowBlur => "box-shadow-blur",
      CssProperty::BoxShadowColor => "box-shadow-color",
      CssProperty::BoxShadowOffset => "box-shadow-offset",
      CssProperty::BoxShadowPosition => "box-shadow-position",
      CssProperty::BoxShadowSpread => "box-shadow-spread",
      CssProperty::BoxSizing => "box-sizing",
      CssProperty::BoxSnap => "box-snap",
      CssProperty::BreakAfter => "break-after",
      CssProperty::BreakBefore => "break-before",
      CssProperty::BreakInside => "break-inside",
      CssProperty::CaptionSide => "caption-side",
      CssProperty::Caret => "caret",
      CssProperty::CaretAnimation => "caret-animation",
      CssProperty::CaretColor => "caret-color",
      CssProperty::CaretShape => "caret-shape",
      CssProperty::Clear => "clear",
      CssProperty::Clip => "clip",
      CssProperty::ClipPath => "clip-path",
      CssProperty::ClipRule => "clip-rule",
      CssProperty::Color => "color",
      CssProperty::ColorAdjust => "color-adjust",
      CssProperty::ColorInterpolation => "color-interpolation",
      CssProperty::ColorInterpolationFilters => "color-interpolation-filters",
      CssProperty::ColorScheme => "color-scheme",
      CssProperty::ColumnCount => "column-count",
      CssProperty::ColumnFill => "column-fill",
      CssProperty::ColumnGap => "column-gap",
      CssProperty::ColumnHeight => "column-height",
      CssProperty::ColumnRule => "column-rule",
      CssProperty::ColumnRuleBreak => "column-rule-break",
      CssProperty::ColumnRuleColor => "column-rule-color",
      CssProperty::ColumnRuleInset => "column-rule-inset",
      CssProperty::ColumnRuleInsetCap => "column-rule-inset-cap",
      CssProperty::ColumnRuleInsetCapEnd => "column-rule-inset-cap-end",
      CssProperty::ColumnRuleInsetCapStart => "column-rule-inset-cap-start",
      CssProperty::ColumnRuleInsetEnd => "column-rule-inset-end",
      CssProperty::ColumnRuleInsetJunction => "column-rule-inset-junction",
      CssProperty::ColumnRuleInsetJunctionEnd => "column-rule-inset-junction-end",
      CssProperty::ColumnRuleInsetJunctionStart => "column-rule-inset-junction-start",
      CssProperty::ColumnRuleInsetStart => "column-rule-inset-start",
      CssProperty::ColumnRuleStyle => "column-rule-style",
      CssProperty::ColumnRuleVisibilityItems => "column-rule-visibility-items",
      CssProperty::ColumnRuleWidth => "column-rule-width",
      CssProperty::ColumnSpan => "column-span",
      CssProperty::ColumnWidth => "column-width",
      CssProperty::ColumnWrap => "column-wrap",
      CssProperty::Columns => "columns",
      CssProperty::Contain => "contain",
      CssProperty::ContainIntrinsicBlockSize => "contain-intrinsic-block-size",
      CssProperty::ContainIntrinsicHeight => "contain-intrinsic-height",
      CssProperty::ContainIntrinsicInlineSize => "contain-intrinsic-inline-size",
      CssProperty::ContainIntrinsicSize => "contain-intrinsic-size",
      CssProperty::ContainIntrinsicWidth => "contain-intrinsic-width",
      CssProperty::Container => "container",
      CssProperty::ContainerName => "container-name",
      CssProperty::ContainerType => "container-type",
      CssProperty::Content => "content",
      CssProperty::ContentVisibility => "content-visibility",
      CssProperty::Continue => "continue",
      CssProperty::CopyInto => "copy-into",
      CssProperty::Corner => "corner",
      CssProperty::CornerBlockEnd => "corner-block-end",
      CssProperty::CornerBlockEndShape => "corner-block-end-shape",
      CssProperty::CornerBlockStart => "corner-block-start",
      CssProperty::CornerBlockStartShape => "corner-block-start-shape",
      CssProperty::CornerBottom => "corner-bottom",
      CssProperty::CornerBottomLeft => "corner-bottom-left",
      CssProperty::CornerBottomLeftShape => "corner-bottom-left-shape",
      CssProperty::CornerBottomRight => "corner-bottom-right",
      CssProperty::CornerBottomRightShape => "corner-bottom-right-shape",
      CssProperty::CornerBottomShape => "corner-bottom-shape",
      CssProperty::CornerEndEnd => "corner-end-end",
      CssProperty::CornerEndEndShape => "corner-end-end-shape",
      CssProperty::CornerEndStart => "corner-end-start",
      CssProperty::CornerEndStartShape => "corner-end-start-shape",
      CssProperty::CornerInlineEnd => "corner-inline-end",
      CssProperty::CornerInlineEndShape => "corner-inline-end-shape",
      CssProperty::CornerInlineStart => "corner-inline-start",
      CssProperty::CornerInlineStartShape => "corner-inline-start-shape",
      CssProperty::CornerLeft => "corner-left",
      CssProperty::CornerLeftShape => "corner-left-shape",
      CssProperty::CornerRight => "corner-right",
      CssProperty::CornerRightShape => "corner-right-shape",
      CssProperty::CornerShape => "corner-shape",
      CssProperty::CornerStartEnd => "corner-start-end",
      CssProperty::CornerStartEndShape => "corner-start-end-shape",
      CssProperty::CornerStartStart => "corner-start-start",
      CssProperty::CornerStartStartShape => "corner-start-start-shape",
      CssProperty::CornerTop => "corner-top",
      CssProperty::CornerTopLeft => "corner-top-left",
      CssProperty::CornerTopLeftShape => "corner-top-left-shape",
      CssProperty::CornerTopRight => "corner-top-right",
      CssProperty::CornerTopRightShape => "corner-top-right-shape",
      CssProperty::CornerTopShape => "corner-top-shape",
      CssProperty::CounterIncrement => "counter-increment",
      CssProperty::CounterReset => "counter-reset",
      CssProperty::CounterSet => "counter-set",
      CssProperty::Cue => "cue",
      CssProperty::CueAfter => "cue-after",
      CssProperty::CueBefore => "cue-before",
      CssProperty::Cursor => "cursor",
      CssProperty::Cx => "cx",
      CssProperty::Cy => "cy",
      CssProperty::D => "d",
      CssProperty::Direction => "direction",
      CssProperty::Display => "display",
      CssProperty::DominantBaseline => "dominant-baseline",
      CssProperty::DynamicRangeLimit => "dynamic-range-limit",
      CssProperty::EmptyCells => "empty-cells",
      CssProperty::EventTrigger => "event-trigger",
      CssProperty::EventTriggerName => "event-trigger-name",
      CssProperty::EventTriggerSource => "event-trigger-source",
      CssProperty::FieldSizing => "field-sizing",
      CssProperty::Fill => "fill",
      CssProperty::FillBreak => "fill-break",
      CssProperty::FillColor => "fill-color",
      CssProperty::FillImage => "fill-image",
      CssProperty::FillOpacity => "fill-opacity",
      CssProperty::FillOrigin => "fill-origin",
      CssProperty::FillPosition => "fill-position",
      CssProperty::FillRepeat => "fill-repeat",
      CssProperty::FillRule => "fill-rule",
      CssProperty::FillSize => "fill-size",
      CssProperty::Filter => "filter",
      CssProperty::Flex => "flex",
      CssProperty::FlexBasis => "flex-basis",
      CssProperty::FlexDirection => "flex-direction",
      CssProperty::FlexFlow => "flex-flow",
      CssProperty::FlexGrow => "flex-grow",
      CssProperty::FlexShrink => "flex-shrink",
      CssProperty::FlexWrap => "flex-wrap",
      CssProperty::Float => "float",
      CssProperty::FloatDefer => "float-defer",
      CssProperty::FloatOffset => "float-offset",
      CssProperty::FloatReference => "float-reference",
      CssProperty::FloodColor => "flood-color",
      CssProperty::FloodOpacity => "flood-opacity",
      CssProperty::FlowFrom => "flow-from",
      CssProperty::FlowInto => "flow-into",
      CssProperty::FlowTolerance => "flow-tolerance",
      CssProperty::Font => "font",
      CssProperty::FontFamily => "font-family",
      CssProperty::FontFeatureSettings => "font-feature-settings",
      CssProperty::FontKerning => "font-kerning",
      CssProperty::FontLanguageOverride => "font-language-override",
      CssProperty::FontOpticalSizing => "font-optical-sizing",
      CssProperty::FontPalette => "font-palette",
      CssProperty::FontSize => "font-size",
      CssProperty::FontSizeAdjust => "font-size-adjust",
      CssProperty::FontStretch => "font-stretch",
      CssProperty::FontStyle => "font-style",
      CssProperty::FontSynthesis => "font-synthesis",
      CssProperty::FontSynthesisPosition => "font-synthesis-position",
      CssProperty::FontSynthesisSmallCaps => "font-synthesis-small-caps",
      CssProperty::FontSynthesisStyle => "font-synthesis-style",
      CssProperty::FontSynthesisWeight => "font-synthesis-weight",
      CssProperty::FontVariant => "font-variant",
      CssProperty::FontVariantAlternates => "font-variant-alternates",
      CssProperty::FontVariantCaps => "font-variant-caps",
      CssProperty::FontVariantEastAsian => "font-variant-east-asian",
      CssProperty::FontVariantEmoji => "font-variant-emoji",
      CssProperty::FontVariantLigatures => "font-variant-ligatures",
      CssProperty::FontVariantNumeric => "font-variant-numeric",
      CssProperty::FontVariantPosition => "font-variant-position",
      CssProperty::FontVariationSettings => "font-variation-settings",
      CssProperty::FontWeight => "font-weight",
      CssProperty::FontWidth => "font-width",
      CssProperty::FootnoteDisplay => "footnote-display",
      CssProperty::FootnotePolicy => "footnote-policy",
      CssProperty::ForcedColorAdjust => "forced-color-adjust",
      CssProperty::FrameSizing => "frame-sizing",
      CssProperty::Gap => "gap",
      CssProperty::GlyphOrientationVertical => "glyph-orientation-vertical",
      CssProperty::Grid => "grid",
      CssProperty::GridArea => "grid-area",
      CssProperty::GridAutoColumns => "grid-auto-columns",
      CssProperty::GridAutoFlow => "grid-auto-flow",
      CssProperty::GridAutoRows => "grid-auto-rows",
      CssProperty::GridColumn => "grid-column",
      CssProperty::GridColumnEnd => "grid-column-end",
      CssProperty::GridColumnGap => "grid-column-gap",
      CssProperty::GridColumnStart => "grid-column-start",
      CssProperty::GridGap => "grid-gap",
      CssProperty::GridRow => "grid-row",
      CssProperty::GridRowEnd => "grid-row-end",
      CssProperty::GridRowGap => "grid-row-gap",
      CssProperty::GridRowStart => "grid-row-start",
      CssProperty::GridTemplate => "grid-template",
      CssProperty::GridTemplateAreas => "grid-template-areas",
      CssProperty::GridTemplateColumns => "grid-template-columns",
      CssProperty::GridTemplateRows => "grid-template-rows",
      CssProperty::HangingPunctuation => "hanging-punctuation",
      CssProperty::Height => "height",
      CssProperty::HyphenateCharacter => "hyphenate-character",
      CssProperty::HyphenateLimitChars => "hyphenate-limit-chars",
      CssProperty::HyphenateLimitLast => "hyphenate-limit-last",
      CssProperty::HyphenateLimitLines => "hyphenate-limit-lines",
      CssProperty::HyphenateLimitZone => "hyphenate-limit-zone",
      CssProperty::Hyphens => "hyphens",
      CssProperty::ImageAnimation => "image-animation",
      CssProperty::ImageOrientation => "image-orientation",
      CssProperty::ImageRendering => "image-rendering",
      CssProperty::ImageResolution => "image-resolution",
      CssProperty::InitialLetter => "initial-letter",
      CssProperty::InitialLetterAlign => "initial-letter-align",
      CssProperty::InitialLetterWrap => "initial-letter-wrap",
      CssProperty::InlineSize => "inline-size",
      CssProperty::InlineSizing => "inline-sizing",
      CssProperty::InputSecurity => "input-security",
      CssProperty::Inset => "inset",
      CssProperty::InsetBlock => "inset-block",
      CssProperty::InsetBlockEnd => "inset-block-end",
      CssProperty::InsetBlockStart => "inset-block-start",
      CssProperty::InsetInline => "inset-inline",
      CssProperty::InsetInlineEnd => "inset-inline-end",
      CssProperty::InsetInlineStart => "inset-inline-start",
      CssProperty::Interactivity => "interactivity",
      CssProperty::InterestDelay => "interest-delay",
      CssProperty::InterestDelayEnd => "interest-delay-end",
      CssProperty::InterestDelayStart => "interest-delay-start",
      CssProperty::InterpolateSize => "interpolate-size",
      CssProperty::Isolation => "isolation",
      CssProperty::JustifyContent => "justify-content",
      CssProperty::JustifyItems => "justify-items",
      CssProperty::JustifySelf => "justify-self",
      CssProperty::Left => "left",
      CssProperty::LetterSpacing => "letter-spacing",
      CssProperty::LightingColor => "lighting-color",
      CssProperty::LineBreak => "line-break",
      CssProperty::LineClamp => "line-clamp",
      CssProperty::LineFitEdge => "line-fit-edge",
      CssProperty::LineGrid => "line-grid",
      CssProperty::LineHeight => "line-height",
      CssProperty::LineHeightStep => "line-height-step",
      CssProperty::LinePadding => "line-padding",
      CssProperty::LineSnap => "line-snap",
      CssProperty::LinkParameters => "link-parameters",
      CssProperty::ListStyle => "list-style",
      CssProperty::ListStyleImage => "list-style-image",
      CssProperty::ListStylePosition => "list-style-position",
      CssProperty::ListStyleType => "list-style-type",
      CssProperty::Margin => "margin",
      CssProperty::MarginBlock => "margin-block",
      CssProperty::MarginBlockEnd => "margin-block-end",
      CssProperty::MarginBlockStart => "margin-block-start",
      CssProperty::MarginBottom => "margin-bottom",
      CssProperty::MarginBreak => "margin-break",
      CssProperty::MarginInline => "margin-inline",
      CssProperty::MarginInlineEnd => "margin-inline-end",
      CssProperty::MarginInlineStart => "margin-inline-start",
      CssProperty::MarginLeft => "margin-left",
      CssProperty::MarginRight => "margin-right",
      CssProperty::MarginTop => "margin-top",
      CssProperty::MarginTrim => "margin-trim",
      CssProperty::Marker => "marker",
      CssProperty::MarkerEnd => "marker-end",
      CssProperty::MarkerMid => "marker-mid",
      CssProperty::MarkerSide => "marker-side",
      CssProperty::MarkerStart => "marker-start",
      CssProperty::Mask => "mask",
      CssProperty::MaskBorder => "mask-border",
      CssProperty::MaskBorderMode => "mask-border-mode",
      CssProperty::MaskBorderOutset => "mask-border-outset",
      CssProperty::MaskBorderRepeat => "mask-border-repeat",
      CssProperty::MaskBorderSlice => "mask-border-slice",
      CssProperty::MaskBorderSource => "mask-border-source",
      CssProperty::MaskBorderWidth => "mask-border-width",
      CssProperty::MaskClip => "mask-clip",
      CssProperty::MaskComposite => "mask-composite",
      CssProperty::MaskImage => "mask-image",
      CssProperty::MaskMode => "mask-mode",
      CssProperty::MaskOrigin => "mask-origin",
      CssProperty::MaskPosition => "mask-position",
      CssProperty::MaskRepeat => "mask-repeat",
      CssProperty::MaskSize => "mask-size",
      CssProperty::MaskType => "mask-type",
      CssProperty::MathDepth => "math-depth",
      CssProperty::MathShift => "math-shift",
      CssProperty::MathStyle => "math-style",
      CssProperty::MaxBlockSize => "max-block-size",
      CssProperty::MaxHeight => "max-height",
      CssProperty::MaxInlineSize => "max-inline-size",
      CssProperty::MaxLines => "max-lines",
      CssProperty::MaxWidth => "max-width",
      CssProperty::MinBlockSize => "min-block-size",
      CssProperty::MinHeight => "min-height",
      CssProperty::MinInlineSize => "min-inline-size",
      CssProperty::MinIntrinsicSizing => "min-intrinsic-sizing",
      CssProperty::MinWidth => "min-width",
      CssProperty::MixBlendMode => "mix-blend-mode",
      CssProperty::NavDown => "nav-down",
      CssProperty::NavLeft => "nav-left",
      CssProperty::NavRight => "nav-right",
      CssProperty::NavUp => "nav-up",
      CssProperty::ObjectFit => "object-fit",
      CssProperty::ObjectPosition => "object-position",
      CssProperty::ObjectViewBox => "object-view-box",
      CssProperty::Offset => "offset",
      CssProperty::OffsetAnchor => "offset-anchor",
      CssProperty::OffsetDistance => "offset-distance",
      CssProperty::OffsetPath => "offset-path",
      CssProperty::OffsetPosition => "offset-position",
      CssProperty::OffsetRotate => "offset-rotate",
      CssProperty::Opacity => "opacity",
      CssProperty::Order => "order",
      CssProperty::Orphans => "orphans",
      CssProperty::Outline => "outline",
      CssProperty::OutlineColor => "outline-color",
      CssProperty::OutlineOffset => "outline-offset",
      CssProperty::OutlineStyle => "outline-style",
      CssProperty::OutlineWidth => "outline-width",
      CssProperty::Overflow => "overflow",
      CssProperty::OverflowAnchor => "overflow-anchor",
      CssProperty::OverflowBlock => "overflow-block",
      CssProperty::OverflowClipMargin => "overflow-clip-margin",
      CssProperty::OverflowClipMarginBlock => "overflow-clip-margin-block",
      CssProperty::OverflowClipMarginBlockEnd => "overflow-clip-margin-block-end",
      CssProperty::OverflowClipMarginBlockStart => "overflow-clip-margin-block-start",
      CssProperty::OverflowClipMarginBottom => "overflow-clip-margin-bottom",
      CssProperty::OverflowClipMarginInline => "overflow-clip-margin-inline",
      CssProperty::OverflowClipMarginInlineEnd => "overflow-clip-margin-inline-end",
      CssProperty::OverflowClipMarginInlineStart => "overflow-clip-margin-inline-start",
      CssProperty::OverflowClipMarginLeft => "overflow-clip-margin-left",
      CssProperty::OverflowClipMarginRight => "overflow-clip-margin-right",
      CssProperty::OverflowClipMarginTop => "overflow-clip-margin-top",
      CssProperty::OverflowInline => "overflow-inline",
      CssProperty::OverflowWrap => "overflow-wrap",
      CssProperty::OverflowX => "overflow-x",
      CssProperty::OverflowY => "overflow-y",
      CssProperty::Overlay => "overlay",
      CssProperty::OverscrollBehavior => "overscroll-behavior",
      CssProperty::OverscrollBehaviorBlock => "overscroll-behavior-block",
      CssProperty::OverscrollBehaviorInline => "overscroll-behavior-inline",
      CssProperty::OverscrollBehaviorX => "overscroll-behavior-x",
      CssProperty::OverscrollBehaviorY => "overscroll-behavior-y",
      CssProperty::Padding => "padding",
      CssProperty::PaddingBlock => "padding-block",
      CssProperty::PaddingBlockEnd => "padding-block-end",
      CssProperty::PaddingBlockStart => "padding-block-start",
      CssProperty::PaddingBottom => "padding-bottom",
      CssProperty::PaddingInline => "padding-inline",
      CssProperty::PaddingInlineEnd => "padding-inline-end",
      CssProperty::PaddingInlineStart => "padding-inline-start",
      CssProperty::PaddingLeft => "padding-left",
      CssProperty::PaddingRight => "padding-right",
      CssProperty::PaddingTop => "padding-top",
      CssProperty::Page => "page",
      CssProperty::PageBreakAfter => "page-break-after",
      CssProperty::PageBreakBefore => "page-break-before",
      CssProperty::PageBreakInside => "page-break-inside",
      CssProperty::PaintOrder => "paint-order",
      CssProperty::PathLength => "path-length",
      CssProperty::Pause => "pause",
      CssProperty::PauseAfter => "pause-after",
      CssProperty::PauseBefore => "pause-before",
      CssProperty::Perspective => "perspective",
      CssProperty::PerspectiveOrigin => "perspective-origin",
      CssProperty::PlaceContent => "place-content",
      CssProperty::PlaceItems => "place-items",
      CssProperty::PlaceSelf => "place-self",
      CssProperty::PointerEvents => "pointer-events",
      CssProperty::PointerTimeline => "pointer-timeline",
      CssProperty::PointerTimelineAxis => "pointer-timeline-axis",
      CssProperty::PointerTimelineName => "pointer-timeline-name",
      CssProperty::Position => "position",
      CssProperty::PositionAnchor => "position-anchor",
      CssProperty::PositionArea => "position-area",
      CssProperty::PositionTry => "position-try",
      CssProperty::PositionTryFallbacks => "position-try-fallbacks",
      CssProperty::PositionTryOrder => "position-try-order",
      CssProperty::PositionVisibility => "position-visibility",
      CssProperty::PrintColorAdjust => "print-color-adjust",
      CssProperty::Quotes => "quotes",
      CssProperty::R => "r",
      CssProperty::ReadingFlow => "reading-flow",
      CssProperty::ReadingOrder => "reading-order",
      CssProperty::RegionFragment => "region-fragment",
      CssProperty::Resize => "resize",
      CssProperty::Rest => "rest",
      CssProperty::RestAfter => "rest-after",
      CssProperty::RestBefore => "rest-before",
      CssProperty::Right => "right",
      CssProperty::Rotate => "rotate",
      CssProperty::RowGap => "row-gap",
      CssProperty::RowRule => "row-rule",
      CssProperty::RowRuleBreak => "row-rule-break",
      CssProperty::RowRuleColor => "row-rule-color",
      CssProperty::RowRuleInset => "row-rule-inset",
      CssProperty::RowRuleInsetCap => "row-rule-inset-cap",
      CssProperty::RowRuleInsetCapEnd => "row-rule-inset-cap-end",
      CssProperty::RowRuleInsetCapStart => "row-rule-inset-cap-start",
      CssProperty::RowRuleInsetEnd => "row-rule-inset-end",
      CssProperty::RowRuleInsetJunction => "row-rule-inset-junction",
      CssProperty::RowRuleInsetJunctionEnd => "row-rule-inset-junction-end",
      CssProperty::RowRuleInsetJunctionStart => "row-rule-inset-junction-start",
      CssProperty::RowRuleInsetStart => "row-rule-inset-start",
      CssProperty::RowRuleStyle => "row-rule-style",
      CssProperty::RowRuleVisibilityItems => "row-rule-visibility-items",
      CssProperty::RowRuleWidth => "row-rule-width",
      CssProperty::RubyAlign => "ruby-align",
      CssProperty::RubyMerge => "ruby-merge",
      CssProperty::RubyOverhang => "ruby-overhang",
      CssProperty::RubyPosition => "ruby-position",
      CssProperty::Rule => "rule",
      CssProperty::RuleBreak => "rule-break",
      CssProperty::RuleColor => "rule-color",
      CssProperty::RuleInset => "rule-inset",
      CssProperty::RuleInsetCap => "rule-inset-cap",
      CssProperty::RuleInsetEnd => "rule-inset-end",
      CssProperty::RuleInsetJunction => "rule-inset-junction",
      CssProperty::RuleInsetStart => "rule-inset-start",
      CssProperty::RuleOverlap => "rule-overlap",
      CssProperty::RuleStyle => "rule-style",
      CssProperty::RuleVisibilityItems => "rule-visibility-items",
      CssProperty::RuleWidth => "rule-width",
      CssProperty::Rx => "rx",
      CssProperty::Ry => "ry",
      CssProperty::Scale => "scale",
      CssProperty::ScrollBehavior => "scroll-behavior",
      CssProperty::ScrollInitialTarget => "scroll-initial-target",
      CssProperty::ScrollMargin => "scroll-margin",
      CssProperty::ScrollMarginBlock => "scroll-margin-block",
      CssProperty::ScrollMarginBlockEnd => "scroll-margin-block-end",
      CssProperty::ScrollMarginBlockStart => "scroll-margin-block-start",
      CssProperty::ScrollMarginBottom => "scroll-margin-bottom",
      CssProperty::ScrollMarginInline => "scroll-margin-inline",
      CssProperty::ScrollMarginInlineEnd => "scroll-margin-inline-end",
      CssProperty::ScrollMarginInlineStart => "scroll-margin-inline-start",
      CssProperty::ScrollMarginLeft => "scroll-margin-left",
      CssProperty::ScrollMarginRight => "scroll-margin-right",
      CssProperty::ScrollMarginTop => "scroll-margin-top",
      CssProperty::ScrollMarkerGroup => "scroll-marker-group",
      CssProperty::ScrollPadding => "scroll-padding",
      CssProperty::ScrollPaddingBlock => "scroll-padding-block",
      CssProperty::ScrollPaddingBlockEnd => "scroll-padding-block-end",
      CssProperty::ScrollPaddingBlockStart => "scroll-padding-block-start",
      CssProperty::ScrollPaddingBottom => "scroll-padding-bottom",
      CssProperty::ScrollPaddingInline => "scroll-padding-inline",
      CssProperty::ScrollPaddingInlineEnd => "scroll-padding-inline-end",
      CssProperty::ScrollPaddingInlineStart => "scroll-padding-inline-start",
      CssProperty::ScrollPaddingLeft => "scroll-padding-left",
      CssProperty::ScrollPaddingRight => "scroll-padding-right",
      CssProperty::ScrollPaddingTop => "scroll-padding-top",
      CssProperty::ScrollSnapAlign => "scroll-snap-align",
      CssProperty::ScrollSnapStop => "scroll-snap-stop",
      CssProperty::ScrollSnapType => "scroll-snap-type",
      CssProperty::ScrollTargetGroup => "scroll-target-group",
      CssProperty::ScrollTimeline => "scroll-timeline",
      CssProperty::ScrollTimelineAxis => "scroll-timeline-axis",
      CssProperty::ScrollTimelineName => "scroll-timeline-name",
      CssProperty::ScrollbarColor => "scrollbar-color",
      CssProperty::ScrollbarGutter => "scrollbar-gutter",
      CssProperty::ScrollbarInset => "scrollbar-inset",
      CssProperty::ScrollbarMinThumbSize => "scrollbar-min-thumb-size",
      CssProperty::ScrollbarMode => "scrollbar-mode",
      CssProperty::ScrollbarWidth => "scrollbar-width",
      CssProperty::ShapeImageThreshold => "shape-image-threshold",
      CssProperty::ShapeInside => "shape-inside",
      CssProperty::ShapeMargin => "shape-margin",
      CssProperty::ShapeOutside => "shape-outside",
      CssProperty::ShapePadding => "shape-padding",
      CssProperty::ShapeRendering => "shape-rendering",
      CssProperty::SliderOrientation => "slider-orientation",
      CssProperty::SpatialNavigationAction => "spatial-navigation-action",
      CssProperty::SpatialNavigationContain => "spatial-navigation-contain",
      CssProperty::SpatialNavigationFunction => "spatial-navigation-function",
      CssProperty::Speak => "speak",
      CssProperty::SpeakAs => "speak-as",
      CssProperty::StopColor => "stop-color",
      CssProperty::StopOpacity => "stop-opacity",
      CssProperty::StringSet => "string-set",
      CssProperty::Stroke => "stroke",
      CssProperty::StrokeAlign => "stroke-align",
      CssProperty::StrokeAlignment => "stroke-alignment",
      CssProperty::StrokeBreak => "stroke-break",
      CssProperty::StrokeColor => "stroke-color",
      CssProperty::StrokeDashCornerPropdefStrokeDashCorner => "stroke-dash-corner",
      CssProperty::StrokeDashJustify => "stroke-dash-justify",
      CssProperty::StrokeDashadjust => "stroke-dashadjust",
      CssProperty::StrokeDasharray => "stroke-dasharray",
      CssProperty::StrokeDashcornerStrokedashcornerproperty => "stroke-dashcorner",
      CssProperty::StrokeDashoffset => "stroke-dashoffset",
      CssProperty::StrokeImage => "stroke-image",
      CssProperty::StrokeLinecap => "stroke-linecap",
      CssProperty::StrokeLinejoin => "stroke-linejoin",
      CssProperty::StrokeMiterlimit => "stroke-miterlimit",
      CssProperty::StrokeOpacity => "stroke-opacity",
      CssProperty::StrokeOrigin => "stroke-origin",
      CssProperty::StrokePosition => "stroke-position",
      CssProperty::StrokeRepeat => "stroke-repeat",
      CssProperty::StrokeSize => "stroke-size",
      CssProperty::StrokeWidth => "stroke-width",
      CssProperty::TabSize => "tab-size",
      CssProperty::TableLayout => "table-layout",
      CssProperty::TextAlign => "text-align",
      CssProperty::TextAlignAll => "text-align-all",
      CssProperty::TextAlignLast => "text-align-last",
      CssProperty::TextAnchor => "text-anchor",
      CssProperty::TextAutospace => "text-autospace",
      CssProperty::TextBox => "text-box",
      CssProperty::TextBoxEdge => "text-box-edge",
      CssProperty::TextBoxTrim => "text-box-trim",
      CssProperty::TextCombineUpright => "text-combine-upright",
      CssProperty::TextDecoration => "text-decoration",
      CssProperty::TextDecorationColor => "text-decoration-color",
      CssProperty::TextDecorationInset => "text-decoration-inset",
      CssProperty::TextDecorationLine => "text-decoration-line",
      CssProperty::TextDecorationSkip => "text-decoration-skip",
      CssProperty::TextDecorationSkipBox => "text-decoration-skip-box",
      CssProperty::TextDecorationSkipInk => "text-decoration-skip-ink",
      CssProperty::TextDecorationSkipSelf => "text-decoration-skip-self",
      CssProperty::TextDecorationSkipSpaces => "text-decoration-skip-spaces",
      CssProperty::TextDecorationStyle => "text-decoration-style",
      CssProperty::TextDecorationThickness => "text-decoration-thickness",
      CssProperty::TextEmphasis => "text-emphasis",
      CssProperty::TextEmphasisColor => "text-emphasis-color",
      CssProperty::TextEmphasisPosition => "text-emphasis-position",
      CssProperty::TextEmphasisSkip => "text-emphasis-skip",
      CssProperty::TextEmphasisStyle => "text-emphasis-style",
      CssProperty::TextFit => "text-fit",
      CssProperty::TextGroupAlign => "text-group-align",
      CssProperty::TextIndent => "text-indent",
      CssProperty::TextJustify => "text-justify",
      CssProperty::TextOrientation => "text-orientation",
      CssProperty::TextOverflow => "text-overflow",
      CssProperty::TextRendering => "text-rendering",
      CssProperty::TextShadow => "text-shadow",
      CssProperty::TextSizeAdjust => "text-size-adjust",
      CssProperty::TextSpacing => "text-spacing",
      CssProperty::TextSpacingTrim => "text-spacing-trim",
      CssProperty::TextTransform => "text-transform",
      CssProperty::TextUnderlineOffset => "text-underline-offset",
      CssProperty::TextUnderlinePosition => "text-underline-position",
      CssProperty::TextWrap => "text-wrap",
      CssProperty::TextWrapMode => "text-wrap-mode",
      CssProperty::TextWrapStyle => "text-wrap-style",
      CssProperty::TimelineScope => "timeline-scope",
      CssProperty::TimelineTrigger => "timeline-trigger",
      CssProperty::TimelineTriggerActivationRange => "timeline-trigger-activation-range",
      CssProperty::TimelineTriggerActivationRangeEnd => "timeline-trigger-activation-range-end",
      CssProperty::TimelineTriggerActivationRangeStart => "timeline-trigger-activation-range-start",
      CssProperty::TimelineTriggerActiveRange => "timeline-trigger-active-range",
      CssProperty::TimelineTriggerActiveRangeEnd => "timeline-trigger-active-range-end",
      CssProperty::TimelineTriggerActiveRangeStart => "timeline-trigger-active-range-start",
      CssProperty::TimelineTriggerName => "timeline-trigger-name",
      CssProperty::TimelineTriggerSource => "timeline-trigger-source",
      CssProperty::Top => "top",
      CssProperty::TouchAction => "touch-action",
      CssProperty::Transform => "transform",
      CssProperty::TransformBox => "transform-box",
      CssProperty::TransformOrigin => "transform-origin",
      CssProperty::TransformStyle => "transform-style",
      CssProperty::Transition => "transition",
      CssProperty::TransitionBehavior => "transition-behavior",
      CssProperty::TransitionDelay => "transition-delay",
      CssProperty::TransitionDuration => "transition-duration",
      CssProperty::TransitionProperty => "transition-property",
      CssProperty::TransitionTimingFunction => "transition-timing-function",
      CssProperty::Translate => "translate",
      CssProperty::TriggerScope => "trigger-scope",
      CssProperty::UnicodeBidi => "unicode-bidi",
      CssProperty::UserSelect => "user-select",
      CssProperty::VectorEffect => "vector-effect",
      CssProperty::VerticalAlign => "vertical-align",
      CssProperty::ViewTimeline => "view-timeline",
      CssProperty::ViewTimelineAxis => "view-timeline-axis",
      CssProperty::ViewTimelineInset => "view-timeline-inset",
      CssProperty::ViewTimelineName => "view-timeline-name",
      CssProperty::ViewTransitionClass => "view-transition-class",
      CssProperty::ViewTransitionGroup => "view-transition-group",
      CssProperty::ViewTransitionName => "view-transition-name",
      CssProperty::ViewTransitionScope => "view-transition-scope",
      CssProperty::Visibility => "visibility",
      CssProperty::VoiceBalance => "voice-balance",
      CssProperty::VoiceDuration => "voice-duration",
      CssProperty::VoiceFamily => "voice-family",
      CssProperty::VoicePitch => "voice-pitch",
      CssProperty::VoiceRange => "voice-range",
      CssProperty::VoiceRate => "voice-rate",
      CssProperty::VoiceStress => "voice-stress",
      CssProperty::VoiceVolume => "voice-volume",
      CssProperty::WhiteSpace => "white-space",
      CssProperty::WhiteSpaceCollapse => "white-space-collapse",
      CssProperty::WhiteSpaceTrim => "white-space-trim",
      CssProperty::Widows => "widows",
      CssProperty::Width => "width",
      CssProperty::WillChange => "will-change",
      CssProperty::WordBreak => "word-break",
      CssProperty::WordSpaceTransform => "word-space-transform",
      CssProperty::WordSpacing => "word-spacing",
      CssProperty::WordWrap => "word-wrap",
      CssProperty::WrapAfter => "wrap-after",
      CssProperty::WrapBefore => "wrap-before",
      CssProperty::WrapFlow => "wrap-flow",
      CssProperty::WrapInside => "wrap-inside",
      CssProperty::WrapThrough => "wrap-through",
      CssProperty::WritingMode => "writing-mode",
      CssProperty::X => "x",
      CssProperty::Y => "y",
      CssProperty::ZIndex => "z-index",
      CssProperty::Zoom => "zoom",
      CssProperty::Unknown(_) => "",
    }
  }

  pub fn from_name(name: &str) -> Self {
    const ENTRIES: &[(&str, CssProperty)] = &[
      ("-webkit-align-content", CssProperty::WebkitAlignContent),
      ("-webkit-align-items", CssProperty::WebkitAlignItems),
      ("-webkit-align-self", CssProperty::WebkitAlignSelf),
      ("-webkit-animation", CssProperty::WebkitAnimation),
      ("-webkit-animation-delay", CssProperty::WebkitAnimationDelay),
      ("-webkit-animation-direction", CssProperty::WebkitAnimationDirection),
      ("-webkit-animation-duration", CssProperty::WebkitAnimationDuration),
      ("-webkit-animation-fill-mode", CssProperty::WebkitAnimationFillMode),
      (
        "-webkit-animation-iteration-count",
        CssProperty::WebkitAnimationIterationCount,
      ),
      ("-webkit-animation-name", CssProperty::WebkitAnimationName),
      ("-webkit-animation-play-state", CssProperty::WebkitAnimationPlayState),
      (
        "-webkit-animation-timing-function",
        CssProperty::WebkitAnimationTimingFunction,
      ),
      ("-webkit-appearance", CssProperty::WebkitAppearance),
      ("-webkit-backface-visibility", CssProperty::WebkitBackfaceVisibility),
      ("-webkit-background-clip", CssProperty::WebkitBackgroundClip),
      ("-webkit-background-origin", CssProperty::WebkitBackgroundOrigin),
      ("-webkit-background-size", CssProperty::WebkitBackgroundSize),
      (
        "-webkit-border-bottom-left-radius",
        CssProperty::WebkitBorderBottomLeftRadius,
      ),
      (
        "-webkit-border-bottom-right-radius",
        CssProperty::WebkitBorderBottomRightRadius,
      ),
      ("-webkit-border-radius", CssProperty::WebkitBorderRadius),
      ("-webkit-border-top-left-radius", CssProperty::WebkitBorderTopLeftRadius),
      (
        "-webkit-border-top-right-radius",
        CssProperty::WebkitBorderTopRightRadius,
      ),
      ("-webkit-box-align", CssProperty::WebkitBoxAlign),
      ("-webkit-box-flex", CssProperty::WebkitBoxFlex),
      ("-webkit-box-ordinal-group", CssProperty::WebkitBoxOrdinalGroup),
      ("-webkit-box-orient", CssProperty::WebkitBoxOrient),
      ("-webkit-box-pack", CssProperty::WebkitBoxPack),
      ("-webkit-box-shadow", CssProperty::WebkitBoxShadow),
      ("-webkit-box-sizing", CssProperty::WebkitBoxSizing),
      ("-webkit-filter", CssProperty::WebkitFilter),
      ("-webkit-flex", CssProperty::WebkitFlex),
      ("-webkit-flex-basis", CssProperty::WebkitFlexBasis),
      ("-webkit-flex-direction", CssProperty::WebkitFlexDirection),
      ("-webkit-flex-flow", CssProperty::WebkitFlexFlow),
      ("-webkit-flex-grow", CssProperty::WebkitFlexGrow),
      ("-webkit-flex-shrink", CssProperty::WebkitFlexShrink),
      ("-webkit-flex-wrap", CssProperty::WebkitFlexWrap),
      ("-webkit-justify-content", CssProperty::WebkitJustifyContent),
      ("-webkit-line-clamp", CssProperty::WebkitLineClamp),
      ("-webkit-mask", CssProperty::WebkitMask),
      ("-webkit-mask-box-image", CssProperty::WebkitMaskBoxImage),
      ("-webkit-mask-box-image-outset", CssProperty::WebkitMaskBoxImageOutset),
      ("-webkit-mask-box-image-repeat", CssProperty::WebkitMaskBoxImageRepeat),
      ("-webkit-mask-box-image-slice", CssProperty::WebkitMaskBoxImageSlice),
      ("-webkit-mask-box-image-source", CssProperty::WebkitMaskBoxImageSource),
      ("-webkit-mask-box-image-width", CssProperty::WebkitMaskBoxImageWidth),
      ("-webkit-mask-clip", CssProperty::WebkitMaskClip),
      ("-webkit-mask-composite", CssProperty::WebkitMaskComposite),
      ("-webkit-mask-image", CssProperty::WebkitMaskImage),
      ("-webkit-mask-origin", CssProperty::WebkitMaskOrigin),
      ("-webkit-mask-position", CssProperty::WebkitMaskPosition),
      ("-webkit-mask-repeat", CssProperty::WebkitMaskRepeat),
      ("-webkit-mask-size", CssProperty::WebkitMaskSize),
      ("-webkit-order", CssProperty::WebkitOrder),
      ("-webkit-perspective", CssProperty::WebkitPerspective),
      ("-webkit-perspective-origin", CssProperty::WebkitPerspectiveOrigin),
      ("-webkit-text-fill-color", CssProperty::WebkitTextFillColor),
      ("-webkit-text-size-adjust", CssProperty::WebkitTextSizeAdjust),
      ("-webkit-text-stroke", CssProperty::WebkitTextStroke),
      ("-webkit-text-stroke-color", CssProperty::WebkitTextStrokeColor),
      ("-webkit-text-stroke-width", CssProperty::WebkitTextStrokeWidth),
      ("-webkit-transform", CssProperty::WebkitTransform),
      ("-webkit-transform-origin", CssProperty::WebkitTransformOrigin),
      ("-webkit-transform-style", CssProperty::WebkitTransformStyle),
      ("-webkit-transition", CssProperty::WebkitTransition),
      ("-webkit-transition-delay", CssProperty::WebkitTransitionDelay),
      ("-webkit-transition-duration", CssProperty::WebkitTransitionDuration),
      ("-webkit-transition-property", CssProperty::WebkitTransitionProperty),
      (
        "-webkit-transition-timing-function",
        CssProperty::WebkitTransitionTimingFunction,
      ),
      ("-webkit-user-select", CssProperty::WebkitUserSelect),
      ("accent-color", CssProperty::AccentColor),
      ("align-content", CssProperty::AlignContent),
      ("align-items", CssProperty::AlignItems),
      ("align-self", CssProperty::AlignSelf),
      ("alignment-baseline", CssProperty::AlignmentBaseline),
      ("all", CssProperty::All),
      ("anchor-name", CssProperty::AnchorName),
      ("anchor-scope", CssProperty::AnchorScope),
      ("animation", CssProperty::Animation),
      ("animation-composition", CssProperty::AnimationComposition),
      ("animation-delay", CssProperty::AnimationDelay),
      ("animation-direction", CssProperty::AnimationDirection),
      ("animation-duration", CssProperty::AnimationDuration),
      ("animation-fill-mode", CssProperty::AnimationFillMode),
      ("animation-iteration-count", CssProperty::AnimationIterationCount),
      ("animation-name", CssProperty::AnimationName),
      ("animation-play-state", CssProperty::AnimationPlayState),
      ("animation-range", CssProperty::AnimationRange),
      ("animation-range-center", CssProperty::AnimationRangeCenter),
      ("animation-range-end", CssProperty::AnimationRangeEnd),
      ("animation-range-start", CssProperty::AnimationRangeStart),
      ("animation-timeline", CssProperty::AnimationTimeline),
      ("animation-timing-function", CssProperty::AnimationTimingFunction),
      ("animation-trigger", CssProperty::AnimationTrigger),
      ("appearance", CssProperty::Appearance),
      ("aspect-ratio", CssProperty::AspectRatio),
      ("backdrop-filter", CssProperty::BackdropFilter),
      ("backface-visibility", CssProperty::BackfaceVisibility),
      ("background", CssProperty::Background),
      ("background-attachment", CssProperty::BackgroundAttachment),
      ("background-blend-mode", CssProperty::BackgroundBlendMode),
      ("background-clip", CssProperty::BackgroundClip),
      ("background-color", CssProperty::BackgroundColor),
      ("background-image", CssProperty::BackgroundImage),
      ("background-origin", CssProperty::BackgroundOrigin),
      ("background-position", CssProperty::BackgroundPosition),
      ("background-position-block", CssProperty::BackgroundPositionBlock),
      ("background-position-inline", CssProperty::BackgroundPositionInline),
      ("background-position-x", CssProperty::BackgroundPositionX),
      ("background-position-y", CssProperty::BackgroundPositionY),
      ("background-repeat", CssProperty::BackgroundRepeat),
      ("background-repeat-block", CssProperty::BackgroundRepeatBlock),
      ("background-repeat-inline", CssProperty::BackgroundRepeatInline),
      ("background-repeat-x", CssProperty::BackgroundRepeatX),
      ("background-repeat-y", CssProperty::BackgroundRepeatY),
      ("background-size", CssProperty::BackgroundSize),
      ("background-tbd", CssProperty::BackgroundTbd),
      ("baseline-shift", CssProperty::BaselineShift),
      ("baseline-source", CssProperty::BaselineSource),
      ("block-ellipsis", CssProperty::BlockEllipsis),
      ("block-size", CssProperty::BlockSize),
      ("block-step", CssProperty::BlockStep),
      ("block-step-align", CssProperty::BlockStepAlign),
      ("block-step-insert", CssProperty::BlockStepInsert),
      ("block-step-round", CssProperty::BlockStepRound),
      ("block-step-size", CssProperty::BlockStepSize),
      ("bookmark-label", CssProperty::BookmarkLabel),
      ("bookmark-level", CssProperty::BookmarkLevel),
      ("bookmark-state", CssProperty::BookmarkState),
      ("border", CssProperty::Border),
      ("border-block", CssProperty::BorderBlock),
      ("border-block-clip", CssProperty::BorderBlockClip),
      ("border-block-color", CssProperty::BorderBlockColor),
      ("border-block-end", CssProperty::BorderBlockEnd),
      ("border-block-end-clip", CssProperty::BorderBlockEndClip),
      ("border-block-end-color", CssProperty::BorderBlockEndColor),
      ("border-block-end-radius", CssProperty::BorderBlockEndRadius),
      ("border-block-end-style", CssProperty::BorderBlockEndStyle),
      ("border-block-end-width", CssProperty::BorderBlockEndWidth),
      ("border-block-start", CssProperty::BorderBlockStart),
      ("border-block-start-clip", CssProperty::BorderBlockStartClip),
      ("border-block-start-color", CssProperty::BorderBlockStartColor),
      ("border-block-start-radius", CssProperty::BorderBlockStartRadius),
      ("border-block-start-style", CssProperty::BorderBlockStartStyle),
      ("border-block-start-width", CssProperty::BorderBlockStartWidth),
      ("border-block-style", CssProperty::BorderBlockStyle),
      ("border-block-width", CssProperty::BorderBlockWidth),
      ("border-bottom", CssProperty::BorderBottom),
      ("border-bottom-clip", CssProperty::BorderBottomClip),
      ("border-bottom-color", CssProperty::BorderBottomColor),
      ("border-bottom-left-radius", CssProperty::BorderBottomLeftRadius),
      ("border-bottom-radius", CssProperty::BorderBottomRadius),
      ("border-bottom-right-radius", CssProperty::BorderBottomRightRadius),
      ("border-bottom-style", CssProperty::BorderBottomStyle),
      ("border-bottom-width", CssProperty::BorderBottomWidth),
      ("border-boundary", CssProperty::BorderBoundary),
      ("border-clip", CssProperty::BorderClip),
      ("border-collapse", CssProperty::BorderCollapse),
      ("border-color", CssProperty::BorderColor),
      ("border-end-end-radius", CssProperty::BorderEndEndRadius),
      ("border-end-start-radius", CssProperty::BorderEndStartRadius),
      ("border-image", CssProperty::BorderImage),
      ("border-image-outset", CssProperty::BorderImageOutset),
      ("border-image-repeat", CssProperty::BorderImageRepeat),
      ("border-image-slice", CssProperty::BorderImageSlice),
      ("border-image-source", CssProperty::BorderImageSource),
      ("border-image-width", CssProperty::BorderImageWidth),
      ("border-inline", CssProperty::BorderInline),
      ("border-inline-clip", CssProperty::BorderInlineClip),
      ("border-inline-color", CssProperty::BorderInlineColor),
      ("border-inline-end", CssProperty::BorderInlineEnd),
      ("border-inline-end-clip", CssProperty::BorderInlineEndClip),
      ("border-inline-end-color", CssProperty::BorderInlineEndColor),
      ("border-inline-end-radius", CssProperty::BorderInlineEndRadius),
      ("border-inline-end-style", CssProperty::BorderInlineEndStyle),
      ("border-inline-end-width", CssProperty::BorderInlineEndWidth),
      ("border-inline-start", CssProperty::BorderInlineStart),
      ("border-inline-start-clip", CssProperty::BorderInlineStartClip),
      ("border-inline-start-color", CssProperty::BorderInlineStartColor),
      ("border-inline-start-radius", CssProperty::BorderInlineStartRadius),
      ("border-inline-start-style", CssProperty::BorderInlineStartStyle),
      ("border-inline-start-width", CssProperty::BorderInlineStartWidth),
      ("border-inline-style", CssProperty::BorderInlineStyle),
      ("border-inline-width", CssProperty::BorderInlineWidth),
      ("border-left", CssProperty::BorderLeft),
      ("border-left-clip", CssProperty::BorderLeftClip),
      ("border-left-color", CssProperty::BorderLeftColor),
      ("border-left-radius", CssProperty::BorderLeftRadius),
      ("border-left-style", CssProperty::BorderLeftStyle),
      ("border-left-width", CssProperty::BorderLeftWidth),
      ("border-limit", CssProperty::BorderLimit),
      ("border-radius", CssProperty::BorderRadius),
      ("border-right", CssProperty::BorderRight),
      ("border-right-clip", CssProperty::BorderRightClip),
      ("border-right-color", CssProperty::BorderRightColor),
      ("border-right-radius", CssProperty::BorderRightRadius),
      ("border-right-style", CssProperty::BorderRightStyle),
      ("border-right-width", CssProperty::BorderRightWidth),
      ("border-shape", CssProperty::BorderShape),
      ("border-spacing", CssProperty::BorderSpacing),
      ("border-start-end-radius", CssProperty::BorderStartEndRadius),
      ("border-start-start-radius", CssProperty::BorderStartStartRadius),
      ("border-style", CssProperty::BorderStyle),
      ("border-top", CssProperty::BorderTop),
      ("border-top-clip", CssProperty::BorderTopClip),
      ("border-top-color", CssProperty::BorderTopColor),
      ("border-top-left-radius", CssProperty::BorderTopLeftRadius),
      ("border-top-radius", CssProperty::BorderTopRadius),
      ("border-top-right-radius", CssProperty::BorderTopRightRadius),
      ("border-top-style", CssProperty::BorderTopStyle),
      ("border-top-width", CssProperty::BorderTopWidth),
      ("border-width", CssProperty::BorderWidth),
      ("bottom", CssProperty::Bottom),
      ("box-decoration-break", CssProperty::BoxDecorationBreak),
      ("box-shadow", CssProperty::BoxShadow),
      ("box-shadow-blur", CssProperty::BoxShadowBlur),
      ("box-shadow-color", CssProperty::BoxShadowColor),
      ("box-shadow-offset", CssProperty::BoxShadowOffset),
      ("box-shadow-position", CssProperty::BoxShadowPosition),
      ("box-shadow-spread", CssProperty::BoxShadowSpread),
      ("box-sizing", CssProperty::BoxSizing),
      ("box-snap", CssProperty::BoxSnap),
      ("break-after", CssProperty::BreakAfter),
      ("break-before", CssProperty::BreakBefore),
      ("break-inside", CssProperty::BreakInside),
      ("caption-side", CssProperty::CaptionSide),
      ("caret", CssProperty::Caret),
      ("caret-animation", CssProperty::CaretAnimation),
      ("caret-color", CssProperty::CaretColor),
      ("caret-shape", CssProperty::CaretShape),
      ("clear", CssProperty::Clear),
      ("clip", CssProperty::Clip),
      ("clip-path", CssProperty::ClipPath),
      ("clip-rule", CssProperty::ClipRule),
      ("color", CssProperty::Color),
      ("color-adjust", CssProperty::ColorAdjust),
      ("color-interpolation", CssProperty::ColorInterpolation),
      ("color-interpolation-filters", CssProperty::ColorInterpolationFilters),
      ("color-scheme", CssProperty::ColorScheme),
      ("column-count", CssProperty::ColumnCount),
      ("column-fill", CssProperty::ColumnFill),
      ("column-gap", CssProperty::ColumnGap),
      ("column-height", CssProperty::ColumnHeight),
      ("column-rule", CssProperty::ColumnRule),
      ("column-rule-break", CssProperty::ColumnRuleBreak),
      ("column-rule-color", CssProperty::ColumnRuleColor),
      ("column-rule-inset", CssProperty::ColumnRuleInset),
      ("column-rule-inset-cap", CssProperty::ColumnRuleInsetCap),
      ("column-rule-inset-cap-end", CssProperty::ColumnRuleInsetCapEnd),
      ("column-rule-inset-cap-start", CssProperty::ColumnRuleInsetCapStart),
      ("column-rule-inset-end", CssProperty::ColumnRuleInsetEnd),
      ("column-rule-inset-junction", CssProperty::ColumnRuleInsetJunction),
      (
        "column-rule-inset-junction-end",
        CssProperty::ColumnRuleInsetJunctionEnd,
      ),
      (
        "column-rule-inset-junction-start",
        CssProperty::ColumnRuleInsetJunctionStart,
      ),
      ("column-rule-inset-start", CssProperty::ColumnRuleInsetStart),
      ("column-rule-style", CssProperty::ColumnRuleStyle),
      ("column-rule-visibility-items", CssProperty::ColumnRuleVisibilityItems),
      ("column-rule-width", CssProperty::ColumnRuleWidth),
      ("column-span", CssProperty::ColumnSpan),
      ("column-width", CssProperty::ColumnWidth),
      ("column-wrap", CssProperty::ColumnWrap),
      ("columns", CssProperty::Columns),
      ("contain", CssProperty::Contain),
      ("contain-intrinsic-block-size", CssProperty::ContainIntrinsicBlockSize),
      ("contain-intrinsic-height", CssProperty::ContainIntrinsicHeight),
      ("contain-intrinsic-inline-size", CssProperty::ContainIntrinsicInlineSize),
      ("contain-intrinsic-size", CssProperty::ContainIntrinsicSize),
      ("contain-intrinsic-width", CssProperty::ContainIntrinsicWidth),
      ("container", CssProperty::Container),
      ("container-name", CssProperty::ContainerName),
      ("container-type", CssProperty::ContainerType),
      ("content", CssProperty::Content),
      ("content-visibility", CssProperty::ContentVisibility),
      ("continue", CssProperty::Continue),
      ("copy-into", CssProperty::CopyInto),
      ("corner", CssProperty::Corner),
      ("corner-block-end", CssProperty::CornerBlockEnd),
      ("corner-block-end-shape", CssProperty::CornerBlockEndShape),
      ("corner-block-start", CssProperty::CornerBlockStart),
      ("corner-block-start-shape", CssProperty::CornerBlockStartShape),
      ("corner-bottom", CssProperty::CornerBottom),
      ("corner-bottom-left", CssProperty::CornerBottomLeft),
      ("corner-bottom-left-shape", CssProperty::CornerBottomLeftShape),
      ("corner-bottom-right", CssProperty::CornerBottomRight),
      ("corner-bottom-right-shape", CssProperty::CornerBottomRightShape),
      ("corner-bottom-shape", CssProperty::CornerBottomShape),
      ("corner-end-end", CssProperty::CornerEndEnd),
      ("corner-end-end-shape", CssProperty::CornerEndEndShape),
      ("corner-end-start", CssProperty::CornerEndStart),
      ("corner-end-start-shape", CssProperty::CornerEndStartShape),
      ("corner-inline-end", CssProperty::CornerInlineEnd),
      ("corner-inline-end-shape", CssProperty::CornerInlineEndShape),
      ("corner-inline-start", CssProperty::CornerInlineStart),
      ("corner-inline-start-shape", CssProperty::CornerInlineStartShape),
      ("corner-left", CssProperty::CornerLeft),
      ("corner-left-shape", CssProperty::CornerLeftShape),
      ("corner-right", CssProperty::CornerRight),
      ("corner-right-shape", CssProperty::CornerRightShape),
      ("corner-shape", CssProperty::CornerShape),
      ("corner-start-end", CssProperty::CornerStartEnd),
      ("corner-start-end-shape", CssProperty::CornerStartEndShape),
      ("corner-start-start", CssProperty::CornerStartStart),
      ("corner-start-start-shape", CssProperty::CornerStartStartShape),
      ("corner-top", CssProperty::CornerTop),
      ("corner-top-left", CssProperty::CornerTopLeft),
      ("corner-top-left-shape", CssProperty::CornerTopLeftShape),
      ("corner-top-right", CssProperty::CornerTopRight),
      ("corner-top-right-shape", CssProperty::CornerTopRightShape),
      ("corner-top-shape", CssProperty::CornerTopShape),
      ("counter-increment", CssProperty::CounterIncrement),
      ("counter-reset", CssProperty::CounterReset),
      ("counter-set", CssProperty::CounterSet),
      ("cue", CssProperty::Cue),
      ("cue-after", CssProperty::CueAfter),
      ("cue-before", CssProperty::CueBefore),
      ("cursor", CssProperty::Cursor),
      ("cx", CssProperty::Cx),
      ("cy", CssProperty::Cy),
      ("d", CssProperty::D),
      ("direction", CssProperty::Direction),
      ("display", CssProperty::Display),
      ("dominant-baseline", CssProperty::DominantBaseline),
      ("dynamic-range-limit", CssProperty::DynamicRangeLimit),
      ("empty-cells", CssProperty::EmptyCells),
      ("event-trigger", CssProperty::EventTrigger),
      ("event-trigger-name", CssProperty::EventTriggerName),
      ("event-trigger-source", CssProperty::EventTriggerSource),
      ("field-sizing", CssProperty::FieldSizing),
      ("fill", CssProperty::Fill),
      ("fill-break", CssProperty::FillBreak),
      ("fill-color", CssProperty::FillColor),
      ("fill-image", CssProperty::FillImage),
      ("fill-opacity", CssProperty::FillOpacity),
      ("fill-origin", CssProperty::FillOrigin),
      ("fill-position", CssProperty::FillPosition),
      ("fill-repeat", CssProperty::FillRepeat),
      ("fill-rule", CssProperty::FillRule),
      ("fill-size", CssProperty::FillSize),
      ("filter", CssProperty::Filter),
      ("flex", CssProperty::Flex),
      ("flex-basis", CssProperty::FlexBasis),
      ("flex-direction", CssProperty::FlexDirection),
      ("flex-flow", CssProperty::FlexFlow),
      ("flex-grow", CssProperty::FlexGrow),
      ("flex-shrink", CssProperty::FlexShrink),
      ("flex-wrap", CssProperty::FlexWrap),
      ("float", CssProperty::Float),
      ("float-defer", CssProperty::FloatDefer),
      ("float-offset", CssProperty::FloatOffset),
      ("float-reference", CssProperty::FloatReference),
      ("flood-color", CssProperty::FloodColor),
      ("flood-opacity", CssProperty::FloodOpacity),
      ("flow-from", CssProperty::FlowFrom),
      ("flow-into", CssProperty::FlowInto),
      ("flow-tolerance", CssProperty::FlowTolerance),
      ("font", CssProperty::Font),
      ("font-family", CssProperty::FontFamily),
      ("font-feature-settings", CssProperty::FontFeatureSettings),
      ("font-kerning", CssProperty::FontKerning),
      ("font-language-override", CssProperty::FontLanguageOverride),
      ("font-optical-sizing", CssProperty::FontOpticalSizing),
      ("font-palette", CssProperty::FontPalette),
      ("font-size", CssProperty::FontSize),
      ("font-size-adjust", CssProperty::FontSizeAdjust),
      ("font-stretch", CssProperty::FontStretch),
      ("font-style", CssProperty::FontStyle),
      ("font-synthesis", CssProperty::FontSynthesis),
      ("font-synthesis-position", CssProperty::FontSynthesisPosition),
      ("font-synthesis-small-caps", CssProperty::FontSynthesisSmallCaps),
      ("font-synthesis-style", CssProperty::FontSynthesisStyle),
      ("font-synthesis-weight", CssProperty::FontSynthesisWeight),
      ("font-variant", CssProperty::FontVariant),
      ("font-variant-alternates", CssProperty::FontVariantAlternates),
      ("font-variant-caps", CssProperty::FontVariantCaps),
      ("font-variant-east-asian", CssProperty::FontVariantEastAsian),
      ("font-variant-emoji", CssProperty::FontVariantEmoji),
      ("font-variant-ligatures", CssProperty::FontVariantLigatures),
      ("font-variant-numeric", CssProperty::FontVariantNumeric),
      ("font-variant-position", CssProperty::FontVariantPosition),
      ("font-variation-settings", CssProperty::FontVariationSettings),
      ("font-weight", CssProperty::FontWeight),
      ("font-width", CssProperty::FontWidth),
      ("footnote-display", CssProperty::FootnoteDisplay),
      ("footnote-policy", CssProperty::FootnotePolicy),
      ("forced-color-adjust", CssProperty::ForcedColorAdjust),
      ("frame-sizing", CssProperty::FrameSizing),
      ("gap", CssProperty::Gap),
      ("glyph-orientation-vertical", CssProperty::GlyphOrientationVertical),
      ("grid", CssProperty::Grid),
      ("grid-area", CssProperty::GridArea),
      ("grid-auto-columns", CssProperty::GridAutoColumns),
      ("grid-auto-flow", CssProperty::GridAutoFlow),
      ("grid-auto-rows", CssProperty::GridAutoRows),
      ("grid-column", CssProperty::GridColumn),
      ("grid-column-end", CssProperty::GridColumnEnd),
      ("grid-column-gap", CssProperty::GridColumnGap),
      ("grid-column-start", CssProperty::GridColumnStart),
      ("grid-gap", CssProperty::GridGap),
      ("grid-row", CssProperty::GridRow),
      ("grid-row-end", CssProperty::GridRowEnd),
      ("grid-row-gap", CssProperty::GridRowGap),
      ("grid-row-start", CssProperty::GridRowStart),
      ("grid-template", CssProperty::GridTemplate),
      ("grid-template-areas", CssProperty::GridTemplateAreas),
      ("grid-template-columns", CssProperty::GridTemplateColumns),
      ("grid-template-rows", CssProperty::GridTemplateRows),
      ("hanging-punctuation", CssProperty::HangingPunctuation),
      ("height", CssProperty::Height),
      ("hyphenate-character", CssProperty::HyphenateCharacter),
      ("hyphenate-limit-chars", CssProperty::HyphenateLimitChars),
      ("hyphenate-limit-last", CssProperty::HyphenateLimitLast),
      ("hyphenate-limit-lines", CssProperty::HyphenateLimitLines),
      ("hyphenate-limit-zone", CssProperty::HyphenateLimitZone),
      ("hyphens", CssProperty::Hyphens),
      ("image-animation", CssProperty::ImageAnimation),
      ("image-orientation", CssProperty::ImageOrientation),
      ("image-rendering", CssProperty::ImageRendering),
      ("image-resolution", CssProperty::ImageResolution),
      ("initial-letter", CssProperty::InitialLetter),
      ("initial-letter-align", CssProperty::InitialLetterAlign),
      ("initial-letter-wrap", CssProperty::InitialLetterWrap),
      ("inline-size", CssProperty::InlineSize),
      ("inline-sizing", CssProperty::InlineSizing),
      ("input-security", CssProperty::InputSecurity),
      ("inset", CssProperty::Inset),
      ("inset-block", CssProperty::InsetBlock),
      ("inset-block-end", CssProperty::InsetBlockEnd),
      ("inset-block-start", CssProperty::InsetBlockStart),
      ("inset-inline", CssProperty::InsetInline),
      ("inset-inline-end", CssProperty::InsetInlineEnd),
      ("inset-inline-start", CssProperty::InsetInlineStart),
      ("interactivity", CssProperty::Interactivity),
      ("interest-delay", CssProperty::InterestDelay),
      ("interest-delay-end", CssProperty::InterestDelayEnd),
      ("interest-delay-start", CssProperty::InterestDelayStart),
      ("interpolate-size", CssProperty::InterpolateSize),
      ("isolation", CssProperty::Isolation),
      ("justify-content", CssProperty::JustifyContent),
      ("justify-items", CssProperty::JustifyItems),
      ("justify-self", CssProperty::JustifySelf),
      ("left", CssProperty::Left),
      ("letter-spacing", CssProperty::LetterSpacing),
      ("lighting-color", CssProperty::LightingColor),
      ("line-break", CssProperty::LineBreak),
      ("line-clamp", CssProperty::LineClamp),
      ("line-fit-edge", CssProperty::LineFitEdge),
      ("line-grid", CssProperty::LineGrid),
      ("line-height", CssProperty::LineHeight),
      ("line-height-step", CssProperty::LineHeightStep),
      ("line-padding", CssProperty::LinePadding),
      ("line-snap", CssProperty::LineSnap),
      ("link-parameters", CssProperty::LinkParameters),
      ("list-style", CssProperty::ListStyle),
      ("list-style-image", CssProperty::ListStyleImage),
      ("list-style-position", CssProperty::ListStylePosition),
      ("list-style-type", CssProperty::ListStyleType),
      ("margin", CssProperty::Margin),
      ("margin-block", CssProperty::MarginBlock),
      ("margin-block-end", CssProperty::MarginBlockEnd),
      ("margin-block-start", CssProperty::MarginBlockStart),
      ("margin-bottom", CssProperty::MarginBottom),
      ("margin-break", CssProperty::MarginBreak),
      ("margin-inline", CssProperty::MarginInline),
      ("margin-inline-end", CssProperty::MarginInlineEnd),
      ("margin-inline-start", CssProperty::MarginInlineStart),
      ("margin-left", CssProperty::MarginLeft),
      ("margin-right", CssProperty::MarginRight),
      ("margin-top", CssProperty::MarginTop),
      ("margin-trim", CssProperty::MarginTrim),
      ("marker", CssProperty::Marker),
      ("marker-end", CssProperty::MarkerEnd),
      ("marker-mid", CssProperty::MarkerMid),
      ("marker-side", CssProperty::MarkerSide),
      ("marker-start", CssProperty::MarkerStart),
      ("mask", CssProperty::Mask),
      ("mask-border", CssProperty::MaskBorder),
      ("mask-border-mode", CssProperty::MaskBorderMode),
      ("mask-border-outset", CssProperty::MaskBorderOutset),
      ("mask-border-repeat", CssProperty::MaskBorderRepeat),
      ("mask-border-slice", CssProperty::MaskBorderSlice),
      ("mask-border-source", CssProperty::MaskBorderSource),
      ("mask-border-width", CssProperty::MaskBorderWidth),
      ("mask-clip", CssProperty::MaskClip),
      ("mask-composite", CssProperty::MaskComposite),
      ("mask-image", CssProperty::MaskImage),
      ("mask-mode", CssProperty::MaskMode),
      ("mask-origin", CssProperty::MaskOrigin),
      ("mask-position", CssProperty::MaskPosition),
      ("mask-repeat", CssProperty::MaskRepeat),
      ("mask-size", CssProperty::MaskSize),
      ("mask-type", CssProperty::MaskType),
      ("math-depth", CssProperty::MathDepth),
      ("math-shift", CssProperty::MathShift),
      ("math-style", CssProperty::MathStyle),
      ("max-block-size", CssProperty::MaxBlockSize),
      ("max-height", CssProperty::MaxHeight),
      ("max-inline-size", CssProperty::MaxInlineSize),
      ("max-lines", CssProperty::MaxLines),
      ("max-width", CssProperty::MaxWidth),
      ("min-block-size", CssProperty::MinBlockSize),
      ("min-height", CssProperty::MinHeight),
      ("min-inline-size", CssProperty::MinInlineSize),
      ("min-intrinsic-sizing", CssProperty::MinIntrinsicSizing),
      ("min-width", CssProperty::MinWidth),
      ("mix-blend-mode", CssProperty::MixBlendMode),
      ("nav-down", CssProperty::NavDown),
      ("nav-left", CssProperty::NavLeft),
      ("nav-right", CssProperty::NavRight),
      ("nav-up", CssProperty::NavUp),
      ("object-fit", CssProperty::ObjectFit),
      ("object-position", CssProperty::ObjectPosition),
      ("object-view-box", CssProperty::ObjectViewBox),
      ("offset", CssProperty::Offset),
      ("offset-anchor", CssProperty::OffsetAnchor),
      ("offset-distance", CssProperty::OffsetDistance),
      ("offset-path", CssProperty::OffsetPath),
      ("offset-position", CssProperty::OffsetPosition),
      ("offset-rotate", CssProperty::OffsetRotate),
      ("opacity", CssProperty::Opacity),
      ("order", CssProperty::Order),
      ("orphans", CssProperty::Orphans),
      ("outline", CssProperty::Outline),
      ("outline-color", CssProperty::OutlineColor),
      ("outline-offset", CssProperty::OutlineOffset),
      ("outline-style", CssProperty::OutlineStyle),
      ("outline-width", CssProperty::OutlineWidth),
      ("overflow", CssProperty::Overflow),
      ("overflow-anchor", CssProperty::OverflowAnchor),
      ("overflow-block", CssProperty::OverflowBlock),
      ("overflow-clip-margin", CssProperty::OverflowClipMargin),
      ("overflow-clip-margin-block", CssProperty::OverflowClipMarginBlock),
      (
        "overflow-clip-margin-block-end",
        CssProperty::OverflowClipMarginBlockEnd,
      ),
      (
        "overflow-clip-margin-block-start",
        CssProperty::OverflowClipMarginBlockStart,
      ),
      ("overflow-clip-margin-bottom", CssProperty::OverflowClipMarginBottom),
      ("overflow-clip-margin-inline", CssProperty::OverflowClipMarginInline),
      (
        "overflow-clip-margin-inline-end",
        CssProperty::OverflowClipMarginInlineEnd,
      ),
      (
        "overflow-clip-margin-inline-start",
        CssProperty::OverflowClipMarginInlineStart,
      ),
      ("overflow-clip-margin-left", CssProperty::OverflowClipMarginLeft),
      ("overflow-clip-margin-right", CssProperty::OverflowClipMarginRight),
      ("overflow-clip-margin-top", CssProperty::OverflowClipMarginTop),
      ("overflow-inline", CssProperty::OverflowInline),
      ("overflow-wrap", CssProperty::OverflowWrap),
      ("overflow-x", CssProperty::OverflowX),
      ("overflow-y", CssProperty::OverflowY),
      ("overlay", CssProperty::Overlay),
      ("overscroll-behavior", CssProperty::OverscrollBehavior),
      ("overscroll-behavior-block", CssProperty::OverscrollBehaviorBlock),
      ("overscroll-behavior-inline", CssProperty::OverscrollBehaviorInline),
      ("overscroll-behavior-x", CssProperty::OverscrollBehaviorX),
      ("overscroll-behavior-y", CssProperty::OverscrollBehaviorY),
      ("padding", CssProperty::Padding),
      ("padding-block", CssProperty::PaddingBlock),
      ("padding-block-end", CssProperty::PaddingBlockEnd),
      ("padding-block-start", CssProperty::PaddingBlockStart),
      ("padding-bottom", CssProperty::PaddingBottom),
      ("padding-inline", CssProperty::PaddingInline),
      ("padding-inline-end", CssProperty::PaddingInlineEnd),
      ("padding-inline-start", CssProperty::PaddingInlineStart),
      ("padding-left", CssProperty::PaddingLeft),
      ("padding-right", CssProperty::PaddingRight),
      ("padding-top", CssProperty::PaddingTop),
      ("page", CssProperty::Page),
      ("page-break-after", CssProperty::PageBreakAfter),
      ("page-break-before", CssProperty::PageBreakBefore),
      ("page-break-inside", CssProperty::PageBreakInside),
      ("paint-order", CssProperty::PaintOrder),
      ("path-length", CssProperty::PathLength),
      ("pause", CssProperty::Pause),
      ("pause-after", CssProperty::PauseAfter),
      ("pause-before", CssProperty::PauseBefore),
      ("perspective", CssProperty::Perspective),
      ("perspective-origin", CssProperty::PerspectiveOrigin),
      ("place-content", CssProperty::PlaceContent),
      ("place-items", CssProperty::PlaceItems),
      ("place-self", CssProperty::PlaceSelf),
      ("pointer-events", CssProperty::PointerEvents),
      ("pointer-timeline", CssProperty::PointerTimeline),
      ("pointer-timeline-axis", CssProperty::PointerTimelineAxis),
      ("pointer-timeline-name", CssProperty::PointerTimelineName),
      ("position", CssProperty::Position),
      ("position-anchor", CssProperty::PositionAnchor),
      ("position-area", CssProperty::PositionArea),
      ("position-try", CssProperty::PositionTry),
      ("position-try-fallbacks", CssProperty::PositionTryFallbacks),
      ("position-try-order", CssProperty::PositionTryOrder),
      ("position-visibility", CssProperty::PositionVisibility),
      ("print-color-adjust", CssProperty::PrintColorAdjust),
      ("quotes", CssProperty::Quotes),
      ("r", CssProperty::R),
      ("reading-flow", CssProperty::ReadingFlow),
      ("reading-order", CssProperty::ReadingOrder),
      ("region-fragment", CssProperty::RegionFragment),
      ("resize", CssProperty::Resize),
      ("rest", CssProperty::Rest),
      ("rest-after", CssProperty::RestAfter),
      ("rest-before", CssProperty::RestBefore),
      ("right", CssProperty::Right),
      ("rotate", CssProperty::Rotate),
      ("row-gap", CssProperty::RowGap),
      ("row-rule", CssProperty::RowRule),
      ("row-rule-break", CssProperty::RowRuleBreak),
      ("row-rule-color", CssProperty::RowRuleColor),
      ("row-rule-inset", CssProperty::RowRuleInset),
      ("row-rule-inset-cap", CssProperty::RowRuleInsetCap),
      ("row-rule-inset-cap-end", CssProperty::RowRuleInsetCapEnd),
      ("row-rule-inset-cap-start", CssProperty::RowRuleInsetCapStart),
      ("row-rule-inset-end", CssProperty::RowRuleInsetEnd),
      ("row-rule-inset-junction", CssProperty::RowRuleInsetJunction),
      ("row-rule-inset-junction-end", CssProperty::RowRuleInsetJunctionEnd),
      ("row-rule-inset-junction-start", CssProperty::RowRuleInsetJunctionStart),
      ("row-rule-inset-start", CssProperty::RowRuleInsetStart),
      ("row-rule-style", CssProperty::RowRuleStyle),
      ("row-rule-visibility-items", CssProperty::RowRuleVisibilityItems),
      ("row-rule-width", CssProperty::RowRuleWidth),
      ("ruby-align", CssProperty::RubyAlign),
      ("ruby-merge", CssProperty::RubyMerge),
      ("ruby-overhang", CssProperty::RubyOverhang),
      ("ruby-position", CssProperty::RubyPosition),
      ("rule", CssProperty::Rule),
      ("rule-break", CssProperty::RuleBreak),
      ("rule-color", CssProperty::RuleColor),
      ("rule-inset", CssProperty::RuleInset),
      ("rule-inset-cap", CssProperty::RuleInsetCap),
      ("rule-inset-end", CssProperty::RuleInsetEnd),
      ("rule-inset-junction", CssProperty::RuleInsetJunction),
      ("rule-inset-start", CssProperty::RuleInsetStart),
      ("rule-overlap", CssProperty::RuleOverlap),
      ("rule-style", CssProperty::RuleStyle),
      ("rule-visibility-items", CssProperty::RuleVisibilityItems),
      ("rule-width", CssProperty::RuleWidth),
      ("rx", CssProperty::Rx),
      ("ry", CssProperty::Ry),
      ("scale", CssProperty::Scale),
      ("scroll-behavior", CssProperty::ScrollBehavior),
      ("scroll-initial-target", CssProperty::ScrollInitialTarget),
      ("scroll-margin", CssProperty::ScrollMargin),
      ("scroll-margin-block", CssProperty::ScrollMarginBlock),
      ("scroll-margin-block-end", CssProperty::ScrollMarginBlockEnd),
      ("scroll-margin-block-start", CssProperty::ScrollMarginBlockStart),
      ("scroll-margin-bottom", CssProperty::ScrollMarginBottom),
      ("scroll-margin-inline", CssProperty::ScrollMarginInline),
      ("scroll-margin-inline-end", CssProperty::ScrollMarginInlineEnd),
      ("scroll-margin-inline-start", CssProperty::ScrollMarginInlineStart),
      ("scroll-margin-left", CssProperty::ScrollMarginLeft),
      ("scroll-margin-right", CssProperty::ScrollMarginRight),
      ("scroll-margin-top", CssProperty::ScrollMarginTop),
      ("scroll-marker-group", CssProperty::ScrollMarkerGroup),
      ("scroll-padding", CssProperty::ScrollPadding),
      ("scroll-padding-block", CssProperty::ScrollPaddingBlock),
      ("scroll-padding-block-end", CssProperty::ScrollPaddingBlockEnd),
      ("scroll-padding-block-start", CssProperty::ScrollPaddingBlockStart),
      ("scroll-padding-bottom", CssProperty::ScrollPaddingBottom),
      ("scroll-padding-inline", CssProperty::ScrollPaddingInline),
      ("scroll-padding-inline-end", CssProperty::ScrollPaddingInlineEnd),
      ("scroll-padding-inline-start", CssProperty::ScrollPaddingInlineStart),
      ("scroll-padding-left", CssProperty::ScrollPaddingLeft),
      ("scroll-padding-right", CssProperty::ScrollPaddingRight),
      ("scroll-padding-top", CssProperty::ScrollPaddingTop),
      ("scroll-snap-align", CssProperty::ScrollSnapAlign),
      ("scroll-snap-stop", CssProperty::ScrollSnapStop),
      ("scroll-snap-type", CssProperty::ScrollSnapType),
      ("scroll-target-group", CssProperty::ScrollTargetGroup),
      ("scroll-timeline", CssProperty::ScrollTimeline),
      ("scroll-timeline-axis", CssProperty::ScrollTimelineAxis),
      ("scroll-timeline-name", CssProperty::ScrollTimelineName),
      ("scrollbar-color", CssProperty::ScrollbarColor),
      ("scrollbar-gutter", CssProperty::ScrollbarGutter),
      ("scrollbar-inset", CssProperty::ScrollbarInset),
      ("scrollbar-min-thumb-size", CssProperty::ScrollbarMinThumbSize),
      ("scrollbar-mode", CssProperty::ScrollbarMode),
      ("scrollbar-width", CssProperty::ScrollbarWidth),
      ("shape-image-threshold", CssProperty::ShapeImageThreshold),
      ("shape-inside", CssProperty::ShapeInside),
      ("shape-margin", CssProperty::ShapeMargin),
      ("shape-outside", CssProperty::ShapeOutside),
      ("shape-padding", CssProperty::ShapePadding),
      ("shape-rendering", CssProperty::ShapeRendering),
      ("slider-orientation", CssProperty::SliderOrientation),
      ("spatial-navigation-action", CssProperty::SpatialNavigationAction),
      ("spatial-navigation-contain", CssProperty::SpatialNavigationContain),
      ("spatial-navigation-function", CssProperty::SpatialNavigationFunction),
      ("speak", CssProperty::Speak),
      ("speak-as", CssProperty::SpeakAs),
      ("stop-color", CssProperty::StopColor),
      ("stop-opacity", CssProperty::StopOpacity),
      ("string-set", CssProperty::StringSet),
      ("stroke", CssProperty::Stroke),
      ("stroke-align", CssProperty::StrokeAlign),
      ("stroke-alignment", CssProperty::StrokeAlignment),
      ("stroke-break", CssProperty::StrokeBreak),
      ("stroke-color", CssProperty::StrokeColor),
      (
        "stroke-dash-corner",
        CssProperty::StrokeDashCornerPropdefStrokeDashCorner,
      ),
      ("stroke-dash-justify", CssProperty::StrokeDashJustify),
      ("stroke-dashadjust", CssProperty::StrokeDashadjust),
      ("stroke-dasharray", CssProperty::StrokeDasharray),
      (
        "stroke-dashcorner",
        CssProperty::StrokeDashcornerStrokedashcornerproperty,
      ),
      ("stroke-dashoffset", CssProperty::StrokeDashoffset),
      ("stroke-image", CssProperty::StrokeImage),
      ("stroke-linecap", CssProperty::StrokeLinecap),
      ("stroke-linejoin", CssProperty::StrokeLinejoin),
      ("stroke-miterlimit", CssProperty::StrokeMiterlimit),
      ("stroke-opacity", CssProperty::StrokeOpacity),
      ("stroke-origin", CssProperty::StrokeOrigin),
      ("stroke-position", CssProperty::StrokePosition),
      ("stroke-repeat", CssProperty::StrokeRepeat),
      ("stroke-size", CssProperty::StrokeSize),
      ("stroke-width", CssProperty::StrokeWidth),
      ("tab-size", CssProperty::TabSize),
      ("table-layout", CssProperty::TableLayout),
      ("text-align", CssProperty::TextAlign),
      ("text-align-all", CssProperty::TextAlignAll),
      ("text-align-last", CssProperty::TextAlignLast),
      ("text-anchor", CssProperty::TextAnchor),
      ("text-autospace", CssProperty::TextAutospace),
      ("text-box", CssProperty::TextBox),
      ("text-box-edge", CssProperty::TextBoxEdge),
      ("text-box-trim", CssProperty::TextBoxTrim),
      ("text-combine-upright", CssProperty::TextCombineUpright),
      ("text-decoration", CssProperty::TextDecoration),
      ("text-decoration-color", CssProperty::TextDecorationColor),
      ("text-decoration-inset", CssProperty::TextDecorationInset),
      ("text-decoration-line", CssProperty::TextDecorationLine),
      ("text-decoration-skip", CssProperty::TextDecorationSkip),
      ("text-decoration-skip-box", CssProperty::TextDecorationSkipBox),
      ("text-decoration-skip-ink", CssProperty::TextDecorationSkipInk),
      ("text-decoration-skip-self", CssProperty::TextDecorationSkipSelf),
      ("text-decoration-skip-spaces", CssProperty::TextDecorationSkipSpaces),
      ("text-decoration-style", CssProperty::TextDecorationStyle),
      ("text-decoration-thickness", CssProperty::TextDecorationThickness),
      ("text-emphasis", CssProperty::TextEmphasis),
      ("text-emphasis-color", CssProperty::TextEmphasisColor),
      ("text-emphasis-position", CssProperty::TextEmphasisPosition),
      ("text-emphasis-skip", CssProperty::TextEmphasisSkip),
      ("text-emphasis-style", CssProperty::TextEmphasisStyle),
      ("text-fit", CssProperty::TextFit),
      ("text-group-align", CssProperty::TextGroupAlign),
      ("text-indent", CssProperty::TextIndent),
      ("text-justify", CssProperty::TextJustify),
      ("text-orientation", CssProperty::TextOrientation),
      ("text-overflow", CssProperty::TextOverflow),
      ("text-rendering", CssProperty::TextRendering),
      ("text-shadow", CssProperty::TextShadow),
      ("text-size-adjust", CssProperty::TextSizeAdjust),
      ("text-spacing", CssProperty::TextSpacing),
      ("text-spacing-trim", CssProperty::TextSpacingTrim),
      ("text-transform", CssProperty::TextTransform),
      ("text-underline-offset", CssProperty::TextUnderlineOffset),
      ("text-underline-position", CssProperty::TextUnderlinePosition),
      ("text-wrap", CssProperty::TextWrap),
      ("text-wrap-mode", CssProperty::TextWrapMode),
      ("text-wrap-style", CssProperty::TextWrapStyle),
      ("timeline-scope", CssProperty::TimelineScope),
      ("timeline-trigger", CssProperty::TimelineTrigger),
      (
        "timeline-trigger-activation-range",
        CssProperty::TimelineTriggerActivationRange,
      ),
      (
        "timeline-trigger-activation-range-end",
        CssProperty::TimelineTriggerActivationRangeEnd,
      ),
      (
        "timeline-trigger-activation-range-start",
        CssProperty::TimelineTriggerActivationRangeStart,
      ),
      ("timeline-trigger-active-range", CssProperty::TimelineTriggerActiveRange),
      (
        "timeline-trigger-active-range-end",
        CssProperty::TimelineTriggerActiveRangeEnd,
      ),
      (
        "timeline-trigger-active-range-start",
        CssProperty::TimelineTriggerActiveRangeStart,
      ),
      ("timeline-trigger-name", CssProperty::TimelineTriggerName),
      ("timeline-trigger-source", CssProperty::TimelineTriggerSource),
      ("top", CssProperty::Top),
      ("touch-action", CssProperty::TouchAction),
      ("transform", CssProperty::Transform),
      ("transform-box", CssProperty::TransformBox),
      ("transform-origin", CssProperty::TransformOrigin),
      ("transform-style", CssProperty::TransformStyle),
      ("transition", CssProperty::Transition),
      ("transition-behavior", CssProperty::TransitionBehavior),
      ("transition-delay", CssProperty::TransitionDelay),
      ("transition-duration", CssProperty::TransitionDuration),
      ("transition-property", CssProperty::TransitionProperty),
      ("transition-timing-function", CssProperty::TransitionTimingFunction),
      ("translate", CssProperty::Translate),
      ("trigger-scope", CssProperty::TriggerScope),
      ("unicode-bidi", CssProperty::UnicodeBidi),
      ("user-select", CssProperty::UserSelect),
      ("vector-effect", CssProperty::VectorEffect),
      ("vertical-align", CssProperty::VerticalAlign),
      ("view-timeline", CssProperty::ViewTimeline),
      ("view-timeline-axis", CssProperty::ViewTimelineAxis),
      ("view-timeline-inset", CssProperty::ViewTimelineInset),
      ("view-timeline-name", CssProperty::ViewTimelineName),
      ("view-transition-class", CssProperty::ViewTransitionClass),
      ("view-transition-group", CssProperty::ViewTransitionGroup),
      ("view-transition-name", CssProperty::ViewTransitionName),
      ("view-transition-scope", CssProperty::ViewTransitionScope),
      ("visibility", CssProperty::Visibility),
      ("voice-balance", CssProperty::VoiceBalance),
      ("voice-duration", CssProperty::VoiceDuration),
      ("voice-family", CssProperty::VoiceFamily),
      ("voice-pitch", CssProperty::VoicePitch),
      ("voice-range", CssProperty::VoiceRange),
      ("voice-rate", CssProperty::VoiceRate),
      ("voice-stress", CssProperty::VoiceStress),
      ("voice-volume", CssProperty::VoiceVolume),
      ("white-space", CssProperty::WhiteSpace),
      ("white-space-collapse", CssProperty::WhiteSpaceCollapse),
      ("white-space-trim", CssProperty::WhiteSpaceTrim),
      ("widows", CssProperty::Widows),
      ("width", CssProperty::Width),
      ("will-change", CssProperty::WillChange),
      ("word-break", CssProperty::WordBreak),
      ("word-space-transform", CssProperty::WordSpaceTransform),
      ("word-spacing", CssProperty::WordSpacing),
      ("word-wrap", CssProperty::WordWrap),
      ("wrap-after", CssProperty::WrapAfter),
      ("wrap-before", CssProperty::WrapBefore),
      ("wrap-flow", CssProperty::WrapFlow),
      ("wrap-inside", CssProperty::WrapInside),
      ("wrap-through", CssProperty::WrapThrough),
      ("writing-mode", CssProperty::WritingMode),
      ("x", CssProperty::X),
      ("y", CssProperty::Y),
      ("z-index", CssProperty::ZIndex),
      ("zoom", CssProperty::Zoom),
    ];
    match ENTRIES.binary_search_by_key(&name, |(n, _)| n) {
      Ok(i) => ENTRIES[i].1.clone(),
      Err(_) => CssProperty::Unknown(name.to_string()),
    }
  }
  pub fn syntax(self) -> &'static str {
    match self {
      CssProperty::WebkitAlignContent => {
        "normal | <baseline-position> | <content-distribution> | <overflow-position>? <content-position>"
      }
      CssProperty::WebkitAlignItems => "normal | stretch | <baseline-position> | <overflow-position>? <self-position>",
      CssProperty::WebkitAlignSelf => {
        "auto | <overflow-position>? [ normal | <self-position> ]| stretch | <baseline-position>"
      }
      CssProperty::WebkitAnimation => "<single-animation>#",
      CssProperty::WebkitAnimationDelay => "<time>#",
      CssProperty::WebkitAnimationDirection => "<single-animation-direction>#",
      CssProperty::WebkitAnimationDuration => "<time [0s,∞]>#",
      CssProperty::WebkitAnimationFillMode => "<single-animation-fill-mode>#",
      CssProperty::WebkitAnimationIterationCount => "<single-animation-iteration-count>#",
      CssProperty::WebkitAnimationName => "[ none | <keyframes-name> ]#",
      CssProperty::WebkitAnimationPlayState => "<single-animation-play-state>#",
      CssProperty::WebkitAnimationTimingFunction => "<easing-function>#",
      CssProperty::WebkitAppearance => "none | auto | base | base-select | <compat-auto> | <compat-special> | base",
      CssProperty::WebkitBackfaceVisibility => "visible | hidden",
      CssProperty::WebkitBackgroundClip => "<visual-box>#",
      CssProperty::WebkitBackgroundOrigin => "<visual-box>#",
      CssProperty::WebkitBackgroundSize => "<bg-size>#",
      CssProperty::WebkitBorderBottomLeftRadius => "<border-radius>",
      CssProperty::WebkitBorderBottomRightRadius => "<border-radius>",
      CssProperty::WebkitBorderRadius => "<length-percentage [0,∞]>{1,4} [ / <length-percentage [0,∞]>{1,4} ]?",
      CssProperty::WebkitBorderTopLeftRadius => "<border-radius>",
      CssProperty::WebkitBorderTopRightRadius => "<border-radius>",
      CssProperty::WebkitBoxAlign => "",
      CssProperty::WebkitBoxFlex => "",
      CssProperty::WebkitBoxOrdinalGroup => "",
      CssProperty::WebkitBoxOrient => "",
      CssProperty::WebkitBoxPack => "",
      CssProperty::WebkitBoxShadow => "<spread-shadow>#",
      CssProperty::WebkitBoxSizing => "content-box | border-box",
      CssProperty::WebkitFilter => "none | <filter-value-list>",
      CssProperty::WebkitFlex => "none | [ <'flex-grow'> <'flex-shrink'>? || <'flex-basis'> ]",
      CssProperty::WebkitFlexBasis => "content | <'width'>",
      CssProperty::WebkitFlexDirection => "row | row-reverse | column | column-reverse",
      CssProperty::WebkitFlexFlow => "<'flex-direction'> || <'flex-wrap'>",
      CssProperty::WebkitFlexGrow => "<number [0,∞]>",
      CssProperty::WebkitFlexShrink => "<number [0,∞]>",
      CssProperty::WebkitFlexWrap => "nowrap | wrap | wrap-reverse",
      CssProperty::WebkitJustifyContent => {
        "normal | <content-distribution> | <overflow-position>? [ <content-position> | left | right ]"
      }
      CssProperty::WebkitLineClamp => "none | <integer [1,∞]>",
      CssProperty::WebkitMask => "<mask-layer>#",
      CssProperty::WebkitMaskBoxImage => {
        "<'mask-border-source'> || <'mask-border-slice'> [ / <'mask-border-width'>? [ / <'mask-border-outset'> ]? ]? || <'mask-border-repeat'> || <'mask-border-mode'>"
      }
      CssProperty::WebkitMaskBoxImageOutset => "<'border-image-outset'>",
      CssProperty::WebkitMaskBoxImageRepeat => "<'border-image-repeat'>",
      CssProperty::WebkitMaskBoxImageSlice => "<'border-image-slice'>",
      CssProperty::WebkitMaskBoxImageSource => "<'border-image-source'>",
      CssProperty::WebkitMaskBoxImageWidth => "<'border-image-width'>",
      CssProperty::WebkitMaskClip => "[ <coord-box> | no-clip ]#",
      CssProperty::WebkitMaskComposite => "<compositing-operator>#",
      CssProperty::WebkitMaskImage => "<mask-reference>#",
      CssProperty::WebkitMaskOrigin => "<coord-box>#",
      CssProperty::WebkitMaskPosition => "<position>#",
      CssProperty::WebkitMaskRepeat => "<repeat-style>#",
      CssProperty::WebkitMaskSize => "<bg-size>#",
      CssProperty::WebkitOrder => "<integer>",
      CssProperty::WebkitPerspective => "none | <length [0,∞]>",
      CssProperty::WebkitPerspectiveOrigin => "<position>",
      CssProperty::WebkitTextFillColor => "<color>",
      CssProperty::WebkitTextSizeAdjust => "auto | none | <percentage [0,∞]>",
      CssProperty::WebkitTextStroke => "<line-width> || <color>",
      CssProperty::WebkitTextStrokeColor => "<color>",
      CssProperty::WebkitTextStrokeWidth => "<line-width>",
      CssProperty::WebkitTransform => "none | <transform-list>",
      CssProperty::WebkitTransformOrigin => {
        "[ left | center | right | top | bottom | <length-percentage> ] | [ left | center | right | <length-percentage> ] [ top | center | bottom | <length-percentage> ] <length>? | [ [ center | left | right ] && [ center | top | bottom ] ] <length>?"
      }
      CssProperty::WebkitTransformStyle => "flat | preserve-3d",
      CssProperty::WebkitTransition => "<single-transition>#",
      CssProperty::WebkitTransitionDelay => "<time>#",
      CssProperty::WebkitTransitionDuration => "<time [0s,∞]>#",
      CssProperty::WebkitTransitionProperty => "none | <single-transition-property>#",
      CssProperty::WebkitTransitionTimingFunction => "<easing-function>#",
      CssProperty::WebkitUserSelect => "auto | text | none | contain | all",
      CssProperty::AccentColor => "auto | <color>",
      CssProperty::AlignContent => {
        "normal | <baseline-position> | <content-distribution> | <overflow-position>? <content-position>"
      }
      CssProperty::AlignItems => "normal | stretch | <baseline-position> | <overflow-position>? <self-position>",
      CssProperty::AlignSelf => {
        "auto | <overflow-position>? [ normal | <self-position> ]| stretch | <baseline-position> | anchor-center"
      }
      CssProperty::AlignmentBaseline => "baseline | <baseline-metric>",
      CssProperty::All => "initial | inherit | unset | revert | revert-layer | revert-rule",
      CssProperty::AnchorName => "none | <anchor-name>#",
      CssProperty::AnchorScope => "none | all | <anchor-name>#",
      CssProperty::Animation => "<single-animation>#",
      CssProperty::AnimationComposition => "<single-animation-composition>#",
      CssProperty::AnimationDelay => "<time>#",
      CssProperty::AnimationDirection => "<single-animation-direction>#",
      CssProperty::AnimationDuration => "[ auto | <time [0s,∞]> ]#",
      CssProperty::AnimationFillMode => "<single-animation-fill-mode>#",
      CssProperty::AnimationIterationCount => "<single-animation-iteration-count>#",
      CssProperty::AnimationName => "[ none | <keyframes-name> ]#",
      CssProperty::AnimationPlayState => "<single-animation-play-state>#",
      CssProperty::AnimationRange => "[ <'animation-range-start'> <'animation-range-end'>? ]#",
      CssProperty::AnimationRangeCenter => {
        "[ normal | [ <length-percentage> | <timeline-range-center-subject> <length-percentage>? ] ]#"
      }
      CssProperty::AnimationRangeEnd => {
        "[ normal | <length-percentage> | <timeline-range-name> <length-percentage>? ]#"
      }
      CssProperty::AnimationRangeStart => {
        "[ normal | <length-percentage> | <timeline-range-name> <length-percentage>? ]#"
      }
      CssProperty::AnimationTimeline => "<single-animation-timeline>#",
      CssProperty::AnimationTimingFunction => "<easing-function>#",
      CssProperty::AnimationTrigger => "[ none | [ <dashed-ident> <animation-action>+ ]+ ]#",
      CssProperty::Appearance => "none | auto | base | base-select | <compat-auto> | <compat-special> | base",
      CssProperty::AspectRatio => "auto || <ratio>",
      CssProperty::BackdropFilter => "none | <filter-value-list>",
      CssProperty::BackfaceVisibility => "visible | hidden",
      CssProperty::Background => "<bg-layer>#? , <final-bg-layer>",
      CssProperty::BackgroundAttachment => "<attachment>#",
      CssProperty::BackgroundBlendMode => "<'mix-blend-mode'>#",
      CssProperty::BackgroundClip => "<bg-clip>#",
      CssProperty::BackgroundColor => "<color>",
      CssProperty::BackgroundImage => "<bg-image>#",
      CssProperty::BackgroundOrigin => "<visual-box>#",
      CssProperty::BackgroundPosition => "<bg-position>#",
      CssProperty::BackgroundPositionBlock => "[ center | [ [ start | end ]? <length-percentage>? ]! ]#",
      CssProperty::BackgroundPositionInline => "[ center | [ [ start | end ]? <length-percentage>? ]! ]#",
      CssProperty::BackgroundPositionX => "[ center | [ [ left | right | x-start | x-end ]? <length-percentage>? ]! ]#",
      CssProperty::BackgroundPositionY => "[ center | [ [ top | bottom | y-start | y-end ]? <length-percentage>? ]! ]#",
      CssProperty::BackgroundRepeat => "<repeat-style>#",
      CssProperty::BackgroundRepeatBlock => "<repetition>#",
      CssProperty::BackgroundRepeatInline => "<repetition>#",
      CssProperty::BackgroundRepeatX => "<repetition>#",
      CssProperty::BackgroundRepeatY => "<repetition>#",
      CssProperty::BackgroundSize => "<bg-size>#",
      CssProperty::BackgroundTbd => "<bg-layer>#",
      CssProperty::BaselineShift => "<length-percentage> | sub | super | top | center | bottom",
      CssProperty::BaselineSource => "auto | first | last",
      CssProperty::BlockEllipsis => "no-ellipsis | auto | <string>",
      CssProperty::BlockSize => "<'width'>",
      CssProperty::BlockStep => {
        "<'block-step-size'> || <'block-step-insert'> || <'block-step-align'> || <'block-step-round'>"
      }
      CssProperty::BlockStepAlign => "auto | center | start | end",
      CssProperty::BlockStepInsert => "margin-box | padding-box | content-box",
      CssProperty::BlockStepRound => "up | down | nearest",
      CssProperty::BlockStepSize => "none | <length [0,∞]>",
      CssProperty::BookmarkLabel => "<content-list>",
      CssProperty::BookmarkLevel => "none | <integer [1,∞]>",
      CssProperty::BookmarkState => "open | closed",
      CssProperty::Border => "<line-width> || <line-style> || <color>",
      CssProperty::BorderBlock => "<'border-block-start'>",
      CssProperty::BorderBlockClip => "<'border-top-clip'>",
      CssProperty::BorderBlockColor => "<'border-top-color'>{1,2}",
      CssProperty::BorderBlockEnd => "<line-width> || <line-style> || <color>",
      CssProperty::BorderBlockEndClip => "none | [ <length-percentage [0,∞]> | <flex> ]+",
      CssProperty::BorderBlockEndColor => "<color> | <image-1D>",
      CssProperty::BorderBlockEndRadius => "<length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?",
      CssProperty::BorderBlockEndStyle => "<line-style>",
      CssProperty::BorderBlockEndWidth => "<line-width>",
      CssProperty::BorderBlockStart => "<line-width> || <line-style> || <color>",
      CssProperty::BorderBlockStartClip => "none | [ <length-percentage [0,∞]> | <flex> ]+",
      CssProperty::BorderBlockStartColor => "<color> | <image-1D>",
      CssProperty::BorderBlockStartRadius => "<length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?",
      CssProperty::BorderBlockStartStyle => "<line-style>",
      CssProperty::BorderBlockStartWidth => "<line-width>",
      CssProperty::BorderBlockStyle => "<'border-top-style'>{1,2}",
      CssProperty::BorderBlockWidth => "<'border-top-width'>{1,2}",
      CssProperty::BorderBottom => "<line-width> || <line-style> || <color>",
      CssProperty::BorderBottomClip => "none | [ <length-percentage [0,∞]> | <flex> ]+",
      CssProperty::BorderBottomColor => "<color> | <image-1D>",
      CssProperty::BorderBottomLeftRadius => "<border-radius>",
      CssProperty::BorderBottomRadius => "<length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?",
      CssProperty::BorderBottomRightRadius => "<border-radius>",
      CssProperty::BorderBottomStyle => "<line-style>",
      CssProperty::BorderBottomWidth => "<line-width>",
      CssProperty::BorderBoundary => "none | parent | display",
      CssProperty::BorderClip => "<'border-top-clip'>",
      CssProperty::BorderCollapse => "separate | collapse",
      CssProperty::BorderColor => "[ <color> | <image-1D> ]{1,4}",
      CssProperty::BorderEndEndRadius => "<border-radius>",
      CssProperty::BorderEndStartRadius => "<border-radius>",
      CssProperty::BorderImage => {
        "<'border-image-source'> || <'border-image-slice'> [ / <'border-image-width'> | / <'border-image-width'>? / <'border-image-outset'> ]? || <'border-image-repeat'>"
      }
      CssProperty::BorderImageOutset => "[ <length [0,∞]> | <number [0,∞]> ]{1,4}",
      CssProperty::BorderImageRepeat => "[ stretch | repeat | round | space ]{1,2}",
      CssProperty::BorderImageSlice => "[<number [0,∞]> | <percentage [0,∞]>]{1,4} && fill?",
      CssProperty::BorderImageSource => "none | <image>",
      CssProperty::BorderImageWidth => "[ <length-percentage [0,∞]> | <number [0,∞]> | auto ]{1,4}",
      CssProperty::BorderInline => "<'border-block-start'>",
      CssProperty::BorderInlineClip => "<'border-top-clip'>",
      CssProperty::BorderInlineColor => "<'border-top-color'>{1,2}",
      CssProperty::BorderInlineEnd => "<line-width> || <line-style> || <color>",
      CssProperty::BorderInlineEndClip => "none | [ <length-percentage [0,∞]> | <flex> ]+",
      CssProperty::BorderInlineEndColor => "<color> | <image-1D>",
      CssProperty::BorderInlineEndRadius => "<length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?",
      CssProperty::BorderInlineEndStyle => "<line-style>",
      CssProperty::BorderInlineEndWidth => "<line-width>",
      CssProperty::BorderInlineStart => "<line-width> || <line-style> || <color>",
      CssProperty::BorderInlineStartClip => "none | [ <length-percentage [0,∞]> | <flex> ]+",
      CssProperty::BorderInlineStartColor => "<color> | <image-1D>",
      CssProperty::BorderInlineStartRadius => "<length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?",
      CssProperty::BorderInlineStartStyle => "<line-style>",
      CssProperty::BorderInlineStartWidth => "<line-width>",
      CssProperty::BorderInlineStyle => "<'border-top-style'>{1,2}",
      CssProperty::BorderInlineWidth => "<'border-top-width'>{1,2}",
      CssProperty::BorderLeft => "<line-width> || <line-style> || <color>",
      CssProperty::BorderLeftClip => "none | [ <length-percentage [0,∞]> | <flex> ]+",
      CssProperty::BorderLeftColor => "<color> | <image-1D>",
      CssProperty::BorderLeftRadius => "<length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?",
      CssProperty::BorderLeftStyle => "<line-style>",
      CssProperty::BorderLeftWidth => "<line-width>",
      CssProperty::BorderLimit => {
        "all | [ sides | corners ] <length-percentage [0,∞]>? | [ top | right | bottom | left ] <length-percentage [0,∞]>"
      }
      CssProperty::BorderRadius => "<length-percentage [0,∞]>{1,4} [ / <length-percentage [0,∞]>{1,4} ]?",
      CssProperty::BorderRight => "<line-width> || <line-style> || <color>",
      CssProperty::BorderRightClip => "none | [ <length-percentage [0,∞]> | <flex> ]+",
      CssProperty::BorderRightColor => "<color> | <image-1D>",
      CssProperty::BorderRightRadius => "<length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?",
      CssProperty::BorderRightStyle => "<line-style>",
      CssProperty::BorderRightWidth => "<line-width>",
      CssProperty::BorderShape => "none | [ <basic-shape> <geometry-box>?]{1,2}",
      CssProperty::BorderSpacing => "<length [0,∞]>{1,2}",
      CssProperty::BorderStartEndRadius => "<border-radius>",
      CssProperty::BorderStartStartRadius => "<border-radius>",
      CssProperty::BorderStyle => "<'border-top-style'>{1,4}",
      CssProperty::BorderTop => "<line-width> || <line-style> || <color>",
      CssProperty::BorderTopClip => "none | [ <length-percentage [0,∞]> | <flex> ]+",
      CssProperty::BorderTopColor => "<color> | <image-1D>",
      CssProperty::BorderTopLeftRadius => "<border-radius>",
      CssProperty::BorderTopRadius => "<length-percentage [0,∞]>{1,2} [ / <length-percentage [0,∞]>{1,2} ]?",
      CssProperty::BorderTopRightRadius => "<border-radius>",
      CssProperty::BorderTopStyle => "<line-style>",
      CssProperty::BorderTopWidth => "<line-width>",
      CssProperty::BorderWidth => "<'border-top-width'>{1,4}",
      CssProperty::Bottom => "auto | <length-percentage> | <anchor()> | <anchor-size()>",
      CssProperty::BoxDecorationBreak => "slice | clone",
      CssProperty::BoxShadow => "<spread-shadow>#",
      CssProperty::BoxShadowBlur => "<length [0,∞]>#",
      CssProperty::BoxShadowColor => "<color>#",
      CssProperty::BoxShadowOffset => "[ none | <length>{1,2} ]#",
      CssProperty::BoxShadowPosition => "[ outset | inset ]#",
      CssProperty::BoxShadowSpread => "<length>#",
      CssProperty::BoxSizing => "content-box | border-box",
      CssProperty::BoxSnap => "none | block-start | block-end | center | baseline | last-baseline",
      CssProperty::BreakAfter => {
        "auto | avoid | always | all | avoid-page | page | left | right | recto | verso | avoid-column | column | avoid-region | region"
      }
      CssProperty::BreakBefore => {
        "auto | avoid | always | all | avoid-page | page | left | right | recto | verso | avoid-column | column | avoid-region | region"
      }
      CssProperty::BreakInside => "auto | avoid | avoid-page | avoid-column | avoid-region",
      CssProperty::CaptionSide => "top | bottom",
      CssProperty::Caret => "<'caret-color'> || <'caret-animation'> || <'caret-shape'>",
      CssProperty::CaretAnimation => "auto | manual",
      CssProperty::CaretColor => "auto | <color> [auto | <color>]?",
      CssProperty::CaretShape => "auto | bar | block | underscore",
      CssProperty::Clear => {
        "inline-start | inline-end | block-start | block-end | left | right | top | bottom | both-inline | both-block | both | none"
      }
      CssProperty::Clip => "<rect()> | auto",
      CssProperty::ClipPath => "<clip-source> | [ <basic-shape> || <geometry-box> ] | none",
      CssProperty::ClipRule => "nonzero | evenodd",
      CssProperty::Color => "<color>",
      CssProperty::ColorAdjust => "<'print-color-adjust'>",
      CssProperty::ColorInterpolation => "auto | sRGB | linearRGB",
      CssProperty::ColorInterpolationFilters => "auto | sRGB | linearRGB",
      CssProperty::ColorScheme => "normal | [ light | dark | <custom-ident> ]+ && only?",
      CssProperty::ColumnCount => "auto | <integer [1,∞]>",
      CssProperty::ColumnFill => "auto | balance | balance-all",
      CssProperty::ColumnGap => "normal | <length-percentage [0,∞]> | <line-width>",
      CssProperty::ColumnHeight => "auto | <length [0,∞]>",
      CssProperty::ColumnRule => "<gap-rule-list> | <gap-auto-rule-list>",
      CssProperty::ColumnRuleBreak => "none | normal | intersection",
      CssProperty::ColumnRuleColor => "<line-color-list> | <auto-line-color-list>",
      CssProperty::ColumnRuleInset => "<'column-rule-inset-cap'> [ / <'column-rule-inset-junction'> ]?",
      CssProperty::ColumnRuleInsetCap => "<length-percentage> [ <length-percentage> ]?",
      CssProperty::ColumnRuleInsetCapEnd => "<length-percentage>",
      CssProperty::ColumnRuleInsetCapStart => "<length-percentage>",
      CssProperty::ColumnRuleInsetEnd => "<length-percentage>",
      CssProperty::ColumnRuleInsetJunction => "<length-percentage> [ <length-percentage> ]?",
      CssProperty::ColumnRuleInsetJunctionEnd => "<length-percentage>",
      CssProperty::ColumnRuleInsetJunctionStart => "<length-percentage>",
      CssProperty::ColumnRuleInsetStart => "<length-percentage>",
      CssProperty::ColumnRuleStyle => "<line-style-list> | <auto-line-style-list>",
      CssProperty::ColumnRuleVisibilityItems => "all | around | between | normal",
      CssProperty::ColumnRuleWidth => "<line-width-list> | <auto-line-width-list>",
      CssProperty::ColumnSpan => "none | <integer [1,∞]> | all | auto",
      CssProperty::ColumnWidth => {
        "auto | <length [0,∞]> | min-content | max-content | fit-content(<length-percentage>)"
      }
      CssProperty::ColumnWrap => "auto | nowrap | wrap",
      CssProperty::Columns => "[ <'column-width'> || <'column-count'> ] [ / <'column-height'> ]?",
      CssProperty::Contain => "none | strict | content | [ [size | inline-size] || layout || style || paint ]",
      CssProperty::ContainIntrinsicBlockSize => "auto? [ none | <length [0,∞]> ]",
      CssProperty::ContainIntrinsicHeight => "auto? [ none | <length [0,∞]> ]",
      CssProperty::ContainIntrinsicInlineSize => "auto? [ none | <length [0,∞]> ]",
      CssProperty::ContainIntrinsicSize => "[ auto? [ none | <length [0,∞]> ] ]{1,2}",
      CssProperty::ContainIntrinsicWidth => "auto? [ none | <length [0,∞]> ]",
      CssProperty::Container => "<'container-name'> [ / <'container-type'> ]?",
      CssProperty::ContainerName => "none | <custom-ident>+",
      CssProperty::ContainerType => "normal | [ [ size | inline-size ] || scroll-state ]",
      CssProperty::Content => {
        "normal | none | [ <content-replacement> | <content-list> ] [/ [ <string> | <counter> | <attr()> ]+ ]? | <element()>"
      }
      CssProperty::ContentVisibility => "visible | auto | hidden",
      CssProperty::Continue => "auto | discard | collapse | -webkit-legacy | overflow | paginate | fragments",
      CssProperty::CopyInto => "none | [ [ <custom-ident> <content-level>] [, <custom-ident> <content-level>]* ]?",
      CssProperty::Corner => "<'border-radius'> || <'corner-shape'>",
      CssProperty::CornerBlockEnd => "<'border-top-radius'> || <'corner-top-shape'>",
      CssProperty::CornerBlockEndShape => "<'corner-top-left-shape'>{1,2}",
      CssProperty::CornerBlockStart => "<'border-top-radius'> || <'corner-top-shape'>",
      CssProperty::CornerBlockStartShape => "<'corner-top-left-shape'>{1,2}",
      CssProperty::CornerBottom => "<'border-top-radius'> || <'corner-top-shape'>",
      CssProperty::CornerBottomLeft => "<'border-top-left-radius'> || <'corner-top-left-shape'>",
      CssProperty::CornerBottomLeftShape => "<corner-shape-value>",
      CssProperty::CornerBottomRight => "<'border-top-left-radius'> || <'corner-top-left-shape'>",
      CssProperty::CornerBottomRightShape => "<corner-shape-value>",
      CssProperty::CornerBottomShape => "<'corner-top-left-shape'>{1,2}",
      CssProperty::CornerEndEnd => "<'border-top-left-radius'> || <'corner-top-left-shape'>",
      CssProperty::CornerEndEndShape => "<corner-shape-value>",
      CssProperty::CornerEndStart => "<'border-top-left-radius'> || <'corner-top-left-shape'>",
      CssProperty::CornerEndStartShape => "<corner-shape-value>",
      CssProperty::CornerInlineEnd => "<'border-top-radius'> || <'corner-top-shape'>",
      CssProperty::CornerInlineEndShape => "<'corner-top-left-shape'>{1,2}",
      CssProperty::CornerInlineStart => "<'border-top-radius'> || <'corner-top-shape'>",
      CssProperty::CornerInlineStartShape => "<'corner-top-left-shape'>{1,2}",
      CssProperty::CornerLeft => "<'border-top-radius'> || <'corner-top-shape'>",
      CssProperty::CornerLeftShape => "<'corner-top-left-shape'>{1,2}",
      CssProperty::CornerRight => "<'border-top-radius'> || <'corner-top-shape'>",
      CssProperty::CornerRightShape => "<'corner-top-left-shape'>{1,2}",
      CssProperty::CornerShape => "<'corner-top-left-shape'>{1,4}",
      CssProperty::CornerStartEnd => "<'border-top-left-radius'> || <'corner-top-left-shape'>",
      CssProperty::CornerStartEndShape => "<corner-shape-value>",
      CssProperty::CornerStartStart => "<'border-top-left-radius'> || <'corner-top-left-shape'>",
      CssProperty::CornerStartStartShape => "<corner-shape-value>",
      CssProperty::CornerTop => "<'border-top-radius'> || <'corner-top-shape'>",
      CssProperty::CornerTopLeft => "<'border-top-left-radius'> || <'corner-top-left-shape'>",
      CssProperty::CornerTopLeftShape => "<corner-shape-value>",
      CssProperty::CornerTopRight => "<'border-top-left-radius'> || <'corner-top-left-shape'>",
      CssProperty::CornerTopRightShape => "<corner-shape-value>",
      CssProperty::CornerTopShape => "<'corner-top-left-shape'>{1,2}",
      CssProperty::CounterIncrement => "[ <counter-name> <integer>? ]+ | none",
      CssProperty::CounterReset => "[ <counter-name> <integer>? | <reversed-counter-name> <integer>? ]+ | none",
      CssProperty::CounterSet => "[ <counter-name> <integer>? ]+ | none",
      CssProperty::Cue => "<'cue-before'> <'cue-after'>?",
      CssProperty::CueAfter => "<url> <decibel>? | none",
      CssProperty::CueBefore => "<url> <decibel>? | none",
      CssProperty::Cursor => "[<cursor-image>,]* <cursor-predefined>",
      CssProperty::Cx => "<length-percentage>",
      CssProperty::Cy => "<length-percentage>",
      CssProperty::D => "none | <string>",
      CssProperty::Direction => "ltr | rtl",
      CssProperty::Display => {
        "[ <display-outside> || <display-inside> ] | <display-listitem> | <display-internal> | <display-box> | <display-legacy> | grid-lanes | inline-grid-lanes | <display-outside> || [ <display-inside> | math ]"
      }
      CssProperty::DominantBaseline => "auto | <baseline-metric>",
      CssProperty::DynamicRangeLimit => "standard | no-limit | constrained | <dynamic-range-limit-mix()>",
      CssProperty::EmptyCells => "show | hide",
      CssProperty::EventTrigger => "none | [ <'event-trigger-name'> <'event-trigger-source'> ]#",
      CssProperty::EventTriggerName => "none | <dashed-ident>#",
      CssProperty::EventTriggerSource => "[ none | <event-trigger-event>+ [ / <event-trigger-event>+ ]? ]#",
      CssProperty::FieldSizing => "fixed | content",
      CssProperty::Fill => "<paint>",
      CssProperty::FillBreak => "bounding-box | slice | clone",
      CssProperty::FillColor => "<color>",
      CssProperty::FillImage => "<paint>#",
      CssProperty::FillOpacity => "<'opacity'>",
      CssProperty::FillOrigin => "match-parent | fill-box | stroke-box | content-box | padding-box | border-box",
      CssProperty::FillPosition => "<position>#",
      CssProperty::FillRepeat => "<repeat-style>#",
      CssProperty::FillRule => "nonzero | evenodd",
      CssProperty::FillSize => "<bg-size>#",
      CssProperty::Filter => "none | <filter-value-list>",
      CssProperty::Flex => "none | [ <'flex-grow'> <'flex-shrink'>? || <'flex-basis'> ]",
      CssProperty::FlexBasis => "content | <'width'>",
      CssProperty::FlexDirection => "row | row-reverse | column | column-reverse",
      CssProperty::FlexFlow => "<'flex-direction'> || <'flex-wrap'>",
      CssProperty::FlexGrow => "<number [0,∞]>",
      CssProperty::FlexShrink => "<number [0,∞]>",
      CssProperty::FlexWrap => "nowrap | wrap | wrap-reverse",
      CssProperty::Float => {
        "block-start | block-end | inline-start | inline-end | snap-block | <snap-block()> | snap-inline | <snap-inline()> | left | right | top | bottom | none | footnote"
      }
      CssProperty::FloatDefer => "<integer> | last | none",
      CssProperty::FloatOffset => "<length-percentage>",
      CssProperty::FloatReference => "inline | column | region | page",
      CssProperty::FloodColor => "<color>",
      CssProperty::FloodOpacity => "<'opacity'>",
      CssProperty::FlowFrom => "<custom-ident> | none",
      CssProperty::FlowInto => "none | <custom-ident> [element | content]?",
      CssProperty::FlowTolerance => "normal | <length-percentage> | infinite",
      CssProperty::Font => {
        "[ [ <'font-style'> || <font-variant-css2> || <'font-weight'> || <font-width-css3> ]? <'font-size'> [ / <'line-height'> ]? <'font-family'># ] | <system-font-family-name>"
      }
      CssProperty::FontFamily => "[ <font-family-name> | <generic-font-family> ]#",
      CssProperty::FontFeatureSettings => "normal | <feature-tag-value>#",
      CssProperty::FontKerning => "auto | normal | none",
      CssProperty::FontLanguageOverride => "normal | <string>",
      CssProperty::FontOpticalSizing => "auto | none",
      CssProperty::FontPalette => "normal | light | dark | <palette-identifier> | <palette-mix()>",
      CssProperty::FontSize => "<absolute-size> | <relative-size> | <length-percentage [0,∞]> | math",
      CssProperty::FontSizeAdjust => {
        "none | [ ex-height | cap-height | ch-width | ic-width | ic-height ]? [ from-font | <number [0,∞]> ]"
      }
      CssProperty::FontStretch => {
        "normal | <percentage [0,∞]> | ultra-condensed | extra-condensed | condensed | semi-condensed | semi-expanded | expanded | extra-expanded | ultra-expanded"
      }
      CssProperty::FontStyle => "normal | italic | left | right | oblique <angle [-90deg,90deg]>?",
      CssProperty::FontSynthesis => "none | [ weight || style || small-caps || position]",
      CssProperty::FontSynthesisPosition => "auto | none",
      CssProperty::FontSynthesisSmallCaps => "auto | none",
      CssProperty::FontSynthesisStyle => "auto | none | oblique-only",
      CssProperty::FontSynthesisWeight => "auto | none",
      CssProperty::FontVariant => {
        "normal | none | [ [ <common-lig-values> || <discretionary-lig-values> || <historical-lig-values> || <contextual-alt-values> ] || [ small-caps | all-small-caps | petite-caps | all-petite-caps | unicase | titling-caps ] || [ stylistic(<font-feature-value-name>) || historical-forms || styleset(<font-feature-value-name>#) || character-variant(<font-feature-value-name>#) || swash(<font-feature-value-name>) || ornaments(<font-feature-value-name>) || annotation(<font-feature-value-name>) ] || [ <numeric-figure-values> || <numeric-spacing-values> || <numeric-fraction-values> || ordinal || slashed-zero ] || [ <east-asian-variant-values> || <east-asian-width-values> || ruby ] || [ sub | super ] || [ text | emoji | unicode ] ]"
      }
      CssProperty::FontVariantAlternates => {
        "normal | [ stylistic(<font-feature-value-name>) || historical-forms || styleset(<font-feature-value-name>#) || character-variant(<font-feature-value-name>#) || swash(<font-feature-value-name>) || ornaments(<font-feature-value-name>) || annotation(<font-feature-value-name>) ]"
      }
      CssProperty::FontVariantCaps => {
        "normal | small-caps | all-small-caps | petite-caps | all-petite-caps | unicase | titling-caps"
      }
      CssProperty::FontVariantEastAsian => {
        "normal | [ <east-asian-variant-values> || <east-asian-width-values> || ruby ]"
      }
      CssProperty::FontVariantEmoji => "normal | text | emoji | unicode",
      CssProperty::FontVariantLigatures => {
        "normal | none | [ <common-lig-values> || <discretionary-lig-values> || <historical-lig-values> || <contextual-alt-values> ]"
      }
      CssProperty::FontVariantNumeric => {
        "normal | [ <numeric-figure-values> || <numeric-spacing-values> || <numeric-fraction-values> || ordinal || slashed-zero ]"
      }
      CssProperty::FontVariantPosition => "normal | sub | super",
      CssProperty::FontVariationSettings => "normal | [ <opentype-tag> <number> ]#",
      CssProperty::FontWeight => "<font-weight-absolute> | bolder | lighter",
      CssProperty::FontWidth => {
        "normal | <percentage [0,∞]> | ultra-condensed | extra-condensed | condensed | semi-condensed | semi-expanded | expanded | extra-expanded | ultra-expanded"
      }
      CssProperty::FootnoteDisplay => "block | inline | compact",
      CssProperty::FootnotePolicy => "auto | line | block",
      CssProperty::ForcedColorAdjust => "auto | none | preserve-parent-color",
      CssProperty::FrameSizing => "auto | content-width | content-height | content-block-size | content-inline-size",
      CssProperty::Gap => "<'row-gap'> <'column-gap'>?",
      CssProperty::GlyphOrientationVertical => "auto | 0deg | 90deg | 0 | 90",
      CssProperty::Grid => {
        "<'grid-template'> | <'grid-template-rows'> / [ auto-flow && dense? ] <'grid-auto-columns'>? | [ auto-flow && dense? ] <'grid-auto-rows'>? / <'grid-template-columns'>"
      }
      CssProperty::GridArea => "<grid-line> [ / <grid-line> ]{0,3}",
      CssProperty::GridAutoColumns => "<track-size>+",
      CssProperty::GridAutoFlow => "[ row | column ] || dense",
      CssProperty::GridAutoRows => "<track-size>+",
      CssProperty::GridColumn => "<grid-line> [ / <grid-line> ]?",
      CssProperty::GridColumnEnd => "<grid-line>",
      CssProperty::GridColumnGap => "normal | <length-percentage [0,∞]> | <line-width>",
      CssProperty::GridColumnStart => "<grid-line>",
      CssProperty::GridGap => "<'row-gap'> <'column-gap'>?",
      CssProperty::GridRow => "<grid-line> [ / <grid-line> ]?",
      CssProperty::GridRowEnd => "<grid-line>",
      CssProperty::GridRowGap => "normal | <length-percentage [0,∞]> | <line-width>",
      CssProperty::GridRowStart => "<grid-line>",
      CssProperty::GridTemplate => {
        "none | [ <'grid-template-rows'> / <'grid-template-columns'> ] | [ <line-names>? <string> <track-size>? <line-names>? ]+ [ / <explicit-track-list> ]?"
      }
      CssProperty::GridTemplateAreas => "none | <string>+",
      CssProperty::GridTemplateColumns => "none | <track-list> | <auto-track-list> | subgrid <line-name-list>?",
      CssProperty::GridTemplateRows => "none | <track-list> | <auto-track-list> | subgrid <line-name-list>?",
      CssProperty::HangingPunctuation => "none | [ first || [ force-end | allow-end ] || last ]",
      CssProperty::Height => {
        "auto | <length-percentage [0,∞]> | min-content | max-content | fit-content(<length-percentage [0,∞]>) | <calc-size()> | <anchor-size()> | stretch | fit-content | contain"
      }
      CssProperty::HyphenateCharacter => "auto | <string>",
      CssProperty::HyphenateLimitChars => "[ auto | <integer> ]{1,3}",
      CssProperty::HyphenateLimitLast => "none | always | column | page | spread",
      CssProperty::HyphenateLimitLines => "no-limit | <integer>",
      CssProperty::HyphenateLimitZone => "<length-percentage>",
      CssProperty::Hyphens => "none | manual | auto",
      CssProperty::ImageAnimation => "normal | paused | stopped | running",
      CssProperty::ImageOrientation => "from-image | none | [ <angle> || flip ]",
      CssProperty::ImageRendering => "auto | smooth | high-quality | pixelated | crisp-edges",
      CssProperty::ImageResolution => "[ from-image || <resolution> ] && snap?",
      CssProperty::InitialLetter => "normal | <number [1,∞]> <integer [1,∞]> | <number [1,∞]> && [ drop | raise ]?",
      CssProperty::InitialLetterAlign => "[ border-box? [ alphabetic | ideographic | hanging | leading ]? ]!",
      CssProperty::InitialLetterWrap => "none | first | all | grid | <length-percentage>",
      CssProperty::InlineSize => "<'width'>",
      CssProperty::InlineSizing => "normal | stretch",
      CssProperty::InputSecurity => "auto | none",
      CssProperty::Inset => "<'top'>{1,4}",
      CssProperty::InsetBlock => "<'top'>{1,2}",
      CssProperty::InsetBlockEnd => "auto | <length-percentage>",
      CssProperty::InsetBlockStart => "auto | <length-percentage>",
      CssProperty::InsetInline => "<'top'>{1,2}",
      CssProperty::InsetInlineEnd => "auto | <length-percentage>",
      CssProperty::InsetInlineStart => "auto | <length-percentage>",
      CssProperty::Interactivity => "auto | inert",
      CssProperty::InterestDelay => "<'interest-delay-start'>{1,2}",
      CssProperty::InterestDelayEnd => "normal | <time>",
      CssProperty::InterestDelayStart => "normal | <time>",
      CssProperty::InterpolateSize => "numeric-only | allow-keywords",
      CssProperty::Isolation => "<isolation-mode>",
      CssProperty::JustifyContent => {
        "normal | <content-distribution> | <overflow-position>? [ <content-position> | left | right ]"
      }
      CssProperty::JustifyItems => {
        "normal | stretch | <baseline-position> | <overflow-position>? [ <self-position> | left | right ] | legacy | legacy && [ left | right | center ]"
      }
      CssProperty::JustifySelf => {
        "auto | <overflow-position>? [ normal | <self-position> | left | right ] | stretch | <baseline-position> | anchor-center"
      }
      CssProperty::Left => "auto | <length-percentage> | <anchor()> | <anchor-size()>",
      CssProperty::LetterSpacing => "normal | <length-percentage>",
      CssProperty::LightingColor => "<color>",
      CssProperty::LineBreak => "auto | loose | normal | strict | anywhere",
      CssProperty::LineClamp => "none | [<integer [1,∞]> || <'block-ellipsis'>] -webkit-legacy?",
      CssProperty::LineFitEdge => "leading | <text-edge>",
      CssProperty::LineGrid => "match-parent | create",
      CssProperty::LineHeight => "normal | <number [0,∞]> | <length-percentage [0,∞]>",
      CssProperty::LineHeightStep => "<length [0,∞]>",
      CssProperty::LinePadding => "<length>",
      CssProperty::LineSnap => "none | baseline | contain",
      CssProperty::LinkParameters => "none | <param()>#",
      CssProperty::ListStyle => "<'list-style-position'> || <'list-style-image'> || <'list-style-type'>",
      CssProperty::ListStyleImage => "<image> | none",
      CssProperty::ListStylePosition => "inside | outside",
      CssProperty::ListStyleType => "<counter-style> | <string> | none",
      CssProperty::Margin => "<'margin-top'>{1,4}",
      CssProperty::MarginBlock => "<'margin-top'>{1,2}",
      CssProperty::MarginBlockEnd => "<'margin-top'>",
      CssProperty::MarginBlockStart => "<'margin-top'>",
      CssProperty::MarginBottom => "<length-percentage> | auto | <anchor-size()>",
      CssProperty::MarginBreak => "auto | keep | discard",
      CssProperty::MarginInline => "<'margin-top'>{1,2}",
      CssProperty::MarginInlineEnd => "<'margin-top'>",
      CssProperty::MarginInlineStart => "<'margin-top'>",
      CssProperty::MarginLeft => "<length-percentage> | auto | <anchor-size()>",
      CssProperty::MarginRight => "<length-percentage> | auto | <anchor-size()>",
      CssProperty::MarginTop => "<length-percentage> | auto | <anchor-size()>",
      CssProperty::MarginTrim => {
        "none | [ block || inline ] | [ block-start || inline-start || block-end || inline-end ]"
      }
      CssProperty::Marker => "none | <marker-ref>",
      CssProperty::MarkerEnd => "none | <marker-ref>",
      CssProperty::MarkerMid => "none | <marker-ref>",
      CssProperty::MarkerSide => "match-self | match-parent",
      CssProperty::MarkerStart => "none | <marker-ref>",
      CssProperty::Mask => "<mask-layer>#",
      CssProperty::MaskBorder => {
        "<'mask-border-source'> || <'mask-border-slice'> [ / <'mask-border-width'>? [ / <'mask-border-outset'> ]? ]? || <'mask-border-repeat'> || <'mask-border-mode'>"
      }
      CssProperty::MaskBorderMode => "luminance | alpha",
      CssProperty::MaskBorderOutset => "<'border-image-outset'>",
      CssProperty::MaskBorderRepeat => "<'border-image-repeat'>",
      CssProperty::MaskBorderSlice => "<'border-image-slice'>",
      CssProperty::MaskBorderSource => "<'border-image-source'>",
      CssProperty::MaskBorderWidth => "<'border-image-width'>",
      CssProperty::MaskClip => "[ <coord-box> | no-clip ]#",
      CssProperty::MaskComposite => "<compositing-operator>#",
      CssProperty::MaskImage => "<mask-reference>#",
      CssProperty::MaskMode => "<masking-mode>#",
      CssProperty::MaskOrigin => "<coord-box>#",
      CssProperty::MaskPosition => "<position>#",
      CssProperty::MaskRepeat => "<repeat-style>#",
      CssProperty::MaskSize => "<bg-size>#",
      CssProperty::MaskType => "luminance | alpha",
      CssProperty::MathDepth => "auto-add | add(<integer>) | <integer>",
      CssProperty::MathShift => "normal | compact",
      CssProperty::MathStyle => "normal | compact",
      CssProperty::MaxBlockSize => "<'max-width'>",
      CssProperty::MaxHeight => {
        "none | <length-percentage [0,∞]> | min-content | max-content | fit-content(<length-percentage [0,∞]>) | <calc-size()> | <anchor-size()> | stretch | fit-content | contain"
      }
      CssProperty::MaxInlineSize => "<'max-width'>",
      CssProperty::MaxLines => "none | <integer [1,∞]>",
      CssProperty::MaxWidth => {
        "none | <length-percentage [0,∞]> | min-content | max-content | fit-content(<length-percentage [0,∞]>) | <calc-size()> | <anchor-size()> | stretch | fit-content | contain"
      }
      CssProperty::MinBlockSize => "<'min-width'>",
      CssProperty::MinHeight => {
        "auto | <length-percentage [0,∞]> | min-content | max-content | fit-content(<length-percentage [0,∞]>) | <calc-size()> | <anchor-size()> | stretch | fit-content | contain"
      }
      CssProperty::MinInlineSize => "<'min-width'>",
      CssProperty::MinIntrinsicSizing => "legacy | zero-if-scroll || zero-if-extrinsic",
      CssProperty::MinWidth => {
        "auto | <length-percentage [0,∞]> | min-content | max-content | fit-content(<length-percentage [0,∞]>) | <calc-size()> | <anchor-size()> | stretch | fit-content | contain"
      }
      CssProperty::MixBlendMode => "<blend-mode> | plus-lighter",
      CssProperty::NavDown => "auto | <id> [ current | root | <target-name> ]?",
      CssProperty::NavLeft => "auto | <id> [ current | root | <target-name> ]?",
      CssProperty::NavRight => "auto | <id> [ current | root | <target-name> ]?",
      CssProperty::NavUp => "auto | <id> [ current | root | <target-name> ]?",
      CssProperty::ObjectFit => "fill | none | [contain | cover] || scale-down",
      CssProperty::ObjectPosition => "<position>",
      CssProperty::ObjectViewBox => "none | <basic-shape-rect>",
      CssProperty::Offset => {
        "[ <'offset-position'>? [ <'offset-path'> [ <'offset-distance'> || <'offset-rotate'> ]? ]? ]! [ / <'offset-anchor'> ]?"
      }
      CssProperty::OffsetAnchor => "auto | <position>",
      CssProperty::OffsetDistance => "<length-percentage>",
      CssProperty::OffsetPath => "none | <offset-path> || <coord-box>",
      CssProperty::OffsetPosition => "normal | auto | <position>",
      CssProperty::OffsetRotate => "[ auto | reverse ] || <angle>",
      CssProperty::Opacity => "<opacity-value>",
      CssProperty::Order => "<integer>",
      CssProperty::Orphans => "<integer [1,∞]>",
      CssProperty::Outline => "<'outline-width'> || <'outline-style'> || <'outline-color'>",
      CssProperty::OutlineColor => "auto | <'border-top-color'>",
      CssProperty::OutlineOffset => "<length>",
      CssProperty::OutlineStyle => "auto | <outline-line-style>",
      CssProperty::OutlineWidth => "<line-width>",
      CssProperty::Overflow => "<'overflow-block'>{1,2}",
      CssProperty::OverflowAnchor => "auto | none",
      CssProperty::OverflowBlock => "visible | hidden | clip | scroll | auto",
      CssProperty::OverflowClipMargin => "<visual-box> || <length>",
      CssProperty::OverflowClipMarginBlock => "<visual-box> || <length>",
      CssProperty::OverflowClipMarginBlockEnd => "<visual-box> || <length>",
      CssProperty::OverflowClipMarginBlockStart => "<visual-box> || <length>",
      CssProperty::OverflowClipMarginBottom => "<visual-box> || <length>",
      CssProperty::OverflowClipMarginInline => "<visual-box> || <length>",
      CssProperty::OverflowClipMarginInlineEnd => "<visual-box> || <length>",
      CssProperty::OverflowClipMarginInlineStart => "<visual-box> || <length>",
      CssProperty::OverflowClipMarginLeft => "<visual-box> || <length>",
      CssProperty::OverflowClipMarginRight => "<visual-box> || <length>",
      CssProperty::OverflowClipMarginTop => "<visual-box> || <length>",
      CssProperty::OverflowInline => "visible | hidden | clip | scroll | auto",
      CssProperty::OverflowWrap => "normal | break-word | anywhere",
      CssProperty::OverflowX => "visible | hidden | clip | scroll | auto",
      CssProperty::OverflowY => "visible | hidden | clip | scroll | auto",
      CssProperty::Overlay => "none | auto",
      CssProperty::OverscrollBehavior => "[ contain | none | auto | chain ]{1,2}",
      CssProperty::OverscrollBehaviorBlock => "contain | none | auto | chain",
      CssProperty::OverscrollBehaviorInline => "contain | none | auto | chain",
      CssProperty::OverscrollBehaviorX => "contain | none | auto | chain",
      CssProperty::OverscrollBehaviorY => "contain | none | auto | chain",
      CssProperty::Padding => "<'padding-top'>{1,4}",
      CssProperty::PaddingBlock => "<'padding-top'>{1,2}",
      CssProperty::PaddingBlockEnd => "<'padding-top'>",
      CssProperty::PaddingBlockStart => "<'padding-top'>",
      CssProperty::PaddingBottom => "<length-percentage [0,∞]>",
      CssProperty::PaddingInline => "<'padding-top'>{1,2}",
      CssProperty::PaddingInlineEnd => "<'padding-top'>",
      CssProperty::PaddingInlineStart => "<'padding-top'>",
      CssProperty::PaddingLeft => "<length-percentage [0,∞]>",
      CssProperty::PaddingRight => "<length-percentage [0,∞]>",
      CssProperty::PaddingTop => "<length-percentage [0,∞]>",
      CssProperty::Page => "auto | <custom-ident>",
      CssProperty::PageBreakAfter => "auto | always | avoid | left | right | inherit",
      CssProperty::PageBreakBefore => "auto | always | avoid | left | right | inherit",
      CssProperty::PageBreakInside => "avoid | auto | inherit",
      CssProperty::PaintOrder => "normal | [ fill || stroke || markers ]",
      CssProperty::PathLength => "none | @@ unknown symbol \"number [0,∞]\"",
      CssProperty::Pause => "<'pause-before'> <'pause-after'>?",
      CssProperty::PauseAfter => "<time [0s,∞]> | none | x-weak | weak | medium | strong | x-strong",
      CssProperty::PauseBefore => "<time [0s,∞]> | none | x-weak | weak | medium | strong | x-strong",
      CssProperty::Perspective => "none | <length [0,∞]>",
      CssProperty::PerspectiveOrigin => "<position>",
      CssProperty::PlaceContent => "<'align-content'> <'justify-content'>?",
      CssProperty::PlaceItems => "<'align-items'> <'justify-items'>?",
      CssProperty::PlaceSelf => "<'align-self'> <'justify-self'>?",
      CssProperty::PointerEvents => {
        "auto | bounding-box | visiblePainted | visibleFill | visibleStroke | visible | painted | fill | stroke | all | none"
      }
      CssProperty::PointerTimeline => "[ <'pointer-timeline-name'> <'pointer-timeline-axis'>? ]#",
      CssProperty::PointerTimelineAxis => "[ block | inline | x | y ]#",
      CssProperty::PointerTimelineName => "[ none | <dashed-ident> ]#",
      CssProperty::Position => "static | relative | absolute | sticky | fixed | <running()>",
      CssProperty::PositionAnchor => "normal | none | auto | <anchor-name> | match-parent",
      CssProperty::PositionArea => "none | <position-area>",
      CssProperty::PositionTry => "<'position-try-order'>? <'position-try-fallbacks'>",
      CssProperty::PositionTryFallbacks => "none | [ [<dashed-ident> || <try-tactic>] | <position-area> ]#",
      CssProperty::PositionTryOrder => "normal | <try-size>",
      CssProperty::PositionVisibility => "always | [ anchors-valid || anchors-visible || no-overflow ]",
      CssProperty::PrintColorAdjust => "economy | exact",
      CssProperty::Quotes => "auto | none | match-parent | [ <string> <string> ]+",
      CssProperty::R => "<length-percentage>",
      CssProperty::ReadingFlow => {
        "normal | source-order | flex-visual | flex-flow | grid-rows | grid-columns | grid-order"
      }
      CssProperty::ReadingOrder => "<integer>",
      CssProperty::RegionFragment => "auto | break",
      CssProperty::Resize => "none | both | horizontal | vertical | block | inline",
      CssProperty::Rest => "<'rest-before'> <'rest-after'>?",
      CssProperty::RestAfter => "<time [0s,∞]> | none | x-weak | weak | medium | strong | x-strong",
      CssProperty::RestBefore => "<time [0s,∞]> | none | x-weak | weak | medium | strong | x-strong",
      CssProperty::Right => "auto | <length-percentage> | <anchor()> | <anchor-size()>",
      CssProperty::Rotate => "none | <angle> | [ x | y | z | <number>{3} ] && <angle>",
      CssProperty::RowGap => "normal | <length-percentage [0,∞]> | <line-width>",
      CssProperty::RowRule => "<gap-rule-list> | <gap-auto-rule-list>",
      CssProperty::RowRuleBreak => "none | normal | intersection",
      CssProperty::RowRuleColor => "<line-color-list> | <auto-line-color-list>",
      CssProperty::RowRuleInset => "<'column-rule-inset-cap'> [ / <'column-rule-inset-junction'> ]?",
      CssProperty::RowRuleInsetCap => "<length-percentage> [ <length-percentage> ]?",
      CssProperty::RowRuleInsetCapEnd => "<length-percentage>",
      CssProperty::RowRuleInsetCapStart => "<length-percentage>",
      CssProperty::RowRuleInsetEnd => "<length-percentage>",
      CssProperty::RowRuleInsetJunction => "<length-percentage> [ <length-percentage> ]?",
      CssProperty::RowRuleInsetJunctionEnd => "<length-percentage>",
      CssProperty::RowRuleInsetJunctionStart => "<length-percentage>",
      CssProperty::RowRuleInsetStart => "<length-percentage>",
      CssProperty::RowRuleStyle => "<line-style-list> | <auto-line-style-list>",
      CssProperty::RowRuleVisibilityItems => "all | around | between | normal",
      CssProperty::RowRuleWidth => "<line-width-list> | <auto-line-width-list>",
      CssProperty::RubyAlign => "start | center | space-between | space-around",
      CssProperty::RubyMerge => "separate | merge | auto",
      CssProperty::RubyOverhang => "auto | spaces",
      CssProperty::RubyPosition => "[ alternate || [ over | under ] ] | inter-character",
      CssProperty::Rule => "<'column-rule'>",
      CssProperty::RuleBreak => "<'column-rule-break'>",
      CssProperty::RuleColor => "<'column-rule-color'>",
      CssProperty::RuleInset => "<'column-rule-inset'>",
      CssProperty::RuleInsetCap => "<'column-rule-inset-cap'>",
      CssProperty::RuleInsetEnd => "<'column-rule-inset-end'>",
      CssProperty::RuleInsetJunction => "<'column-rule-inset-junction'>",
      CssProperty::RuleInsetStart => "<'column-rule-inset-start'>",
      CssProperty::RuleOverlap => "row-over-column | column-over-row",
      CssProperty::RuleStyle => "<'column-rule-style'>",
      CssProperty::RuleVisibilityItems => "<'column-rule-visibility-items'>",
      CssProperty::RuleWidth => "<'column-rule-width'>",
      CssProperty::Rx => "<length-percentage> | auto",
      CssProperty::Ry => "<length-percentage> | auto",
      CssProperty::Scale => "none | [ <number> | <percentage> ]{1,3}",
      CssProperty::ScrollBehavior => "auto | smooth",
      CssProperty::ScrollInitialTarget => "none | nearest",
      CssProperty::ScrollMargin => "<length>{1,4}",
      CssProperty::ScrollMarginBlock => "<length>{1,2}",
      CssProperty::ScrollMarginBlockEnd => "<length>",
      CssProperty::ScrollMarginBlockStart => "<length>",
      CssProperty::ScrollMarginBottom => "<length>",
      CssProperty::ScrollMarginInline => "<length>{1,2}",
      CssProperty::ScrollMarginInlineEnd => "<length>",
      CssProperty::ScrollMarginInlineStart => "<length>",
      CssProperty::ScrollMarginLeft => "<length>",
      CssProperty::ScrollMarginRight => "<length>",
      CssProperty::ScrollMarginTop => "<length>",
      CssProperty::ScrollMarkerGroup => "none | [ [ before | after ] || [ links | tabs ] ]",
      CssProperty::ScrollPadding => "[ auto | <length-percentage [0,∞]> ]{1,4}",
      CssProperty::ScrollPaddingBlock => "[ auto | <length-percentage [0,∞]> ]{1,2}",
      CssProperty::ScrollPaddingBlockEnd => "auto | <length-percentage [0,∞]>",
      CssProperty::ScrollPaddingBlockStart => "auto | <length-percentage [0,∞]>",
      CssProperty::ScrollPaddingBottom => "auto | <length-percentage [0,∞]>",
      CssProperty::ScrollPaddingInline => "[ auto | <length-percentage [0,∞]> ]{1,2}",
      CssProperty::ScrollPaddingInlineEnd => "auto | <length-percentage [0,∞]>",
      CssProperty::ScrollPaddingInlineStart => "auto | <length-percentage [0,∞]>",
      CssProperty::ScrollPaddingLeft => "auto | <length-percentage [0,∞]>",
      CssProperty::ScrollPaddingRight => "auto | <length-percentage [0,∞]>",
      CssProperty::ScrollPaddingTop => "auto | <length-percentage [0,∞]>",
      CssProperty::ScrollSnapAlign => "[ none | start | end | center ]{1,2}",
      CssProperty::ScrollSnapStop => "normal | always",
      CssProperty::ScrollSnapType => "none | [ x | y | block | inline | both ] [ mandatory | proximity ]?",
      CssProperty::ScrollTargetGroup => "none | auto",
      CssProperty::ScrollTimeline => "[ <'scroll-timeline-name'> <'scroll-timeline-axis'>? ]#",
      CssProperty::ScrollTimelineAxis => "[ block | inline | x | y ]#",
      CssProperty::ScrollTimelineName => "[ none | <dashed-ident> ]#",
      CssProperty::ScrollbarColor => "auto | <color>{2}",
      CssProperty::ScrollbarGutter => "auto | stable && both-edges?",
      CssProperty::ScrollbarInset => "<length>{1,4}",
      CssProperty::ScrollbarMinThumbSize => "<length>",
      CssProperty::ScrollbarMode => "auto | classic | overlay | none",
      CssProperty::ScrollbarWidth => "auto | thin | none",
      CssProperty::ShapeImageThreshold => "<opacity-value>",
      CssProperty::ShapeInside => "auto | outside-shape | [ <basic-shape> || shape-box ] | <image> | display",
      CssProperty::ShapeMargin => "<length-percentage [0,∞]>",
      CssProperty::ShapeOutside => "none | [ <basic-shape> || <shape-box> ] | <image>",
      CssProperty::ShapePadding => "<length-percentage [0,∞]>",
      CssProperty::ShapeRendering => "auto | optimizeSpeed | crispEdges | geometricPrecision",
      CssProperty::SliderOrientation => "auto | left-to-right | right-to-left | top-to-bottom | bottom-to-top",
      CssProperty::SpatialNavigationAction => "auto | focus | scroll",
      CssProperty::SpatialNavigationContain => "auto | contain",
      CssProperty::SpatialNavigationFunction => "normal | grid",
      CssProperty::Speak => "auto | never | always",
      CssProperty::SpeakAs => "normal | spell-out || digits || [ literal-punctuation | no-punctuation ]",
      CssProperty::StopColor => "<'color'>",
      CssProperty::StopOpacity => "<'opacity'>",
      CssProperty::StringSet => "none | [ <custom-ident> <string>+ ]#",
      CssProperty::Stroke => "<paint>",
      CssProperty::StrokeAlign => "center | inset | outset",
      CssProperty::StrokeAlignment => "center | inner | outer",
      CssProperty::StrokeBreak => "bounding-box | slice | clone",
      CssProperty::StrokeColor => "<color>#",
      CssProperty::StrokeDashCornerPropdefStrokeDashCorner => "none | <length>",
      CssProperty::StrokeDashJustify => "none | [ stretch | compress ] || [ dashes || gaps ]",
      CssProperty::StrokeDashadjust => "none | [stretch | compress] [dashes | gaps]?",
      CssProperty::StrokeDasharray => "none | [<length-percentage> | <number>]+#",
      CssProperty::StrokeDashcornerStrokedashcornerproperty => "none | <length>",
      CssProperty::StrokeDashoffset => "<length-percentage> | <number>",
      CssProperty::StrokeImage => "<paint>#",
      CssProperty::StrokeLinecap => "butt | round | square",
      CssProperty::StrokeLinejoin => "[ crop | arcs | miter ] || [ bevel | round | fallback ]",
      CssProperty::StrokeMiterlimit => "<number>",
      CssProperty::StrokeOpacity => "<'opacity'>",
      CssProperty::StrokeOrigin => "match-parent | fill-box | stroke-box | content-box | padding-box | border-box",
      CssProperty::StrokePosition => "<position>#",
      CssProperty::StrokeRepeat => "<repeat-style>#",
      CssProperty::StrokeSize => "<bg-size>#",
      CssProperty::StrokeWidth => "[ <length-percentage> | <line-width> | <number> ]#",
      CssProperty::TabSize => "<number [0,∞]> | <length [0,∞]>",
      CssProperty::TableLayout => "auto | fixed",
      CssProperty::TextAlign => "start | end | left | right | center | <string> | justify | match-parent | justify-all",
      CssProperty::TextAlignAll => "start | end | left | right | center | <string> | justify | match-parent",
      CssProperty::TextAlignLast => "auto | start | end | left | right | center | justify | match-parent",
      CssProperty::TextAnchor => "start | middle | end",
      CssProperty::TextAutospace => "normal | <autospace> | auto",
      CssProperty::TextBox => "normal | <'text-box-trim'> || <'text-box-edge'>",
      CssProperty::TextBoxEdge => "auto | <text-edge>",
      CssProperty::TextBoxTrim => "none | trim-start | trim-end | trim-both",
      CssProperty::TextCombineUpright => "none | all | [ digits <integer [2,4]>? ]",
      CssProperty::TextDecoration => {
        "<'text-decoration-line'> || <'text-decoration-thickness'> || <'text-decoration-style'> || <'text-decoration-color'>"
      }
      CssProperty::TextDecorationColor => "<color>",
      CssProperty::TextDecorationInset => "<length>{1,2} | auto",
      CssProperty::TextDecorationLine => {
        "none | [ underline || overline || line-through || blink ] | spelling-error | grammar-error"
      }
      CssProperty::TextDecorationSkip => "none | auto",
      CssProperty::TextDecorationSkipBox => "none | all",
      CssProperty::TextDecorationSkipInk => "auto | none | all",
      CssProperty::TextDecorationSkipSelf => {
        "auto | skip-all | [ skip-underline || skip-overline || skip-line-through ] | no-skip"
      }
      CssProperty::TextDecorationSkipSpaces => "none | all | [ start || end ]",
      CssProperty::TextDecorationStyle => "solid | double | dotted | dashed | wavy",
      CssProperty::TextDecorationThickness => "auto | from-font | <length-percentage> | <line-width>",
      CssProperty::TextEmphasis => "<'text-emphasis-style'> || <'text-emphasis-color'>",
      CssProperty::TextEmphasisColor => "<color>",
      CssProperty::TextEmphasisPosition => "[ over | under ] && [ right | left ]?",
      CssProperty::TextEmphasisSkip => "spaces || punctuation || symbols || narrow",
      CssProperty::TextEmphasisStyle => {
        "none | [ [ filled | open ] || [ dot | circle | double-circle | triangle | sesame ] ] | <string>"
      }
      CssProperty::TextFit => "[ none | grow | shrink ] [consistent | per-line | per-line-all]? <percentage>?",
      CssProperty::TextGroupAlign => "none | start | end | left | right | center",
      CssProperty::TextIndent => "[ <length-percentage> ] && hanging? && each-line?",
      CssProperty::TextJustify => "[ auto | none | inter-word | inter-character | ruby ] || no-compress",
      CssProperty::TextOrientation => "mixed | upright | sideways",
      CssProperty::TextOverflow => "[ clip | ellipsis | <string> | fade | <fade()> ]{1,2}",
      CssProperty::TextRendering => "auto | optimizeSpeed | optimizeLegibility | geometricPrecision",
      CssProperty::TextShadow => "none | <shadow>#",
      CssProperty::TextSizeAdjust => "auto | none | <percentage [0,∞]>",
      CssProperty::TextSpacing => "none | auto | <spacing-trim> || <autospace>",
      CssProperty::TextSpacingTrim => "<spacing-trim> | auto",
      CssProperty::TextTransform => {
        "none | [capitalize | uppercase | lowercase ] || full-width || full-size-kana | math-auto"
      }
      CssProperty::TextUnderlineOffset => "auto | <length-percentage>",
      CssProperty::TextUnderlinePosition => "auto | [ from-font | under ] || [ left | right ]",
      CssProperty::TextWrap => "<'text-wrap-mode'> || <'text-wrap-style'>",
      CssProperty::TextWrapMode => "wrap | nowrap",
      CssProperty::TextWrapStyle => "auto | balance | stable | pretty | avoid-orphans",
      CssProperty::TimelineScope => "none | all | <dashed-ident>#",
      CssProperty::TimelineTrigger => {
        "none | [ <'timeline-trigger-name'> <'timeline-trigger-source'> <'timeline-trigger-activation-range'> [ '/' <'timeline-trigger-active-range'> ]? ]#"
      }
      CssProperty::TimelineTriggerActivationRange => {
        "[ <'timeline-trigger-activation-range-start'> <'timeline-trigger-activation-range-end'>? ]#"
      }
      CssProperty::TimelineTriggerActivationRangeEnd => {
        "[ normal | <length-percentage> | <timeline-range-name> <length-percentage>? ]#"
      }
      CssProperty::TimelineTriggerActivationRangeStart => {
        "[ normal | <length-percentage> | <timeline-range-name> <length-percentage>? ]#"
      }
      CssProperty::TimelineTriggerActiveRange => {
        "[ <'timeline-trigger-active-range-start'> <'timeline-trigger-active-range-end'>? ]#"
      }
      CssProperty::TimelineTriggerActiveRangeEnd => {
        "[ auto | normal | <length-percentage> | <timeline-range-name> <length-percentage>? ]#"
      }
      CssProperty::TimelineTriggerActiveRangeStart => {
        "[ auto | normal | <length-percentage> | <timeline-range-name> <length-percentage>? ]#"
      }
      CssProperty::TimelineTriggerName => "none | <dashed-ident>#",
      CssProperty::TimelineTriggerSource => "<single-animation-timeline>#",
      CssProperty::Top => "auto | <length-percentage> | <anchor()> | <anchor-size()>",
      CssProperty::TouchAction => {
        "auto | none | [ [ pan-x | pan-left | pan-right ] || [ pan-y | pan-up | pan-down ] || pinch-zoom ] | manipulation"
      }
      CssProperty::Transform => "none | <transform-list>",
      CssProperty::TransformBox => "content-box | border-box | fill-box | stroke-box | view-box",
      CssProperty::TransformOrigin => {
        "[ left | center | right | top | bottom | <length-percentage> ] | [ left | center | right | <length-percentage> ] [ top | center | bottom | <length-percentage> ] <length>? | [ [ center | left | right ] && [ center | top | bottom ] ] <length>?"
      }
      CssProperty::TransformStyle => "flat | preserve-3d",
      CssProperty::Transition => "<single-transition>#",
      CssProperty::TransitionBehavior => "<transition-behavior-value>#",
      CssProperty::TransitionDelay => "<time>#",
      CssProperty::TransitionDuration => "<time [0s,∞]>#",
      CssProperty::TransitionProperty => "none | <single-transition-property>#",
      CssProperty::TransitionTimingFunction => "<easing-function>#",
      CssProperty::Translate => "none | <length-percentage> [ <length-percentage> <length>? ]?",
      CssProperty::TriggerScope => "none | all | <dashed-ident>#",
      CssProperty::UnicodeBidi => "normal | embed | isolate | bidi-override | isolate-override | plaintext",
      CssProperty::UserSelect => "auto | text | none | contain | all",
      CssProperty::VectorEffect => "none | non-scaling-stroke | non-scaling-size | non-rotation | fixed-position",
      CssProperty::VerticalAlign => "[ first | last] || <'alignment-baseline'> || <'baseline-shift'>",
      CssProperty::ViewTimeline => "[ <'view-timeline-name'> [ <'view-timeline-axis'> || <'view-timeline-inset'> ]? ]#",
      CssProperty::ViewTimelineAxis => "[ block | inline | x | y ]#",
      CssProperty::ViewTimelineInset => "[ [ auto | <length-percentage> ]{1,2} ]#",
      CssProperty::ViewTimelineName => "[ none | <dashed-ident> ]#",
      CssProperty::ViewTransitionClass => "none | <custom-ident>+",
      CssProperty::ViewTransitionGroup => "normal | contain | nearest | <custom-ident>",
      CssProperty::ViewTransitionName => "none | <custom-ident>",
      CssProperty::ViewTransitionScope => "none | all",
      CssProperty::Visibility => "visible | hidden | force-hidden | collapse",
      CssProperty::VoiceBalance => "<number> | left | center | right | leftwards | rightwards",
      CssProperty::VoiceDuration => "auto | <time [0s,∞]>",
      CssProperty::VoiceFamily => "[ <voice-family-name> | <generic-voice> ]# | preserve",
      CssProperty::VoicePitch => {
        "<frequency [0Hz,∞]> && absolute | [ [ x-low | low | medium | high | x-high ] || [ <frequency [0Hz,∞]> | <semitones> | <percentage> ] ]"
      }
      CssProperty::VoiceRange => {
        "<frequency [0Hz,∞]> && absolute | [ [ x-low | low | medium | high | x-high ] || [ <frequency [0Hz,∞]> | <semitones> | <percentage> ] ]"
      }
      CssProperty::VoiceRate => "[ normal | x-slow | slow | medium | fast | x-fast ] || <percentage [0,∞]>",
      CssProperty::VoiceStress => "normal | strong | moderate | none | reduced",
      CssProperty::VoiceVolume => "silent | [ [ x-soft | soft | medium | loud | x-loud ] || <decibel> ]",
      CssProperty::WhiteSpace => {
        "normal | pre | pre-wrap | pre-line | <'white-space-collapse'> || <'text-wrap-mode'> || <'white-space-trim'>"
      }
      CssProperty::WhiteSpaceCollapse => {
        "collapse | discard | preserve | preserve-breaks | preserve-spaces | break-spaces"
      }
      CssProperty::WhiteSpaceTrim => "none | discard-before || discard-after || discard-inner",
      CssProperty::Widows => "<integer [1,∞]>",
      CssProperty::Width => {
        "auto | <length-percentage [0,∞]> | min-content | max-content | fit-content(<length-percentage [0,∞]>) | <calc-size()> | <anchor-size()> | stretch | fit-content | contain"
      }
      CssProperty::WillChange => "auto | <animateable-feature>#",
      CssProperty::WordBreak => "normal | break-all | keep-all | manual | auto-phrase | break-word",
      CssProperty::WordSpaceTransform => "none | [ space | ideographic-space ] && auto-phrase?",
      CssProperty::WordSpacing => "normal | <length-percentage>",
      CssProperty::WordWrap => "normal | break-word | anywhere",
      CssProperty::WrapAfter => "auto | avoid | avoid-line | avoid-flex | line | flex",
      CssProperty::WrapBefore => "auto | avoid | avoid-line | avoid-flex | line | flex",
      CssProperty::WrapFlow => "auto | both | start | end | minimum | maximum | clear",
      CssProperty::WrapInside => "auto | avoid",
      CssProperty::WrapThrough => "wrap | none",
      CssProperty::WritingMode => "horizontal-tb | vertical-rl | vertical-lr | sideways-rl | sideways-lr",
      CssProperty::X => "<length-percentage>",
      CssProperty::Y => "<length-percentage>",
      CssProperty::ZIndex => "auto | <integer> | inherit",
      CssProperty::Zoom => "<number [0,∞]> | <percentage [0,∞]>",
      CssProperty::Unknown(_) => "",
    }
  }

  pub fn initial(self) -> &'static str {
    match self {
      CssProperty::WebkitAlignContent => "",
      CssProperty::WebkitAlignItems => "",
      CssProperty::WebkitAlignSelf => "",
      CssProperty::WebkitAnimation => "",
      CssProperty::WebkitAnimationDelay => "",
      CssProperty::WebkitAnimationDirection => "",
      CssProperty::WebkitAnimationDuration => "",
      CssProperty::WebkitAnimationFillMode => "",
      CssProperty::WebkitAnimationIterationCount => "",
      CssProperty::WebkitAnimationName => "",
      CssProperty::WebkitAnimationPlayState => "",
      CssProperty::WebkitAnimationTimingFunction => "",
      CssProperty::WebkitAppearance => "",
      CssProperty::WebkitBackfaceVisibility => "",
      CssProperty::WebkitBackgroundClip => "",
      CssProperty::WebkitBackgroundOrigin => "",
      CssProperty::WebkitBackgroundSize => "",
      CssProperty::WebkitBorderBottomLeftRadius => "",
      CssProperty::WebkitBorderBottomRightRadius => "",
      CssProperty::WebkitBorderRadius => "",
      CssProperty::WebkitBorderTopLeftRadius => "",
      CssProperty::WebkitBorderTopRightRadius => "",
      CssProperty::WebkitBoxAlign => "",
      CssProperty::WebkitBoxFlex => "",
      CssProperty::WebkitBoxOrdinalGroup => "",
      CssProperty::WebkitBoxOrient => "",
      CssProperty::WebkitBoxPack => "",
      CssProperty::WebkitBoxShadow => "",
      CssProperty::WebkitBoxSizing => "",
      CssProperty::WebkitFilter => "",
      CssProperty::WebkitFlex => "",
      CssProperty::WebkitFlexBasis => "",
      CssProperty::WebkitFlexDirection => "",
      CssProperty::WebkitFlexFlow => "",
      CssProperty::WebkitFlexGrow => "",
      CssProperty::WebkitFlexShrink => "",
      CssProperty::WebkitFlexWrap => "",
      CssProperty::WebkitJustifyContent => "",
      CssProperty::WebkitLineClamp => "none",
      CssProperty::WebkitMask => "",
      CssProperty::WebkitMaskBoxImage => "",
      CssProperty::WebkitMaskBoxImageOutset => "",
      CssProperty::WebkitMaskBoxImageRepeat => "",
      CssProperty::WebkitMaskBoxImageSlice => "",
      CssProperty::WebkitMaskBoxImageSource => "",
      CssProperty::WebkitMaskBoxImageWidth => "",
      CssProperty::WebkitMaskClip => "",
      CssProperty::WebkitMaskComposite => "",
      CssProperty::WebkitMaskImage => "",
      CssProperty::WebkitMaskOrigin => "",
      CssProperty::WebkitMaskPosition => "",
      CssProperty::WebkitMaskRepeat => "",
      CssProperty::WebkitMaskSize => "",
      CssProperty::WebkitOrder => "",
      CssProperty::WebkitPerspective => "",
      CssProperty::WebkitPerspectiveOrigin => "",
      CssProperty::WebkitTextFillColor => "currentcolor",
      CssProperty::WebkitTextSizeAdjust => "",
      CssProperty::WebkitTextStroke => "See individual properties",
      CssProperty::WebkitTextStrokeColor => "currentcolor",
      CssProperty::WebkitTextStrokeWidth => "0",
      CssProperty::WebkitTransform => "",
      CssProperty::WebkitTransformOrigin => "",
      CssProperty::WebkitTransformStyle => "",
      CssProperty::WebkitTransition => "",
      CssProperty::WebkitTransitionDelay => "",
      CssProperty::WebkitTransitionDuration => "",
      CssProperty::WebkitTransitionProperty => "",
      CssProperty::WebkitTransitionTimingFunction => "",
      CssProperty::WebkitUserSelect => "",
      CssProperty::AccentColor => "auto",
      CssProperty::AlignContent => "normal",
      CssProperty::AlignItems => "normal",
      CssProperty::AlignSelf => "auto",
      CssProperty::AlignmentBaseline => "baseline",
      CssProperty::All => "see individual properties",
      CssProperty::AnchorName => "none",
      CssProperty::AnchorScope => "none",
      CssProperty::Animation => "see individual properties",
      CssProperty::AnimationComposition => "replace",
      CssProperty::AnimationDelay => "0s",
      CssProperty::AnimationDirection => "normal",
      CssProperty::AnimationDuration => "auto",
      CssProperty::AnimationFillMode => "none",
      CssProperty::AnimationIterationCount => "1",
      CssProperty::AnimationName => "none",
      CssProperty::AnimationPlayState => "running",
      CssProperty::AnimationRange => "see individual properties",
      CssProperty::AnimationRangeCenter => "normal",
      CssProperty::AnimationRangeEnd => "normal",
      CssProperty::AnimationRangeStart => "normal",
      CssProperty::AnimationTimeline => "auto",
      CssProperty::AnimationTimingFunction => "ease",
      CssProperty::AnimationTrigger => "none",
      CssProperty::Appearance => "none",
      CssProperty::AspectRatio => "auto",
      CssProperty::BackdropFilter => "none",
      CssProperty::BackfaceVisibility => "visible",
      CssProperty::Background => "see individual properties",
      CssProperty::BackgroundAttachment => "scroll",
      CssProperty::BackgroundBlendMode => "normal",
      CssProperty::BackgroundClip => "border-box",
      CssProperty::BackgroundColor => "transparent",
      CssProperty::BackgroundImage => "none",
      CssProperty::BackgroundOrigin => "padding-box",
      CssProperty::BackgroundPosition => "0% 0%",
      CssProperty::BackgroundPositionBlock => "0%",
      CssProperty::BackgroundPositionInline => "0%",
      CssProperty::BackgroundPositionX => "0%",
      CssProperty::BackgroundPositionY => "0%",
      CssProperty::BackgroundRepeat => "repeat",
      CssProperty::BackgroundRepeatBlock => "repeat",
      CssProperty::BackgroundRepeatInline => "repeat",
      CssProperty::BackgroundRepeatX => "repeat",
      CssProperty::BackgroundRepeatY => "repeat",
      CssProperty::BackgroundSize => "auto",
      CssProperty::BackgroundTbd => "see individual properties",
      CssProperty::BaselineShift => "0",
      CssProperty::BaselineSource => "auto",
      CssProperty::BlockEllipsis => "no-ellipsis",
      CssProperty::BlockSize => "auto",
      CssProperty::BlockStep => "see individual properties",
      CssProperty::BlockStepAlign => "auto",
      CssProperty::BlockStepInsert => "margin-box",
      CssProperty::BlockStepRound => "up",
      CssProperty::BlockStepSize => "none",
      CssProperty::BookmarkLabel => "content(text)",
      CssProperty::BookmarkLevel => "none",
      CssProperty::BookmarkState => "open",
      CssProperty::Border => "see individual properties",
      CssProperty::BorderBlock => "see individual properties",
      CssProperty::BorderBlockClip => "see individual properties",
      CssProperty::BorderBlockColor => "see individual properties",
      CssProperty::BorderBlockEnd => "See individual properties",
      CssProperty::BorderBlockEndClip => "none",
      CssProperty::BorderBlockEndColor => "currentcolor",
      CssProperty::BorderBlockEndRadius => "0",
      CssProperty::BorderBlockEndStyle => "none",
      CssProperty::BorderBlockEndWidth => "medium",
      CssProperty::BorderBlockStart => "See individual properties",
      CssProperty::BorderBlockStartClip => "none",
      CssProperty::BorderBlockStartColor => "currentcolor",
      CssProperty::BorderBlockStartRadius => "0",
      CssProperty::BorderBlockStartStyle => "none",
      CssProperty::BorderBlockStartWidth => "medium",
      CssProperty::BorderBlockStyle => "see individual properties",
      CssProperty::BorderBlockWidth => "see individual properties",
      CssProperty::BorderBottom => "See individual properties",
      CssProperty::BorderBottomClip => "none",
      CssProperty::BorderBottomColor => "currentcolor",
      CssProperty::BorderBottomLeftRadius => "0",
      CssProperty::BorderBottomRadius => "0",
      CssProperty::BorderBottomRightRadius => "0",
      CssProperty::BorderBottomStyle => "none",
      CssProperty::BorderBottomWidth => "medium",
      CssProperty::BorderBoundary => "none",
      CssProperty::BorderClip => "see individual properties",
      CssProperty::BorderCollapse => "separate",
      CssProperty::BorderColor => "see individual properties",
      CssProperty::BorderEndEndRadius => "0",
      CssProperty::BorderEndStartRadius => "0",
      CssProperty::BorderImage => "See individual properties",
      CssProperty::BorderImageOutset => "0",
      CssProperty::BorderImageRepeat => "stretch",
      CssProperty::BorderImageSlice => "100%",
      CssProperty::BorderImageSource => "none",
      CssProperty::BorderImageWidth => "1",
      CssProperty::BorderInline => "see individual properties",
      CssProperty::BorderInlineClip => "see individual properties",
      CssProperty::BorderInlineColor => "see individual properties",
      CssProperty::BorderInlineEnd => "See individual properties",
      CssProperty::BorderInlineEndClip => "none",
      CssProperty::BorderInlineEndColor => "currentcolor",
      CssProperty::BorderInlineEndRadius => "0",
      CssProperty::BorderInlineEndStyle => "none",
      CssProperty::BorderInlineEndWidth => "medium",
      CssProperty::BorderInlineStart => "See individual properties",
      CssProperty::BorderInlineStartClip => "none",
      CssProperty::BorderInlineStartColor => "currentcolor",
      CssProperty::BorderInlineStartRadius => "0",
      CssProperty::BorderInlineStartStyle => "none",
      CssProperty::BorderInlineStartWidth => "medium",
      CssProperty::BorderInlineStyle => "see individual properties",
      CssProperty::BorderInlineWidth => "see individual properties",
      CssProperty::BorderLeft => "See individual properties",
      CssProperty::BorderLeftClip => "none",
      CssProperty::BorderLeftColor => "currentcolor",
      CssProperty::BorderLeftRadius => "0",
      CssProperty::BorderLeftStyle => "none",
      CssProperty::BorderLeftWidth => "medium",
      CssProperty::BorderLimit => "all",
      CssProperty::BorderRadius => "see individual properties",
      CssProperty::BorderRight => "See individual properties",
      CssProperty::BorderRightClip => "none",
      CssProperty::BorderRightColor => "currentcolor",
      CssProperty::BorderRightRadius => "0",
      CssProperty::BorderRightStyle => "none",
      CssProperty::BorderRightWidth => "medium",
      CssProperty::BorderShape => "none",
      CssProperty::BorderSpacing => "0px 0px",
      CssProperty::BorderStartEndRadius => "0",
      CssProperty::BorderStartStartRadius => "0",
      CssProperty::BorderStyle => "see individual properties",
      CssProperty::BorderTop => "See individual properties",
      CssProperty::BorderTopClip => "none",
      CssProperty::BorderTopColor => "currentcolor",
      CssProperty::BorderTopLeftRadius => "0",
      CssProperty::BorderTopRadius => "0",
      CssProperty::BorderTopRightRadius => "0",
      CssProperty::BorderTopStyle => "none",
      CssProperty::BorderTopWidth => "medium",
      CssProperty::BorderWidth => "see individual properties",
      CssProperty::Bottom => "auto",
      CssProperty::BoxDecorationBreak => "slice",
      CssProperty::BoxShadow => "none",
      CssProperty::BoxShadowBlur => "0",
      CssProperty::BoxShadowColor => "currentcolor",
      CssProperty::BoxShadowOffset => "none",
      CssProperty::BoxShadowPosition => "outset",
      CssProperty::BoxShadowSpread => "0",
      CssProperty::BoxSizing => "content-box",
      CssProperty::BoxSnap => "none",
      CssProperty::BreakAfter => "auto",
      CssProperty::BreakBefore => "auto",
      CssProperty::BreakInside => "auto",
      CssProperty::CaptionSide => "top",
      CssProperty::Caret => "auto",
      CssProperty::CaretAnimation => "auto",
      CssProperty::CaretColor => "auto",
      CssProperty::CaretShape => "auto",
      CssProperty::Clear => "none",
      CssProperty::Clip => "auto",
      CssProperty::ClipPath => "none",
      CssProperty::ClipRule => "nonzero",
      CssProperty::Color => "CanvasText",
      CssProperty::ColorAdjust => "see individual properties",
      CssProperty::ColorInterpolation => "sRGB",
      CssProperty::ColorInterpolationFilters => "linearRGB",
      CssProperty::ColorScheme => "normal",
      CssProperty::ColumnCount => "auto",
      CssProperty::ColumnFill => "balance",
      CssProperty::ColumnGap => "normal",
      CssProperty::ColumnHeight => "auto",
      CssProperty::ColumnRule => "see individual properties",
      CssProperty::ColumnRuleBreak => "normal",
      CssProperty::ColumnRuleColor => "currentcolor",
      CssProperty::ColumnRuleInset => "see individual properties",
      CssProperty::ColumnRuleInsetCap => "see individual properties",
      CssProperty::ColumnRuleInsetCapEnd => "0",
      CssProperty::ColumnRuleInsetCapStart => "0",
      CssProperty::ColumnRuleInsetEnd => "see individual properties",
      CssProperty::ColumnRuleInsetJunction => "see individual properties",
      CssProperty::ColumnRuleInsetJunctionEnd => "0",
      CssProperty::ColumnRuleInsetJunctionStart => "0",
      CssProperty::ColumnRuleInsetStart => "see individual properties",
      CssProperty::ColumnRuleStyle => "none",
      CssProperty::ColumnRuleVisibilityItems => "normal",
      CssProperty::ColumnRuleWidth => "medium",
      CssProperty::ColumnSpan => "none",
      CssProperty::ColumnWidth => "auto",
      CssProperty::ColumnWrap => "auto",
      CssProperty::Columns => "see individual properties",
      CssProperty::Contain => "none",
      CssProperty::ContainIntrinsicBlockSize => "none",
      CssProperty::ContainIntrinsicHeight => "none",
      CssProperty::ContainIntrinsicInlineSize => "none",
      CssProperty::ContainIntrinsicSize => "see individual properties",
      CssProperty::ContainIntrinsicWidth => "none",
      CssProperty::Container => "see individual properties",
      CssProperty::ContainerName => "none",
      CssProperty::ContainerType => "normal",
      CssProperty::Content => "normal",
      CssProperty::ContentVisibility => "visible",
      CssProperty::Continue => "auto",
      CssProperty::CopyInto => "none",
      CssProperty::Corner => "0",
      CssProperty::CornerBlockEnd => "0",
      CssProperty::CornerBlockEndShape => "see individual properties",
      CssProperty::CornerBlockStart => "0",
      CssProperty::CornerBlockStartShape => "see individual properties",
      CssProperty::CornerBottom => "0",
      CssProperty::CornerBottomLeft => "0",
      CssProperty::CornerBottomLeftShape => "round",
      CssProperty::CornerBottomRight => "0",
      CssProperty::CornerBottomRightShape => "round",
      CssProperty::CornerBottomShape => "see individual properties",
      CssProperty::CornerEndEnd => "0",
      CssProperty::CornerEndEndShape => "round",
      CssProperty::CornerEndStart => "0",
      CssProperty::CornerEndStartShape => "round",
      CssProperty::CornerInlineEnd => "0",
      CssProperty::CornerInlineEndShape => "see individual properties",
      CssProperty::CornerInlineStart => "0",
      CssProperty::CornerInlineStartShape => "see individual properties",
      CssProperty::CornerLeft => "0",
      CssProperty::CornerLeftShape => "see individual properties",
      CssProperty::CornerRight => "0",
      CssProperty::CornerRightShape => "see individual properties",
      CssProperty::CornerShape => "round",
      CssProperty::CornerStartEnd => "0",
      CssProperty::CornerStartEndShape => "round",
      CssProperty::CornerStartStart => "0",
      CssProperty::CornerStartStartShape => "round",
      CssProperty::CornerTop => "0",
      CssProperty::CornerTopLeft => "0",
      CssProperty::CornerTopLeftShape => "round",
      CssProperty::CornerTopRight => "0",
      CssProperty::CornerTopRightShape => "round",
      CssProperty::CornerTopShape => "see individual properties",
      CssProperty::CounterIncrement => "none",
      CssProperty::CounterReset => "none",
      CssProperty::CounterSet => "none",
      CssProperty::Cue => "see individual properties",
      CssProperty::CueAfter => "none",
      CssProperty::CueBefore => "none",
      CssProperty::Cursor => "auto",
      CssProperty::Cx => "0",
      CssProperty::Cy => "0",
      CssProperty::D => "none",
      CssProperty::Direction => "ltr",
      CssProperty::Display => "inline",
      CssProperty::DominantBaseline => "auto",
      CssProperty::DynamicRangeLimit => "no-limit",
      CssProperty::EmptyCells => "show",
      CssProperty::EventTrigger => "none",
      CssProperty::EventTriggerName => "none",
      CssProperty::EventTriggerSource => "none",
      CssProperty::FieldSizing => "fixed",
      CssProperty::Fill => "black",
      CssProperty::FillBreak => "bounding-box",
      CssProperty::FillColor => "currentcolor",
      CssProperty::FillImage => "none",
      CssProperty::FillOpacity => "1",
      CssProperty::FillOrigin => "match-parent",
      CssProperty::FillPosition => "0% 0%",
      CssProperty::FillRepeat => "repeat",
      CssProperty::FillRule => "nonzero",
      CssProperty::FillSize => "auto",
      CssProperty::Filter => "none",
      CssProperty::Flex => "0 1 auto",
      CssProperty::FlexBasis => "auto",
      CssProperty::FlexDirection => "row",
      CssProperty::FlexFlow => "see individual properties",
      CssProperty::FlexGrow => "0",
      CssProperty::FlexShrink => "1",
      CssProperty::FlexWrap => "nowrap",
      CssProperty::Float => "none",
      CssProperty::FloatDefer => "none",
      CssProperty::FloatOffset => "0",
      CssProperty::FloatReference => "inline",
      CssProperty::FloodColor => "black",
      CssProperty::FloodOpacity => "1",
      CssProperty::FlowFrom => "none",
      CssProperty::FlowInto => "none",
      CssProperty::FlowTolerance => "normal",
      CssProperty::Font => "see individual properties",
      CssProperty::FontFamily => "depends on user agent",
      CssProperty::FontFeatureSettings => "normal",
      CssProperty::FontKerning => "auto",
      CssProperty::FontLanguageOverride => "normal",
      CssProperty::FontOpticalSizing => "auto",
      CssProperty::FontPalette => "normal",
      CssProperty::FontSize => "medium",
      CssProperty::FontSizeAdjust => "none",
      CssProperty::FontStretch => "",
      CssProperty::FontStyle => "normal",
      CssProperty::FontSynthesis => "weight style small-caps position",
      CssProperty::FontSynthesisPosition => "auto",
      CssProperty::FontSynthesisSmallCaps => "auto",
      CssProperty::FontSynthesisStyle => "auto",
      CssProperty::FontSynthesisWeight => "auto",
      CssProperty::FontVariant => "normal",
      CssProperty::FontVariantAlternates => "normal",
      CssProperty::FontVariantCaps => "normal",
      CssProperty::FontVariantEastAsian => "normal",
      CssProperty::FontVariantEmoji => "normal",
      CssProperty::FontVariantLigatures => "normal",
      CssProperty::FontVariantNumeric => "normal",
      CssProperty::FontVariantPosition => "normal",
      CssProperty::FontVariationSettings => "normal",
      CssProperty::FontWeight => "normal",
      CssProperty::FontWidth => "normal",
      CssProperty::FootnoteDisplay => "block",
      CssProperty::FootnotePolicy => "auto",
      CssProperty::ForcedColorAdjust => "auto",
      CssProperty::FrameSizing => "auto",
      CssProperty::Gap => "see individual properties",
      CssProperty::GlyphOrientationVertical => "n/a",
      CssProperty::Grid => "none",
      CssProperty::GridArea => "auto",
      CssProperty::GridAutoColumns => "auto",
      CssProperty::GridAutoFlow => "row",
      CssProperty::GridAutoRows => "auto",
      CssProperty::GridColumn => "auto",
      CssProperty::GridColumnEnd => "auto",
      CssProperty::GridColumnGap => "",
      CssProperty::GridColumnStart => "auto",
      CssProperty::GridGap => "",
      CssProperty::GridRow => "auto",
      CssProperty::GridRowEnd => "auto",
      CssProperty::GridRowGap => "",
      CssProperty::GridRowStart => "auto",
      CssProperty::GridTemplate => "none",
      CssProperty::GridTemplateAreas => "none",
      CssProperty::GridTemplateColumns => "none",
      CssProperty::GridTemplateRows => "none",
      CssProperty::HangingPunctuation => "none",
      CssProperty::Height => "auto",
      CssProperty::HyphenateCharacter => "auto",
      CssProperty::HyphenateLimitChars => "auto",
      CssProperty::HyphenateLimitLast => "none",
      CssProperty::HyphenateLimitLines => "no-limit",
      CssProperty::HyphenateLimitZone => "0",
      CssProperty::Hyphens => "manual",
      CssProperty::ImageAnimation => "normal",
      CssProperty::ImageOrientation => "from-image",
      CssProperty::ImageRendering => "auto",
      CssProperty::ImageResolution => "1dppx",
      CssProperty::InitialLetter => "normal",
      CssProperty::InitialLetterAlign => "alphabetic",
      CssProperty::InitialLetterWrap => "none",
      CssProperty::InlineSize => "auto",
      CssProperty::InlineSizing => "normal",
      CssProperty::InputSecurity => "auto",
      CssProperty::Inset => "auto",
      CssProperty::InsetBlock => "auto",
      CssProperty::InsetBlockEnd => "auto",
      CssProperty::InsetBlockStart => "auto",
      CssProperty::InsetInline => "auto",
      CssProperty::InsetInlineEnd => "auto",
      CssProperty::InsetInlineStart => "auto",
      CssProperty::Interactivity => "auto",
      CssProperty::InterestDelay => "see individual properties",
      CssProperty::InterestDelayEnd => "normal",
      CssProperty::InterestDelayStart => "normal",
      CssProperty::InterpolateSize => "numeric-only",
      CssProperty::Isolation => "auto",
      CssProperty::JustifyContent => "normal",
      CssProperty::JustifyItems => "legacy",
      CssProperty::JustifySelf => "auto",
      CssProperty::Left => "auto",
      CssProperty::LetterSpacing => "normal",
      CssProperty::LightingColor => "white",
      CssProperty::LineBreak => "auto",
      CssProperty::LineClamp => "none",
      CssProperty::LineFitEdge => "leading",
      CssProperty::LineGrid => "match-parent",
      CssProperty::LineHeight => "normal",
      CssProperty::LineHeightStep => "0",
      CssProperty::LinePadding => "0",
      CssProperty::LineSnap => "none",
      CssProperty::LinkParameters => "none",
      CssProperty::ListStyle => "see individual properties",
      CssProperty::ListStyleImage => "none",
      CssProperty::ListStylePosition => "outside",
      CssProperty::ListStyleType => "disc",
      CssProperty::Margin => "0",
      CssProperty::MarginBlock => "see individual properties",
      CssProperty::MarginBlockEnd => "0",
      CssProperty::MarginBlockStart => "0",
      CssProperty::MarginBottom => "0",
      CssProperty::MarginBreak => "auto",
      CssProperty::MarginInline => "see individual properties",
      CssProperty::MarginInlineEnd => "0",
      CssProperty::MarginInlineStart => "0",
      CssProperty::MarginLeft => "0",
      CssProperty::MarginRight => "0",
      CssProperty::MarginTop => "0",
      CssProperty::MarginTrim => "none",
      CssProperty::Marker => "not defined for shorthand properties",
      CssProperty::MarkerEnd => "none",
      CssProperty::MarkerMid => "none",
      CssProperty::MarkerSide => "match-self",
      CssProperty::MarkerStart => "none",
      CssProperty::Mask => "see individual properties",
      CssProperty::MaskBorder => "See individual properties",
      CssProperty::MaskBorderMode => "alpha",
      CssProperty::MaskBorderOutset => "0",
      CssProperty::MaskBorderRepeat => "stretch",
      CssProperty::MaskBorderSlice => "0",
      CssProperty::MaskBorderSource => "none",
      CssProperty::MaskBorderWidth => "auto",
      CssProperty::MaskClip => "border-box",
      CssProperty::MaskComposite => "add",
      CssProperty::MaskImage => "none",
      CssProperty::MaskMode => "match-source",
      CssProperty::MaskOrigin => "border-box",
      CssProperty::MaskPosition => "0% 0%",
      CssProperty::MaskRepeat => "repeat",
      CssProperty::MaskSize => "auto",
      CssProperty::MaskType => "luminance",
      CssProperty::MathDepth => "0",
      CssProperty::MathShift => "normal",
      CssProperty::MathStyle => "normal",
      CssProperty::MaxBlockSize => "none",
      CssProperty::MaxHeight => "none",
      CssProperty::MaxInlineSize => "none",
      CssProperty::MaxLines => "none",
      CssProperty::MaxWidth => "none",
      CssProperty::MinBlockSize => "0",
      CssProperty::MinHeight => "auto",
      CssProperty::MinInlineSize => "0",
      CssProperty::MinIntrinsicSizing => "legacy",
      CssProperty::MinWidth => "auto",
      CssProperty::MixBlendMode => "normal",
      CssProperty::NavDown => "auto",
      CssProperty::NavLeft => "auto",
      CssProperty::NavRight => "auto",
      CssProperty::NavUp => "auto",
      CssProperty::ObjectFit => "fill",
      CssProperty::ObjectPosition => "50% 50%",
      CssProperty::ObjectViewBox => "none",
      CssProperty::Offset => "see individual properties",
      CssProperty::OffsetAnchor => "auto",
      CssProperty::OffsetDistance => "0",
      CssProperty::OffsetPath => "none",
      CssProperty::OffsetPosition => "normal",
      CssProperty::OffsetRotate => "auto",
      CssProperty::Opacity => "1",
      CssProperty::Order => "0",
      CssProperty::Orphans => "2",
      CssProperty::Outline => "see individual properties",
      CssProperty::OutlineColor => "auto",
      CssProperty::OutlineOffset => "0",
      CssProperty::OutlineStyle => "none",
      CssProperty::OutlineWidth => "medium",
      CssProperty::Overflow => "visible",
      CssProperty::OverflowAnchor => "auto",
      CssProperty::OverflowBlock => "visible",
      CssProperty::OverflowClipMargin => "0px",
      CssProperty::OverflowClipMarginBlock => "0px",
      CssProperty::OverflowClipMarginBlockEnd => "0px",
      CssProperty::OverflowClipMarginBlockStart => "0px",
      CssProperty::OverflowClipMarginBottom => "0px",
      CssProperty::OverflowClipMarginInline => "0px",
      CssProperty::OverflowClipMarginInlineEnd => "0px",
      CssProperty::OverflowClipMarginInlineStart => "0px",
      CssProperty::OverflowClipMarginLeft => "0px",
      CssProperty::OverflowClipMarginRight => "0px",
      CssProperty::OverflowClipMarginTop => "0px",
      CssProperty::OverflowInline => "visible",
      CssProperty::OverflowWrap => "normal",
      CssProperty::OverflowX => "visible",
      CssProperty::OverflowY => "visible",
      CssProperty::Overlay => "none",
      CssProperty::OverscrollBehavior => "auto auto",
      CssProperty::OverscrollBehaviorBlock => "auto",
      CssProperty::OverscrollBehaviorInline => "auto",
      CssProperty::OverscrollBehaviorX => "auto",
      CssProperty::OverscrollBehaviorY => "auto",
      CssProperty::Padding => "0",
      CssProperty::PaddingBlock => "see individual properties",
      CssProperty::PaddingBlockEnd => "0",
      CssProperty::PaddingBlockStart => "0",
      CssProperty::PaddingBottom => "0",
      CssProperty::PaddingInline => "see individual properties",
      CssProperty::PaddingInlineEnd => "0",
      CssProperty::PaddingInlineStart => "0",
      CssProperty::PaddingLeft => "0",
      CssProperty::PaddingRight => "0",
      CssProperty::PaddingTop => "0",
      CssProperty::Page => "auto",
      CssProperty::PageBreakAfter => "auto",
      CssProperty::PageBreakBefore => "auto",
      CssProperty::PageBreakInside => "auto",
      CssProperty::PaintOrder => "normal",
      CssProperty::PathLength => "none",
      CssProperty::Pause => "see individual properties",
      CssProperty::PauseAfter => "none",
      CssProperty::PauseBefore => "none",
      CssProperty::Perspective => "none",
      CssProperty::PerspectiveOrigin => "50% 50%",
      CssProperty::PlaceContent => "normal",
      CssProperty::PlaceItems => "see individual properties",
      CssProperty::PlaceSelf => "auto",
      CssProperty::PointerEvents => "auto",
      CssProperty::PointerTimeline => "see individual properties",
      CssProperty::PointerTimelineAxis => "block",
      CssProperty::PointerTimelineName => "none",
      CssProperty::Position => "static",
      CssProperty::PositionAnchor => "normal",
      CssProperty::PositionArea => "none",
      CssProperty::PositionTry => "see individual properties",
      CssProperty::PositionTryFallbacks => "none",
      CssProperty::PositionTryOrder => "normal",
      CssProperty::PositionVisibility => "anchors-visible",
      CssProperty::PrintColorAdjust => "economy",
      CssProperty::Quotes => "auto",
      CssProperty::R => "0",
      CssProperty::ReadingFlow => "normal",
      CssProperty::ReadingOrder => "0",
      CssProperty::RegionFragment => "auto",
      CssProperty::Resize => "none",
      CssProperty::Rest => "see individual properties",
      CssProperty::RestAfter => "none",
      CssProperty::RestBefore => "none",
      CssProperty::Right => "auto",
      CssProperty::Rotate => "none",
      CssProperty::RowGap => "normal",
      CssProperty::RowRule => "see individual properties",
      CssProperty::RowRuleBreak => "normal",
      CssProperty::RowRuleColor => "currentcolor",
      CssProperty::RowRuleInset => "see individual properties",
      CssProperty::RowRuleInsetCap => "see individual properties",
      CssProperty::RowRuleInsetCapEnd => "0",
      CssProperty::RowRuleInsetCapStart => "0",
      CssProperty::RowRuleInsetEnd => "see individual properties",
      CssProperty::RowRuleInsetJunction => "see individual properties",
      CssProperty::RowRuleInsetJunctionEnd => "0",
      CssProperty::RowRuleInsetJunctionStart => "0",
      CssProperty::RowRuleInsetStart => "see individual properties",
      CssProperty::RowRuleStyle => "none",
      CssProperty::RowRuleVisibilityItems => "normal",
      CssProperty::RowRuleWidth => "medium",
      CssProperty::RubyAlign => "space-around",
      CssProperty::RubyMerge => "separate",
      CssProperty::RubyOverhang => "auto",
      CssProperty::RubyPosition => "alternate",
      CssProperty::Rule => "see individual properties",
      CssProperty::RuleBreak => "see individual properties",
      CssProperty::RuleColor => "see individual properties",
      CssProperty::RuleInset => "see individual properties",
      CssProperty::RuleInsetCap => "see individual properties",
      CssProperty::RuleInsetEnd => "see individual properties",
      CssProperty::RuleInsetJunction => "see individual properties",
      CssProperty::RuleInsetStart => "see individual properties",
      CssProperty::RuleOverlap => "row-over-column",
      CssProperty::RuleStyle => "see individual properties",
      CssProperty::RuleVisibilityItems => "see individual properties",
      CssProperty::RuleWidth => "see individual properties",
      CssProperty::Rx => "auto",
      CssProperty::Ry => "auto",
      CssProperty::Scale => "none",
      CssProperty::ScrollBehavior => "auto",
      CssProperty::ScrollInitialTarget => "none",
      CssProperty::ScrollMargin => "0",
      CssProperty::ScrollMarginBlock => "0",
      CssProperty::ScrollMarginBlockEnd => "0",
      CssProperty::ScrollMarginBlockStart => "0",
      CssProperty::ScrollMarginBottom => "0",
      CssProperty::ScrollMarginInline => "0",
      CssProperty::ScrollMarginInlineEnd => "0",
      CssProperty::ScrollMarginInlineStart => "0",
      CssProperty::ScrollMarginLeft => "0",
      CssProperty::ScrollMarginRight => "0",
      CssProperty::ScrollMarginTop => "0",
      CssProperty::ScrollMarkerGroup => "none",
      CssProperty::ScrollPadding => "auto",
      CssProperty::ScrollPaddingBlock => "auto",
      CssProperty::ScrollPaddingBlockEnd => "auto",
      CssProperty::ScrollPaddingBlockStart => "auto",
      CssProperty::ScrollPaddingBottom => "auto",
      CssProperty::ScrollPaddingInline => "auto",
      CssProperty::ScrollPaddingInlineEnd => "auto",
      CssProperty::ScrollPaddingInlineStart => "auto",
      CssProperty::ScrollPaddingLeft => "auto",
      CssProperty::ScrollPaddingRight => "auto",
      CssProperty::ScrollPaddingTop => "auto",
      CssProperty::ScrollSnapAlign => "none",
      CssProperty::ScrollSnapStop => "normal",
      CssProperty::ScrollSnapType => "none",
      CssProperty::ScrollTargetGroup => "none",
      CssProperty::ScrollTimeline => "see individual properties",
      CssProperty::ScrollTimelineAxis => "block",
      CssProperty::ScrollTimelineName => "none",
      CssProperty::ScrollbarColor => "auto",
      CssProperty::ScrollbarGutter => "auto",
      CssProperty::ScrollbarInset => "0px",
      CssProperty::ScrollbarMinThumbSize => "20px",
      CssProperty::ScrollbarMode => "auto",
      CssProperty::ScrollbarWidth => "auto",
      CssProperty::ShapeImageThreshold => "0",
      CssProperty::ShapeInside => "auto",
      CssProperty::ShapeMargin => "0",
      CssProperty::ShapeOutside => "none",
      CssProperty::ShapePadding => "0",
      CssProperty::ShapeRendering => "auto",
      CssProperty::SliderOrientation => "auto",
      CssProperty::SpatialNavigationAction => "auto",
      CssProperty::SpatialNavigationContain => "auto",
      CssProperty::SpatialNavigationFunction => "normal",
      CssProperty::Speak => "auto",
      CssProperty::SpeakAs => "normal",
      CssProperty::StopColor => "",
      CssProperty::StopOpacity => "",
      CssProperty::StringSet => "none",
      CssProperty::Stroke => "none",
      CssProperty::StrokeAlign => "center",
      CssProperty::StrokeAlignment => "center",
      CssProperty::StrokeBreak => "bounding-box",
      CssProperty::StrokeColor => "transparent",
      CssProperty::StrokeDashCornerPropdefStrokeDashCorner => "none",
      CssProperty::StrokeDashJustify => "none",
      CssProperty::StrokeDashadjust => "none",
      CssProperty::StrokeDasharray => "none",
      CssProperty::StrokeDashcornerStrokedashcornerproperty => "none",
      CssProperty::StrokeDashoffset => "0",
      CssProperty::StrokeImage => "none",
      CssProperty::StrokeLinecap => "butt",
      CssProperty::StrokeLinejoin => "miter",
      CssProperty::StrokeMiterlimit => "4",
      CssProperty::StrokeOpacity => "1",
      CssProperty::StrokeOrigin => "match-parent",
      CssProperty::StrokePosition => "0% 0%",
      CssProperty::StrokeRepeat => "repeat",
      CssProperty::StrokeSize => "auto",
      CssProperty::StrokeWidth => "1px",
      CssProperty::TabSize => "8",
      CssProperty::TableLayout => "auto",
      CssProperty::TextAlign => "start",
      CssProperty::TextAlignAll => "start",
      CssProperty::TextAlignLast => "auto",
      CssProperty::TextAnchor => "start",
      CssProperty::TextAutospace => "normal",
      CssProperty::TextBox => "normal",
      CssProperty::TextBoxEdge => "auto",
      CssProperty::TextBoxTrim => "none",
      CssProperty::TextCombineUpright => "none",
      CssProperty::TextDecoration => "see individual properties",
      CssProperty::TextDecorationColor => "currentcolor",
      CssProperty::TextDecorationInset => "0",
      CssProperty::TextDecorationLine => "none",
      CssProperty::TextDecorationSkip => "See individual properties",
      CssProperty::TextDecorationSkipBox => "none",
      CssProperty::TextDecorationSkipInk => "auto",
      CssProperty::TextDecorationSkipSelf => "auto",
      CssProperty::TextDecorationSkipSpaces => "start end",
      CssProperty::TextDecorationStyle => "solid",
      CssProperty::TextDecorationThickness => "auto",
      CssProperty::TextEmphasis => "see individual properties",
      CssProperty::TextEmphasisColor => "currentcolor",
      CssProperty::TextEmphasisPosition => "over right",
      CssProperty::TextEmphasisSkip => "spaces punctuation",
      CssProperty::TextEmphasisStyle => "none",
      CssProperty::TextFit => "none",
      CssProperty::TextGroupAlign => "none",
      CssProperty::TextIndent => "0",
      CssProperty::TextJustify => "auto",
      CssProperty::TextOrientation => "mixed",
      CssProperty::TextOverflow => "clip",
      CssProperty::TextRendering => "auto",
      CssProperty::TextShadow => "none",
      CssProperty::TextSizeAdjust => "auto",
      CssProperty::TextSpacing => "see individual properties",
      CssProperty::TextSpacingTrim => "normal",
      CssProperty::TextTransform => "none",
      CssProperty::TextUnderlineOffset => "auto",
      CssProperty::TextUnderlinePosition => "auto",
      CssProperty::TextWrap => "wrap",
      CssProperty::TextWrapMode => "wrap",
      CssProperty::TextWrapStyle => "auto",
      CssProperty::TimelineScope => "none",
      CssProperty::TimelineTrigger => "see individual properties",
      CssProperty::TimelineTriggerActivationRange => "see individual properties",
      CssProperty::TimelineTriggerActivationRangeEnd => "normal",
      CssProperty::TimelineTriggerActivationRangeStart => "normal",
      CssProperty::TimelineTriggerActiveRange => "see individual properties",
      CssProperty::TimelineTriggerActiveRangeEnd => "auto",
      CssProperty::TimelineTriggerActiveRangeStart => "auto",
      CssProperty::TimelineTriggerName => "none",
      CssProperty::TimelineTriggerSource => "auto",
      CssProperty::Top => "auto",
      CssProperty::TouchAction => "auto",
      CssProperty::Transform => "none",
      CssProperty::TransformBox => "view-box",
      CssProperty::TransformOrigin => "50% 50%",
      CssProperty::TransformStyle => "flat",
      CssProperty::Transition => "see individual properties",
      CssProperty::TransitionBehavior => "normal",
      CssProperty::TransitionDelay => "0s",
      CssProperty::TransitionDuration => "0s",
      CssProperty::TransitionProperty => "all",
      CssProperty::TransitionTimingFunction => "ease",
      CssProperty::Translate => "none",
      CssProperty::TriggerScope => "none",
      CssProperty::UnicodeBidi => "normal",
      CssProperty::UserSelect => "auto",
      CssProperty::VectorEffect => "none",
      CssProperty::VerticalAlign => "baseline",
      CssProperty::ViewTimeline => "see individual properties",
      CssProperty::ViewTimelineAxis => "block",
      CssProperty::ViewTimelineInset => "auto",
      CssProperty::ViewTimelineName => "none",
      CssProperty::ViewTransitionClass => "none",
      CssProperty::ViewTransitionGroup => "normal",
      CssProperty::ViewTransitionName => "none",
      CssProperty::ViewTransitionScope => "none",
      CssProperty::Visibility => "visible",
      CssProperty::VoiceBalance => "center",
      CssProperty::VoiceDuration => "auto",
      CssProperty::VoiceFamily => "implementation-dependent",
      CssProperty::VoicePitch => "medium",
      CssProperty::VoiceRange => "medium",
      CssProperty::VoiceRate => "normal",
      CssProperty::VoiceStress => "normal",
      CssProperty::VoiceVolume => "medium",
      CssProperty::WhiteSpace => "normal",
      CssProperty::WhiteSpaceCollapse => "collapse",
      CssProperty::WhiteSpaceTrim => "none",
      CssProperty::Widows => "2",
      CssProperty::Width => "auto",
      CssProperty::WillChange => "auto",
      CssProperty::WordBreak => "normal",
      CssProperty::WordSpaceTransform => "none",
      CssProperty::WordSpacing => "normal",
      CssProperty::WordWrap => "normal",
      CssProperty::WrapAfter => "auto",
      CssProperty::WrapBefore => "auto",
      CssProperty::WrapFlow => "auto",
      CssProperty::WrapInside => "auto",
      CssProperty::WrapThrough => "wrap",
      CssProperty::WritingMode => "horizontal-tb",
      CssProperty::X => "0",
      CssProperty::Y => "0",
      CssProperty::ZIndex => "auto",
      CssProperty::Zoom => "1",
      CssProperty::Unknown(_) => "",
    }
  }

  pub fn inherited(self) -> bool {
    match self {
      CssProperty::WebkitAlignContent => false,
      CssProperty::WebkitAlignItems => false,
      CssProperty::WebkitAlignSelf => false,
      CssProperty::WebkitAnimation => false,
      CssProperty::WebkitAnimationDelay => false,
      CssProperty::WebkitAnimationDirection => false,
      CssProperty::WebkitAnimationDuration => false,
      CssProperty::WebkitAnimationFillMode => false,
      CssProperty::WebkitAnimationIterationCount => false,
      CssProperty::WebkitAnimationName => false,
      CssProperty::WebkitAnimationPlayState => false,
      CssProperty::WebkitAnimationTimingFunction => false,
      CssProperty::WebkitAppearance => false,
      CssProperty::WebkitBackfaceVisibility => false,
      CssProperty::WebkitBackgroundClip => false,
      CssProperty::WebkitBackgroundOrigin => false,
      CssProperty::WebkitBackgroundSize => false,
      CssProperty::WebkitBorderBottomLeftRadius => false,
      CssProperty::WebkitBorderBottomRightRadius => false,
      CssProperty::WebkitBorderRadius => false,
      CssProperty::WebkitBorderTopLeftRadius => false,
      CssProperty::WebkitBorderTopRightRadius => false,
      CssProperty::WebkitBoxAlign => false,
      CssProperty::WebkitBoxFlex => false,
      CssProperty::WebkitBoxOrdinalGroup => false,
      CssProperty::WebkitBoxOrient => false,
      CssProperty::WebkitBoxPack => false,
      CssProperty::WebkitBoxShadow => false,
      CssProperty::WebkitBoxSizing => false,
      CssProperty::WebkitFilter => false,
      CssProperty::WebkitFlex => false,
      CssProperty::WebkitFlexBasis => false,
      CssProperty::WebkitFlexDirection => false,
      CssProperty::WebkitFlexFlow => false,
      CssProperty::WebkitFlexGrow => false,
      CssProperty::WebkitFlexShrink => false,
      CssProperty::WebkitFlexWrap => false,
      CssProperty::WebkitJustifyContent => false,
      CssProperty::WebkitLineClamp => false,
      CssProperty::WebkitMask => false,
      CssProperty::WebkitMaskBoxImage => false,
      CssProperty::WebkitMaskBoxImageOutset => false,
      CssProperty::WebkitMaskBoxImageRepeat => false,
      CssProperty::WebkitMaskBoxImageSlice => false,
      CssProperty::WebkitMaskBoxImageSource => false,
      CssProperty::WebkitMaskBoxImageWidth => false,
      CssProperty::WebkitMaskClip => false,
      CssProperty::WebkitMaskComposite => false,
      CssProperty::WebkitMaskImage => false,
      CssProperty::WebkitMaskOrigin => false,
      CssProperty::WebkitMaskPosition => false,
      CssProperty::WebkitMaskRepeat => false,
      CssProperty::WebkitMaskSize => false,
      CssProperty::WebkitOrder => false,
      CssProperty::WebkitPerspective => false,
      CssProperty::WebkitPerspectiveOrigin => false,
      CssProperty::WebkitTextFillColor => true,
      CssProperty::WebkitTextSizeAdjust => false,
      CssProperty::WebkitTextStroke => true,
      CssProperty::WebkitTextStrokeColor => true,
      CssProperty::WebkitTextStrokeWidth => true,
      CssProperty::WebkitTransform => false,
      CssProperty::WebkitTransformOrigin => false,
      CssProperty::WebkitTransformStyle => false,
      CssProperty::WebkitTransition => false,
      CssProperty::WebkitTransitionDelay => false,
      CssProperty::WebkitTransitionDuration => false,
      CssProperty::WebkitTransitionProperty => false,
      CssProperty::WebkitTransitionTimingFunction => false,
      CssProperty::WebkitUserSelect => false,
      CssProperty::AccentColor => true,
      CssProperty::AlignContent => false,
      CssProperty::AlignItems => false,
      CssProperty::AlignSelf => false,
      CssProperty::AlignmentBaseline => false,
      CssProperty::All => false,
      CssProperty::AnchorName => false,
      CssProperty::AnchorScope => false,
      CssProperty::Animation => false,
      CssProperty::AnimationComposition => false,
      CssProperty::AnimationDelay => false,
      CssProperty::AnimationDirection => false,
      CssProperty::AnimationDuration => false,
      CssProperty::AnimationFillMode => false,
      CssProperty::AnimationIterationCount => false,
      CssProperty::AnimationName => false,
      CssProperty::AnimationPlayState => false,
      CssProperty::AnimationRange => false,
      CssProperty::AnimationRangeCenter => false,
      CssProperty::AnimationRangeEnd => false,
      CssProperty::AnimationRangeStart => false,
      CssProperty::AnimationTimeline => false,
      CssProperty::AnimationTimingFunction => false,
      CssProperty::AnimationTrigger => false,
      CssProperty::Appearance => false,
      CssProperty::AspectRatio => false,
      CssProperty::BackdropFilter => false,
      CssProperty::BackfaceVisibility => false,
      CssProperty::Background => false,
      CssProperty::BackgroundAttachment => false,
      CssProperty::BackgroundBlendMode => false,
      CssProperty::BackgroundClip => false,
      CssProperty::BackgroundColor => false,
      CssProperty::BackgroundImage => false,
      CssProperty::BackgroundOrigin => false,
      CssProperty::BackgroundPosition => false,
      CssProperty::BackgroundPositionBlock => false,
      CssProperty::BackgroundPositionInline => false,
      CssProperty::BackgroundPositionX => false,
      CssProperty::BackgroundPositionY => false,
      CssProperty::BackgroundRepeat => false,
      CssProperty::BackgroundRepeatBlock => false,
      CssProperty::BackgroundRepeatInline => false,
      CssProperty::BackgroundRepeatX => false,
      CssProperty::BackgroundRepeatY => false,
      CssProperty::BackgroundSize => false,
      CssProperty::BackgroundTbd => false,
      CssProperty::BaselineShift => false,
      CssProperty::BaselineSource => false,
      CssProperty::BlockEllipsis => true,
      CssProperty::BlockSize => false,
      CssProperty::BlockStep => false,
      CssProperty::BlockStepAlign => false,
      CssProperty::BlockStepInsert => false,
      CssProperty::BlockStepRound => false,
      CssProperty::BlockStepSize => false,
      CssProperty::BookmarkLabel => false,
      CssProperty::BookmarkLevel => false,
      CssProperty::BookmarkState => false,
      CssProperty::Border => false,
      CssProperty::BorderBlock => false,
      CssProperty::BorderBlockClip => false,
      CssProperty::BorderBlockColor => false,
      CssProperty::BorderBlockEnd => false,
      CssProperty::BorderBlockEndClip => false,
      CssProperty::BorderBlockEndColor => false,
      CssProperty::BorderBlockEndRadius => false,
      CssProperty::BorderBlockEndStyle => false,
      CssProperty::BorderBlockEndWidth => false,
      CssProperty::BorderBlockStart => false,
      CssProperty::BorderBlockStartClip => false,
      CssProperty::BorderBlockStartColor => false,
      CssProperty::BorderBlockStartRadius => false,
      CssProperty::BorderBlockStartStyle => false,
      CssProperty::BorderBlockStartWidth => false,
      CssProperty::BorderBlockStyle => false,
      CssProperty::BorderBlockWidth => false,
      CssProperty::BorderBottom => false,
      CssProperty::BorderBottomClip => false,
      CssProperty::BorderBottomColor => false,
      CssProperty::BorderBottomLeftRadius => false,
      CssProperty::BorderBottomRadius => false,
      CssProperty::BorderBottomRightRadius => false,
      CssProperty::BorderBottomStyle => false,
      CssProperty::BorderBottomWidth => false,
      CssProperty::BorderBoundary => true,
      CssProperty::BorderClip => false,
      CssProperty::BorderCollapse => true,
      CssProperty::BorderColor => false,
      CssProperty::BorderEndEndRadius => false,
      CssProperty::BorderEndStartRadius => false,
      CssProperty::BorderImage => false,
      CssProperty::BorderImageOutset => false,
      CssProperty::BorderImageRepeat => false,
      CssProperty::BorderImageSlice => false,
      CssProperty::BorderImageSource => false,
      CssProperty::BorderImageWidth => false,
      CssProperty::BorderInline => false,
      CssProperty::BorderInlineClip => false,
      CssProperty::BorderInlineColor => false,
      CssProperty::BorderInlineEnd => false,
      CssProperty::BorderInlineEndClip => false,
      CssProperty::BorderInlineEndColor => false,
      CssProperty::BorderInlineEndRadius => false,
      CssProperty::BorderInlineEndStyle => false,
      CssProperty::BorderInlineEndWidth => false,
      CssProperty::BorderInlineStart => false,
      CssProperty::BorderInlineStartClip => false,
      CssProperty::BorderInlineStartColor => false,
      CssProperty::BorderInlineStartRadius => false,
      CssProperty::BorderInlineStartStyle => false,
      CssProperty::BorderInlineStartWidth => false,
      CssProperty::BorderInlineStyle => false,
      CssProperty::BorderInlineWidth => false,
      CssProperty::BorderLeft => false,
      CssProperty::BorderLeftClip => false,
      CssProperty::BorderLeftColor => false,
      CssProperty::BorderLeftRadius => false,
      CssProperty::BorderLeftStyle => false,
      CssProperty::BorderLeftWidth => false,
      CssProperty::BorderLimit => false,
      CssProperty::BorderRadius => false,
      CssProperty::BorderRight => false,
      CssProperty::BorderRightClip => false,
      CssProperty::BorderRightColor => false,
      CssProperty::BorderRightRadius => false,
      CssProperty::BorderRightStyle => false,
      CssProperty::BorderRightWidth => false,
      CssProperty::BorderShape => false,
      CssProperty::BorderSpacing => true,
      CssProperty::BorderStartEndRadius => false,
      CssProperty::BorderStartStartRadius => false,
      CssProperty::BorderStyle => false,
      CssProperty::BorderTop => false,
      CssProperty::BorderTopClip => false,
      CssProperty::BorderTopColor => false,
      CssProperty::BorderTopLeftRadius => false,
      CssProperty::BorderTopRadius => false,
      CssProperty::BorderTopRightRadius => false,
      CssProperty::BorderTopStyle => false,
      CssProperty::BorderTopWidth => false,
      CssProperty::BorderWidth => false,
      CssProperty::Bottom => false,
      CssProperty::BoxDecorationBreak => false,
      CssProperty::BoxShadow => false,
      CssProperty::BoxShadowBlur => false,
      CssProperty::BoxShadowColor => false,
      CssProperty::BoxShadowOffset => false,
      CssProperty::BoxShadowPosition => false,
      CssProperty::BoxShadowSpread => false,
      CssProperty::BoxSizing => false,
      CssProperty::BoxSnap => true,
      CssProperty::BreakAfter => false,
      CssProperty::BreakBefore => false,
      CssProperty::BreakInside => false,
      CssProperty::CaptionSide => true,
      CssProperty::Caret => true,
      CssProperty::CaretAnimation => true,
      CssProperty::CaretColor => true,
      CssProperty::CaretShape => true,
      CssProperty::Clear => false,
      CssProperty::Clip => false,
      CssProperty::ClipPath => false,
      CssProperty::ClipRule => true,
      CssProperty::Color => true,
      CssProperty::ColorAdjust => false,
      CssProperty::ColorInterpolation => true,
      CssProperty::ColorInterpolationFilters => true,
      CssProperty::ColorScheme => true,
      CssProperty::ColumnCount => false,
      CssProperty::ColumnFill => false,
      CssProperty::ColumnGap => false,
      CssProperty::ColumnHeight => false,
      CssProperty::ColumnRule => false,
      CssProperty::ColumnRuleBreak => false,
      CssProperty::ColumnRuleColor => false,
      CssProperty::ColumnRuleInset => false,
      CssProperty::ColumnRuleInsetCap => false,
      CssProperty::ColumnRuleInsetCapEnd => false,
      CssProperty::ColumnRuleInsetCapStart => false,
      CssProperty::ColumnRuleInsetEnd => false,
      CssProperty::ColumnRuleInsetJunction => false,
      CssProperty::ColumnRuleInsetJunctionEnd => false,
      CssProperty::ColumnRuleInsetJunctionStart => false,
      CssProperty::ColumnRuleInsetStart => false,
      CssProperty::ColumnRuleStyle => false,
      CssProperty::ColumnRuleVisibilityItems => false,
      CssProperty::ColumnRuleWidth => false,
      CssProperty::ColumnSpan => false,
      CssProperty::ColumnWidth => false,
      CssProperty::ColumnWrap => false,
      CssProperty::Columns => false,
      CssProperty::Contain => false,
      CssProperty::ContainIntrinsicBlockSize => false,
      CssProperty::ContainIntrinsicHeight => false,
      CssProperty::ContainIntrinsicInlineSize => false,
      CssProperty::ContainIntrinsicSize => false,
      CssProperty::ContainIntrinsicWidth => false,
      CssProperty::Container => false,
      CssProperty::ContainerName => false,
      CssProperty::ContainerType => false,
      CssProperty::Content => false,
      CssProperty::ContentVisibility => false,
      CssProperty::Continue => false,
      CssProperty::CopyInto => false,
      CssProperty::Corner => false,
      CssProperty::CornerBlockEnd => false,
      CssProperty::CornerBlockEndShape => false,
      CssProperty::CornerBlockStart => false,
      CssProperty::CornerBlockStartShape => false,
      CssProperty::CornerBottom => false,
      CssProperty::CornerBottomLeft => false,
      CssProperty::CornerBottomLeftShape => false,
      CssProperty::CornerBottomRight => false,
      CssProperty::CornerBottomRightShape => false,
      CssProperty::CornerBottomShape => false,
      CssProperty::CornerEndEnd => false,
      CssProperty::CornerEndEndShape => false,
      CssProperty::CornerEndStart => false,
      CssProperty::CornerEndStartShape => false,
      CssProperty::CornerInlineEnd => false,
      CssProperty::CornerInlineEndShape => false,
      CssProperty::CornerInlineStart => false,
      CssProperty::CornerInlineStartShape => false,
      CssProperty::CornerLeft => false,
      CssProperty::CornerLeftShape => false,
      CssProperty::CornerRight => false,
      CssProperty::CornerRightShape => false,
      CssProperty::CornerShape => false,
      CssProperty::CornerStartEnd => false,
      CssProperty::CornerStartEndShape => false,
      CssProperty::CornerStartStart => false,
      CssProperty::CornerStartStartShape => false,
      CssProperty::CornerTop => false,
      CssProperty::CornerTopLeft => false,
      CssProperty::CornerTopLeftShape => false,
      CssProperty::CornerTopRight => false,
      CssProperty::CornerTopRightShape => false,
      CssProperty::CornerTopShape => false,
      CssProperty::CounterIncrement => false,
      CssProperty::CounterReset => false,
      CssProperty::CounterSet => false,
      CssProperty::Cue => false,
      CssProperty::CueAfter => false,
      CssProperty::CueBefore => false,
      CssProperty::Cursor => true,
      CssProperty::Cx => false,
      CssProperty::Cy => false,
      CssProperty::D => false,
      CssProperty::Direction => true,
      CssProperty::Display => false,
      CssProperty::DominantBaseline => true,
      CssProperty::DynamicRangeLimit => true,
      CssProperty::EmptyCells => true,
      CssProperty::EventTrigger => false,
      CssProperty::EventTriggerName => false,
      CssProperty::EventTriggerSource => false,
      CssProperty::FieldSizing => false,
      CssProperty::Fill => true,
      CssProperty::FillBreak => false,
      CssProperty::FillColor => true,
      CssProperty::FillImage => true,
      CssProperty::FillOpacity => true,
      CssProperty::FillOrigin => false,
      CssProperty::FillPosition => true,
      CssProperty::FillRepeat => true,
      CssProperty::FillRule => true,
      CssProperty::FillSize => true,
      CssProperty::Filter => false,
      CssProperty::Flex => false,
      CssProperty::FlexBasis => false,
      CssProperty::FlexDirection => false,
      CssProperty::FlexFlow => false,
      CssProperty::FlexGrow => false,
      CssProperty::FlexShrink => false,
      CssProperty::FlexWrap => false,
      CssProperty::Float => false,
      CssProperty::FloatDefer => false,
      CssProperty::FloatOffset => false,
      CssProperty::FloatReference => false,
      CssProperty::FloodColor => false,
      CssProperty::FloodOpacity => false,
      CssProperty::FlowFrom => false,
      CssProperty::FlowInto => false,
      CssProperty::FlowTolerance => false,
      CssProperty::Font => true,
      CssProperty::FontFamily => true,
      CssProperty::FontFeatureSettings => true,
      CssProperty::FontKerning => true,
      CssProperty::FontLanguageOverride => true,
      CssProperty::FontOpticalSizing => true,
      CssProperty::FontPalette => true,
      CssProperty::FontSize => true,
      CssProperty::FontSizeAdjust => true,
      CssProperty::FontStretch => false,
      CssProperty::FontStyle => true,
      CssProperty::FontSynthesis => true,
      CssProperty::FontSynthesisPosition => true,
      CssProperty::FontSynthesisSmallCaps => true,
      CssProperty::FontSynthesisStyle => true,
      CssProperty::FontSynthesisWeight => true,
      CssProperty::FontVariant => true,
      CssProperty::FontVariantAlternates => true,
      CssProperty::FontVariantCaps => true,
      CssProperty::FontVariantEastAsian => true,
      CssProperty::FontVariantEmoji => true,
      CssProperty::FontVariantLigatures => true,
      CssProperty::FontVariantNumeric => true,
      CssProperty::FontVariantPosition => true,
      CssProperty::FontVariationSettings => true,
      CssProperty::FontWeight => true,
      CssProperty::FontWidth => true,
      CssProperty::FootnoteDisplay => false,
      CssProperty::FootnotePolicy => false,
      CssProperty::ForcedColorAdjust => true,
      CssProperty::FrameSizing => false,
      CssProperty::Gap => false,
      CssProperty::GlyphOrientationVertical => false,
      CssProperty::Grid => false,
      CssProperty::GridArea => false,
      CssProperty::GridAutoColumns => false,
      CssProperty::GridAutoFlow => false,
      CssProperty::GridAutoRows => false,
      CssProperty::GridColumn => false,
      CssProperty::GridColumnEnd => false,
      CssProperty::GridColumnGap => false,
      CssProperty::GridColumnStart => false,
      CssProperty::GridGap => false,
      CssProperty::GridRow => false,
      CssProperty::GridRowEnd => false,
      CssProperty::GridRowGap => false,
      CssProperty::GridRowStart => false,
      CssProperty::GridTemplate => false,
      CssProperty::GridTemplateAreas => false,
      CssProperty::GridTemplateColumns => false,
      CssProperty::GridTemplateRows => false,
      CssProperty::HangingPunctuation => true,
      CssProperty::Height => false,
      CssProperty::HyphenateCharacter => true,
      CssProperty::HyphenateLimitChars => true,
      CssProperty::HyphenateLimitLast => true,
      CssProperty::HyphenateLimitLines => true,
      CssProperty::HyphenateLimitZone => true,
      CssProperty::Hyphens => true,
      CssProperty::ImageAnimation => true,
      CssProperty::ImageOrientation => true,
      CssProperty::ImageRendering => true,
      CssProperty::ImageResolution => true,
      CssProperty::InitialLetter => false,
      CssProperty::InitialLetterAlign => true,
      CssProperty::InitialLetterWrap => true,
      CssProperty::InlineSize => false,
      CssProperty::InlineSizing => true,
      CssProperty::InputSecurity => false,
      CssProperty::Inset => false,
      CssProperty::InsetBlock => false,
      CssProperty::InsetBlockEnd => false,
      CssProperty::InsetBlockStart => false,
      CssProperty::InsetInline => false,
      CssProperty::InsetInlineEnd => false,
      CssProperty::InsetInlineStart => false,
      CssProperty::Interactivity => true,
      CssProperty::InterestDelay => false,
      CssProperty::InterestDelayEnd => true,
      CssProperty::InterestDelayStart => true,
      CssProperty::InterpolateSize => true,
      CssProperty::Isolation => false,
      CssProperty::JustifyContent => false,
      CssProperty::JustifyItems => false,
      CssProperty::JustifySelf => false,
      CssProperty::Left => false,
      CssProperty::LetterSpacing => true,
      CssProperty::LightingColor => false,
      CssProperty::LineBreak => true,
      CssProperty::LineClamp => false,
      CssProperty::LineFitEdge => true,
      CssProperty::LineGrid => false,
      CssProperty::LineHeight => true,
      CssProperty::LineHeightStep => true,
      CssProperty::LinePadding => true,
      CssProperty::LineSnap => true,
      CssProperty::LinkParameters => false,
      CssProperty::ListStyle => false,
      CssProperty::ListStyleImage => true,
      CssProperty::ListStylePosition => true,
      CssProperty::ListStyleType => true,
      CssProperty::Margin => false,
      CssProperty::MarginBlock => false,
      CssProperty::MarginBlockEnd => false,
      CssProperty::MarginBlockStart => false,
      CssProperty::MarginBottom => false,
      CssProperty::MarginBreak => false,
      CssProperty::MarginInline => false,
      CssProperty::MarginInlineEnd => false,
      CssProperty::MarginInlineStart => false,
      CssProperty::MarginLeft => false,
      CssProperty::MarginRight => false,
      CssProperty::MarginTop => false,
      CssProperty::MarginTrim => false,
      CssProperty::Marker => true,
      CssProperty::MarkerEnd => true,
      CssProperty::MarkerMid => true,
      CssProperty::MarkerSide => true,
      CssProperty::MarkerStart => true,
      CssProperty::Mask => false,
      CssProperty::MaskBorder => false,
      CssProperty::MaskBorderMode => false,
      CssProperty::MaskBorderOutset => false,
      CssProperty::MaskBorderRepeat => false,
      CssProperty::MaskBorderSlice => false,
      CssProperty::MaskBorderSource => false,
      CssProperty::MaskBorderWidth => false,
      CssProperty::MaskClip => false,
      CssProperty::MaskComposite => false,
      CssProperty::MaskImage => false,
      CssProperty::MaskMode => false,
      CssProperty::MaskOrigin => false,
      CssProperty::MaskPosition => false,
      CssProperty::MaskRepeat => false,
      CssProperty::MaskSize => false,
      CssProperty::MaskType => false,
      CssProperty::MathDepth => true,
      CssProperty::MathShift => true,
      CssProperty::MathStyle => true,
      CssProperty::MaxBlockSize => false,
      CssProperty::MaxHeight => false,
      CssProperty::MaxInlineSize => false,
      CssProperty::MaxLines => false,
      CssProperty::MaxWidth => false,
      CssProperty::MinBlockSize => false,
      CssProperty::MinHeight => false,
      CssProperty::MinInlineSize => false,
      CssProperty::MinIntrinsicSizing => false,
      CssProperty::MinWidth => false,
      CssProperty::MixBlendMode => false,
      CssProperty::NavDown => false,
      CssProperty::NavLeft => false,
      CssProperty::NavRight => false,
      CssProperty::NavUp => false,
      CssProperty::ObjectFit => false,
      CssProperty::ObjectPosition => false,
      CssProperty::ObjectViewBox => false,
      CssProperty::Offset => false,
      CssProperty::OffsetAnchor => false,
      CssProperty::OffsetDistance => false,
      CssProperty::OffsetPath => false,
      CssProperty::OffsetPosition => false,
      CssProperty::OffsetRotate => false,
      CssProperty::Opacity => false,
      CssProperty::Order => false,
      CssProperty::Orphans => true,
      CssProperty::Outline => false,
      CssProperty::OutlineColor => false,
      CssProperty::OutlineOffset => false,
      CssProperty::OutlineStyle => false,
      CssProperty::OutlineWidth => false,
      CssProperty::Overflow => false,
      CssProperty::OverflowAnchor => false,
      CssProperty::OverflowBlock => false,
      CssProperty::OverflowClipMargin => false,
      CssProperty::OverflowClipMarginBlock => false,
      CssProperty::OverflowClipMarginBlockEnd => false,
      CssProperty::OverflowClipMarginBlockStart => false,
      CssProperty::OverflowClipMarginBottom => false,
      CssProperty::OverflowClipMarginInline => false,
      CssProperty::OverflowClipMarginInlineEnd => false,
      CssProperty::OverflowClipMarginInlineStart => false,
      CssProperty::OverflowClipMarginLeft => false,
      CssProperty::OverflowClipMarginRight => false,
      CssProperty::OverflowClipMarginTop => false,
      CssProperty::OverflowInline => false,
      CssProperty::OverflowWrap => true,
      CssProperty::OverflowX => false,
      CssProperty::OverflowY => false,
      CssProperty::Overlay => false,
      CssProperty::OverscrollBehavior => false,
      CssProperty::OverscrollBehaviorBlock => false,
      CssProperty::OverscrollBehaviorInline => false,
      CssProperty::OverscrollBehaviorX => false,
      CssProperty::OverscrollBehaviorY => false,
      CssProperty::Padding => false,
      CssProperty::PaddingBlock => false,
      CssProperty::PaddingBlockEnd => false,
      CssProperty::PaddingBlockStart => false,
      CssProperty::PaddingBottom => false,
      CssProperty::PaddingInline => false,
      CssProperty::PaddingInlineEnd => false,
      CssProperty::PaddingInlineStart => false,
      CssProperty::PaddingLeft => false,
      CssProperty::PaddingRight => false,
      CssProperty::PaddingTop => false,
      CssProperty::Page => false,
      CssProperty::PageBreakAfter => false,
      CssProperty::PageBreakBefore => false,
      CssProperty::PageBreakInside => false,
      CssProperty::PaintOrder => true,
      CssProperty::PathLength => false,
      CssProperty::Pause => false,
      CssProperty::PauseAfter => false,
      CssProperty::PauseBefore => false,
      CssProperty::Perspective => false,
      CssProperty::PerspectiveOrigin => false,
      CssProperty::PlaceContent => false,
      CssProperty::PlaceItems => false,
      CssProperty::PlaceSelf => false,
      CssProperty::PointerEvents => true,
      CssProperty::PointerTimeline => false,
      CssProperty::PointerTimelineAxis => false,
      CssProperty::PointerTimelineName => false,
      CssProperty::Position => false,
      CssProperty::PositionAnchor => false,
      CssProperty::PositionArea => false,
      CssProperty::PositionTry => false,
      CssProperty::PositionTryFallbacks => false,
      CssProperty::PositionTryOrder => false,
      CssProperty::PositionVisibility => false,
      CssProperty::PrintColorAdjust => true,
      CssProperty::Quotes => true,
      CssProperty::R => false,
      CssProperty::ReadingFlow => false,
      CssProperty::ReadingOrder => false,
      CssProperty::RegionFragment => false,
      CssProperty::Resize => false,
      CssProperty::Rest => false,
      CssProperty::RestAfter => false,
      CssProperty::RestBefore => false,
      CssProperty::Right => false,
      CssProperty::Rotate => false,
      CssProperty::RowGap => false,
      CssProperty::RowRule => false,
      CssProperty::RowRuleBreak => false,
      CssProperty::RowRuleColor => false,
      CssProperty::RowRuleInset => false,
      CssProperty::RowRuleInsetCap => false,
      CssProperty::RowRuleInsetCapEnd => false,
      CssProperty::RowRuleInsetCapStart => false,
      CssProperty::RowRuleInsetEnd => false,
      CssProperty::RowRuleInsetJunction => false,
      CssProperty::RowRuleInsetJunctionEnd => false,
      CssProperty::RowRuleInsetJunctionStart => false,
      CssProperty::RowRuleInsetStart => false,
      CssProperty::RowRuleStyle => false,
      CssProperty::RowRuleVisibilityItems => false,
      CssProperty::RowRuleWidth => false,
      CssProperty::RubyAlign => true,
      CssProperty::RubyMerge => true,
      CssProperty::RubyOverhang => true,
      CssProperty::RubyPosition => true,
      CssProperty::Rule => false,
      CssProperty::RuleBreak => false,
      CssProperty::RuleColor => false,
      CssProperty::RuleInset => false,
      CssProperty::RuleInsetCap => false,
      CssProperty::RuleInsetEnd => false,
      CssProperty::RuleInsetJunction => false,
      CssProperty::RuleInsetStart => false,
      CssProperty::RuleOverlap => false,
      CssProperty::RuleStyle => false,
      CssProperty::RuleVisibilityItems => false,
      CssProperty::RuleWidth => false,
      CssProperty::Rx => false,
      CssProperty::Ry => false,
      CssProperty::Scale => false,
      CssProperty::ScrollBehavior => false,
      CssProperty::ScrollInitialTarget => false,
      CssProperty::ScrollMargin => false,
      CssProperty::ScrollMarginBlock => false,
      CssProperty::ScrollMarginBlockEnd => false,
      CssProperty::ScrollMarginBlockStart => false,
      CssProperty::ScrollMarginBottom => false,
      CssProperty::ScrollMarginInline => false,
      CssProperty::ScrollMarginInlineEnd => false,
      CssProperty::ScrollMarginInlineStart => false,
      CssProperty::ScrollMarginLeft => false,
      CssProperty::ScrollMarginRight => false,
      CssProperty::ScrollMarginTop => false,
      CssProperty::ScrollMarkerGroup => false,
      CssProperty::ScrollPadding => false,
      CssProperty::ScrollPaddingBlock => false,
      CssProperty::ScrollPaddingBlockEnd => false,
      CssProperty::ScrollPaddingBlockStart => false,
      CssProperty::ScrollPaddingBottom => false,
      CssProperty::ScrollPaddingInline => false,
      CssProperty::ScrollPaddingInlineEnd => false,
      CssProperty::ScrollPaddingInlineStart => false,
      CssProperty::ScrollPaddingLeft => false,
      CssProperty::ScrollPaddingRight => false,
      CssProperty::ScrollPaddingTop => false,
      CssProperty::ScrollSnapAlign => false,
      CssProperty::ScrollSnapStop => false,
      CssProperty::ScrollSnapType => false,
      CssProperty::ScrollTargetGroup => false,
      CssProperty::ScrollTimeline => false,
      CssProperty::ScrollTimelineAxis => false,
      CssProperty::ScrollTimelineName => false,
      CssProperty::ScrollbarColor => true,
      CssProperty::ScrollbarGutter => false,
      CssProperty::ScrollbarInset => false,
      CssProperty::ScrollbarMinThumbSize => false,
      CssProperty::ScrollbarMode => false,
      CssProperty::ScrollbarWidth => false,
      CssProperty::ShapeImageThreshold => false,
      CssProperty::ShapeInside => false,
      CssProperty::ShapeMargin => false,
      CssProperty::ShapeOutside => false,
      CssProperty::ShapePadding => false,
      CssProperty::ShapeRendering => true,
      CssProperty::SliderOrientation => false,
      CssProperty::SpatialNavigationAction => false,
      CssProperty::SpatialNavigationContain => false,
      CssProperty::SpatialNavigationFunction => false,
      CssProperty::Speak => true,
      CssProperty::SpeakAs => true,
      CssProperty::StopColor => false,
      CssProperty::StopOpacity => false,
      CssProperty::StringSet => false,
      CssProperty::Stroke => true,
      CssProperty::StrokeAlign => true,
      CssProperty::StrokeAlignment => true,
      CssProperty::StrokeBreak => false,
      CssProperty::StrokeColor => true,
      CssProperty::StrokeDashCornerPropdefStrokeDashCorner => true,
      CssProperty::StrokeDashJustify => true,
      CssProperty::StrokeDashadjust => true,
      CssProperty::StrokeDasharray => true,
      CssProperty::StrokeDashcornerStrokedashcornerproperty => true,
      CssProperty::StrokeDashoffset => true,
      CssProperty::StrokeImage => true,
      CssProperty::StrokeLinecap => true,
      CssProperty::StrokeLinejoin => true,
      CssProperty::StrokeMiterlimit => true,
      CssProperty::StrokeOpacity => true,
      CssProperty::StrokeOrigin => false,
      CssProperty::StrokePosition => true,
      CssProperty::StrokeRepeat => true,
      CssProperty::StrokeSize => true,
      CssProperty::StrokeWidth => true,
      CssProperty::TabSize => true,
      CssProperty::TableLayout => false,
      CssProperty::TextAlign => true,
      CssProperty::TextAlignAll => true,
      CssProperty::TextAlignLast => true,
      CssProperty::TextAnchor => true,
      CssProperty::TextAutospace => true,
      CssProperty::TextBox => false,
      CssProperty::TextBoxEdge => true,
      CssProperty::TextBoxTrim => false,
      CssProperty::TextCombineUpright => true,
      CssProperty::TextDecoration => false,
      CssProperty::TextDecorationColor => false,
      CssProperty::TextDecorationInset => false,
      CssProperty::TextDecorationLine => false,
      CssProperty::TextDecorationSkip => true,
      CssProperty::TextDecorationSkipBox => true,
      CssProperty::TextDecorationSkipInk => true,
      CssProperty::TextDecorationSkipSelf => false,
      CssProperty::TextDecorationSkipSpaces => true,
      CssProperty::TextDecorationStyle => false,
      CssProperty::TextDecorationThickness => false,
      CssProperty::TextEmphasis => false,
      CssProperty::TextEmphasisColor => true,
      CssProperty::TextEmphasisPosition => true,
      CssProperty::TextEmphasisSkip => true,
      CssProperty::TextEmphasisStyle => true,
      CssProperty::TextFit => false,
      CssProperty::TextGroupAlign => false,
      CssProperty::TextIndent => true,
      CssProperty::TextJustify => true,
      CssProperty::TextOrientation => true,
      CssProperty::TextOverflow => false,
      CssProperty::TextRendering => true,
      CssProperty::TextShadow => true,
      CssProperty::TextSizeAdjust => true,
      CssProperty::TextSpacing => true,
      CssProperty::TextSpacingTrim => true,
      CssProperty::TextTransform => true,
      CssProperty::TextUnderlineOffset => true,
      CssProperty::TextUnderlinePosition => true,
      CssProperty::TextWrap => false,
      CssProperty::TextWrapMode => true,
      CssProperty::TextWrapStyle => true,
      CssProperty::TimelineScope => false,
      CssProperty::TimelineTrigger => false,
      CssProperty::TimelineTriggerActivationRange => false,
      CssProperty::TimelineTriggerActivationRangeEnd => false,
      CssProperty::TimelineTriggerActivationRangeStart => false,
      CssProperty::TimelineTriggerActiveRange => false,
      CssProperty::TimelineTriggerActiveRangeEnd => false,
      CssProperty::TimelineTriggerActiveRangeStart => false,
      CssProperty::TimelineTriggerName => false,
      CssProperty::TimelineTriggerSource => false,
      CssProperty::Top => false,
      CssProperty::TouchAction => false,
      CssProperty::Transform => false,
      CssProperty::TransformBox => false,
      CssProperty::TransformOrigin => false,
      CssProperty::TransformStyle => false,
      CssProperty::Transition => false,
      CssProperty::TransitionBehavior => false,
      CssProperty::TransitionDelay => false,
      CssProperty::TransitionDuration => false,
      CssProperty::TransitionProperty => false,
      CssProperty::TransitionTimingFunction => false,
      CssProperty::Translate => false,
      CssProperty::TriggerScope => false,
      CssProperty::UnicodeBidi => false,
      CssProperty::UserSelect => false,
      CssProperty::VectorEffect => false,
      CssProperty::VerticalAlign => false,
      CssProperty::ViewTimeline => false,
      CssProperty::ViewTimelineAxis => false,
      CssProperty::ViewTimelineInset => false,
      CssProperty::ViewTimelineName => false,
      CssProperty::ViewTransitionClass => false,
      CssProperty::ViewTransitionGroup => false,
      CssProperty::ViewTransitionName => false,
      CssProperty::ViewTransitionScope => false,
      CssProperty::Visibility => true,
      CssProperty::VoiceBalance => true,
      CssProperty::VoiceDuration => false,
      CssProperty::VoiceFamily => true,
      CssProperty::VoicePitch => true,
      CssProperty::VoiceRange => true,
      CssProperty::VoiceRate => true,
      CssProperty::VoiceStress => true,
      CssProperty::VoiceVolume => true,
      CssProperty::WhiteSpace => true,
      CssProperty::WhiteSpaceCollapse => true,
      CssProperty::WhiteSpaceTrim => false,
      CssProperty::Widows => true,
      CssProperty::Width => false,
      CssProperty::WillChange => false,
      CssProperty::WordBreak => true,
      CssProperty::WordSpaceTransform => true,
      CssProperty::WordSpacing => true,
      CssProperty::WordWrap => true,
      CssProperty::WrapAfter => false,
      CssProperty::WrapBefore => false,
      CssProperty::WrapFlow => false,
      CssProperty::WrapInside => false,
      CssProperty::WrapThrough => false,
      CssProperty::WritingMode => true,
      CssProperty::X => false,
      CssProperty::Y => false,
      CssProperty::ZIndex => false,
      CssProperty::Zoom => false,
      CssProperty::Unknown(_) => false,
    }
  }
}
