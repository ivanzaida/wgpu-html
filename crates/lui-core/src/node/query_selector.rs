use crate::ArcStr;

pub struct QuerySelector {}

impl From<&str> for QuerySelector {
  fn from(selector: &str) -> Self {
    todo!()
  }
}

impl From<ArcStr> for QuerySelector {
  fn from(selector: ArcStr) -> Self {
    todo!()
  }
}

impl From<String> for QuerySelector {
  fn from(selector: String) -> Self {
    todo!()
  }
}

impl From<&QuerySelector> for QuerySelector {
  fn from(selector: &QuerySelector) -> Self {
    todo!()
  }
}
