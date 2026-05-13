use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct GamepadEventInit { pub base: EventInit, pub gamepad: Option<String> }
impl Default for GamepadEventInit { fn default() -> Self { Self { base: EventInit::default(), gamepad: None } } }

#[derive(Debug, Clone)]
pub struct MediaStreamTrackEventInit { pub base: EventInit, pub track: Option<String> }
impl Default for MediaStreamTrackEventInit { fn default() -> Self { Self { base: EventInit::default(), track: None } } }

#[derive(Debug, Clone)]
pub struct MediaEncryptedEventInit { pub base: EventInit, pub init_data_type: String, pub init_data: Option<String> }
impl Default for MediaEncryptedEventInit { fn default() -> Self { Self { base: EventInit::default(), init_data_type: String::new(), init_data: None } } }

#[derive(Debug, Clone)]
pub struct MediaKeyMessageEventInit { pub base: EventInit, pub message_type: String, pub message: Option<String> }
impl Default for MediaKeyMessageEventInit { fn default() -> Self { Self { base: EventInit::default(), message_type: String::new(), message: None } } }

#[derive(Debug, Clone)]
pub struct MediaQueryListEventInit { pub base: EventInit, pub media: String, pub matches: bool }
impl Default for MediaQueryListEventInit { fn default() -> Self { Self { base: EventInit::default(), media: String::new(), matches: false } } }

#[derive(Debug, Clone)]
pub struct PictureInPictureEventInit { pub base: EventInit, pub picture_in_picture_window: Option<String> }
impl Default for PictureInPictureEventInit { fn default() -> Self { Self { base: EventInit::default(), picture_in_picture_window: None } } }

#[derive(Debug, Clone)]
pub struct DocumentPictureInPictureEventInit { pub base: EventInit, pub window: Option<String> }
impl Default for DocumentPictureInPictureEventInit { fn default() -> Self { Self { base: EventInit::default(), window: None } } }

#[derive(Debug, Clone)]
pub struct PortalActivateEventInit { pub base: EventInit, pub data: Option<String> }
impl Default for PortalActivateEventInit { fn default() -> Self { Self { base: EventInit::default(), data: None } } }

#[derive(Debug, Clone)]
pub struct PageTransitionEventInit { pub base: EventInit, pub persisted: bool }
impl Default for PageTransitionEventInit { fn default() -> Self { Self { base: EventInit::default(), persisted: false } } }

#[derive(Debug, Clone)]
pub struct PageRevealEventInit { pub base: EventInit }
impl Default for PageRevealEventInit { fn default() -> Self { Self { base: EventInit::default() } } }

#[derive(Debug, Clone)]
pub struct PageSwapEventInit { pub base: EventInit, pub activation: Option<String>, pub view_transition: Option<String> }
impl Default for PageSwapEventInit { fn default() -> Self { Self { base: EventInit::default(), activation: None, view_transition: None } } }

#[derive(Debug, Clone)]
pub struct NavigateEventInit { pub base: EventInit, pub navigation_type: String, pub destination: Option<String>, pub can_intercept: bool, pub user_initiated: bool, pub hash_change: bool, pub download_request: Option<String>, pub form_data: Option<String>, pub info: Option<String> }
impl Default for NavigateEventInit { fn default() -> Self { Self { base: EventInit::default(), navigation_type: String::new(), destination: None, can_intercept: false, user_initiated: false, hash_change: false, download_request: None, form_data: None, info: None } } }

#[derive(Debug, Clone)]
pub struct NavigationCurrentEntryChangeEventInit { pub base: EventInit, pub navigation_type: String, pub from: Option<String> }
impl Default for NavigationCurrentEntryChangeEventInit { fn default() -> Self { Self { base: EventInit::default(), navigation_type: String::new(), from: None } } }

#[derive(Debug, Clone)]
pub struct BeforeUnloadEventInit { pub base: EventInit }
impl Default for BeforeUnloadEventInit { fn default() -> Self { Self { base: EventInit::default() } } }

#[derive(Debug, Clone)]
pub struct BeforeInstallPromptEventInit { pub base: EventInit, pub platforms: Vec<String> }
impl Default for BeforeInstallPromptEventInit { fn default() -> Self { Self { base: EventInit::default(), platforms: Vec::new() } } }

#[derive(Debug, Clone)]
pub struct SecurityPolicyViolationEventInit { pub base: EventInit, pub document_uri: String, pub referrer: String, pub blocked_uri: String, pub violated_directive: String, pub effective_directive: String, pub original_policy: String, pub disposition: String, pub source_file: String, pub status_code: u16, pub line_number: u32, pub column_number: u32 }
impl Default for SecurityPolicyViolationEventInit { fn default() -> Self { Self { base: EventInit::default(), document_uri: String::new(), referrer: String::new(), blocked_uri: String::new(), violated_directive: String::new(), effective_directive: String::new(), original_policy: String::new(), disposition: String::new(), source_file: String::new(), status_code: 0, line_number: 0, column_number: 0 } } }

#[derive(Debug, Clone)]
pub struct PromiseRejectionEventInit { pub base: EventInit, pub promise: Option<String>, pub reason: Option<String> }
impl Default for PromiseRejectionEventInit { fn default() -> Self { Self { base: EventInit::default(), promise: None, reason: None } } }

#[derive(Debug, Clone)]
pub struct TaskPriorityChangeEventInit { pub base: EventInit, pub previous_priority: String }
impl Default for TaskPriorityChangeEventInit { fn default() -> Self { Self { base: EventInit::default(), previous_priority: String::new() } } }
