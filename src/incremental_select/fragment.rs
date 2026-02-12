#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum CharClassPriority {
    Whitespace,
    Operator,
    Dash,
    Underscore,
    Word,
}

#[derive(PartialEq, Eq)]
pub enum CharClass {
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

    pub fn priority(self) -> CharClassPriority {
        match self {
            CharClass::Lower | CharClass::Cap | CharClass::Caseless | CharClass::Number => {
                CharClassPriority::Word
            }
            CharClass::Underscore => CharClassPriority::Underscore,
            CharClass::Dash => CharClassPriority::Dash,
            CharClass::Operator => CharClassPriority::Operator,
            CharClass::Whitespace => CharClassPriority::Whitespace,
        }
    }
}

#[derive(Copy, Clone)]
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
}

impl FragmentKind {
    pub fn of(string: impl IntoIterator<Item = char>, next_char: Option<char>) -> Option<Self> {
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

        if next_char.is_some_and(|c| CharClass::of(c) == CharClass::Lower) {
            match state {
                State::AllCap => {
                    return Some(FragmentKind::Superword);
                }
                State::OneCap => {
                    return Some(FragmentKind::UpperWord);
                }
                _ => (),
            }
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
