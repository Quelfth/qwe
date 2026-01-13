#[derive(Copy, Clone, Default, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct Pos {
    pub line: usize,
    pub column: usize,
}
