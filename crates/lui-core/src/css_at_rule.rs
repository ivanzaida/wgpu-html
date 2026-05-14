// Auto-generated from atrules.json. DO NOT EDIT.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CssAtRule {
  ///
  /// href: https://compat.spec.whatwg.org/#at-ruledef--webkit-keyframes
  WebkitKeyframes,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#at-ruledef-font-feature-values-annotation
  /// syntax: @annotation { <declaration-list> }
  Annotation,
  ///
  /// href: https://drafts.csswg.org/css-mixins-1/#at-ruledef-apply
  /// syntax: @apply [ <dashed-ident> | <dashed-function> ] [ { <declaration-list> } ]?;
  Apply,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-bottom-center
  /// syntax: @bottom-center { <declaration-list> };
  BottomCenter,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-bottom-left
  /// syntax: @bottom-left { <declaration-list> };
  BottomLeft,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-bottom-left-corner
  /// syntax: @bottom-left-corner { <declaration-list> };
  BottomLeftCorner,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-bottom-right
  /// syntax: @bottom-right { <declaration-list> };
  BottomRight,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-bottom-right-corner
  /// syntax: @bottom-right-corner { <declaration-list> };
  BottomRightCorner,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#at-ruledef-font-feature-values-character-variant
  /// syntax: @character-variant { <declaration-list> }
  CharacterVariant,
  ///
  /// href: https://drafts.csswg.org/css-syntax-3/#at-ruledef-charset
  /// prose: However, there is no actual at-rule named @charset. When a stylesheet is actually parsed, any occurrences
  /// of an @charset rule must be treated as an unrecognized rule, and thus dropped as invalid when the stylesheet is
  /// grammar-checked.
  Charset,
  ///
  /// href: https://drafts.csswg.org/css-color-5/#at-ruledef-profile
  /// syntax: @color-profile [<dashed-ident> | device-cmyk] { <declaration-list> }
  /// prose: The @color-profile rule defines and names a color profile which can later be used in the color() function
  /// to specify a color.
  ColorProfile,
  ///
  /// href: https://drafts.csswg.org/css-conditional-5/#at-ruledef-container
  /// syntax: @container <container-condition># { <block-contents> }
  /// prose: The @container rule is a conditional group rule whose condition contains a container query, which is a
  /// boolean combination of container size queries and/or container style queries. Style declarations within the
  /// <block-contents> block of an @container rule are filtered by its condition to only match when the container query
  /// is true for their element’s query container.
  Container,
  ///
  /// href: https://drafts.csswg.org/css-mixins-1/#at-ruledef-contents
  /// syntax: @contents [ { <declaration-list> } ]?
  Contents,
  ///
  /// href: https://drafts.csswg.org/css-counter-styles-3/#at-ruledef-counter-style
  /// syntax: @counter-style <counter-style-name> { <declaration-list> }
  /// prose: The @counter-style rule allows authors to define a custom counter style. The components of a counter style
  /// are specified by descriptors in the @counter-style rule. The algorithm is specified implicitly by a combination of
  /// the system, symbols, and additive-symbols properties.
  CounterStyle,
  ///
  /// href: https://drafts.csswg.org/mediaqueries-5/#at-ruledef-custom-media
  /// syntax: @custom-media <extension-name> [ <media-query-list> | true | false ] ;
  CustomMedia,
  ///
  /// href: https://drafts.csswg.org/css-extensions-1/#at-ruledef-custom-selector
  /// syntax: @custom-selector <custom-selector> <selector-list> ;
  CustomSelector,
  ///
  /// href: https://drafts.csswg.org/css-conditional-5/#at-ruledef-else
  /// syntax: @else <boolean-condition>? { <rule-list> }
  /// prose: The @else rule is a conditional group rule used to form conditional rule chains, which associate multiple
  /// conditional group rules and guarantee that only the first one that matches will evaluate its condition as true. It
  /// is defined as:
  Else,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#at-font-face-rule
  /// syntax: @font-face { <declaration-list> }
  FontFace,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#at-ruledef-font-feature-values
  /// syntax: @font-feature-values <font-family-name># { <declaration-rule-list> }
  FontFeatureValues,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#at-ruledef-font-palette-values
  /// syntax: @font-palette-values <dashed-ident> { <declaration-list> }
  /// prose: The @font-palette-values rule defines a color palette and associates that color palette with a specific
  /// font. This allows a web author to select arbitrary <color>s to use inside a color font rather than being limited
  /// to the preexisting palettes inside font files.
  FontPaletteValues,
  ///
  /// href: https://drafts.csswg.org/css-mixins-1/#at-ruledef-function
  /// syntax: @function <function-token> <function-parameter>#? ) [ returns <css-type> ]? { <declaration-rule-list> }
  Function,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#at-ruledef-font-feature-values-historical-forms
  /// syntax: @historical-forms { <declaration-list> }
  HistoricalForms,
  ///
  /// href: https://drafts.csswg.org/css-cascade-6/#at-ruledef-import
  /// syntax: @import [ <url> | <string> ] [[ layer | layer( <layer-name> ) ] || [ scope | scope( <scope-start> |
  /// <scope-boundaries> ) ] || <supports-import-condition>]? <media-import-condition> ; prose: The @import rule
  /// allows users to import style rules from other style sheets. If an @import rule refers to a valid stylesheet, user
  /// agents must treat the contents of the stylesheet as if they were written in place of the @import rule, with two
  /// exceptions:
  Import,
  ///
  /// href: https://drafts.csswg.org/css-animations-1/#at-ruledef-keyframes
  /// syntax: @keyframes <keyframes-name> { <qualified-rule-list> }
  /// prose: Keyframes are specified using the @keyframes at-rule, defined as follows:
  Keyframes,
  ///
  /// href: https://drafts.csswg.org/css-cascade-5/#at-ruledef-layer
  /// syntax: @layer <layer-name>? { <rule-list> } | @layer <layer-name>#;
  /// prose: The @layer rule declares a cascade layer, with the option to assign style rules.
  Layer,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-left-bottom
  /// syntax: @left-bottom { <declaration-list> };
  LeftBottom,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-left-middle
  /// syntax: @left-middle { <declaration-list> };
  LeftMiddle,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-left-top
  /// syntax: @left-top { <declaration-list> };
  LeftTop,
  ///
  /// href: https://drafts.csswg.org/css-mixins-1/#macro-rule
  /// syntax: @macro <dashed-ident> { <declaration-rule-list> }
  Macro,
  ///
  /// href: https://drafts.csswg.org/css-conditional-3/#at-ruledef-media
  /// syntax: @media <media-query-list> { <rule-list> }
  /// prose: The @media rule is a conditional group rule whose condition is a media query. Its syntax is:
  Media,
  ///
  /// href: https://drafts.csswg.org/css-mixins-1/#at-ruledef-mixin
  /// syntax: @mixin <function-token> <function-parameter>#? ) { <declaration-rule-list> }
  Mixin,
  ///
  /// href: https://drafts.csswg.org/css-namespaces-3/#at-ruledef-namespace
  /// syntax: @namespace <namespace-prefix>? [ <string> | <url> ] ;
  /// prose: The @namespace at-rule declares a namespace prefix and associates it with a given namespace name (a
  /// string). This namespace prefix can then be used in namespace-qualified names such as the CSS qualified names
  /// defined below.
  Namespace,
  ///
  /// href: https://drafts.csswg.org/css-navigation-1/#at-ruledef-navigation
  /// syntax: @navigation <navigation-condition> { <rule-list> }
  /// prose: The @navigation rule is a conditional group rule whose condition tests characteristics of the current URL
  /// or of the state of navigation between two URLs. These queries are called navigation queries.
  Navigation,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#at-ruledef-font-feature-values-ornaments
  /// syntax: @ornaments { <declaration-list> }
  Ornaments,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-page
  /// syntax: @page <page-selector-list>? { <declaration-rule-list> }
  /// prose: Authors can specify various aspects of a page box, such as its dimensions, orientation, and margins, within
  /// an @page rule. @page rules are allowed wherever rule-sets are allowed. An @page rule consists of the keyword
  /// @page, an optional comma-separated list of page selectors and a block of declarations (said to be in the page
  /// context). An @page rule can also contain other at-rules, interleaved between declarations. The current level of
  /// this specification only allows margin at-rules inside @page.
  Page,
  ///
  /// href: https://drafts.csswg.org/css-anchor-position-1/#at-ruledef-position-try
  /// syntax: @position-try <dashed-ident> { <declaration-list> }
  /// prose: The @position-try rule defines a position option with a given name, specifying one or more sets of
  /// positioning properties that can be applied to a box via position-try-fallbacks,
  PositionTry,
  ///
  /// href: https://drafts.css-houdini.org/css-properties-values-api-1/#at-ruledef-property
  /// syntax: @property <custom-property-name> { <declaration-list> }
  Property,
  ///
  /// href: https://drafts.csswg.org/css-mixins-1/#result-rule
  /// syntax: @result { <declaration-rule-list> }
  Result,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-right-bottom
  /// syntax: @right-bottom { <declaration-list> };
  RightBottom,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-right-middle
  /// syntax: @right-middle { <declaration-list> };
  RightMiddle,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-right-top
  /// syntax: @right-top { <declaration-list> };
  RightTop,
  ///
  /// href: https://drafts.csswg.org/css-navigation-1/#at-ruledef-route
  /// syntax: @route <dashed-ident> { [ <pattern-descriptors> | <init-descriptors> ] }
  /// prose: The @route rule is an at-rule that associates a name with a URL pattern. This name can be referenced in
  /// @navigation rules and in :link-to() pseudo-classes.
  Route,
  ///
  /// href: https://drafts.csswg.org/css-cascade-6/#at-ruledef-scope
  /// syntax: @scope <scope-boundaries>? { <block-contents> }
  /// prose: Scoped styles are described in CSS using the @scope block at-rule, which declares a scoping root and
  /// optional scoping limits associated with a set of style rules.
  Scope,
  ///
  /// href: https://drafts.csswg.org/css-transitions-2/#at-ruledef-starting-style
  /// prose: The @starting-style rule is a grouping rule. The style rules inside it are used to establish styles to
  /// transition from, if the previous style change event did not establish a before-change style for the element whose
  /// styles are being computed.
  StartingStyle,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#at-ruledef-font-feature-values-styleset
  /// syntax: @styleset { <declaration-list> }
  Styleset,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#at-ruledef-font-feature-values-stylistic
  /// syntax: @stylistic { <declaration-list> }
  Stylistic,
  ///
  /// href: https://drafts.csswg.org/css-conditional-3/#at-ruledef-supports
  /// syntax: @supports <supports-condition> { <rule-list> }
  /// prose: The @supports rule is a conditional group rule whose condition tests whether the user agent supports CSS
  /// property:value pairs. Authors can use it to write style sheets that use new features when available but degrade
  /// gracefully when those features are not supported. These queries are called CSS feature queries or (colloquially)
  /// supports queries.
  Supports,
  ///
  /// href: https://drafts.csswg.org/css-conditional-5/#at-ruledef-supports-condition
  /// syntax: @supports-condition <supports-condition-name> { <block-contents> }
  /// prose: The @supports-condition at-rule is a conditional group rule that allows authors to define and name a
  /// supports query for later reuse, creating a named supports condition. This enables complex or frequently-used
  /// feature queries to be referenced by name, improving maintainability and readability.
  SupportsCondition,
  ///
  /// href: https://drafts.csswg.org/css-fonts-4/#at-ruledef-font-feature-values-swash
  /// syntax: @swash { <declaration-list> }
  Swash,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-top-center
  /// syntax: @top-center { <declaration-list> };
  TopCenter,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-top-left
  /// syntax: @top-left { <declaration-list> };
  TopLeft,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-top-left-corner
  /// syntax: @top-left-corner { <declaration-list> };
  TopLeftCorner,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-top-right
  /// syntax: @top-right { <declaration-list> };
  TopRight,
  ///
  /// href: https://drafts.csswg.org/css-page-3/#at-ruledef-top-right-corner
  /// syntax: @top-right-corner { <declaration-list> };
  TopRightCorner,
  ///
  /// href: https://drafts.csswg.org/css-view-transitions-2/#at-view-transition-rule
  /// syntax: @view-transition { <declaration-list> }
  ViewTransition,
  ///
  /// href: https://drafts.csswg.org/css-conditional-5/#at-ruledef-when
  /// syntax: @when <boolean-condition> { <rule-list> }
  /// prose: The @when at-rule is a conditional group rule that generalizes the individual conditional group rules such
  /// as @media and @supports. It is defined as:
  When,
  Unknown(String),
}

impl CssAtRule {
  pub fn name(&self) -> &'static str {
    match self {
      CssAtRule::WebkitKeyframes => "@-webkit-keyframes",
      CssAtRule::Annotation => "@annotation",
      CssAtRule::Apply => "@apply",
      CssAtRule::BottomCenter => "@bottom-center",
      CssAtRule::BottomLeft => "@bottom-left",
      CssAtRule::BottomLeftCorner => "@bottom-left-corner",
      CssAtRule::BottomRight => "@bottom-right",
      CssAtRule::BottomRightCorner => "@bottom-right-corner",
      CssAtRule::CharacterVariant => "@character-variant",
      CssAtRule::Charset => "@charset",
      CssAtRule::ColorProfile => "@color-profile",
      CssAtRule::Container => "@container",
      CssAtRule::Contents => "@contents",
      CssAtRule::CounterStyle => "@counter-style",
      CssAtRule::CustomMedia => "@custom-media",
      CssAtRule::CustomSelector => "@custom-selector",
      CssAtRule::Else => "@else",
      CssAtRule::FontFace => "@font-face",
      CssAtRule::FontFeatureValues => "@font-feature-values",
      CssAtRule::FontPaletteValues => "@font-palette-values",
      CssAtRule::Function => "@function",
      CssAtRule::HistoricalForms => "@historical-forms",
      CssAtRule::Import => "@import",
      CssAtRule::Keyframes => "@keyframes",
      CssAtRule::Layer => "@layer",
      CssAtRule::LeftBottom => "@left-bottom",
      CssAtRule::LeftMiddle => "@left-middle",
      CssAtRule::LeftTop => "@left-top",
      CssAtRule::Macro => "@macro",
      CssAtRule::Media => "@media",
      CssAtRule::Mixin => "@mixin",
      CssAtRule::Namespace => "@namespace",
      CssAtRule::Navigation => "@navigation",
      CssAtRule::Ornaments => "@ornaments",
      CssAtRule::Page => "@page",
      CssAtRule::PositionTry => "@position-try",
      CssAtRule::Property => "@property",
      CssAtRule::Result => "@result",
      CssAtRule::RightBottom => "@right-bottom",
      CssAtRule::RightMiddle => "@right-middle",
      CssAtRule::RightTop => "@right-top",
      CssAtRule::Route => "@route",
      CssAtRule::Scope => "@scope",
      CssAtRule::StartingStyle => "@starting-style",
      CssAtRule::Styleset => "@styleset",
      CssAtRule::Stylistic => "@stylistic",
      CssAtRule::Supports => "@supports",
      CssAtRule::SupportsCondition => "@supports-condition",
      CssAtRule::Swash => "@swash",
      CssAtRule::TopCenter => "@top-center",
      CssAtRule::TopLeft => "@top-left",
      CssAtRule::TopLeftCorner => "@top-left-corner",
      CssAtRule::TopRight => "@top-right",
      CssAtRule::TopRightCorner => "@top-right-corner",
      CssAtRule::ViewTransition => "@view-transition",
      CssAtRule::When => "@when",
      CssAtRule::Unknown(_) => "",
    }
  }

  pub fn from_name(name: &str) -> Self {
    const ENTRIES: &[(&str, CssAtRule)] = &[
      ("@-webkit-keyframes", CssAtRule::WebkitKeyframes),
      ("@annotation", CssAtRule::Annotation),
      ("@apply", CssAtRule::Apply),
      ("@bottom-center", CssAtRule::BottomCenter),
      ("@bottom-left", CssAtRule::BottomLeft),
      ("@bottom-left-corner", CssAtRule::BottomLeftCorner),
      ("@bottom-right", CssAtRule::BottomRight),
      ("@bottom-right-corner", CssAtRule::BottomRightCorner),
      ("@character-variant", CssAtRule::CharacterVariant),
      ("@charset", CssAtRule::Charset),
      ("@color-profile", CssAtRule::ColorProfile),
      ("@container", CssAtRule::Container),
      ("@contents", CssAtRule::Contents),
      ("@counter-style", CssAtRule::CounterStyle),
      ("@custom-media", CssAtRule::CustomMedia),
      ("@custom-selector", CssAtRule::CustomSelector),
      ("@else", CssAtRule::Else),
      ("@font-face", CssAtRule::FontFace),
      ("@font-feature-values", CssAtRule::FontFeatureValues),
      ("@font-palette-values", CssAtRule::FontPaletteValues),
      ("@function", CssAtRule::Function),
      ("@historical-forms", CssAtRule::HistoricalForms),
      ("@import", CssAtRule::Import),
      ("@keyframes", CssAtRule::Keyframes),
      ("@layer", CssAtRule::Layer),
      ("@left-bottom", CssAtRule::LeftBottom),
      ("@left-middle", CssAtRule::LeftMiddle),
      ("@left-top", CssAtRule::LeftTop),
      ("@macro", CssAtRule::Macro),
      ("@media", CssAtRule::Media),
      ("@mixin", CssAtRule::Mixin),
      ("@namespace", CssAtRule::Namespace),
      ("@navigation", CssAtRule::Navigation),
      ("@ornaments", CssAtRule::Ornaments),
      ("@page", CssAtRule::Page),
      ("@position-try", CssAtRule::PositionTry),
      ("@property", CssAtRule::Property),
      ("@result", CssAtRule::Result),
      ("@right-bottom", CssAtRule::RightBottom),
      ("@right-middle", CssAtRule::RightMiddle),
      ("@right-top", CssAtRule::RightTop),
      ("@route", CssAtRule::Route),
      ("@scope", CssAtRule::Scope),
      ("@starting-style", CssAtRule::StartingStyle),
      ("@styleset", CssAtRule::Styleset),
      ("@stylistic", CssAtRule::Stylistic),
      ("@supports", CssAtRule::Supports),
      ("@supports-condition", CssAtRule::SupportsCondition),
      ("@swash", CssAtRule::Swash),
      ("@top-center", CssAtRule::TopCenter),
      ("@top-left", CssAtRule::TopLeft),
      ("@top-left-corner", CssAtRule::TopLeftCorner),
      ("@top-right", CssAtRule::TopRight),
      ("@top-right-corner", CssAtRule::TopRightCorner),
      ("@view-transition", CssAtRule::ViewTransition),
      ("@when", CssAtRule::When),
    ];
    match ENTRIES.binary_search_by_key(&name, |(n, _)| n) {
      Ok(i) => ENTRIES[i].1.clone(),
      Err(_) => CssAtRule::Unknown(name.to_string()),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtRuleKind {
  Block,
  Statement,
}

impl CssAtRule {
  pub fn kind(&self) -> AtRuleKind {
    match self {
      CssAtRule::WebkitKeyframes
      | CssAtRule::Annotation
      | CssAtRule::BottomCenter
      | CssAtRule::BottomLeft
      | CssAtRule::BottomLeftCorner
      | CssAtRule::BottomRight
      | CssAtRule::BottomRightCorner
      | CssAtRule::CharacterVariant
      | CssAtRule::ColorProfile
      | CssAtRule::Container
      | CssAtRule::Contents
      | CssAtRule::CounterStyle
      | CssAtRule::Else
      | CssAtRule::FontFace
      | CssAtRule::FontFeatureValues
      | CssAtRule::FontPaletteValues
      | CssAtRule::Function
      | CssAtRule::HistoricalForms
      | CssAtRule::Keyframes
      | CssAtRule::LeftBottom
      | CssAtRule::LeftMiddle
      | CssAtRule::LeftTop
      | CssAtRule::Macro
      | CssAtRule::Media
      | CssAtRule::Mixin
      | CssAtRule::Navigation
      | CssAtRule::Ornaments
      | CssAtRule::Page
      | CssAtRule::PositionTry
      | CssAtRule::Property
      | CssAtRule::Result
      | CssAtRule::RightBottom
      | CssAtRule::RightMiddle
      | CssAtRule::RightTop
      | CssAtRule::Route
      | CssAtRule::Scope
      | CssAtRule::Styleset
      | CssAtRule::Stylistic
      | CssAtRule::Supports
      | CssAtRule::SupportsCondition
      | CssAtRule::Swash
      | CssAtRule::TopCenter
      | CssAtRule::TopLeft
      | CssAtRule::TopLeftCorner
      | CssAtRule::TopRight
      | CssAtRule::TopRightCorner
      | CssAtRule::ViewTransition
      | CssAtRule::When => AtRuleKind::Block,
      _ => AtRuleKind::Statement,
    }
  }
}
