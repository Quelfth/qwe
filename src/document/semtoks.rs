use std::sync::Arc;

#[derive(Clone)]
pub struct SemanticToken {
    pub r#type: Arc<str>,
    pub mods: Vec<Arc<str>>,
}
