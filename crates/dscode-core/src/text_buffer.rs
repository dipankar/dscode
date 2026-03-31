/// A rope-based text buffer for efficient editing operations.
///
/// Uses `ropey` internally for fast insertions, deletions, and character counting
/// on large texts.
pub struct TextBuffer {
    rope: ropey::Rope,
}

impl TextBuffer {
    /// Create a new text buffer with the given content.
    pub fn new(content: &str) -> Self {
        Self { rope: ropey::Rope::from_str(content) }
    }

    /// Get the full text content of the buffer.
    pub fn get_text(&self) -> String {
        self.rope.to_string()
    }

    /// Get the number of characters in the buffer.
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_buffer_new_empty() {
        let buffer = TextBuffer::new("");
        assert!(buffer.is_empty());
        assert_eq!(buffer.len_chars(), 0);
        assert_eq!(buffer.get_text(), "");
    }

    #[test]
    fn test_text_buffer_with_content() {
        let content = "Hello, world!";
        let buffer = TextBuffer::new(content);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.len_chars(), 13);
        assert_eq!(buffer.get_text(), content);
    }

    #[test]
    fn test_text_buffer_len_chars() {
        // Single line
        let buffer = TextBuffer::new("abcdef");
        assert_eq!(buffer.len_chars(), 6);

        // Multi-line with newlines
        let buffer = TextBuffer::new("line1\nline2\nline3");
        assert_eq!(buffer.len_chars(), 17);

        // Unicode content
        let buffer = TextBuffer::new("café");
        assert_eq!(buffer.len_chars(), 4);

        // Emoji
        let buffer = TextBuffer::new("🦀");
        assert_eq!(buffer.len_chars(), 1);
    }

    #[test]
    fn test_text_buffer_large_content() {
        // Create a large buffer with 10,000 lines
        let content: String = (0..10_000)
            .map(|i| format!("Line number {}", i))
            .collect::<Vec<_>>()
            .join("\n");

        let buffer = TextBuffer::new(&content);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.len_chars(), content.len());

        // Verify the content is preserved
        let text = buffer.get_text();
        assert_eq!(text, content);
        assert!(text.starts_with("Line number 0"));
        assert!(text.contains("Line number 5000"));
        assert!(text.ends_with("Line number 9999"));
    }
}