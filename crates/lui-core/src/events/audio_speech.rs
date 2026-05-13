use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct SpeechRecognitionEventInit { pub base: EventInit, pub result_index: u32, pub results: Option<String> }
impl Default for SpeechRecognitionEventInit { fn default() -> Self { Self { base: EventInit::default(), result_index: 0, results: None } } }

#[derive(Debug, Clone)]
pub struct SpeechRecognitionErrorEventInit { pub base: EventInit, pub error: String, pub message: String }
impl Default for SpeechRecognitionErrorEventInit { fn default() -> Self { Self { base: EventInit::default(), error: String::new(), message: String::new() } } }

#[derive(Debug, Clone)]
pub struct SpeechSynthesisEventInit { pub base: EventInit, pub utterance: Option<String>, pub char_index: u32, pub char_length: u32, pub elapsed_time: f32, pub name: String }
impl Default for SpeechSynthesisEventInit { fn default() -> Self { Self { base: EventInit::default(), utterance: None, char_index: 0, char_length: 0, elapsed_time: 0.0, name: String::new() } } }

#[derive(Debug, Clone)]
pub struct SpeechSynthesisErrorEventInit { pub base: SpeechSynthesisEventInit, pub error: String }
impl Default for SpeechSynthesisErrorEventInit { fn default() -> Self { Self { base: SpeechSynthesisEventInit::default(), error: String::new() } } }

#[derive(Debug, Clone)]
pub struct AudioProcessingEventInit { pub base: EventInit, pub playback_time: f64, pub input_buffer: Option<String>, pub output_buffer: Option<String> }
impl Default for AudioProcessingEventInit { fn default() -> Self { Self { base: EventInit::default(), playback_time: 0.0, input_buffer: None, output_buffer: None } } }

#[derive(Debug, Clone)]
pub struct OfflineAudioCompletionEventInit { pub base: EventInit, pub rendered_buffer: Option<String> }
impl Default for OfflineAudioCompletionEventInit { fn default() -> Self { Self { base: EventInit::default(), rendered_buffer: None } } }

#[derive(Debug, Clone)]
pub struct MIDIConnectionEventInit { pub base: EventInit, pub port: Option<String> }
impl Default for MIDIConnectionEventInit { fn default() -> Self { Self { base: EventInit::default(), port: None } } }

#[derive(Debug, Clone)]
pub struct MIDIMessageEventInit { pub base: EventInit, pub data: Option<String> }
impl Default for MIDIMessageEventInit { fn default() -> Self { Self { base: EventInit::default(), data: None } } }
