use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct RTCDataChannelEventInit { pub base: EventInit, pub channel: Option<String> }
impl Default for RTCDataChannelEventInit { fn default() -> Self { Self { base: EventInit::default(), channel: None } } }

#[derive(Debug, Clone)]
pub struct RTCDTMFToneChangeEventInit { pub base: EventInit, pub tone: String }
impl Default for RTCDTMFToneChangeEventInit { fn default() -> Self { Self { base: EventInit::default(), tone: String::new() } } }

#[derive(Debug, Clone)]
pub struct RTCErrorEventInit { pub base: EventInit, pub error: Option<String> }
impl Default for RTCErrorEventInit { fn default() -> Self { Self { base: EventInit::default(), error: None } } }

#[derive(Debug, Clone)]
pub struct RTCPeerConnectionIceEventInit { pub base: EventInit, pub candidate: Option<String>, pub url: String }
impl Default for RTCPeerConnectionIceEventInit { fn default() -> Self { Self { base: EventInit::default(), candidate: None, url: String::new() } } }

#[derive(Debug, Clone)]
pub struct RTCPeerConnectionIceErrorEventInit { pub base: EventInit, pub address: String, pub port: Option<u16>, pub url: String, pub error_code: u16, pub error_text: String }
impl Default for RTCPeerConnectionIceErrorEventInit { fn default() -> Self { Self { base: EventInit::default(), address: String::new(), port: None, url: String::new(), error_code: 0, error_text: String::new() } } }

#[derive(Debug, Clone)]
pub struct RTCTrackEventInit { pub base: EventInit, pub receiver: Option<String>, pub track: Option<String>, pub streams: Vec<String>, pub transceiver: Option<String> }
impl Default for RTCTrackEventInit { fn default() -> Self { Self { base: EventInit::default(), receiver: None, track: None, streams: Vec::new(), transceiver: None } } }

#[derive(Debug, Clone)]
pub struct RTCTransformEventInit { pub base: EventInit, pub readable: Option<String>, pub writable: Option<String> }
impl Default for RTCTransformEventInit { fn default() -> Self { Self { base: EventInit::default(), readable: None, writable: None } } }
