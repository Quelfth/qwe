use std::{collections::HashSet, sync::LazyLock};

use mutx::Mutex;

use crate::lang::Language;

pub static GLOBAL_CONFIG: LazyLock<GlobalConfig> = LazyLock::new(Default::default);

#[derive(Default)]
pub struct GlobalConfig {
    pub autosave_langs: Mutex<HashSet<Language>>,
}