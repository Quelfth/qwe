use crate::{
    pos::{Pos, Utf16Pos},
    rope::Rope,
};

pub enum ConvertableToPos {
    Pos(Pos),
    Utf16(Utf16Pos),
}

impl From<Pos> for ConvertableToPos {
    fn from(value: Pos) -> Self {
        Self::Pos(value)
    }
}

impl From<Utf16Pos> for ConvertableToPos {
    fn from(value: Utf16Pos) -> Self {
        Self::Utf16(value)
    }
}

pub trait TextConvertablePos<T> {
    fn convert(self, text: &Rope) -> T;
}

impl<T> TextConvertablePos<T> for T {
    fn convert(self, _: &Rope) -> T {
        self
    }
}

impl TextConvertablePos<Pos> for Utf16Pos {
    fn convert(self, text: &Rope) -> Pos {
        text.pos_of_utf16_pos_saturating(self)
    }
}

impl TextConvertablePos<Pos> for ConvertableToPos {
    fn convert(self, text: &Rope) -> Pos {
        match self {
            ConvertableToPos::Pos(pos) => pos.convert(text),
            ConvertableToPos::Utf16(utf16_pos) => utf16_pos.convert(text),
        }
    }
}
