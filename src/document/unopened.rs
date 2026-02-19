use crate::{
    document::diagnostics::Diagnostic,
    ix::{Byte, Ix},
    range_tree::RangeTree,
};

pub struct UnopenedDocument {
    diagnostics: Option<RangeTree<Ix<Byte>, Diagnostic>>,
}
