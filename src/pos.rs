#[derive(Copy, Clone, Default, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct Pos {
    pub line: usize,
    pub column: usize,
}

impl Pos {
    pub const ZERO: Self = Self { line: 0, column: 0 };
}
