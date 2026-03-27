pub struct TextBuffer {
    rope: ropey::Rope,
}

impl TextBuffer {
    pub fn new(content: &str) -> Self {
        Self { rope: ropey::Rope::from_str(content) }
    }

    pub fn get_text(&self) -> String {
        self.rope.to_string()
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }
}
