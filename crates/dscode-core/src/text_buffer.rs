/// A rope-based text buffer for efficient editing operations.
///
/// `TextBuffer` wraps a [`ropey::Rope`] to provide fast insertions, deletions,
/// and character counting on texts of any size — from short snippets to files
/// with millions of lines.
///
/// # Invariants
///
/// - The internal rope is always well-formed (never `None` or in an undefined
///   state) after construction via [`TextBuffer::new`].
/// - Character counts returned by [`TextBuffer::len_chars`] count Unicode
///   scalar values, so multi-byte characters like emoji are counted as a
///   single character.
///
/// # Example
///
/// ```rust
/// use dscode_core::TextBuffer;
///
/// let buffer = TextBuffer::new("Hello, world!");
/// assert_eq!(buffer.len_chars(), 13);
/// assert_eq!(buffer.get_text(), "Hello, world!");
/// assert!(!buffer.is_empty());
/// ```
pub struct TextBuffer {
    rope: ropey::Rope,
}

impl TextBuffer {
    /// Create a new text buffer initialised with the given content.
    ///
    /// The entire `content` string is copied into an internal rope data
    /// structure. For an empty buffer, pass `""`.
    ///
    /// # Arguments
    ///
    /// * `content` — The initial text content of the buffer. An empty string
    ///   produces an empty buffer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dscode_core::TextBuffer;
    ///
    /// let buffer = TextBuffer::new("some text");
    /// ```
    pub fn new(content: &str) -> Self {
        Self {
            rope: ropey::Rope::from_str(content),
        }
    }

    /// Get the full text content of the buffer as a [`String`].
    ///
    /// This copies the entire rope contents into a new string. For very large
    /// buffers, consider whether you truly need the whole text at once.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dscode_core::TextBuffer;
    ///
    /// let buffer = TextBuffer::new("abc");
    /// assert_eq!(buffer.get_text(), "abc");
    /// ```
    pub fn get_text(&self) -> String {
        self.rope.to_string()
    }

    /// Get the number of Unicode scalar values in the buffer.
    ///
    /// This counts characters, not bytes. For example, `"café"` returns `4`,
    /// and `"🦀"` returns `1`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dscode_core::TextBuffer;
    ///
    /// let buffer = TextBuffer::new("café");
    /// assert_eq!(buffer.len_chars(), 4);
    /// ```
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// Check whether the buffer contains no characters.
    ///
    /// Returns `true` when [`TextBuffer::len_chars`] is zero.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dscode_core::TextBuffer;
    ///
    /// let buffer = TextBuffer::new("");
    /// assert!(buffer.is_empty());
    /// ```
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

    #[test]
    fn test_text_buffer_new_returns_valid_buffer() {
        // Verify new() constructs a buffer whose get_text() matches the input
        let content = "Hello, DSCode!";
        let buffer = TextBuffer::new(content);
        assert_eq!(buffer.get_text(), content);
    }

    #[test]
    fn test_text_buffer_get_text_preserves_unicode() {
        // get_text() should faithfully reproduce multi-byte content
        let content = "cafe\u{301} 🦀 Rust"; // café with combining accent + emoji
        let buffer = TextBuffer::new(content);
        assert_eq!(buffer.get_text(), content);
    }

    #[test]
    fn test_text_buffer_is_empty_variations() {
        // Only truly empty string should report is_empty
        assert!(TextBuffer::new("").is_empty());
        assert!(!TextBuffer::new(" ").is_empty()); // space is a character
        assert!(!TextBuffer::new("\n").is_empty()); // newline is a character
        assert!(!TextBuffer::new("\t").is_empty()); // tab is a character
    }

    #[test]
    fn test_text_buffer_len_chars_multibyte() {
        // Emoji and CJK characters should count as single chars
        let buffer = TextBuffer::new("🦀🦀🦀");
        assert_eq!(buffer.len_chars(), 3);

        let buffer = TextBuffer::new("日本語");
        assert_eq!(buffer.len_chars(), 3);
    }

    #[test]
    fn test_text_buffer_large_multiline() {
        // Verify line-count consistency for a large multi-line buffer
        let line_count = 50_000;
        let content: String = (0..line_count)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let buffer = TextBuffer::new(&content);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.get_text(), content);
    }
}
