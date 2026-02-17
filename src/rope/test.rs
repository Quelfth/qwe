use crate::{
    ix::{Ix, Line},
    rope::Rope,
};

const fn ix(n: usize) -> Ix<Line> {
    Ix::new(n)
}

#[test]
fn lines_empty() {
    let rope: Rope = "".into();
    assert_eq!(rope.max_line_number(), ix(0));
}

#[test]
fn lines_one() {
    let rope: Rope = "Some text".into();
    assert_eq!(rope.max_line_number(), ix(1));
}

#[test]
fn lines_one_feed() {
    let rope: Rope = "Some Text\n".into();
    assert_eq!(rope.max_line_number(), ix(2));
}

#[test]
fn lines_lone_feed() {
    let rope: Rope = "\n".into();
    assert_eq!(rope.max_line_number(), ix(2));
}

#[test]
fn lines_feed_and_line() {
    let rope: Rope = "\n Some Text".into();
    assert_eq!(rope.max_line_number(), ix(2));
}

#[test]
fn lines_two() {
    let rope: Rope = "Some Text\n Some More Text".into();
    assert_eq!(rope.max_line_number(), ix(2));
}

#[test]
fn lines_two_feed() {
    let rope: Rope = "Some Text\n Some More Text\n".into();
    assert_eq!(rope.max_line_number(), ix(3));
}
