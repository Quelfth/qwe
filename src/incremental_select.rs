use std::ops::Range;

use crate::{
    incremental_select::fragment::{CharClass, CharClassPriority, FragmentKind},
    ix::{Byte, Ix},
    rope::Rope,
};

mod fragment;

pub fn increment_range(text: &Rope, range: Range<Ix<Byte>>) -> Range<Ix<Byte>> {
    increment(text, range.clone()).unwrap_or(range)
}

fn increment(text: &Rope, mut range: Range<Ix<Byte>>) -> Option<Range<Ix<Byte>>> {
    struct State<'a> {
        text: &'a Rope,
        range: Range<Ix<Byte>>,
        kind: Option<FragmentKind>,
    }

    let mut state = State {
        text,
        kind: FragmentKind::of(
            text.byte_slice(range.clone())?.chars(),
            text.byte_slice(range.end..)?.chars().next(),
        ),
        range,
    };
    if state.kind.is_none() {
        let mut lr = 0;
        loop {
            let (left, right) = state.left_right_chars();
            match (left, right) {
                (None, None) => return None,
                (None, Some(CharClass::Whitespace)) => {
                    state.extend_right(|_| true);
                }
                (None, Some(_)) => {
                    state.extend_right(|_| true);
                    state.trim_left();
                    break;
                }
                (Some(CharClass::Whitespace), None) => {
                    state.extend_left(|_| true);
                }
                (Some(_), None) => {
                    state.extend_left(|_| true);
                    state.trim_right();
                    break;
                }
                (Some(CharClass::Whitespace), Some(CharClass::Whitespace)) => {
                    if lr <= 0 {
                        state.extend_right(|_| true);
                        lr += 1;
                    } else {
                        state.extend_left(|_| true);
                        lr -= 1;
                    }
                }
                (Some(CharClass::Whitespace), Some(_)) => {
                    state.extend_right(|_| true);
                    state.trim_left();
                    break;
                }
                (Some(_), Some(CharClass::Whitespace)) => {
                    state.extend_left(|_| true);
                    state.trim_right();
                    break;
                }
                (Some(_), Some(_)) => {
                    state.extend_right(|_| true);
                    break;
                }
            }
        }
        state.rekind();
    }

    impl State<'_> {
        fn trim_left(&mut self) {
            while let Some(char) = self
                .text
                .byte_slice(self.range.clone())
                .unwrap()
                .chars()
                .next()
                && char.is_whitespace()
            {
                self.range.start += Ix::new(char.len_utf8());
            }
        }

        fn trim_right(&mut self) {
            while let Some(char) = self
                .text
                .byte_slice(self.range.clone())
                .unwrap()
                .chars()
                .next_back()
                && char.is_whitespace()
            {
                self.range.end -= Ix::new(char.len_utf8());
            }
        }

        fn rekind(&mut self) {
            if let Some(kind) = try {
                FragmentKind::of(
                    self.text.byte_slice(self.range.clone())?.chars(),
                    self.text.byte_slice(self.range.end..)?.chars().next(),
                )
            } {
                self.kind = kind
            }
        }

        fn left_right_chars(&self) -> (Option<CharClass>, Option<CharClass>) {
            (
                self.text
                    .byte_slice(..self.range.start)
                    .unwrap()
                    .chars()
                    .next_back()
                    .map(CharClass::of),
                self.text
                    .byte_slice(self.range.end..)
                    .unwrap()
                    .chars()
                    .next()
                    .map(CharClass::of),
            )
        }

        fn extend_left(&mut self, predicate: impl Fn(char) -> bool) -> Option<char> {
            let char = self
                .text
                .byte_slice(..self.range.start)
                .unwrap()
                .chars()
                .next_back()?;
            if !predicate(char) {
                return None;
            }

            self.range.start -= Ix::new(char.len_utf8());
            Some(char)
        }
        fn extend_right(&mut self, predicate: impl Fn(char) -> bool) -> Option<char> {
            let char = self
                .text
                .byte_slice(self.range.end..)
                .unwrap()
                .chars()
                .next()?;
            if !predicate(char) {
                return None;
            }

            self.range.end += Ix::new(char.len_utf8());
            Some(char)
        }

        fn extend_right_unless_lower_next(
            &mut self,
            predicate: impl Fn(char) -> bool,
        ) -> Option<char> {
            let char = self
                .text
                .byte_slice(self.range.end..)
                .unwrap()
                .chars()
                .next()?;
            if !predicate(char) {
                return None;
            }
            let char_width = Ix::new(char.len_utf8());
            if self
                .text
                .byte_slice(self.range.end + char_width..)
                .unwrap()
                .chars()
                .next()?
                .is_lowercase()
            {
                return None;
            }

            self.range.end += char_width;
            Some(char)
        }

        fn extend(&mut self) {
            loop {
                let Some(kind) = self.kind else { panic!() };
                match kind {
                    FragmentKind::LowerWord => {
                        let Some(char) = self.extend_left(|c| c.is_uppercase() || c.is_lowercase())
                        else {
                            if self.extend_right(|c| c.is_lowercase()).is_some() {
                                continue;
                            }
                            break;
                        };
                        if char.is_uppercase() {
                            self.kind = Some(FragmentKind::UpperWord);
                        }
                    }
                    FragmentKind::UpperWord => {
                        if self.extend_right(|c| c.is_lowercase()).is_none() {
                            break;
                        }
                    }
                    FragmentKind::AllCapsWord => {
                        if self.extend_left(char::is_uppercase).is_none() {
                            if self
                                .extend_right_unless_lower_next(char::is_uppercase)
                                .is_some()
                            {
                                continue;
                            }
                            break;
                        }
                    }
                    FragmentKind::CaselessWord => {
                        let pred =
                            |c: char| c.is_alphabetic() && !c.is_uppercase() && !c.is_lowercase();
                        if self.extend_left(pred).is_none() {
                            if self.extend_right(pred).is_some() {
                                continue;
                            }
                            break;
                        }
                    }
                    FragmentKind::NumberWord => {
                        if self.extend_left(char::is_numeric).is_none() {
                            if self.extend_right(char::is_numeric).is_some() {
                                continue;
                            }
                            break;
                        }
                    }
                    FragmentKind::Superword => {
                        if self.extend_left(|c| c.is_alphanumeric()).is_none() {
                            if self.extend_right(|c| c.is_alphanumeric()).is_some() {
                                continue;
                            }
                            break;
                        }
                    }
                    FragmentKind::Subphrase => {
                        if self
                            .extend_left(|c| c.is_alphanumeric() || c == '_')
                            .is_none()
                        {
                            if self
                                .extend_right(|c| c.is_alphanumeric() || c == '_')
                                .is_some()
                            {
                                continue;
                            }
                            break;
                        }
                    }
                    FragmentKind::Phrase => {
                        if self
                            .extend_left(|c| c.is_alphanumeric() || matches!(c, '_' | '-'))
                            .is_none()
                        {
                            if self
                                .extend_right(|c| c.is_alphanumeric() || matches!(c, '_' | '-'))
                                .is_some()
                            {
                                continue;
                            }
                            break;
                        }
                    }
                    FragmentKind::Clause => {
                        if self
                            .extend_left(|c| !c.is_whitespace() && !matches!(c, '(' | '[' | '{'))
                            .is_none()
                        {
                            if self
                                .extend_right(|c| {
                                    !c.is_whitespace() && !matches!(c, ')' | ']' | '}')
                                })
                                .is_some()
                            {
                                continue;
                            }
                            break;
                        }
                    }
                    FragmentKind::Sentence => {
                        if self
                            .extend_left(|c| !matches!(c, '(' | '[' | '{'))
                            .is_none()
                        {
                            if self
                                .extend_right(|c| !matches!(c, ')' | ']' | '}'))
                                .is_some()
                            {
                                continue;
                            }
                            break;
                        }
                    }
                    FragmentKind::Group => todo!(),
                }
            }
        }
    }

    let range = state.range.clone();
    state.extend();

    if state.range != range {
        return Some(state.range);
    }

    let (left, right) = state.left_right_chars();
    match (left, right) {
        (None, None) => return Some(state.range),
        (None, Some(_)) => {
            state.extend_right(|_| true);
        }
        (Some(_), None) => {
            state.extend_left(|_| true);
        }
        (Some(l), Some(r)) => {
            if l.priority() > r.priority() {
                state.extend_left(|_| true);
            } else {
                state.extend_right(|_| true);
            }
        }
    }
    state.rekind();
    state.extend();

    Some(state.range)
}
