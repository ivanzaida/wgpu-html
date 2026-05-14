//! Per-spec DOM event structs + union enum. Every interface in events.json.

mod animation;
mod audio_speech;
mod base;
mod clipboard;
mod composition;
mod custom;
mod device;
mod dom_events;
mod drag;
mod error;
mod focus;
mod hash_change;
mod input;
mod keyboard;
mod media_events;
mod message;
mod misc;
mod mouse;
mod pointer;
mod pop_state;
mod progress;
mod touch;
mod transition;
mod ui;
mod webrtc;
mod webxr;
mod wheel;
mod worker;

pub use animation::AnimationEventInit;
pub use audio_speech::*;
pub use base::EventInit;
pub use clipboard::ClipboardEventInit;
pub use composition::CompositionEventInit;
pub use custom::CustomEventInit;
pub use device::{DeviceAccel, DeviceMotionEventInit, DeviceOrientationEventInit, DeviceRotation};
pub use dom_events::{CommandEventInit, FormDataEventInit, StorageEventInit, SubmitEventInit, ToggleEventInit};
pub use drag::DragEventInit;
pub use error::ErrorEventInit;
pub use focus::FocusEventInit;
pub use hash_change::HashChangeEventInit;
pub use input::InputEventInit;
pub use keyboard::KeyboardEventInit;
pub use media_events::*;
pub use message::MessageEventInit;
pub use misc::*;
pub use mouse::MouseEventInit;
pub use pointer::PointerEventInit;
pub use pop_state::PopStateEventInit;
pub use progress::ProgressEventInit;
pub use touch::{TouchEventInit, TouchInit};
pub use transition::TransitionEventInit;
pub use ui::UiEventInit;
pub use webrtc::*;
pub use webxr::*;
pub use wheel::WheelEventInit;
pub use worker::*;

#[derive(Debug, Clone)]
pub enum DocumentEvent {
  Event(EventInit),
  UiEvent(UiEventInit),
  MouseEvent(MouseEventInit),
  WheelEvent(WheelEventInit),
  KeyboardEvent(KeyboardEventInit),
  FocusEvent(FocusEventInit),
  InputEvent(InputEventInit),
  CompositionEvent(CompositionEventInit),
  PointerEvent(PointerEventInit),
  TouchEvent(TouchEventInit),
  DragEvent(DragEventInit),
  ClipboardEvent(ClipboardEventInit),
  ProgressEvent(ProgressEventInit),
  MessageEvent(MessageEventInit),
  HashChangeEvent(HashChangeEventInit),
  PopStateEvent(PopStateEventInit),
  AnimationEvent(AnimationEventInit),
  TransitionEvent(TransitionEventInit),
  ErrorEvent(ErrorEventInit),
  Custom(CustomEventInit),
  DeviceMotion(DeviceMotionEventInit),
  DeviceOrientation(DeviceOrientationEventInit),
  Storage(StorageEventInit),
  Submit(SubmitEventInit),
  FormData(FormDataEventInit),
  Toggle(ToggleEventInit),
  Command(CommandEventInit),
  Close(CloseEventInit),
  Fetch(FetchEventInit),
  Extendable(ExtendableEventInit),
  ExtendableMessage(ExtendableMessageEventInit),
  ExtendableCookieChange(ExtendableCookieChangeEventInit),
  Install(InstallEventInit),
  Sync(SyncEventInit),
  PeriodicSync(PeriodicSyncEventInit),
  Push(PushEventInit),
  PushSubscriptionChange(PushSubscriptionChangeEventInit),
  Notification(NotificationEventInit),
  BackgroundFetch(BackgroundFetchEventInit),
  BackgroundFetchUpdateUI(BackgroundFetchUpdateUIEventInit),
  ContentIndex(ContentIndexEventInit),
  Gamepad(GamepadEventInit),
  MediaStreamTrack(MediaStreamTrackEventInit),
  MediaEncrypted(MediaEncryptedEventInit),
  MediaKeyMessage(MediaKeyMessageEventInit),
  MediaQueryList(MediaQueryListEventInit),
  PictureInPicture(PictureInPictureEventInit),
  DocumentPictureInPicture(DocumentPictureInPictureEventInit),
  PortalActivate(PortalActivateEventInit),
  PageTransition(PageTransitionEventInit),
  PageReveal(PageRevealEventInit),
  PageSwap(PageSwapEventInit),
  Navigate(NavigateEventInit),
  NavigationCurrentEntryChange(NavigationCurrentEntryChangeEventInit),
  BeforeUnload(BeforeUnloadEventInit),
  BeforeInstallPrompt(BeforeInstallPromptEventInit),
  SecurityPolicyViolation(SecurityPolicyViolationEventInit),
  PromiseRejection(PromiseRejectionEventInit),
  TaskPriorityChange(TaskPriorityChangeEventInit),
  RTCDataChannel(RTCDataChannelEventInit),
  RTCDTMFToneChange(RTCDTMFToneChangeEventInit),
  RTCError(RTCErrorEventInit),
  RTCPeerConnectionIce(RTCPeerConnectionIceEventInit),
  RTCPeerConnectionIceError(RTCPeerConnectionIceErrorEventInit),
  RTCTrack(RTCTrackEventInit),
  RTCTransform(RTCTransformEventInit),
  XRInputSource(XRInputSourceEventInit),
  XRInputSourcesChange(XRInputSourcesChangeEventInit),
  XRLayer(XRLayerEventInit),
  XRReferenceSpace(XRReferenceSpaceEventInit),
  XRSession(XRSessionEventInit),
  XRVisibilityMaskChange(XRVisibilityMaskChangeEventInit),
  SpeechRecognition(SpeechRecognitionEventInit),
  SpeechRecognitionError(SpeechRecognitionErrorEventInit),
  SpeechSynthesis(SpeechSynthesisEventInit),
  SpeechSynthesisError(SpeechSynthesisErrorEventInit),
  AudioProcessing(AudioProcessingEventInit),
  OfflineAudioCompletion(OfflineAudioCompletionEventInit),
  MIDIConnection(MIDIConnectionEventInit),
  MIDIMessage(MIDIMessageEventInit),
  PaymentRequest(PaymentRequestEventInit),
  PaymentRequestUpdate(PaymentRequestUpdateEventInit),
  PaymentMethodChange(PaymentMethodChangeEventInit),
  CanMakePayment(CanMakePaymentEventInit),
  PresentationConnectionAvailable(PresentationConnectionAvailableEventInit),
  PresentationConnectionClose(PresentationConnectionCloseEventInit),
  BluetoothAdvertising(BluetoothAdvertisingEventInit),
  HIDConnection(HIDConnectionEventInit),
  HIDInputReport(HIDInputReportEventInit),
  USBConnection(USBConnectionEventInit),
  NDEFReading(NDEFReadingEventInit),
  WebGLContext(WebGLContextEventInit),
  GPUUncapturedError(GPUUncapturedErrorEventInit),
  FontFaceSetLoad(FontFaceSetLoadEventInit),
  TextEvent(TextEventInit),
  TextFormatUpdate(TextFormatUpdateEventInit),
  TextUpdate(TextUpdateEventInit),
  SFrameTransformError(SFrameTransformErrorEventInit),
  CharacterBoundsUpdate(CharacterBoundsUpdateEventInit),
  KeyFrameRequest(KeyFrameRequestEventInit),
  WindowControlsOverlayGeometryChange(WindowControlsOverlayGeometryChangeEventInit),
  Autofill(AutofillEventInit),
  Blob(BlobEventInit),
  BufferedChange(BufferedChangeEventInit),
  CaptureAction(CaptureActionEventInit),
  CapturedMouse(CapturedMouseEventInit),
  ClipboardChange(ClipboardChangeEventInit),
  ContentVisibilityAutoStateChange(ContentVisibilityAutoStateChangeEventInit),
  CookieChange(CookieChangeEventInit),
  SensorError(SensorErrorEventInit),
  Value(ValueEventInit),
}

impl DocumentEvent {
  pub fn base(&self) -> &EventInit {
    use DocumentEvent::*;
    match self {
      Event(e) => e,
      UiEvent(e) => &e.base,
      MouseEvent(e) => &e.ui.base,
      WheelEvent(e) => &e.mouse.ui.base,
      KeyboardEvent(e) => &e.ui.base,
      FocusEvent(e) => &e.ui.base,
      InputEvent(e) => &e.ui.base,
      CompositionEvent(e) => &e.ui.base,
      PointerEvent(e) => &e.mouse.ui.base,
      TouchEvent(e) => &e.ui.base,
      DragEvent(e) => &e.mouse.ui.base,
      ClipboardEvent(e) => &e.base,
      ProgressEvent(e) => &e.base,
      MessageEvent(e) => &e.base,
      HashChangeEvent(e) => &e.base,
      PopStateEvent(e) => &e.base,
      AnimationEvent(e) => &e.base,
      TransitionEvent(e) => &e.base,
      ErrorEvent(e) => &e.base,
      Custom(e) => &e.base,
      DeviceMotion(e) => &e.base,
      DeviceOrientation(e) => &e.base,
      Storage(e) => &e.base,
      Submit(e) => &e.base,
      FormData(e) => &e.base,
      Toggle(e) => &e.base,
      Command(e) => &e.base,
      Close(e) => &e.base,
      Fetch(e) => &e.base.base,
      Extendable(e) => &e.base,
      ExtendableMessage(e) => &e.message.base,
      ExtendableCookieChange(e) => &e.base.base,
      Install(e) => &e.base.base,
      Sync(e) => &e.base.base,
      PeriodicSync(e) => &e.base.base,
      Push(e) => &e.base.base,
      PushSubscriptionChange(e) => &e.base.base,
      Notification(e) => &e.base.base,
      BackgroundFetch(e) => &e.base.base,
      BackgroundFetchUpdateUI(e) => &e.base.base.base,
      ContentIndex(e) => &e.base.base,
      _ => {
        use std::sync::LazyLock;
        static DEFAULT_EVENT: LazyLock<EventInit> = LazyLock::new(EventInit::default);
        &*DEFAULT_EVENT
      }
    }
  }

  pub fn base_mut(&mut self) -> &mut EventInit {
    use DocumentEvent::*;
    match self {
      Event(e) => e,
      UiEvent(e) => &mut e.base,
      MouseEvent(e) => &mut e.ui.base,
      WheelEvent(e) => &mut e.mouse.ui.base,
      KeyboardEvent(e) => &mut e.ui.base,
      FocusEvent(e) => &mut e.ui.base,
      InputEvent(e) => &mut e.ui.base,
      CompositionEvent(e) => &mut e.ui.base,
      PointerEvent(e) => &mut e.mouse.ui.base,
      TouchEvent(e) => &mut e.ui.base,
      DragEvent(e) => &mut e.mouse.ui.base,
      ClipboardEvent(e) => &mut e.base,
      ProgressEvent(e) => &mut e.base,
      MessageEvent(e) => &mut e.base,
      HashChangeEvent(e) => &mut e.base,
      PopStateEvent(e) => &mut e.base,
      AnimationEvent(e) => &mut e.base,
      TransitionEvent(e) => &mut e.base,
      ErrorEvent(e) => &mut e.base,
      Custom(e) => &mut e.base,
      DeviceMotion(e) => &mut e.base,
      DeviceOrientation(e) => &mut e.base,
      Storage(e) => &mut e.base,
      Submit(e) => &mut e.base,
      FormData(e) => &mut e.base,
      Toggle(e) => &mut e.base,
      Command(e) => &mut e.base,
      Close(e) => &mut e.base,
      Fetch(e) => &mut e.base.base,
      Extendable(e) => &mut e.base,
      ExtendableMessage(e) => &mut e.message.base,
      ExtendableCookieChange(e) => &mut e.base.base,
      Install(e) => &mut e.base.base,
      Sync(e) => &mut e.base.base,
      PeriodicSync(e) => &mut e.base.base,
      Push(e) => &mut e.base.base,
      PushSubscriptionChange(e) => &mut e.base.base,
      Notification(e) => &mut e.base.base,
      BackgroundFetch(e) => &mut e.base.base,
      BackgroundFetchUpdateUI(e) => &mut e.base.base.base,
      ContentIndex(e) => &mut e.base.base,
      _ => unimplemented!("base_mut not implemented for this event variant"),
    }
  }

  pub fn event_type(&self) -> &str {
    &self.base().event_type
  }

  pub fn prevent_default(&mut self) {
    let base = self.base_mut();
    if base.cancelable {
      base.default_prevented = true;
    }
  }

  pub fn is_default_prevented(&self) -> bool {
    self.base().default_prevented
  }

  pub fn stop_propagation(&mut self) {
    self.base_mut().propagation_stopped = true;
  }

  pub fn stop_immediate_propagation(&mut self) {
    let base = self.base_mut();
    base.propagation_stopped = true;
    base.immediate_propagation_stopped = true;
  }

  pub fn is_propagation_stopped(&self) -> bool {
    self.base().propagation_stopped
  }

  pub fn from_interface(name: &str) -> Self {
    use DocumentEvent::*;
    match name {
      "MouseEvent" => MouseEvent(MouseEventInit::default()),
      "WheelEvent" => WheelEvent(WheelEventInit::default()),
      "KeyboardEvent" => KeyboardEvent(KeyboardEventInit::default()),
      "FocusEvent" => FocusEvent(FocusEventInit::default()),
      "InputEvent" => InputEvent(InputEventInit::default()),
      "CompositionEvent" => CompositionEvent(CompositionEventInit::default()),
      "PointerEvent" => PointerEvent(PointerEventInit::default()),
      "TouchEvent" => TouchEvent(TouchEventInit::default()),
      "DragEvent" => DragEvent(DragEventInit::default()),
      "ClipboardEvent" => ClipboardEvent(ClipboardEventInit::default()),
      "ProgressEvent" => ProgressEvent(ProgressEventInit::default()),
      "MessageEvent" => MessageEvent(MessageEventInit::default()),
      "HashChangeEvent" => HashChangeEvent(HashChangeEventInit::default()),
      "PopStateEvent" => PopStateEvent(PopStateEventInit::default()),
      "AnimationEvent" => AnimationEvent(AnimationEventInit::default()),
      "TransitionEvent" => TransitionEvent(TransitionEventInit::default()),
      "ErrorEvent" => ErrorEvent(ErrorEventInit::default()),
      "UIEvent" => UiEvent(UiEventInit::default()),
      "DeviceMotionEvent" => DeviceMotion(DeviceMotionEventInit::default()),
      "DeviceOrientationEvent" => DeviceOrientation(DeviceOrientationEventInit::default()),
      "StorageEvent" => Storage(StorageEventInit::default()),
      "SubmitEvent" => Submit(SubmitEventInit::default()),
      "FormDataEvent" => FormData(FormDataEventInit::default()),
      "ToggleEvent" => Toggle(ToggleEventInit::default()),
      "CommandEvent" => Command(CommandEventInit::default()),
      "CloseEvent" => Close(CloseEventInit::default()),
      "FetchEvent" => Fetch(FetchEventInit::default()),
      "ExtendableEvent" => Extendable(ExtendableEventInit::default()),
      "ExtendableMessageEvent" => ExtendableMessage(ExtendableMessageEventInit::default()),
      "ExtendableCookieChangeEvent" => ExtendableCookieChange(ExtendableCookieChangeEventInit::default()),
      "InstallEvent" => Install(InstallEventInit::default()),
      "SyncEvent" => Sync(SyncEventInit::default()),
      "PeriodicSyncEvent" => PeriodicSync(PeriodicSyncEventInit::default()),
      "PushEvent" => Push(PushEventInit::default()),
      "PushSubscriptionChangeEvent" => PushSubscriptionChange(PushSubscriptionChangeEventInit::default()),
      "NotificationEvent" => Notification(NotificationEventInit::default()),
      "BackgroundFetchEvent" => BackgroundFetch(BackgroundFetchEventInit::default()),
      "BackgroundFetchUpdateUIEvent" => BackgroundFetchUpdateUI(BackgroundFetchUpdateUIEventInit::default()),
      "ContentIndexEvent" => ContentIndex(ContentIndexEventInit::default()),
      "GamepadEvent" => Gamepad(GamepadEventInit::default()),
      "MediaStreamTrackEvent" => MediaStreamTrack(MediaStreamTrackEventInit::default()),
      "MediaEncryptedEvent" => MediaEncrypted(MediaEncryptedEventInit::default()),
      "MediaKeyMessageEvent" => MediaKeyMessage(MediaKeyMessageEventInit::default()),
      "MediaQueryListEvent" => MediaQueryList(MediaQueryListEventInit::default()),
      "PictureInPictureEvent" => PictureInPicture(PictureInPictureEventInit::default()),
      "DocumentPictureInPictureEvent" => DocumentPictureInPicture(DocumentPictureInPictureEventInit::default()),
      "PortalActivateEvent" => PortalActivate(PortalActivateEventInit::default()),
      "PageTransitionEvent" => PageTransition(PageTransitionEventInit::default()),
      "PageRevealEvent" => PageReveal(PageRevealEventInit::default()),
      "PageSwapEvent" => PageSwap(PageSwapEventInit::default()),
      "NavigateEvent" => Navigate(NavigateEventInit::default()),
      "NavigationCurrentEntryChangeEvent" => {
        NavigationCurrentEntryChange(NavigationCurrentEntryChangeEventInit::default())
      }
      "BeforeUnloadEvent" => BeforeUnload(BeforeUnloadEventInit::default()),
      "BeforeInstallPromptEvent" => BeforeInstallPrompt(BeforeInstallPromptEventInit::default()),
      "SecurityPolicyViolationEvent" => SecurityPolicyViolation(SecurityPolicyViolationEventInit::default()),
      "PromiseRejectionEvent" => PromiseRejection(PromiseRejectionEventInit::default()),
      "TaskPriorityChangeEvent" => TaskPriorityChange(TaskPriorityChangeEventInit::default()),
      "RTCDataChannelEvent" => RTCDataChannel(RTCDataChannelEventInit::default()),
      "RTCDTMFToneChangeEvent" => RTCDTMFToneChange(RTCDTMFToneChangeEventInit::default()),
      "RTCErrorEvent" => RTCError(RTCErrorEventInit::default()),
      "RTCPeerConnectionIceEvent" => RTCPeerConnectionIce(RTCPeerConnectionIceEventInit::default()),
      "RTCPeerConnectionIceErrorEvent" => RTCPeerConnectionIceError(RTCPeerConnectionIceErrorEventInit::default()),
      "RTCTrackEvent" => RTCTrack(RTCTrackEventInit::default()),
      "RTCTransformEvent" => RTCTransform(RTCTransformEventInit::default()),
      "XRInputSourceEvent" => XRInputSource(XRInputSourceEventInit::default()),
      "XRInputSourcesChangeEvent" => XRInputSourcesChange(XRInputSourcesChangeEventInit::default()),
      "XRLayerEvent" => XRLayer(XRLayerEventInit::default()),
      "XRReferenceSpaceEvent" => XRReferenceSpace(XRReferenceSpaceEventInit::default()),
      "XRSessionEvent" => XRSession(XRSessionEventInit::default()),
      "XRVisibilityMaskChangeEvent" => XRVisibilityMaskChange(XRVisibilityMaskChangeEventInit::default()),
      "SpeechRecognitionEvent" => SpeechRecognition(SpeechRecognitionEventInit::default()),
      "SpeechRecognitionErrorEvent" => SpeechRecognitionError(SpeechRecognitionErrorEventInit::default()),
      "SpeechSynthesisEvent" => SpeechSynthesis(SpeechSynthesisEventInit::default()),
      "SpeechSynthesisErrorEvent" => SpeechSynthesisError(SpeechSynthesisErrorEventInit::default()),
      "AudioProcessingEvent" => AudioProcessing(AudioProcessingEventInit::default()),
      "OfflineAudioCompletionEvent" => OfflineAudioCompletion(OfflineAudioCompletionEventInit::default()),
      "MIDIConnectionEvent" => MIDIConnection(MIDIConnectionEventInit::default()),
      "MIDIMessageEvent" => MIDIMessage(MIDIMessageEventInit::default()),
      "PaymentRequestEvent" => PaymentRequest(PaymentRequestEventInit::default()),
      "PaymentRequestUpdateEvent" => PaymentRequestUpdate(PaymentRequestUpdateEventInit::default()),
      "PaymentMethodChangeEvent" => PaymentMethodChange(PaymentMethodChangeEventInit::default()),
      "CanMakePaymentEvent" => CanMakePayment(CanMakePaymentEventInit::default()),
      "PresentationConnectionAvailableEvent" => {
        PresentationConnectionAvailable(PresentationConnectionAvailableEventInit::default())
      }
      "PresentationConnectionCloseEvent" => {
        PresentationConnectionClose(PresentationConnectionCloseEventInit::default())
      }
      "BluetoothAdvertisingEvent" => BluetoothAdvertising(BluetoothAdvertisingEventInit::default()),
      "HIDConnectionEvent" => HIDConnection(HIDConnectionEventInit::default()),
      "HIDInputReportEvent" => HIDInputReport(HIDInputReportEventInit::default()),
      "USBConnectionEvent" => USBConnection(USBConnectionEventInit::default()),
      "NDEFReadingEvent" => NDEFReading(NDEFReadingEventInit::default()),
      "WebGLContextEvent" => WebGLContext(WebGLContextEventInit::default()),
      "GPUUncapturedErrorEvent" => GPUUncapturedError(GPUUncapturedErrorEventInit::default()),
      "FontFaceSetLoadEvent" => FontFaceSetLoad(FontFaceSetLoadEventInit::default()),
      "TextEvent" => TextEvent(TextEventInit::default()),
      "TextFormatUpdateEvent" => TextFormatUpdate(TextFormatUpdateEventInit::default()),
      "TextUpdateEvent" => TextUpdate(TextUpdateEventInit::default()),
      "SFrameTransformErrorEvent" => SFrameTransformError(SFrameTransformErrorEventInit::default()),
      "CharacterBoundsUpdateEvent" => CharacterBoundsUpdate(CharacterBoundsUpdateEventInit::default()),
      "KeyFrameRequestEvent" => KeyFrameRequest(KeyFrameRequestEventInit::default()),
      "WindowControlsOverlayGeometryChangeEvent" => {
        WindowControlsOverlayGeometryChange(WindowControlsOverlayGeometryChangeEventInit::default())
      }
      "AutofillEvent" => Autofill(AutofillEventInit::default()),
      "BlobEvent" => Blob(BlobEventInit::default()),
      "BufferedChangeEvent" => BufferedChange(BufferedChangeEventInit::default()),
      "CaptureActionEvent" => CaptureAction(CaptureActionEventInit::default()),
      "CapturedMouseEvent" => CapturedMouse(CapturedMouseEventInit::default()),
      "ClipboardChangeEvent" => ClipboardChange(ClipboardChangeEventInit::default()),
      "ContentVisibilityAutoStateChangeEvent" => {
        ContentVisibilityAutoStateChange(ContentVisibilityAutoStateChangeEventInit::default())
      }
      "CookieChangeEvent" => CookieChange(CookieChangeEventInit::default()),
      "SensorErrorEvent" => SensorError(SensorErrorEventInit::default()),
      "ValueEvent" => Value(ValueEventInit::default()),
      _ => Event(EventInit::default()),
    }
  }
}
