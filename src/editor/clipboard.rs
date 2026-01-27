#[derive(Default, Clone)]
pub struct Clip(Vec<String>);

#[derive(Default)]
pub struct Clipboard {
    cursor: usize,
    board: Vec<Clip>,
}

impl Clipboard {
    pub fn append(&mut self, string: String) {
        if self.board.is_empty() {
            self.board.push(Clip::default());
        }

        self.board.last_mut().unwrap().0.push(string);
    }

    pub fn new_clip(&mut self) {
        self.cursor = 0;
        self.board.push(Clip::default());
    }

    pub fn next_clip(&mut self) -> &str {
        let Some(clip) = self.board.last() else {
            return "";
        };

        let str = &**clip.0.get(self.cursor).unwrap();
        self.cursor += 1;
        self.cursor %= clip.0.len();

        str
    }
}
