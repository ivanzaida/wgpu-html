use super::{base::EventInit, worker::ExtendableEventInit};

// Payment
#[derive(Debug, Clone)]
pub struct PaymentRequestEventInit {
  pub base: ExtendableEventInit,
  pub top_origin: String,
  pub payment_request_origin: String,
  pub payment_request_id: String,
  pub method_data: Vec<String>,
  pub total: Option<String>,
  pub modifiers: Vec<String>,
}
impl Default for PaymentRequestEventInit {
  fn default() -> Self {
    Self {
      base: ExtendableEventInit::default(),
      top_origin: String::new(),
      payment_request_origin: String::new(),
      payment_request_id: String::new(),
      method_data: Vec::new(),
      total: None,
      modifiers: Vec::new(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct PaymentRequestUpdateEventInit {
  pub base: EventInit,
}
impl Default for PaymentRequestUpdateEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct PaymentMethodChangeEventInit {
  pub base: EventInit,
  pub method_name: String,
  pub method_details: Option<String>,
}
impl Default for PaymentMethodChangeEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      method_name: String::new(),
      method_details: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct CanMakePaymentEventInit {
  pub base: ExtendableEventInit,
  pub top_origin: String,
  pub payment_request_origin: String,
  pub payment_request_id: String,
  pub method_data: Vec<String>,
}
impl Default for CanMakePaymentEventInit {
  fn default() -> Self {
    Self {
      base: ExtendableEventInit::default(),
      top_origin: String::new(),
      payment_request_origin: String::new(),
      payment_request_id: String::new(),
      method_data: Vec::new(),
    }
  }
}

// Presentation
#[derive(Debug, Clone)]
pub struct PresentationConnectionAvailableEventInit {
  pub base: EventInit,
  pub connection: Option<String>,
}
impl Default for PresentationConnectionAvailableEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      connection: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct PresentationConnectionCloseEventInit {
  pub base: EventInit,
  pub reason: String,
  pub message: String,
}
impl Default for PresentationConnectionCloseEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      reason: String::new(),
      message: String::new(),
    }
  }
}

// Bluetooth
#[derive(Debug, Clone)]
pub struct BluetoothAdvertisingEventInit {
  pub base: EventInit,
  pub device: Option<String>,
  pub uuids: Vec<String>,
  pub name: Option<String>,
  pub appearance: Option<u16>,
  pub tx_power: Option<i8>,
  pub rssi: Option<i8>,
  pub manufacturer_data: Option<String>,
  pub service_data: Option<String>,
}
impl Default for BluetoothAdvertisingEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      device: None,
      uuids: Vec::new(),
      name: None,
      appearance: None,
      tx_power: None,
      rssi: None,
      manufacturer_data: None,
      service_data: None,
    }
  }
}

// HID
#[derive(Debug, Clone)]
pub struct HIDConnectionEventInit {
  pub base: EventInit,
  pub device: Option<String>,
}
impl Default for HIDConnectionEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      device: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct HIDInputReportEventInit {
  pub base: EventInit,
  pub device: Option<String>,
  pub report_id: u8,
  pub data: Option<String>,
}
impl Default for HIDInputReportEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      device: None,
      report_id: 0,
      data: None,
    }
  }
}

// USB
#[derive(Debug, Clone)]
pub struct USBConnectionEventInit {
  pub base: EventInit,
  pub device: Option<String>,
}
impl Default for USBConnectionEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      device: None,
    }
  }
}

// NFC
#[derive(Debug, Clone)]
pub struct NDEFReadingEventInit {
  pub base: EventInit,
  pub serial_number: String,
  pub message: Option<String>,
}
impl Default for NDEFReadingEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      serial_number: String::new(),
      message: None,
    }
  }
}

// WebGL
#[derive(Debug, Clone)]
pub struct WebGLContextEventInit {
  pub base: EventInit,
  pub status_message: String,
}
impl Default for WebGLContextEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      status_message: String::new(),
    }
  }
}

// GPU
#[derive(Debug, Clone)]
pub struct GPUUncapturedErrorEventInit {
  pub base: EventInit,
  pub error: Option<String>,
}
impl Default for GPUUncapturedErrorEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      error: None,
    }
  }
}

// Font
#[derive(Debug, Clone)]
pub struct FontFaceSetLoadEventInit {
  pub base: EventInit,
  pub fontfaces: Vec<String>,
}
impl Default for FontFaceSetLoadEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      fontfaces: Vec::new(),
    }
  }
}

// Text update (EditContext)
#[derive(Debug, Clone)]
pub struct TextEventInit {
  pub base: EventInit,
  pub data: String,
}
impl Default for TextEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      data: String::new(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct TextFormatUpdateEventInit {
  pub base: EventInit,
  pub text_formats: Vec<String>,
}
impl Default for TextFormatUpdateEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      text_formats: Vec::new(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct TextUpdateEventInit {
  pub base: EventInit,
  pub update_range_start: u32,
  pub update_range_end: u32,
  pub text: String,
  pub selection_start: u32,
  pub selection_end: u32,
  pub composition_start: u32,
  pub composition_end: u32,
}
impl Default for TextUpdateEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      update_range_start: 0,
      update_range_end: 0,
      text: String::new(),
      selection_start: 0,
      selection_end: 0,
      composition_start: 0,
      composition_end: 0,
    }
  }
}

// SFrame
#[derive(Debug, Clone)]
pub struct SFrameTransformErrorEventInit {
  pub base: EventInit,
  pub error_type: String,
  pub frame: Option<String>,
  pub key_id: Option<String>,
}
impl Default for SFrameTransformErrorEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      error_type: String::new(),
      frame: None,
      key_id: None,
    }
  }
}

// Character bounds
#[derive(Debug, Clone)]
pub struct CharacterBoundsUpdateEventInit {
  pub base: EventInit,
  pub bounds: Vec<String>,
}
impl Default for CharacterBoundsUpdateEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      bounds: Vec::new(),
    }
  }
}

// Keyframe
#[derive(Debug, Clone)]
pub struct KeyFrameRequestEventInit {
  pub base: EventInit,
}
impl Default for KeyFrameRequestEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
    }
  }
}

// Window controls overlay
#[derive(Debug, Clone)]
pub struct WindowControlsOverlayGeometryChangeEventInit {
  pub base: EventInit,
  pub visible: bool,
  pub bounding_rect: Option<String>,
}
impl Default for WindowControlsOverlayGeometryChangeEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      visible: false,
      bounding_rect: None,
    }
  }
}

// Remaining one-offs
#[derive(Debug, Clone)]
pub struct AutofillEventInit {
  pub base: EventInit,
}
impl Default for AutofillEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct BlobEventInit {
  pub base: EventInit,
  pub data: Option<String>,
  pub timecode: Option<f64>,
}
impl Default for BlobEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      data: None,
      timecode: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct BufferedChangeEventInit {
  pub base: EventInit,
  pub added_ranges: Vec<String>,
  pub removed_ranges: Vec<String>,
}
impl Default for BufferedChangeEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      added_ranges: Vec::new(),
      removed_ranges: Vec::new(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct CaptureActionEventInit {
  pub base: EventInit,
  pub action: String,
}
impl Default for CaptureActionEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      action: String::new(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct CapturedMouseEventInit {
  pub base: EventInit,
  pub surface_x: f64,
  pub surface_y: f64,
}
impl Default for CapturedMouseEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      surface_x: 0.0,
      surface_y: 0.0,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ClipboardChangeEventInit {
  pub base: EventInit,
}
impl Default for ClipboardChangeEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ContentVisibilityAutoStateChangeEventInit {
  pub base: EventInit,
  pub skipped: bool,
}
impl Default for ContentVisibilityAutoStateChangeEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      skipped: false,
    }
  }
}

#[derive(Debug, Clone)]
pub struct CookieChangeEventInit {
  pub base: EventInit,
  pub changed: Vec<String>,
  pub deleted: Vec<String>,
}
impl Default for CookieChangeEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      changed: Vec::new(),
      deleted: Vec::new(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct SensorErrorEventInit {
  pub base: EventInit,
  pub error: Option<String>,
}
impl Default for SensorErrorEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      error: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ValueEventInit {
  pub base: EventInit,
  pub value: Option<String>,
}
impl Default for ValueEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      value: None,
    }
  }
}
