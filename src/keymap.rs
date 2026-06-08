#![allow(unused)]
use std::{array, collections::HashMap, ops::{Index, IndexMut}};

use strum::EnumCount;

use crate::key::Key;

pub struct KeyMap<A> {
    map: [Option<A>; Key::COUNT],
}

pub macro keymap($(..$h:expr,)* $([$($k:tt)*] => $b: expr),* $(,)?) {{
    let mut map = KeyMap::new();
    $( for i in 0..Key::COUNT {
        if let Some(mapping) = $h.map[i] {
            map.map[i] = Some(mapping.into());
        }
    } )*
    $( map[crate::key::key![$($k)*]] = Some($b); )*
    map
}}

impl<A> KeyMap<A> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<A> Default for KeyMap<A> {
    fn default() -> Self {
        Self { map: array::from_fn(|_| None) }
    }
}


impl<A> Index<Key> for KeyMap<A> {
    type Output = Option<A>;

    fn index(&self, index: Key) -> &Self::Output {
        &self.map[index as usize]
    }
}

impl<A> IndexMut<Key> for KeyMap<A> {
    fn index_mut(&mut self, index: Key) -> &mut Self::Output {
        &mut self.map[index as usize]
    }
}

impl<A> KeyMap<A> {
    pub fn bind(&mut self, key: Key, action: impl Into<A>) {
        self[key] = Some(action.into());
    }

    pub fn map(&self, key: Key) -> Option<&A> {
        self[key].as_ref()
    }
}
