use crate::{ArcStr, HtmlNode, node::query_selector::QuerySelector};

impl HtmlNode {
  pub fn get_element_by_id(&self, id: ArcStr) -> Option<&HtmlNode> {
    todo!()
  }

  pub fn get_element_by_id_mut(&mut self, id: ArcStr) -> Option<&mut HtmlNode> {
    todo!()
  }

  pub fn get_elements_by_class_name(&self, class_name: ArcStr) -> Vec<&HtmlNode> {
    todo!()
  }

  pub fn get_elements_by_class_name_mut(&mut self, class_name: ArcStr) -> Vec<&mut HtmlNode> {
    todo!()
  }

  pub fn get_elements_by_tag_name(&self, tag_name: ArcStr) -> Vec<&HtmlNode> {
    todo!()
  }

  pub fn get_elements_by_tag_name_mut(&mut self, tag_name: ArcStr) -> Vec<&HtmlNode> {
    todo!()
  }

  pub fn query_selector(&self, selector: impl Into<QuerySelector>) -> Option<&HtmlNode> {
    todo!()
  }

  pub fn query_selector_mut(&mut self, selector: impl Into<QuerySelector>) -> Option<&mut HtmlNode> {
    todo!()
  }

  pub fn query_selector_all(&self, selector: impl Into<QuerySelector>) -> Vec<&HtmlNode> {
    todo!()
  }

  pub fn query_selector_all_mut(&mut self, selector: impl Into<QuerySelector>) -> Vec<&mut HtmlNode> {
    todo!()
  }
}
