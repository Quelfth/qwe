use std::fmt::{Display, Formatter, Result};

use crate::rope::{Rope, RopeSlice};

impl Display for Rope {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl Display for RopeSlice<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}
