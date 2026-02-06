use std::{ops::Range, sync::Arc};

use crate::ix::{Byte, Ix};

#[derive(Clone)]
pub struct SemanticToken {
    pub range: Range<Ix<Byte>>,
    pub r#type: Arc<str>,
    pub mods: Vec<Arc<str>>,
}
