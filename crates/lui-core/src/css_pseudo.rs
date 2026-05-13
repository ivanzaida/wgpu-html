// Auto-generated from selectors.json. DO NOT EDIT.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CssPseudo {
    ///
    /// href: https://drafts.csswg.org/css-pseudo-4/#selectordef-after
    /// syntax: ::after
    /// prose: Represents a styleable child pseudo-element immediately after the originating element’s actual content.
    After,
    ///
    /// href: https://drafts.csswg.org/css-position-4/#selectordef-backdrop
    /// syntax: ::backdrop
    /// prose: Each element rendered in the top layer has a ::backdrop pseudo-element, for which it is the originating element.
    Backdrop,
    ///
    /// href: https://drafts.csswg.org/css-pseudo-4/#selectordef-before
    /// syntax: ::before
    /// prose: Represents a styleable child pseudo-element immediately before the originating element’s actual content.
    Before,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-checkmark
    /// syntax: ::checkmark
    /// prose: The ::checkmark pseudo-element represents an indicator of whether the item is checked, and is present on checkboxes, radios, and option elements.
    Checkmark,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-clear-icon
    /// syntax: ::clear-icon
    /// prose: The ::clear-icon pseudo-element represents the portion of the input that allows the user to clear the input when clicked if provided by the user agent. With appearance: textfield, the user agent must not generate this part.
    ClearIcon,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-color-swatch
    /// syntax: ::color-swatch
    /// prose: The ::color-swatch pseudo-element represents the portion of the control that displays the chosen color value.
    ColorSwatch,
    ///
    /// href: https://drafts.csswg.org/css-multicol-2/#selectordef-column
    /// syntax: ::column
    /// prose: The ::column pseudo-element represents the individual columns in a multi-column container. It only exists on multi-column containers.
    Column,
    ///
    /// href: https://w3c.github.io/webvtt/#selectordef-cue
    /// syntax: ::cue
    /// prose: The ::cue pseudo-element (with no argument) matches any list of WebVTT Node Objects constructed for the matched element, with the exception that the properties corresponding to the background shorthand must be applied to the WebVTT cue background box rather than the list of WebVTT Node Objects.
    Cue,
    ///
    /// href: https://w3c.github.io/webvtt/#selectordef-cue-region
    /// syntax: ::cue-region
    /// prose: The ::cue-region pseudo-element (with no argument) matches any list of WebVTT region objects constructed for the matched element.
    CueRegion,
    ///
    /// href: https://w3c.github.io/webvtt/#selectordef-cue-region-selector
    /// syntax: ::cue-region(selector)
    CueRegionFn,
    ///
    /// href: https://w3c.github.io/webvtt/#selectordef-cue-selector
    /// syntax: ::cue(selector)
    CueFn,
    ///
    /// href: https://drafts.csswg.org/css-pseudo-4/#selectordef-details-content
    /// syntax: ::details-content
    /// prose: The ::details-content pseudo-element targets the additional information in a details element that can be expanded or collapsed. It is an element-backed pseudo-element.
    DetailsContent,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-field-component
    /// syntax: ::field-component
    /// prose: The ::field-component pseudo-element represents the portions of the control that contain the date/time component values.
    FieldComponent,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-field-separator
    /// syntax: ::field-separator
    /// prose: The ::field-separator pseudo-element represents the portions of the control that separate the date/time component values if the user agent provides those portions.
    FieldSeparator,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-field-text
    /// syntax: ::field-text
    /// prose: The ::field-text pseudo-element represents the portion of the input that contains the editable text.
    FieldText,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-file-selector-button
    /// syntax: ::file-selector-button
    /// prose: The ::file-selector-button pseudo-element represents the button used to open a file picker, if the UA renders such a button.
    FileSelectorButton,
    ///
    /// href: https://drafts.csswg.org/css-pseudo-4/#selectordef-first-letter
    /// syntax: ::first-letter
    /// prose: The ::first-letter pseudo-element represents the first Letter, Number, or Symbol (Unicode category L*, N*, or S*) typographic character unit on the first formatted line of its originating element (the first letter) as well as its associated punctuation. Collectively, this text is the first-letter text. The ::first-letter pseudo-element can be used to create “initial caps” and “drop caps”, which are common typographic effects.
    FirstLetter,
    ///
    /// href: https://drafts.csswg.org/css-pseudo-4/#selectordef-first-line
    /// syntax: ::first-line
    /// prose: The ::first-line pseudo-element represents the contents of the first formatted line of its originating element.
    FirstLine,
    ///
    /// href: https://drafts.csswg.org/css-pseudo-4/#selectordef-grammar-error
    /// syntax: ::grammar-error
    /// prose: The ::grammar-error pseudo-element represents a portion of text that has been flagged by the user agent as grammatically incorrect.
    GrammarError,
    ///
    /// href: https://drafts.csswg.org/css-pseudo-4/#selectordef-highlight-custom-ident
    /// syntax: ::highlight(<custom-ident>)
    Highlight,
    ///
    /// href: https://drafts.csswg.org/css-pseudo-4/#selectordef-marker
    /// syntax: ::marker
    /// prose: The ::marker pseudo-element represents the automatically generated marker box of a list item. (See [CSS-DISPLAY-3] and [CSS-LISTS-3].)
    Marker,
    ///
    /// href: https://drafts.csswg.org/css-overflow-5/#selectordef-nth-fragment
    /// prose: The ::nth-fragment() pseudo-element is a pseudo-element that describes some of the fragment boxes generated by an element. The argument to the pseudo-element takes the same syntax as the argument to the :nth-child() pseudo-class defined in [SELECT], and has the same meaning except that the number is relative to fragment boxes generated by the element instead of siblings of the element.
    NthFragment,
    ///
    /// href: https://drafts.csswg.org/css-shadow-1/#selectordef-part
    /// syntax: ::part( <ident>+ )
    /// prose: The ::part() pseudo-element allows you to select elements that have been exposed via a part attribute. The syntax is:
    Part,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-picker-icon
    /// syntax: ::picker-icon
    /// prose: The ::picker-icon pseudo-element represents the part of the control that represents the icon denoting the presence of the picker. It is only generated when the originating element has basic appearance and if it opens a picker. It is a fully styleable pseudo-element and inherits from its originating element.
    PickerIcon,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-picker
    /// syntax: ::picker( <form-control-identifier>+ )
    /// prose: The ::picker() pseudo-element represents the part of the form control that pops out of the page.
    Picker,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-placeholder
    /// syntax: ::placeholder
    /// prose: The ::placeholder pseudo-element represents the portion of the input that contains the placeholder text.
    Placeholder,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-reveal-icon
    /// syntax: ::reveal-icon
    /// prose: The ::reveal-icon pseudo-element represents the portion of the input that allows the user to temporarily disable obscuring of sensitive text input when clicked if provided by the user agent. User agents providing ::reveal-icon may choose to remove it in some circumstances to help protect sensitive text input from being revealed unintentionally.
    RevealIcon,
    ///
    /// href: https://drafts.csswg.org/css-overflow-5/#selectordef-scroll-button---scroll-button-direction
    /// syntax: ::scroll-button( '*' | <scroll-button-direction> )
    ScrollButton,
    ///
    /// href: https://drafts.csswg.org/css-overflow-5/#selectordef-scroll-marker
    /// syntax: ::scroll-marker
    /// prose: When the computed content value of a ::scroll-marker pseudo-element is not none and its nearest ancestor scroll container scroll container has a computed scroll-marker-group property that is not none, the pseudo-element generates a box attached as a child of the ::scroll-marker-group pseudo-element’s generated box on its nearest ancestor scroll container. These boxes are added in the tree order of their originating element.
    ScrollMarker,
    ///
    /// href: https://drafts.csswg.org/css-overflow-5/#selectordef-scroll-marker-group
    /// syntax: ::scroll-marker-group
    /// prose: The ::scroll-marker-group fully styleable pseudo-element is generated by a scroll container element having a computed scroll-marker-group property that is not none.
    ScrollMarkerGroup,
    ///
    /// href: https://drafts.csswg.org/css-pseudo-4/#selectordef-search-text
    /// syntax: ::search-text
    /// prose: The ::search-text pseudo-element represents text identified by the user agent’s find-in-page feature. Since not all UAs style matched text in ways expressible with the highlight pseudo-elements, this pseudo-element is optional to implement. The :current pseudo-class (but not ::current()) may be combined with ::search-text to represent the currently focused match instance. The :past and :future pseudo-classes are reserved for analogous use in the future. Any unsupported combination of these pseudo-classes with ::search-text must be treated as invalid.
    SearchText,
    ///
    /// href: https://drafts.csswg.org/css-pseudo-4/#selectordef-selection
    /// syntax: ::selection
    /// prose: The ::selection pseudo-element represents the portion of a document that has been selected as the target or object of some possible future user-agent operation(s). It applies, for example, to selected text within an editable text field, which would be copied by a copy operation or replaced by a paste operation.
    Selection,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-slider-fill
    /// syntax: ::slider-fill
    /// prose: The ::slider-fill pseudo-element represents the portion containing the progressed portion of the control. When the progress of control is indeterminate (like with <progress>), the user agent must give this portion an inline-size of zero.
    SliderFill,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-slider-thumb
    /// syntax: ::slider-thumb
    /// prose: The ::slider-thumb pseudo-element represents the portion that allows the user to adjust the progress of the control.
    SliderThumb,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-slider-track
    /// syntax: ::slider-track
    /// prose: The ::slider-track pseudo-element represents the portion containing both the progressed and unprogressed portions of the control.
    SliderTrack,
    ///
    /// href: https://drafts.csswg.org/css-shadow-1/#selectordef-slotted
    /// prose: The ::slotted() pseudo-element represents the elements assigned, after flattening, to a slot. This pseudo-element only exists on slots.
    Slotted,
    ///
    /// href: https://drafts.csswg.org/css-pseudo-4/#selectordef-spelling-error
    /// syntax: ::spelling-error
    /// prose: The ::spelling-error pseudo-element represents a portion of text that has been flagged by the user agent as misspelled.
    SpellingError,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-step-control
    /// syntax: ::step-control
    /// prose: The ::step-control pseudo-element represents the portion of a number input that contains the up and down buttons.
    StepControl,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-step-down
    /// syntax: ::step-down
    /// prose: The ::step-down pseudo-element represents the button that decrements the value inside a number input when activated.
    StepDown,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-step-up
    /// syntax: ::step-up
    /// prose: The ::step-up pseudo-element represents the button that increments the value inside a number input when activated.
    StepUp,
    ///
    /// href: https://drafts.csswg.org/css-pseudo-4/#selectordef-target-text
    /// syntax: ::target-text
    /// prose: The ::target-text pseudo-element represents text directly targeted by the document URL’s fragment, if any.
    TargetText,
    ///
    /// href: https://drafts.csswg.org/css-view-transitions-2/#selectordef-view-transition
    /// syntax: ::view-transition
    /// prose: The ::view-transition pseudo-element is a tree-abiding pseudo-element that is also a pseudo-element root. Its originating element is the document’s document element, and its containing block is the snapshot containing block.
    ViewTransition,
    ///
    /// href: https://drafts.csswg.org/css-view-transitions-2/#selectordef-view-transition-group-children
    /// prose: The ::view-transition-group-children() pseudo-element is created when mandated by the view-transition-group property on either the element itself or on its contained element. It serves as a container for descendant view-transition-group() pseudo-elements. By default, it is sized to the same size as the corresponding view-transition-group() pseudo element. By default, it has a transparent border that matches the size and shape of the border on the element that caused this structure to be created.
    ViewTransitionGroupChildren,
    ///
    /// href: https://drafts.csswg.org/css-view-transitions-2/#selectordef-view-transition-group
    /// prose: The ::view-transition-group() pseudo-element is a named view transition pseudo-element that represents a matching named view transition capture. A ::view-transition-group() pseudo-element is generated for each view transition name as a child of the ::view-transition pseudo-element, and contains a corresponding ::view-transition-image-pair().
    ViewTransitionGroup,
    ///
    /// href: https://drafts.csswg.org/css-view-transitions-2/#selectordef-view-transition-image-pair
    /// prose: The ::view-transition-image-pair() pseudo-element is a named view transition pseudo-element that represents a pair of corresponding old/new view transition captures. This pseudo-element is a child of the corresponding ::view-transition-group() pseudo-element and contains a corresponding ::view-transition-old() pseudo-element and/or a corresponding ::view-transition-new() pseudo-element (in that order).
    ViewTransitionImagePair,
    ///
    /// href: https://drafts.csswg.org/css-view-transitions-2/#selectordef-view-transition-new
    /// prose: The ::view-transition-new() pseudo-element (like the analogous ::view-transition-old() pseudo-element) is an empty named view transition pseudo-element that represents a visual snapshot of the “new” state as a replaced element; it is omitted if there’s no “new” state to represent. Each ::view-transition-new() pseudo-element is a child of the corresponding ::view-transition-image-pair() pseudo-element.
    ViewTransitionNew,
    ///
    /// href: https://drafts.csswg.org/css-view-transitions-2/#selectordef-view-transition-old
    /// prose: The ::view-transition-old() pseudo-element is an empty named view transition pseudo-element that represents a visual snapshot of the “old” state as a replaced element; it is omitted if there’s no “old” state to represent. Each ::view-transition-old() pseudo-element is a child of the corresponding ::view-transition-image-pair() pseudo-element.
    ViewTransitionOld,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#active-pseudo
    /// syntax: :active
    /// prose: The :active pseudo-class applies while an element is being “activated” by the user, as defined by the host language; for example, while a hyperlink is being triggered.
    Active,
    ///
    /// href: https://drafts.csswg.org/css-view-transitions-2/#active-view-transition-pseudo
    /// syntax: :active-view-transition
    /// prose: The :active-view-transition pseudo-class applies to the root element of the document, if it has an active view transition.
    ActiveViewTransition,
    ///
    /// href: https://drafts.csswg.org/css-view-transitions-2/#active-view-transition-type-pseudo
    /// prose: The :active-view-transition-type() pseudo-class applies to the root element of the document, if it has a matching active view transition. It has the following syntax definition:
    ActiveViewTransitionType,
    ///
    /// href: https://drafts.csswg.org/css2/#selectordef-after
    /// syntax: :after
    AfterLegacy,
    ///
    /// href: https://drafts.csswg.org/css-image-animation-1/#selectordef-animated-image
    /// syntax: :animated-image
    /// prose: The :animated-image pseudo-class represents content image elements where a animated image has been loaded. For the animated-image pseudo-class to match, the image must not only be in a format that is capable of animation, but must also be an actually animated image.
    AnimatedImage,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#any-link-pseudo
    /// syntax: :any-link
    /// prose: The :any-link pseudo-class represents an element that acts as the source anchor of a hyperlink. For example, in [HTML5], any a or area elements with an href attribute are hyperlinks, and thus match :any-link. It matches an element if the element would match either :link or :visited, and is equivalent to :is(:link, :visited).
    AnyLink,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#selectordef-autofill
    /// syntax: :autofill
    /// prose: The :autofill pseudo-class represents input elements that have been automatically filled by the user agent, and have not been subsequently altered by the user.
    Autofill,
    ///
    /// href: https://drafts.csswg.org/css2/#selectordef-before
    /// syntax: :before
    BeforeLegacy,
    ///
    /// href: https://drafts.csswg.org/selectors-5/#blank-pseudo
    /// syntax: :blank
    /// prose: The :blank pseudo-class applies to user-input elements whose input value is empty (consists of the empty string or otherwise null input).
    Blank,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#selectordef-buffering
    /// syntax: :buffering
    /// prose: The :buffering pseudo-class represents an element that is capable of being “played” or “paused”, when that element cannot continue playing because it is actively attempting to obtain media data but has not yet obtained enough data to resume playback. (Note that the element is still considered to be “playing” when it is “buffering”. Whenever :buffering matches an element, :playing also matches the element.)
    Buffering,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#checked-pseudo
    /// syntax: :checked
    /// prose: When such elements are toggled “on” the :checked pseudo-class applies. For example, [HTML5] defines that checked checkboxes, radio buttons, and selected <option> elements match :checked. Similarly, when such elements are toggled “off”, the :unchecked pseudo-class applies.
    Checked,
    ///
    /// href: https://drafts.csswg.org/selectors-5/#current-pseudo
    /// syntax: :current
    /// prose: The :current pseudo-class represents the element, or an ancestor of the element, that is currently being displayed.
    Current,
    ///
    /// href: https://drafts.csswg.org/selectors-5/#selectordef-current
    /// prose: Its alternate form :current() takes a list of compound selectors as its argument: it represents the :current element that matches the argument or, if that does not match, the innermost ancestor of the :current element that does. (If neither the :current element nor its ancestors match the argument, then the selector does not represent anything.)
    CurrentFn,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#default-pseudo
    /// syntax: :default
    /// prose: The :default pseudo-class applies to the one or more UI elements that are the default among a set of similar elements. Typically applies to context menu items, buttons and select lists/menus.
    Default,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#defined-pseudo
    /// syntax: :defined
    /// prose: In some host languages, elements can have a distinction between being “defined”/“constructed” or not. The :defined pseudo-class matches elements that are fully defined, as dictated by the host language.
    Defined,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#dir-pseudo
    /// prose: The :dir() pseudo-class allows the author to write selectors that represent an element based on its directionality as determined by the document language. For example, [HTML5] defines how to determine the directionality of an element, based on a combination of the dir attribute, the surrounding text, and other factors. As another example, the its:dir and dirRule element of the Internationalization Tag Set [ITS20] are able to define the directionality of an element in [XML10].
    Dir,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#disabled-pseudo
    /// syntax: :disabled
    /// prose: Conversely, the :disabled pseudo-class represents user interface elements that are in a disabled state; such elements must have a corresponding enabled state.
    Disabled,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#empty-pseudo
    /// syntax: :empty
    /// prose: The :empty pseudo-class represents an element that has no children except, optionally, document white space characters. In terms of the document tree, only element nodes and content nodes (such as [DOM] text nodes, and entity references) whose data has a non-zero length must be considered as affecting emptiness; comments, processing instructions, and other nodes must not affect whether an element is considered empty or not.
    Empty,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#enabled-pseudo
    /// syntax: :enabled
    /// prose: The :enabled pseudo-class represents user interface elements that are in an enabled state; such elements must have a corresponding disabled state.
    Enabled,
    ///
    /// href: https://drafts.csswg.org/css2/#selectordef-first
    /// syntax: :first
    /// prose: Authors may also specify style for the first page of a document with the :first pseudo-class:
    First,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#first-child-pseudo
    /// syntax: :first-child
    /// prose: The :first-child pseudo-class represents an element that is first among its inclusive siblings. Same as :nth-child(1).
    FirstChild,
    ///
    /// href: https://drafts.csswg.org/css2/#selectordef-first-letter
    /// syntax: :first-letter
    FirstLetterLegacy,
    ///
    /// href: https://drafts.csswg.org/css2/#selectordef-first-line
    /// syntax: :first-line
    FirstLineLegacy,
    ///
    /// href: https://drafts.csswg.org/css-gcpm-4/#selectordef-first-of-page
    /// syntax: :first-of-page
    /// prose: Same as :nth-of-page(n), but where n = 1 (it is the first matched element on the page).
    FirstOfPage,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#first-of-type-pseudo
    /// syntax: :first-of-type
    /// prose: The :first-of-type pseudo-class represents the same element as :nth-of-type(1).
    FirstOfType,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#focus-pseudo
    /// syntax: :focus
    /// prose: The :focus pseudo-class applies while an element (or pseudo-element) has the focus (accepts keyboard or other forms of input).
    Focus,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#focus-visible-pseudo
    /// syntax: :focus-visible
    /// prose: While the :focus pseudo-class always matches the currently-focused element, UAs only sometimes visibly indicate focus (such as by drawing a “focus ring”), instead using a variety of heuristics to visibly indicate the focus only when it would be most helpful to the user. The :focus-visible pseudo-class matches a focused element (or pseudo-element) in these situations only, allowing authors to change the appearance of the focus indicator without changing when a focus indicator appears.
    FocusVisible,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#focus-within-pseudo
    /// syntax: :focus-within
    /// prose: The :focus-within pseudo-class applies to any element (or pseudo-element) for which the :focus pseudo-class applies, as well as to an element (or pseudo-element) whose descendant in the flat tree (including non-element nodes, such as text nodes) matches the conditions for matching :focus.
    FocusWithin,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#selectordef-fullscreen
    /// syntax: :fullscreen
    /// prose: The :fullscreen pseudo-class represents an element which is displayed in a mode that takes up most (usually all) of the screen, such as that defined by the Fullscreen API. [FULLSCREEN]
    Fullscreen,
    ///
    /// href: https://drafts.csswg.org/selectors-5/#future-pseudo
    /// syntax: :future
    /// prose: The :future pseudo-class represents any element that is defined to occur entirely after a :current element. For example, the WebVTT spec defines the :future pseudo-class relative to the current playback position of a media element. If a time-based order of elements is not defined by the document language, then this represents any element that is a (possibly indirect) next sibling of a :current element.
    Future,
    ///
    /// href: https://drafts.csswg.org/css-shadow-1/#selectordef-has-slotted
    /// syntax: :has-slotted
    /// prose: The :has-slotted pseudo-class matches slot elements which have a non-empty list of flattened slotted nodes.
    HasSlotted,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#has-pseudo
    /// prose: The relational pseudo-class, :has(), is a functional pseudo-class taking a <relative-selector-list> as an argument. It represents an element if any of the relative selectors would match at least one element when anchored against this element.
    Has,
    ///
    /// href: https://drafts.csswg.org/selectors-5/#heading-pseudo
    /// syntax: :heading
    /// prose: The (non-functional) :heading pseudo-class matches an element which has a heading level, with respect to the semantics defined by the document language (e.g. [HTML5]).
    Heading,
    ///
    /// href: https://drafts.csswg.org/selectors-5/#heading-functional-pseudo
    /// syntax: :heading( <level># )
    /// prose: As a functional pseudo-class, :heading() notation represents elements that have a heading level among matching any of the provided integer values. The syntax is:
    HeadingFn,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-high-value
    /// syntax: :high-value
    /// prose: The :high-value pseudo-class matches on a meter element when its value is over the value specified by the high HTML attribute.
    HighValue,
    ///
    /// href: https://drafts.csswg.org/css-shadow-1/#selectordef-host
    /// syntax: :host
    /// prose: The :host pseudo-class, when evaluated in the context of a shadow tree, matches the shadow tree’s shadow host. In any other context, it matches nothing.
    Host,
    ///
    /// href: https://drafts.csswg.org/css-shadow-1/#selectordef-host-context
    /// prose: The :host-context() functional pseudo-class tests whether there is an ancestor, outside the shadow tree, which matches a particular selector. Its syntax is:
    HostContext,
    ///
    /// href: https://drafts.csswg.org/css-shadow-1/#selectordef-host-function
    /// prose: The :host() function pseudo-class has the syntax:
    HostFn,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#hover-pseudo
    /// syntax: :hover
    /// prose: The :hover pseudo-class applies while the user designates an element (or pseudo-element) with a pointing device, but does not necessarily activate it. For example, a visual user agent could apply this pseudo-class when the cursor (mouse pointer) hovers over a box generated by the element. Interactive user agents that cannot detect hovering due to hardware limitations (e.g., a pen device that does not detect hovering) are still conforming; the selector will simply never match in such a UA.
    Hover,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#in-range-pseudo
    /// syntax: :in-range
    /// prose: The :in-range and :out-of-range pseudo-classes apply only to elements that have range limitations. An element is :in-range or :out-of-range when the value that the element is bound to is in range or out of range with respect to its range limits as defined by the document language. An element that lacks data range limits or is not a form control is neither :in-range nor :out-of-range. E.g. a slider element with a value of 11 presented as a slider control that only represents the values from 1-10 is :out-of-range. Another example is a menu element with a value of "E" that happens to be presented in a popup menu that only has choices "A", "B" and "C".
    InRange,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#indeterminate-pseudo
    /// syntax: :indeterminate
    /// prose: If an element that could match :checked or :unchecked is neither "on" nor "off", the :indeterminate pseudo-class applies. :indeterminate also matches elements which do not have a notion of being "checked", but whose "value" is still in an indeterminate state, such as a progress meter whose progress percentage is unknown.
    Indeterminate,
    ///
    /// href: https://drafts.csswg.org/selectors-5/#selectordef-interest-source
    /// syntax: :interest-source
    /// prose: The :interest-source pseudo-class applies to an interest source element that the user is currently "showing interest" in, and the :interest-target pseudo-class applies to the associated interest target of an element matching :interest-source.
    InterestSource,
    ///
    /// href: https://drafts.csswg.org/selectors-5/#selectordef-interest-target
    /// syntax: :interest-target
    /// prose: The :interest-source pseudo-class applies to an interest source element that the user is currently "showing interest" in, and the :interest-target pseudo-class applies to the associated interest target of an element matching :interest-source.
    InterestTarget,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#invalid-pseudo
    /// syntax: :invalid
    /// prose: An element is :valid or :invalid when its contents or value is, respectively, valid or invalid with respect to data validity semantics defined by the document language (e.g. [XFORMS11] or [HTML5]). An element which lacks data validity semantics is neither :valid nor :invalid.
    Invalid,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#matches-pseudo
    /// prose: The matches-any pseudo-class, :is(), is a functional pseudo-class taking a <forgiving-selector-list> as its sole argument.
    Is,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#lang-pseudo
    /// prose: If the document language specifies how the (human) content language of an element is determined, it is possible to write selectors that represent an element based on its content language. The :lang() pseudo-class, which accepts a comma-separated list of one or more language ranges, represents an element whose content language is one of the languages listed in its argument. Each language range in :lang() is an extended language range, as defined in BCP 47, and must be a valid CSS <ident> or <string>. (Thus language ranges containing asterisks, for example, must be either correctly escaped or quoted as strings, e.g. :lang(\*-Latn) or :lang("*-Latn").)
    Lang,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#last-child-pseudo
    /// syntax: :last-child
    /// prose: The :last-child pseudo-class represents an element that is last among its inclusive siblings. Same as :nth-last-child(1).
    LastChild,
    ///
    /// href: https://drafts.csswg.org/css-gcpm-4/#selectordef-last-of-page
    /// syntax: :last-of-page
    /// prose: The element is the last matched element on the page.
    LastOfPage,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#last-of-type-pseudo
    /// syntax: :last-of-type
    /// prose: The :last-of-type pseudo-class represents the same element as :nth-last-of-type(1).
    LastOfType,
    ///
    /// href: https://drafts.csswg.org/css2/#selectordef-left
    /// syntax: :left
    /// prose: All pages are automatically classified by user agents into either the :left or :right pseudo-class. Whether the first page of a document is :left or :right depends on the major writing direction of the root element. For example, the first page of a document with a left-to-right major writing direction would be a :right page, and the first page of a document with a right-to-left major writing direction would be a :left page. To explicitly force a document to begin printing on a left or right page, authors can insert a page break before the first generated box.
    Left,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#link-pseudo
    /// syntax: :link
    /// prose: User agents commonly display unvisited hyperlinks differently from previously visited ones. Selectors provides the pseudo-classes :link and :visited to distinguish them:
    Link,
    ///
    /// href: https://drafts.csswg.org/css-navigation-1/#link-to-pseudo
    /// prose: This specification defines a new :link-to() functional pseudo-class that matches link elements that link to a certain URL.
    LinkTo,
    ///
    /// href: https://drafts.csswg.org/selectors-5/#local-link-pseudo
    /// syntax: :local-link
    /// prose: The :local-link pseudo-class allows authors to style hyperlinks based on the users current location within a site and to differentiate site-internal versus site-external links.
    LocalLink,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-low-value
    /// syntax: :low-value
    /// prose: The :low-value pseudo-class matches on a meter element when its value is under the value specified by the low HTML attribute.
    LowValue,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#selectordef-matches
    /// prose: As previous drafts of this specification used the name :matches() for this pseudo-class, UAs may additionally implement this obsolete name as a legacy selector alias for :is() if needed for backwards-compatibility.
    Matches,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#selectordef-modal
    /// syntax: :modal
    /// prose: The :modal pseudo-class represents an element which is in a state that excludes all interaction with elements outside it until it has been dismissed. Multiple elements can be :modal simultaneously, with only one of them active (able to receive input).
    Modal,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#selectordef-muted
    /// syntax: :muted
    /// prose: The :muted pseudo-class represents an element that is capable of making sound, but is currently “muted“ (forced silent). (For the audio and video elements of HTML, see muted. [HTML])
    Muted,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#negation-pseudo
    /// prose: The negation pseudo-class, :not(), is a functional pseudo-class taking a <complex-real-selector-list> as an argument. It represents an element that is not represented by its argument.
    Not,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#nth-child-pseudo
    /// syntax: :nth-child(An+B [of S]? )
    NthChild,
    ///
    /// href: https://drafts.csswg.org/selectors-5/#nth-col-pseudo
    /// syntax: :nth-col(An+B)
    NthCol,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#nth-last-child-pseudo
    /// syntax: :nth-last-child(An+B [of S]? )
    NthLastChild,
    ///
    /// href: https://drafts.csswg.org/selectors-5/#nth-last-col-pseudo
    /// syntax: :nth-last-col(An+B)
    NthLastCol,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#nth-last-of-type-pseudo
    /// syntax: :nth-last-of-type(An+B)
    NthLastOfType,
    ///
    /// href: https://drafts.csswg.org/css-gcpm-4/#selectordef-nth-of-page-n
    /// syntax: :nth-of-page(n)
    NthOfPage,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#nth-of-type-pseudo
    /// syntax: :nth-of-type(An+B)
    NthOfType,
    ///
    /// href: https://drafts.csswg.org/css-gcpm-3/#selectordef-nth
    /// syntax: :nth( An+B [of <custom-ident>]? )
    Nth,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#only-child-pseudo
    /// syntax: :only-child
    /// prose: The :only-child pseudo-class represents an element that has no siblings. Same as :first-child:last-child or :nth-child(1):nth-last-child(1), but with a lower specificity.
    OnlyChild,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#only-of-type-pseudo
    /// syntax: :only-of-type
    /// prose: The :only-of-type pseudo-class represents the same element as :first-of-type:last-of-type.
    OnlyOfType,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#selectordef-open
    /// syntax: :open
    /// prose: The :open pseudo-class represents an element that has both “open” and “closed” states, and which is currently in the “open” state.
    Open,
    ///
    /// href: https://drafts.csswg.org/css-forms-1/#selectordef-optimal-value
    /// syntax: :optimal-value
    /// prose: The :optimal-value pseudo-class matches on a meter element when its value is in the range determined by the optimum / low / high HTML attributes.
    OptimalValue,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#optional-pseudo
    /// syntax: :optional
    /// prose: A form element is :required or :optional if a value for it is, respectively, required or optional before the form it belongs to can be validly submitted. Elements that are not form elements are neither required nor optional.
    Optional,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#out-of-range-pseudo
    /// syntax: :out-of-range
    /// prose: The :in-range and :out-of-range pseudo-classes apply only to elements that have range limitations. An element is :in-range or :out-of-range when the value that the element is bound to is in range or out of range with respect to its range limits as defined by the document language. An element that lacks data range limits or is not a form control is neither :in-range nor :out-of-range. E.g. a slider element with a value of 11 presented as a slider control that only represents the values from 1-10 is :out-of-range. Another example is a menu element with a value of "E" that happens to be presented in a popup menu that only has choices "A", "B" and "C".
    OutOfRange,
    ///
    /// href: https://drafts.csswg.org/selectors-5/#past-pseudo
    /// syntax: :past
    /// prose: The :past pseudo-class represents any element that is defined to occur entirely prior to a :current element. For example, the WebVTT spec defines the :past pseudo-class relative to the current playback position of a media element. If a time-based order of elements is not defined by the document language, then this represents any element that is a (possibly indirect) previous sibling of a :current element.
    Past,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#selectordef-paused
    /// syntax: :paused
    /// prose: The :paused pseudo-class represents an element that is capable of being “played” or “paused”, when that element is “paused” (i.e. not ”playing”). (This includes both an explicit “paused” state, and other non-playing states like “loaded, hasn’t been activated yet”, etc.)
    Paused,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#selectordef-picture-in-picture
    /// syntax: :picture-in-picture
    /// prose: The :picture-in-picture pseudo-class represents an element which is displayed in a mode that takes up most (usually all) of the viewport, and whose viewport is confined to part of the screen while being displayed over other content, for example when using the Picture-in-Picture API. [picture-in-picture]
    PictureInPicture,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#placeholder-shown-pseudo
    /// syntax: :placeholder-shown
    /// prose: Input elements can sometimes show placeholder text as a hint to the user on what to type in. See, for example, the placeholder attribute in [HTML5]. The :placeholder-shown pseudo-class matches an input element that is showing such placeholder text, whether that text is given by an attribute or a real element, or is otherwise implied by the UA.
    PlaceholderShown,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#selectordef-playing
    /// syntax: :playing
    /// prose: The :playing pseudo-class represents an element that is capable of being “played” or “paused”, when that element is “playing”. (This includes both when the element is explicitly playing, and when it’s temporarily stopped for some reason not connected to user intent, but will automatically resume when that reason is resolved, such as a “buffering” or “stalled” state.)
    Playing,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#selectordef-popover-open
    /// syntax: :popover-open
    /// prose: The :popover-open pseudo-class represents an element that has both “popover-showing” and “popover-hidden” states and which is currently in the “popover-showing” state.
    PopoverOpen,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#read-only-pseudo
    /// syntax: :read-only
    /// prose: An element matches :read-write if it is user-alterable, as defined by the document language. Otherwise, it is :read-only.
    ReadOnly,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#read-write-pseudo
    /// syntax: :read-write
    /// prose: An element matches :read-write if it is user-alterable, as defined by the document language. Otherwise, it is :read-only.
    ReadWrite,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#required-pseudo
    /// syntax: :required
    /// prose: A form element is :required or :optional if a value for it is, respectively, required or optional before the form it belongs to can be validly submitted. Elements that are not form elements are neither required nor optional.
    Required,
    ///
    /// href: https://drafts.csswg.org/css2/#selectordef-right
    /// syntax: :right
    /// prose: All pages are automatically classified by user agents into either the :left or :right pseudo-class. Whether the first page of a document is :left or :right depends on the major writing direction of the root element. For example, the first page of a document with a left-to-right major writing direction would be a :right page, and the first page of a document with a right-to-left major writing direction would be a :left page. To explicitly force a document to begin printing on a left or right page, authors can insert a page break before the first generated box.
    Right,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#root-pseudo
    /// syntax: :root
    /// prose: The :root pseudo-class represents an element that is the root of the document.
    Root,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#scope-pseudo
    /// syntax: :scope
    /// prose: In some contexts, selectors are matched with respect to one or more scoping roots, such as when calling the querySelector() method in [DOM]. The :scope pseudo-class represents this scoping root, and may be either a true element or a virtual one (such as a DocumentFragment).
    Scope,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#selectordef-seeking
    /// syntax: :seeking
    /// prose: The :seeking pseudo-class represents an element that is capable of ”seeking” when that element is ”seeking”. (For the audio and video elements of HTML, see HTML § 4.8.11.9 Seeking.)
    Seeking,
    ///
    /// href: https://drafts.csswg.org/css-scroll-snap-2/#selectordef-snapped
    /// syntax: :snapped
    /// prose: The :snapped pseudo-class matches any snap targets, regardless of axis. The longform physical and logical pseudo-class selectors allow for more finite snapped children styling as they can target an individual axis.
    Snapped,
    ///
    /// href: https://drafts.csswg.org/css-scroll-snap-2/#selectordef-snapped-block
    /// syntax: :snapped-block
    /// prose: Matches the child snapped on the block axis.
    SnappedBlock,
    ///
    /// href: https://drafts.csswg.org/css-scroll-snap-2/#selectordef-snapped-inline
    /// syntax: :snapped-inline
    /// prose: Matches the child snapped on the inline axis.
    SnappedInline,
    ///
    /// href: https://drafts.csswg.org/css-scroll-snap-2/#selectordef-snapped-x
    /// syntax: :snapped-x
    /// prose: Matches the child snapped on the horizontal axis.
    SnappedX,
    ///
    /// href: https://drafts.csswg.org/css-scroll-snap-2/#selectordef-snapped-y
    /// syntax: :snapped-y
    /// prose: Matches the child snapped on the vertical axis.
    SnappedY,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#selectordef-stalled
    /// syntax: :stalled
    /// prose: The :stalled pseudo-class represents an element when that element cannot continue playing because it is actively attempting to obtain media data but it has failed to receive any data for some amount of time. For the audio and video elements of HTML, this amount of time is the media element stall timeout. [HTML] (Note that, like with the :buffering pseudo-class, the element is still considered to be “playing” when it is “stalled”. Whenever :stalled matches an element, :playing also matches the element.)
    Stalled,
    ///
    /// href: https://drafts.csswg.org/css-gcpm-4/#selectordef-start-of-page
    /// syntax: :start-of-page
    /// prose: The element is the first matched element on the page, and neither it nor its ancestors have any previous siblings that appear on the page.
    StartOfPage,
    ///
    /// href: https://drafts.csswg.org/selectors-5/#state-pseudo
    /// syntax: :state( <ident> )
    State,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#target-pseudo
    /// syntax: :target
    /// prose: The :target pseudo-class matches the document’s target elements. If the document’s URL has no fragment identifier, then the document has no target elements.
    Target,
    ///
    /// href: https://drafts.csswg.org/css-overflow-5/#selectordef-target-after
    /// syntax: :target-after
    /// prose: In addition to the :target-current pseudo-class, this specification introduces the :target-before and :target-after pseudo-classes for use with scroll marker elements.
    TargetAfter,
    ///
    /// href: https://drafts.csswg.org/css-overflow-5/#selectordef-target-before
    /// syntax: :target-before
    /// prose: In addition to the :target-current pseudo-class, this specification introduces the :target-before and :target-after pseudo-classes for use with scroll marker elements.
    TargetBefore,
    ///
    /// href: https://drafts.csswg.org/css-overflow-5/#selectordef-target-current
    /// syntax: :target-current
    /// prose: Exactly one scroll marker within each scroll marker group is determined to be active at a time. Such "active" scroll markers match the :target-current pseudo-class.
    TargetCurrent,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#unchecked-pseudo
    /// syntax: :unchecked
    /// prose: When such elements are toggled “on” the :checked pseudo-class applies. For example, [HTML5] defines that checked checkboxes, radio buttons, and selected <option> elements match :checked. Similarly, when such elements are toggled “off”, the :unchecked pseudo-class applies.
    Unchecked,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#user-invalid-pseudo
    /// syntax: :user-invalid
    /// prose: The :user-invalid and the :user-valid pseudo-classes represent an element with incorrect or correct input, respectively, but only after the user has significantly interacted with it. Their purpose is to help the user identify mistakes in their input.
    UserInvalid,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#user-valid-pseudo
    /// syntax: :user-valid
    /// prose: The :user-invalid and the :user-valid pseudo-classes represent an element with incorrect or correct input, respectively, but only after the user has significantly interacted with it. Their purpose is to help the user identify mistakes in their input.
    UserValid,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#valid-pseudo
    /// syntax: :valid
    /// prose: An element is :valid or :invalid when its contents or value is, respectively, valid or invalid with respect to data validity semantics defined by the document language (e.g. [XFORMS11] or [HTML5]). An element which lacks data validity semantics is neither :valid nor :invalid.
    Valid,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#visited-pseudo
    /// syntax: :visited
    /// prose: User agents commonly display unvisited hyperlinks differently from previously visited ones. Selectors provides the pseudo-classes :link and :visited to distinguish them:
    Visited,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#selectordef-volume-locked
    /// syntax: :volume-locked
    /// prose: The :volume-locked pseudo-class represents an element that is capable of making sound, and currently has its volume "locked" by the UA or the user, so the page author cannot change it. (For the audio and video elements of HTML, see the algorithm for setting the element’s effective media volume. [HTML])
    VolumeLocked,
    ///
    /// href: https://drafts.csswg.org/selectors-4/#where-pseudo
    /// prose: The Specificity-adjustment pseudo-class, :where(), is a functional pseudo-class with the same syntax and functionality as :is(). Unlike :is(), neither the :where() pseudo-class, nor any of its arguments, contribute to the specificity of the selector—​its specificity is always zero.
    Where,
    ///
    /// href: https://immersive-web.github.io/dom-overlays/#selectordef-xr-overlay
    /// syntax: :xr-overlay
    /// prose: The :xr-overlay pseudo-class MUST match the overlay element for the duration of an immersive session using a DOM Overlay.
    XrOverlay,
    ///
    /// href: https://drafts.csswg.org/css-nesting-1/#selectordef-
    /// syntax: '&'
    /// prose: When using a nested style rule, one must be able to refer to the elements matched by the parent rule; that is, after all, the entire point of nesting. To accomplish that, this specification defines a new selector, the nesting selector, written as & (U+0026 AMPERSAND).
    Ampersand,
    /// An unknown pseudo-class or pseudo-element.
    Unknown(String),
}

impl CssPseudo {
    pub fn name(&self) -> &'static str { match self {
            CssPseudo::After => "::after",
            CssPseudo::Backdrop => "::backdrop",
            CssPseudo::Before => "::before",
            CssPseudo::Checkmark => "::checkmark",
            CssPseudo::ClearIcon => "::clear-icon",
            CssPseudo::ColorSwatch => "::color-swatch",
            CssPseudo::Column => "::column",
            CssPseudo::Cue => "::cue",
            CssPseudo::CueRegion => "::cue-region",
            CssPseudo::CueRegionFn => "::cue-region()",
            CssPseudo::CueFn => "::cue()",
            CssPseudo::DetailsContent => "::details-content",
            CssPseudo::FieldComponent => "::field-component",
            CssPseudo::FieldSeparator => "::field-separator",
            CssPseudo::FieldText => "::field-text",
            CssPseudo::FileSelectorButton => "::file-selector-button",
            CssPseudo::FirstLetter => "::first-letter",
            CssPseudo::FirstLine => "::first-line",
            CssPseudo::GrammarError => "::grammar-error",
            CssPseudo::Highlight => "::highlight()",
            CssPseudo::Marker => "::marker",
            CssPseudo::NthFragment => "::nth-fragment()",
            CssPseudo::Part => "::part()",
            CssPseudo::PickerIcon => "::picker-icon",
            CssPseudo::Picker => "::picker()",
            CssPseudo::Placeholder => "::placeholder",
            CssPseudo::RevealIcon => "::reveal-icon",
            CssPseudo::ScrollButton => "::scroll-button()",
            CssPseudo::ScrollMarker => "::scroll-marker",
            CssPseudo::ScrollMarkerGroup => "::scroll-marker-group",
            CssPseudo::SearchText => "::search-text",
            CssPseudo::Selection => "::selection",
            CssPseudo::SliderFill => "::slider-fill",
            CssPseudo::SliderThumb => "::slider-thumb",
            CssPseudo::SliderTrack => "::slider-track",
            CssPseudo::Slotted => "::slotted()",
            CssPseudo::SpellingError => "::spelling-error",
            CssPseudo::StepControl => "::step-control",
            CssPseudo::StepDown => "::step-down",
            CssPseudo::StepUp => "::step-up",
            CssPseudo::TargetText => "::target-text",
            CssPseudo::ViewTransition => "::view-transition",
            CssPseudo::ViewTransitionGroupChildren => "::view-transition-group-children()",
            CssPseudo::ViewTransitionGroup => "::view-transition-group()",
            CssPseudo::ViewTransitionImagePair => "::view-transition-image-pair()",
            CssPseudo::ViewTransitionNew => "::view-transition-new()",
            CssPseudo::ViewTransitionOld => "::view-transition-old()",
            CssPseudo::Active => ":active",
            CssPseudo::ActiveViewTransition => ":active-view-transition",
            CssPseudo::ActiveViewTransitionType => ":active-view-transition-type()",
            CssPseudo::AfterLegacy => ":after",
            CssPseudo::AnimatedImage => ":animated-image",
            CssPseudo::AnyLink => ":any-link",
            CssPseudo::Autofill => ":autofill",
            CssPseudo::BeforeLegacy => ":before",
            CssPseudo::Blank => ":blank",
            CssPseudo::Buffering => ":buffering",
            CssPseudo::Checked => ":checked",
            CssPseudo::Current => ":current",
            CssPseudo::CurrentFn => ":current()",
            CssPseudo::Default => ":default",
            CssPseudo::Defined => ":defined",
            CssPseudo::Dir => ":dir()",
            CssPseudo::Disabled => ":disabled",
            CssPseudo::Empty => ":empty",
            CssPseudo::Enabled => ":enabled",
            CssPseudo::First => ":first",
            CssPseudo::FirstChild => ":first-child",
            CssPseudo::FirstLetterLegacy => ":first-letter",
            CssPseudo::FirstLineLegacy => ":first-line",
            CssPseudo::FirstOfPage => ":first-of-page",
            CssPseudo::FirstOfType => ":first-of-type",
            CssPseudo::Focus => ":focus",
            CssPseudo::FocusVisible => ":focus-visible",
            CssPseudo::FocusWithin => ":focus-within",
            CssPseudo::Fullscreen => ":fullscreen",
            CssPseudo::Future => ":future",
            CssPseudo::HasSlotted => ":has-slotted",
            CssPseudo::Has => ":has()",
            CssPseudo::Heading => ":heading",
            CssPseudo::HeadingFn => ":heading()",
            CssPseudo::HighValue => ":high-value",
            CssPseudo::Host => ":host",
            CssPseudo::HostContext => ":host-context()",
            CssPseudo::HostFn => ":host()",
            CssPseudo::Hover => ":hover",
            CssPseudo::InRange => ":in-range",
            CssPseudo::Indeterminate => ":indeterminate",
            CssPseudo::InterestSource => ":interest-source",
            CssPseudo::InterestTarget => ":interest-target",
            CssPseudo::Invalid => ":invalid",
            CssPseudo::Is => ":is()",
            CssPseudo::Lang => ":lang()",
            CssPseudo::LastChild => ":last-child",
            CssPseudo::LastOfPage => ":last-of-page",
            CssPseudo::LastOfType => ":last-of-type",
            CssPseudo::Left => ":left",
            CssPseudo::Link => ":link",
            CssPseudo::LinkTo => ":link-to()",
            CssPseudo::LocalLink => ":local-link",
            CssPseudo::LowValue => ":low-value",
            CssPseudo::Matches => ":matches()",
            CssPseudo::Modal => ":modal",
            CssPseudo::Muted => ":muted",
            CssPseudo::Not => ":not()",
            CssPseudo::NthChild => ":nth-child()",
            CssPseudo::NthCol => ":nth-col()",
            CssPseudo::NthLastChild => ":nth-last-child()",
            CssPseudo::NthLastCol => ":nth-last-col()",
            CssPseudo::NthLastOfType => ":nth-last-of-type()",
            CssPseudo::NthOfPage => ":nth-of-page()",
            CssPseudo::NthOfType => ":nth-of-type()",
            CssPseudo::Nth => ":nth()",
            CssPseudo::OnlyChild => ":only-child",
            CssPseudo::OnlyOfType => ":only-of-type",
            CssPseudo::Open => ":open",
            CssPseudo::OptimalValue => ":optimal-value",
            CssPseudo::Optional => ":optional",
            CssPseudo::OutOfRange => ":out-of-range",
            CssPseudo::Past => ":past",
            CssPseudo::Paused => ":paused",
            CssPseudo::PictureInPicture => ":picture-in-picture",
            CssPseudo::PlaceholderShown => ":placeholder-shown",
            CssPseudo::Playing => ":playing",
            CssPseudo::PopoverOpen => ":popover-open",
            CssPseudo::ReadOnly => ":read-only",
            CssPseudo::ReadWrite => ":read-write",
            CssPseudo::Required => ":required",
            CssPseudo::Right => ":right",
            CssPseudo::Root => ":root",
            CssPseudo::Scope => ":scope",
            CssPseudo::Seeking => ":seeking",
            CssPseudo::Snapped => ":snapped",
            CssPseudo::SnappedBlock => ":snapped-block",
            CssPseudo::SnappedInline => ":snapped-inline",
            CssPseudo::SnappedX => ":snapped-x",
            CssPseudo::SnappedY => ":snapped-y",
            CssPseudo::Stalled => ":stalled",
            CssPseudo::StartOfPage => ":start-of-page",
            CssPseudo::State => ":state()",
            CssPseudo::Target => ":target",
            CssPseudo::TargetAfter => ":target-after",
            CssPseudo::TargetBefore => ":target-before",
            CssPseudo::TargetCurrent => ":target-current",
            CssPseudo::Unchecked => ":unchecked",
            CssPseudo::UserInvalid => ":user-invalid",
            CssPseudo::UserValid => ":user-valid",
            CssPseudo::Valid => ":valid",
            CssPseudo::Visited => ":visited",
            CssPseudo::VolumeLocked => ":volume-locked",
            CssPseudo::Where => ":where()",
            CssPseudo::XrOverlay => ":xr-overlay",
            CssPseudo::Ampersand => "&",
            CssPseudo::Unknown(_) => "",
        }
    }

    pub fn from_name(name: &str) -> Self {
        const ENTRIES: &[(&str, CssPseudo)] = &[
            ("::after", CssPseudo::After),
            ("::backdrop", CssPseudo::Backdrop),
            ("::before", CssPseudo::Before),
            ("::checkmark", CssPseudo::Checkmark),
            ("::clear-icon", CssPseudo::ClearIcon),
            ("::color-swatch", CssPseudo::ColorSwatch),
            ("::column", CssPseudo::Column),
            ("::cue", CssPseudo::Cue),
            ("::cue()", CssPseudo::CueFn),
            ("::cue-region", CssPseudo::CueRegion),
            ("::cue-region()", CssPseudo::CueRegionFn),
            ("::details-content", CssPseudo::DetailsContent),
            ("::field-component", CssPseudo::FieldComponent),
            ("::field-separator", CssPseudo::FieldSeparator),
            ("::field-text", CssPseudo::FieldText),
            ("::file-selector-button", CssPseudo::FileSelectorButton),
            ("::first-letter", CssPseudo::FirstLetter),
            ("::first-line", CssPseudo::FirstLine),
            ("::grammar-error", CssPseudo::GrammarError),
            ("::highlight()", CssPseudo::Highlight),
            ("::marker", CssPseudo::Marker),
            ("::nth-fragment()", CssPseudo::NthFragment),
            ("::part()", CssPseudo::Part),
            ("::picker()", CssPseudo::Picker),
            ("::picker-icon", CssPseudo::PickerIcon),
            ("::placeholder", CssPseudo::Placeholder),
            ("::reveal-icon", CssPseudo::RevealIcon),
            ("::scroll-button()", CssPseudo::ScrollButton),
            ("::scroll-marker", CssPseudo::ScrollMarker),
            ("::scroll-marker-group", CssPseudo::ScrollMarkerGroup),
            ("::search-text", CssPseudo::SearchText),
            ("::selection", CssPseudo::Selection),
            ("::slider-fill", CssPseudo::SliderFill),
            ("::slider-thumb", CssPseudo::SliderThumb),
            ("::slider-track", CssPseudo::SliderTrack),
            ("::slotted()", CssPseudo::Slotted),
            ("::spelling-error", CssPseudo::SpellingError),
            ("::step-control", CssPseudo::StepControl),
            ("::step-down", CssPseudo::StepDown),
            ("::step-up", CssPseudo::StepUp),
            ("::target-text", CssPseudo::TargetText),
            ("::view-transition", CssPseudo::ViewTransition),
            ("::view-transition-group()", CssPseudo::ViewTransitionGroup),
            ("::view-transition-group-children()", CssPseudo::ViewTransitionGroupChildren),
            ("::view-transition-image-pair()", CssPseudo::ViewTransitionImagePair),
            ("::view-transition-new()", CssPseudo::ViewTransitionNew),
            ("::view-transition-old()", CssPseudo::ViewTransitionOld),
            (":active", CssPseudo::Active),
            (":active-view-transition", CssPseudo::ActiveViewTransition),
            (":active-view-transition-type()", CssPseudo::ActiveViewTransitionType),
            (":after", CssPseudo::AfterLegacy),
            (":animated-image", CssPseudo::AnimatedImage),
            (":any-link", CssPseudo::AnyLink),
            (":autofill", CssPseudo::Autofill),
            (":before", CssPseudo::BeforeLegacy),
            (":blank", CssPseudo::Blank),
            (":buffering", CssPseudo::Buffering),
            (":checked", CssPseudo::Checked),
            (":current", CssPseudo::Current),
            (":current()", CssPseudo::CurrentFn),
            (":default", CssPseudo::Default),
            (":defined", CssPseudo::Defined),
            (":dir()", CssPseudo::Dir),
            (":disabled", CssPseudo::Disabled),
            (":empty", CssPseudo::Empty),
            (":enabled", CssPseudo::Enabled),
            (":first", CssPseudo::First),
            (":first-child", CssPseudo::FirstChild),
            (":first-letter", CssPseudo::FirstLetterLegacy),
            (":first-line", CssPseudo::FirstLineLegacy),
            (":first-of-page", CssPseudo::FirstOfPage),
            (":first-of-type", CssPseudo::FirstOfType),
            (":focus", CssPseudo::Focus),
            (":focus-visible", CssPseudo::FocusVisible),
            (":focus-within", CssPseudo::FocusWithin),
            (":fullscreen", CssPseudo::Fullscreen),
            (":future", CssPseudo::Future),
            (":has()", CssPseudo::Has),
            (":has-slotted", CssPseudo::HasSlotted),
            (":heading", CssPseudo::Heading),
            (":heading()", CssPseudo::HeadingFn),
            (":high-value", CssPseudo::HighValue),
            (":host", CssPseudo::Host),
            (":host()", CssPseudo::HostFn),
            (":host-context()", CssPseudo::HostContext),
            (":hover", CssPseudo::Hover),
            (":in-range", CssPseudo::InRange),
            (":indeterminate", CssPseudo::Indeterminate),
            (":interest-source", CssPseudo::InterestSource),
            (":interest-target", CssPseudo::InterestTarget),
            (":invalid", CssPseudo::Invalid),
            (":is()", CssPseudo::Is),
            (":lang()", CssPseudo::Lang),
            (":last-child", CssPseudo::LastChild),
            (":last-of-page", CssPseudo::LastOfPage),
            (":last-of-type", CssPseudo::LastOfType),
            (":left", CssPseudo::Left),
            (":link", CssPseudo::Link),
            (":link-to()", CssPseudo::LinkTo),
            (":local-link", CssPseudo::LocalLink),
            (":low-value", CssPseudo::LowValue),
            (":matches()", CssPseudo::Matches),
            (":modal", CssPseudo::Modal),
            (":muted", CssPseudo::Muted),
            (":not()", CssPseudo::Not),
            (":nth()", CssPseudo::Nth),
            (":nth-child()", CssPseudo::NthChild),
            (":nth-col()", CssPseudo::NthCol),
            (":nth-last-child()", CssPseudo::NthLastChild),
            (":nth-last-col()", CssPseudo::NthLastCol),
            (":nth-last-of-type()", CssPseudo::NthLastOfType),
            (":nth-of-page()", CssPseudo::NthOfPage),
            (":nth-of-type()", CssPseudo::NthOfType),
            (":only-child", CssPseudo::OnlyChild),
            (":only-of-type", CssPseudo::OnlyOfType),
            (":open", CssPseudo::Open),
            (":optimal-value", CssPseudo::OptimalValue),
            (":optional", CssPseudo::Optional),
            (":out-of-range", CssPseudo::OutOfRange),
            (":past", CssPseudo::Past),
            (":paused", CssPseudo::Paused),
            (":picture-in-picture", CssPseudo::PictureInPicture),
            (":placeholder-shown", CssPseudo::PlaceholderShown),
            (":playing", CssPseudo::Playing),
            (":popover-open", CssPseudo::PopoverOpen),
            (":read-only", CssPseudo::ReadOnly),
            (":read-write", CssPseudo::ReadWrite),
            (":required", CssPseudo::Required),
            (":right", CssPseudo::Right),
            (":root", CssPseudo::Root),
            (":scope", CssPseudo::Scope),
            (":seeking", CssPseudo::Seeking),
            (":snapped", CssPseudo::Snapped),
            (":snapped-block", CssPseudo::SnappedBlock),
            (":snapped-inline", CssPseudo::SnappedInline),
            (":snapped-x", CssPseudo::SnappedX),
            (":snapped-y", CssPseudo::SnappedY),
            (":stalled", CssPseudo::Stalled),
            (":start-of-page", CssPseudo::StartOfPage),
            (":state()", CssPseudo::State),
            (":target", CssPseudo::Target),
            (":target-after", CssPseudo::TargetAfter),
            (":target-before", CssPseudo::TargetBefore),
            (":target-current", CssPseudo::TargetCurrent),
            (":unchecked", CssPseudo::Unchecked),
            (":user-invalid", CssPseudo::UserInvalid),
            (":user-valid", CssPseudo::UserValid),
            (":valid", CssPseudo::Valid),
            (":visited", CssPseudo::Visited),
            (":volume-locked", CssPseudo::VolumeLocked),
            (":where()", CssPseudo::Where),
            (":xr-overlay", CssPseudo::XrOverlay),
            ("&", CssPseudo::Ampersand),
        ];
        match ENTRIES.binary_search_by_key(&name, |(n, _)| n) {
            Ok(i) => ENTRIES[i].1.clone(),
            Err(_) => CssPseudo::Unknown(name.to_string()),
        }
    }
}
