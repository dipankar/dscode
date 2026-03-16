use ropey::Rope;

pub struct TextBuffer {
    rope: Rope,
    file_path: Option<String>,
    is_modified: bool,
}

impl TextBuffer {
    pub fn new(content: &str) -> Self {
        Self {
            rope: Rope::from_str(content),
            file_path: None,
            is_modified: false,
        }
    }

    pub fn from_file(path: &str) -> std::io::Result<Self> {
        let rope = Rope::from_reader(std::fs::File::open(path)?)?;
        Ok(Self {
            rope,
            file_path: Some(path.to_string()),
            is_modified: false,
        })
    }

    pub fn get_text(&self) -> String {
        self.rope.to_string()
    }

    pub fn insert(&mut self, char_idx: usize, text: &str) {
        self.rope.insert(char_idx, text);
        self.is_modified = true;
    }

    pub fn remove(&mut self, start: usize, end: usize) {
        self.rope.remove(start..end);
        self.is_modified = true;
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn is_modified(&self) -> bool {
        self.is_modified
    }
}
