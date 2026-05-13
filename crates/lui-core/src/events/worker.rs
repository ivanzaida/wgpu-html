use super::base::EventInit;
use super::message::MessageEventInit;

#[derive(Debug, Clone)]
pub struct CloseEventInit {
    pub base: EventInit,
    pub was_clean: bool, pub code: u16, pub reason: String,
}
impl Default for CloseEventInit {
    fn default() -> Self { Self { base: EventInit::default(), was_clean: false, code: 0, reason: String::new() } }
}

#[derive(Debug, Clone)]
pub struct FetchEventInit { pub base: ExtendableEventInit, pub request: Option<String>, pub client_id: String, pub resulting_client_id: String, pub replaces_client_id: String, pub handled: Option<String>, pub preload_response: Option<String> }
impl Default for FetchEventInit { fn default() -> Self { Self { base: ExtendableEventInit::default(), request: None, client_id: String::new(), resulting_client_id: String::new(), replaces_client_id: String::new(), handled: None, preload_response: None } } }

#[derive(Debug, Clone)]
pub struct ExtendableEventInit { pub base: EventInit }
impl Default for ExtendableEventInit { fn default() -> Self { Self { base: EventInit::default() } } }

#[derive(Debug, Clone)]
pub struct ExtendableMessageEventInit {
    #[allow(dead_code)] pub extendable: ExtendableEventInit,
    pub message: MessageEventInit,
}
impl Default for ExtendableMessageEventInit { fn default() -> Self { Self { extendable: ExtendableEventInit::default(), message: MessageEventInit::default() } } }

#[derive(Debug, Clone)]
pub struct ExtendableCookieChangeEventInit { pub base: ExtendableEventInit, pub changed: Vec<String>, pub deleted: Vec<String> }
impl Default for ExtendableCookieChangeEventInit { fn default() -> Self { Self { base: ExtendableEventInit::default(), changed: Vec::new(), deleted: Vec::new() } } }

#[derive(Debug, Clone)]
pub struct InstallEventInit { pub base: ExtendableEventInit }
impl Default for InstallEventInit { fn default() -> Self { Self { base: ExtendableEventInit::default() } } }

#[derive(Debug, Clone)]
pub struct SyncEventInit { pub base: ExtendableEventInit, pub tag: String, pub last_chance: bool }
impl Default for SyncEventInit { fn default() -> Self { Self { base: ExtendableEventInit::default(), tag: String::new(), last_chance: false } } }

#[derive(Debug, Clone)]
pub struct PeriodicSyncEventInit { pub base: ExtendableEventInit, pub tag: String }
impl Default for PeriodicSyncEventInit { fn default() -> Self { Self { base: ExtendableEventInit::default(), tag: String::new() } } }

#[derive(Debug, Clone)]
pub struct PushEventInit { pub base: ExtendableEventInit, pub data: Option<String> }
impl Default for PushEventInit { fn default() -> Self { Self { base: ExtendableEventInit::default(), data: None } } }

#[derive(Debug, Clone)]
pub struct PushSubscriptionChangeEventInit { pub base: ExtendableEventInit, pub new_subscription: Option<String>, pub old_subscription: Option<String> }
impl Default for PushSubscriptionChangeEventInit { fn default() -> Self { Self { base: ExtendableEventInit::default(), new_subscription: None, old_subscription: None } } }

#[derive(Debug, Clone)]
pub struct NotificationEventInit { pub base: ExtendableEventInit, pub notification: Option<String>, pub action: String }
impl Default for NotificationEventInit { fn default() -> Self { Self { base: ExtendableEventInit::default(), notification: None, action: String::new() } } }

#[derive(Debug, Clone)]
pub struct BackgroundFetchEventInit { pub base: ExtendableEventInit, pub registration: Option<String> }
impl Default for BackgroundFetchEventInit { fn default() -> Self { Self { base: ExtendableEventInit::default(), registration: None } } }

#[derive(Debug, Clone)]
pub struct BackgroundFetchUpdateUIEventInit { pub base: BackgroundFetchEventInit }
impl Default for BackgroundFetchUpdateUIEventInit { fn default() -> Self { Self { base: BackgroundFetchEventInit::default() } } }

#[derive(Debug, Clone)]
pub struct ContentIndexEventInit { pub base: ExtendableEventInit, pub id: String }
impl Default for ContentIndexEventInit { fn default() -> Self { Self { base: ExtendableEventInit::default(), id: String::new() } } }
