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
