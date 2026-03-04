use std::ops::Range;

use crate::{
    document::diagnostics::Diagnostic,
    ix::{Byte, Ix},
};

#[allow(unused)]
pub struct UnopenedDocument {
    diagnostics: Option<Vec<(Range<Ix<Byte>>, Diagnostic)>>,
}
