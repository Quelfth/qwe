use crate::lsp::Server;

mod roslyn;

#[derive(Copy, Clone)]
pub enum SpecialBehavior {
    NoOp,
    Roslyn,
}

impl Server {
    pub async fn special_init(&mut self, f: SpecialBehavior) {
        match f {
            SpecialBehavior::NoOp => (),
            SpecialBehavior::Roslyn => self.init_roslyn().await,
        }
    }
}
