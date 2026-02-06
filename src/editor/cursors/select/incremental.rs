use crate::rope::Rope;

use super::SelectCursor;

impl SelectCursor {
    pub fn incremental_select(&mut self, text: &Rope) {
        if !self.other_lines.is_empty() {
            todo!()
        }
        let Some(line) = text.line(self.line) else {
            return;
        };
        let current = line.column_slice(self.first_line.start..self.first_line.end);
        let kind = FragmentKind::of(
            current.chars(),
            line.byte_slice(line.columns_to_bytes(self.first_line.end)..)
                .unwrap()
                .chars()
                .next()
                .unwrap_or('\n'),
        );
    }
}

pub enum FragmentKind {
    /// foo
    LowerWord,
    /// Foo
    UpperWord,
    /// FOO
    AllCapsWord,
    /// フー
    CaselessWord,
    /// 700
    NumberWord,
    /// fooBar FOO100
    Superword,
    /// foo_bar
    Subphrase,
    /// foo-bar
    Phrase,
    /// foo+bar
    Clause,
    /// foo + bar
    Sentence,
    /// (foo + bar + baz)
    Group,
}

impl FragmentKind {
    pub fn of(string: impl IntoIterator<Item = char>, next_char: char) -> Option<Self> {
        #[derive(PartialEq, Eq)]
        enum State {
            None,
            AllLower,
            OneCap,
            AllCap,
            CapWord,
            AllCaseless,
            AllNumbers,
            Superword,
            Subphrase,
            Phrase,
            Clause,
            Sentence,
        }
        let mut state = State::None;
        #[derive(PartialEq, Eq)]
        enum CharClass {
            Lower,
            Cap,
            Caseless,
            Number,
            Underscore,
            Dash,
            Operator,
            Whitespace,
        }

        impl CharClass {
            pub fn of(char: char) -> Self {
                if char.is_lowercase() {
                    CharClass::Lower
                } else if char.is_uppercase() {
                    CharClass::Cap
                } else if char.is_alphabetic() {
                    CharClass::Caseless
                } else if char.is_numeric() {
                    CharClass::Number
                } else if char.is_whitespace() {
                    CharClass::Whitespace
                } else {
                    match char {
                        '_' => CharClass::Underscore,
                        '-' => CharClass::Dash,
                        _ => CharClass::Operator,
                    }
                }
            }
        }

        for char in string {
            use {CharClass::*, State::*};
            state = match (state, CharClass::of(char)) {
                (None, Lower) => AllLower,
                (None, Cap) => OneCap,
                (None, Caseless) => AllCaseless,
                (None, Number) => AllNumbers,
                (AllLower, Lower) => AllLower,
                (AllLower, Cap | Caseless | Number) => Superword,
                (OneCap, Lower) => CapWord,
                (OneCap, Cap) => AllCap,
                (OneCap, Caseless | Number) => Superword,
                (AllCap, Lower | Caseless | Number) => Superword,
                (AllCap, Cap) => AllCap,
                (CapWord, Lower) => CapWord,
                (CapWord, Cap | Caseless | Number) => Superword,
                (AllCaseless, Lower | Cap | Number) => Superword,
                (AllCaseless, Caseless) => AllCaseless,
                (AllNumbers, Lower | Cap | Caseless) => Superword,
                (AllNumbers, Number) => AllNumbers,
                (Superword, Lower | Cap | Caseless | Number) => Superword,
                (
                    None | AllLower | OneCap | AllCap | CapWord | AllCaseless | AllNumbers
                    | Superword,
                    Underscore,
                ) => Subphrase,
                (Subphrase, Lower | Cap | Caseless | Number | Underscore) => Subphrase,
                (
                    None | AllLower | OneCap | AllCap | CapWord | AllCaseless | AllNumbers
                    | Superword | Subphrase,
                    Dash,
                ) => Phrase,
                (Phrase, Lower | Cap | Caseless | Number | Underscore | Dash) => Phrase,
                (
                    None | AllLower | OneCap | AllCap | CapWord | AllCaseless | AllNumbers
                    | Superword | Subphrase | Phrase,
                    Operator,
                ) => Clause,
                (Clause, Lower | Cap | Caseless | Number | Underscore | Dash | Operator) => Clause,
                (_, Whitespace) => Sentence,
                (Sentence, _) => Sentence,
            }
        }

        if matches!(state, State::AllCap | State::OneCap)
            && CharClass::of(next_char) == CharClass::Lower
        {
            return Some(FragmentKind::Superword);
        }

        Some(match state {
            State::None => None::<!>?,
            State::AllLower => FragmentKind::LowerWord,
            State::OneCap | State::AllCap => FragmentKind::AllCapsWord,
            State::CapWord => FragmentKind::UpperWord,
            State::AllCaseless => FragmentKind::CaselessWord,
            State::AllNumbers => FragmentKind::NumberWord,
            State::Superword => FragmentKind::Superword,
            State::Subphrase => FragmentKind::Subphrase,
            State::Phrase => FragmentKind::Phrase,
            State::Clause => FragmentKind::Clause,
            State::Sentence => FragmentKind::Sentence,
        })
    }
}
